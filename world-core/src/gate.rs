//! 门禁（`M05`）—— **不可绕过是 world 与 app 之间唯一的硬分界**。
//!
//! 依据 `07/2-依据/15-世界核心的组成与职责.md` 与 `WC-SRS-001`
//! `REQ-F-015/016/017`。
//!
//! ## 门禁不是什么
//!
//! 不是"权限系统"，也不是"登录校验"。它只回答一个问题：
//! **这一条 `act` 事件，现在能不能执行？** 并且把答案变成**可读的流水**。
//!
//! ## 三条设计口径（都是刻意的）
//!
//! 1. **默认拒绝**：`policy.json` 的 `capabilities` 里没有的能力一律不放行。
//!    理由：默认允许的系统，其安全性等于"没人写错规则"；而规则一定会写错。
//! 2. **按不可逆性分级加摩擦**：可逆的动作免检但**必须留痕**；
//!    不可逆的动作由 `irreversible_actors` 白名单主体执行，白名单外的主体走
//!    `AwaitApproval`（加摩擦）——**v1 没有审批通道**（`DEBT-07`），不是"等等就会批"。
//!    理由：不是所有事情都值得拦，但所有事情都值得记。
//! 3. **拒绝也要留痕**：被拒的动作**不写入 `act` 事件**（它没发生），
//!    但会写一条 `notice`（`gate.rejected` / `gate.awaiting-approval`）。
//!    理由：本项目的目标是"事前拦截 + 一条人能读懂的流水"，
//!    只拦不记等于把审计能力丢掉。
//!
//! ## 不可绕过的另一半
//!
//! 本模块只负责**决策**。让决策无法被绕过的，是
//! ① [`crate::World::commit`] 这一处**唯一咽喉**（进程内无第二条写路径），
//! ② [`crate::guard`] 对策略文件与账本的**权限静态检查**（被管者改不动规则）。
//!
//! 三者缺一，"不可绕过"就不成立——所以它们是**一组**，不能只看其中一处。

use crate::guard;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 一个能力的评级。
///
/// 只有一个字段是**刻意的**（`WC-R4-DISP-001` §三 **E-5** 裁定①：删字段）：
/// 原先还有一个 `requires_approval`，但它不参与裁决（`decide` 只读 `reversible`），
/// 却让 `policy` 子命令打印出"不可逆 → 加摩擦（需批准）"——而 v1 **没有审批通道**
/// （`DEBT-07`：不可逆只允许 `irreversible_actors` 白名单主体执行）。
/// 承诺与实现不符 ⇒ 删除该字段，裁决口径只由 `reversible` + 白名单表达。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    /// 该动作是否可逆（可逆 ⇒ 免检但留痕）。
    pub reversible: bool,
}

/// 门禁裁决。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// 放行。
    Allow,
    /// 拒绝（法律里没有这个能力、或主体不在白名单）。
    Reject(String),
    /// 加摩擦：本主体不可执行该不可逆动作。v1 **没有审批通道**，
    /// 须改由 `irreversible_actors` 白名单主体执行（`DEBT-07`）。
    AwaitApproval(String),
}

/// 出厂门禁策略（`policy.json` 的内存形态）。
#[derive(Debug, Clone)]
pub struct Policy {
    version: u64,
    caps: BTreeMap<String, Capability>,
    allow: Vec<String>,
    /// **谁可以写哪些主体**（`change` 家族的授权）。
    ///
    /// 为什么必须有这一段（2026-09-26 由安全评审发现，见 `WC-RV-R2-001` **S-01**）：
    /// 原实现只对 `act` 裁决，而 `change` **才是真正的状态写原语**——
    /// 于是任何"被拒绝的动作"都可以用一条 `change` 静默达成，
    /// `actor` 与 `subject` 之间毫无约束。**当时"门禁不可绕过"是假的。**
    ///
    /// 默认拒绝：本映射里没有列出的主体，**不得写任何主体**。
    writes: BTreeMap<String, Vec<String>>,
    /// **谁可以执行不可逆动作**（2026-09-26 加，见 WC-RV-R2-001 FIND-12 / DEBT-07）。
    ///
    /// 为什么需要：原先 `!reversible` 一律返回 `AwaitApproval`，而 `commit` 收到即 `Err`，
    /// 仓内**没有批准命令、没有批准事件、没有消费路径** ⇒ 被判不可逆的能力
    /// 在 v1 **永远无法执行**（`policy.json` 里三个能力因此全是死号）。
    /// 文档却写"先取得批准再重试"——措辞与实现不符。
    ///
    /// v1 的处置（**明确的最小治理规则**）：不可逆动作**只允许白名单里的主体**执行
    /// （通常是世界的主人）；其他主体拿到的是"加摩擦"，且理由里**说明 v1 没有审批通道**，
    /// 不得让人以为"等等就能批"。
    irreversible_actors: Vec<String>,
    path: PathBuf,
}

/// 模式匹配：`*` 结尾表示前缀匹配，否则要求完全相等。
fn pattern_matches(pattern: &str, value: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => value.starts_with(prefix),
        None => value == pattern,
    }
}

impl Policy {
    /// 加载策略。**含静态防线检查**（文件与目录不得对 group/other 可写）。
    ///
    /// 启动时**一次**读入；运行中不重读（见 `policy.json` 的 `_invariants`）。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("门禁策略（法律）无法读取 {}：{e}", path.display()))?;
        let root: Value = serde_json::from_str(&text)
            .map_err(|e| format!("门禁策略（法律）不是合法 JSON（{}）：{e}", path.display()))?;

        let version = root
            .get("policy")
            .and_then(Value::as_u64)
            .ok_or_else(|| "门禁策略缺少 `policy` 版本号".to_string())?;
        if version != 1 {
            return Err(format!(
                "门禁策略版本不支持：期望 1，实得 {version}（法律版本不符即拒绝启动）"
            ));
        }

        let caps_obj = root
            .get("capabilities")
            .and_then(Value::as_object)
            .ok_or_else(|| "门禁策略缺少 `capabilities` 段".to_string())?;
        let mut caps = BTreeMap::new();
        for (name, spec) in caps_obj {
            let reversible = spec
                .get("reversible")
                .and_then(Value::as_bool)
                .ok_or_else(|| format!("能力 `{name}` 缺少 reversible 布尔值"))?;
            // 本段为手工取值（`serde_json::Value`，**没有** `deny_unknown_fields`）：
            // 能力项里的多余/未知键（例如已删除的 `requires_approval`）一律**忽略**。
            // 理由（E-5 裁定①）：未知键既不该拒载（那是加载期自检的活，而它已被删除），
            // 也不该影响裁决——裁决只看 `reversible` 与 `irreversible_actors`。
            caps.insert(name.clone(), Capability { reversible });
        }
        if caps.is_empty() {
            return Err(
                "门禁策略的 capabilities 为空——空策略不是『什么都不允许』的安全默认，\
                 而是『法律没写好』；请显式写出允许的能力（默认可通过不声明获得）"
                    .to_string(),
            );
        }

        let allow = root
            .get("subjects")
            .and_then(|s| s.get("allow"))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if allow.is_empty() {
            return Err(
                "门禁策略的 subjects.allow 为空——没有主体能做事的世界等于死掉；\
                 请显式列出允许的主体（支持结尾 * 前缀匹配）"
                    .to_string(),
            );
        }

        // ── writes：change 家族的授权（S-01）──
        let writes_obj = root
            .get("writes")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                "门禁策略缺少 `writes` 段——**默认拒绝**要求显式声明\
                 「谁可以写哪些主体」；缺失即拒绝加载（否则 change 可绕过一切裁决）"
                    .to_string()
            })?;
        let mut writes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (actor, subjects) in writes_obj {
            if actor.starts_with('_') {
                continue;
            }
            let list = subjects
                .as_array()
                .ok_or_else(|| format!("writes 的 `{actor}` 不是数组"))?
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>();
            writes.insert(actor.clone(), list);
        }
        if writes.is_empty() {
            return Err(
                "门禁策略的 writes 为空——没有任何主体能写入状态的世界等于死掉；\
                 请显式列出「主体 → 可写主体模式」（默认拒绝）"
                    .to_string(),
            );
        }

        // irreversible_actors：谁能执行不可逆动作（默认拒绝：缺省即无人）
        let irreversible_actors = root
            .get("irreversible_actors")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // 静态防线：规则所在之处，被管者不得能写
        guard::assert_not_other_writable(path, "门禁策略（法律）")?;

        Ok(Policy {
            version,
            caps,
            allow,
            writes,
            irreversible_actors,
            path: path.to_path_buf(),
        })
    }

    /// **写入裁决**：这个 `actor` 能不能写 `subject`（`change` 家族的授权）。
    ///
    /// 这是 2026-09-26 补上的一道闸：在此之前 `change` 只过本体形状校验，
    /// 于是"被拒绝的 `act`"完全可以用一条 `change` 达成（S-01）。
    pub fn authorize_write(&self, actor: &str, subject: &str) -> Decision {
        if !self.subject_allowed(actor) {
            return Decision::Reject(format!(
                "主体 `{actor}` 不在门禁白名单内（policy.json 的 subjects.allow）"
            ));
        }
        match self.writes.get(actor) {
            None => Decision::Reject(format!(
                "主体 `{actor}` 未获写入授权（policy.json 的 writes 里没有它）——\
                 默认拒绝：状态不能被未授权的角色直接改写"
            )),
            Some(patterns) => {
                if patterns.iter().any(|p| pattern_matches(p, subject)) {
                    Decision::Allow
                } else {
                    Decision::Reject(format!(
                        "主体 `{actor}` 无权写 `{subject}`（writes 允许的是 {patterns:?}）——\
                         越界的写入一律不放行"
                    ))
                }
            }
        }
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 遍历已声明的能力（键有序，输出确定）。
    pub fn capabilities(&self) -> impl Iterator<Item = (&str, &Capability)> {
        self.caps.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn allowed_subjects(&self) -> &[String] {
        &self.allow
    }

    /// 主体是否在白名单内。`*` 结尾表示前缀匹配。
    pub fn subject_allowed(&self, actor: &str) -> bool {
        self.allow.iter().any(|pat| pattern_matches(pat, actor))
    }

    /// **裁决**：这条 `act` 的信纸能不能执行。
    pub fn decide(&self, actor: &str, body: &Value) -> Decision {
        let capability = match body.get("capability").and_then(Value::as_str) {
            Some(c) => c,
            None => return Decision::Reject("act 信纸缺少 capability".to_string()),
        };
        let verb = body.get("verb").and_then(Value::as_str).unwrap_or("-");

        if !self.subject_allowed(actor) {
            return Decision::Reject(format!(
                "主体 `{actor}` 不在门禁白名单内（policy.json 的 subjects.allow）"
            ));
        }

        match self.caps.get(capability) {
            None => Decision::Reject(format!(
                "能力 `{capability}` 未在门禁策略中声明（verb={verb}）——\
                 法律里没有的能力一律不放行（默认拒绝）"
            )),
            Some(c) if !c.reversible => {
                if self
                    .irreversible_actors
                    .iter()
                    .any(|a| pattern_matches(a, actor))
                {
                    Decision::Allow
                } else {
                    Decision::AwaitApproval(format!(
                        "能力 `{capability}` 不可逆（verb={verb}），而 `{actor}` 不在\
                         irreversible_actors 白名单内。\n\
                         \x20 ⚠️ v1 **没有审批通道**（无批准命令、无批准事件）——\
                         不要等批准，它不会来。\n\
                         \x20 处置：由白名单主体（{:?}）执行该动作",
                        self.irreversible_actors
                    ))
                }
            }
            Some(_) => Decision::Allow,
        }
    }

    /// 策略摘要（供 CLI 打印与记录归档）。
    pub fn summary(&self) -> Value {
        let mut caps = Map::new();
        for (name, c) in &self.caps {
            caps.insert(
                name.to_string(),
                serde_json::json!({
                    "reversible": c.reversible,
                }),
            );
        }
        serde_json::json!({
            "policy": self.version,
            "path": self.path.display().to_string(),
            "capabilities": Value::Object(caps),
            "subjects_allow": self.allow,
        })
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use serde_json::json;

    fn test_policy() -> Policy {
        let mut caps = BTreeMap::new();
        caps.insert("notice.mute".to_string(), Capability { reversible: true });
        caps.insert(
            "ledger.compact".to_string(),
            Capability { reversible: false },
        );
        let mut writes = BTreeMap::new();
        writes.insert("world://user".to_string(), vec!["world://*".to_string()]);
        writes.insert(
            "world://agent/*".to_string(),
            vec!["world://agent/*".to_string()],
        );
        Policy {
            version: 1,
            caps,
            allow: vec!["world://user".to_string(), "world://agent/*".to_string()],
            writes,
            irreversible_actors: vec!["world://user".to_string()],
            path: PathBuf::from("policy.json"),
        }
    }

    #[test]
    fn declares_default_deny() {
        let p = test_policy();
        match p.decide("world://user", &json!({"capability": "nope", "verb": "do"})) {
            Decision::Reject(m) => assert!(m.contains("未在门禁策略中声明"), "{m}"),
            other => panic!("期望拒绝，实得 {other:?}"),
        }
    }

    #[test]
    fn allows_reversible_and_adds_friction_for_irreversible() {
        let p = test_policy();
        assert_eq!(
            p.decide(
                "world://user",
                &json!({"capability": "notice.mute", "verb": "do"})
            ),
            Decision::Allow
        );
        // 不可逆动作：v1 规则 = 只允许 irreversible_actors 白名单里的主体执行
        // （2026-09-26 起；此前一律 AwaitApproval，等于该能力是死号 —— 见 DEBT-07）
        assert_eq!(
            p.decide(
                "world://user",
                &json!({"capability": "ledger.compact", "verb": "do"})
            ),
            Decision::Allow,
            "白名单主体应能执行不可逆动作，否则该能力是死号"
        );
        match p.decide(
            "world://agent/1",
            &json!({"capability": "ledger.compact", "verb": "do"}),
        ) {
            Decision::AwaitApproval(m) => {
                assert!(m.contains("不可逆"), "{m}");
                assert!(
                    m.contains("没有审批通道"),
                    "拒绝理由必须明说 v1 无审批通道，别让人以为等等就能批：{m}"
                );
            }
            other => panic!("期望加摩擦，实得 {other:?}"),
        }
    }

    #[test]
    fn rejects_subject_outside_whitelist() {
        let p = test_policy();
        match p.decide(
            "world://stranger",
            &json!({"capability": "notice.mute", "verb": "do"}),
        ) {
            Decision::Reject(m) => assert!(m.contains("不在门禁白名单内"), "{m}"),
            other => panic!("期望拒绝，实得 {other:?}"),
        }
    }

    #[test]
    fn prefix_pattern_matches_all_agents() {
        let p = test_policy();
        assert!(p.subject_allowed("world://agent/1"));
        assert!(p.subject_allowed("world://agent/anything"));
        assert!(!p.subject_allowed("world://agentX"));
    }
}
