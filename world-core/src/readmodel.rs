//! 读模型（`M03`）—— **状态的唯一来源是账本**。
//!
//! 世界的真相只有一条：**语义事件账本**。状态不是被"保存"的，而是被**算出来**的：
//!
//! ```text
//! state = fold(events[0..seq])
//! ```
//!
//! 由此推出三条硬性质（`07/4-计划/03` §七、`07/2-依据/15` §三）：
//!
//! 1. **读模型是派生物，不是真相**。它可以随时被删掉、从账本重算，
//!    结果必须逐字节一致——这是本项目的第三条专属验收测试
//!    （"删掉读模型 → 重算一致"）。本模块在 v1 **不持久化任何东西**：
//!    没有"读模型文件"需要维护一致性，也就**不存在"读模型与账本不一致"**这种经典故障。
//! 2. **回滚 = 追加补偿事件**，绝不修改历史。所以折叠里**没有"撤销"逻辑**——
//!    补偿事件本身就是一条普通的 `change`，折叠照常前进。
//! 3. **折叠必须报错而不是猜**。序号断裂、旧值不符、未知家族——一律拒绝折叠。
//!    读模型若默默容忍坏账本，那么"账本是唯一真相"就成了一句空话。
//!
//! 关于 `before` 的核对：`change` 事件按法律**必带旧值**（回滚所需信息当场留下）。
//! 本模块因此在折叠时核对"事件声称的旧值 == 账本折叠出的当前值"——
//! 这是**读模型侧的第二道墙**：即便有人手改了账本，也会在这里被拦下。
//! 该 path **首次出现**时不核对（此前无当前值可比），此时 `before` 允许为 `null`。
//!
//! ⚠️ 快照（`M08`）在 v1 **不存在**。将来若加，它只能是**带 `base_seq` 的缓存**，
//! 且必须永远可被"从账本重算"覆盖验证——否则它就从缓存悄悄变成了第二真相。

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// 世界状态。**只能由 [`State::apply`] / [`State::fold`] 产生**。
///
/// 字段私有：外部拿不到可改的句柄，也就不可能"绕过账本改状态"。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    last_seq: u64,
    seen: u64,
    acts: u64,
    notices: u64,
    /// 主体 → 字段路径 → 当前值。用 `BTreeMap` 是为了**确定性**：
    /// 键有序 ⇒ 同样的账本必然渲染出同样的字节。（`serde_json` 默认的
    /// `Map` 也是有序的，两者共同保证折叠结果可逐字节比对。）
    objects: BTreeMap<String, BTreeMap<String, Value>>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// 已折叠到的账本序号（空状态为 0）。
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// 已折叠的事件条数。
    pub fn seen(&self) -> u64 {
        self.seen
    }

    /// 已折叠的 `act` 条数（动作是事实，本身就是一等事件）。
    pub fn acts(&self) -> u64 {
        self.acts
    }

    /// 已折叠的 `notice` 条数。
    pub fn notices(&self) -> u64 {
        self.notices
    }

    /// 读一个字段的当前值。
    pub fn get(&self, subject: &str, path: &str) -> Option<&Value> {
        self.objects.get(subject).and_then(|m| m.get(path))
    }

    /// 遍历全部 `(主体, 路径, 值)`。顺序**确定**（主体、路径皆为有序键），
    /// 因此同一次折叠必然给出同一个遍历序列。
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str, &Value)> + '_ {
        self.objects.iter().flat_map(|(subject, paths)| {
            paths
                .iter()
                .map(move |(path, v)| (subject.as_str(), path.as_str(), v))
        })
    }

    /// 折叠一条事件。**必须按 `seq` 顺序、且从连续前缀开始**。
    pub fn apply(&mut self, ev: &Value) -> Result<(), String> {
        let seq = ev.get("seq").and_then(Value::as_u64).ok_or_else(|| {
            "ext.world.ReadModel.MissingSeq: 事件缺少 seq：账本不是合法 JSON Lines".to_string()
        })?;

        let expected = self.last_seq + 1;
        if seq != expected {
            return Err(format!(
                "ext.world.ReadModel.SeqGap: 事件序号不连续：已折叠到 {}，期望 {expected}，实际 {seq}\
                 （读模型只折叠**连续的账本前缀**，不猜缺口）",
                self.last_seq
            ));
        }

        let kind = ev
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("事件 seq={seq} 缺少 kind"))?;

        match kind {
            "change" => self.apply_change(seq, ev)?,
            "act" => self.acts += 1,
            "notice" => self.notices += 1,
            other => {
                return Err(format!(
                    "ext.world.ReadModel.UnknownKind: 未知事件家族 `{other}`（seq={seq}）：读模型**拒绝猜测**其语义\
                     ——法律与读模型必须同源"
                ))
            }
        }

        self.last_seq = seq;
        self.seen += 1;
        Ok(())
    }

    fn apply_change(&mut self, seq: u64, ev: &Value) -> Result<(), String> {
        let body = ev
            .get("body")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("change 事件 seq={seq} 缺少 body"))?;
        let subject = body
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("change 事件 seq={seq} 的 body 缺少 subject"))?;
        let path = body
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("change 事件 seq={seq} 的 body 缺少 path"))?;
        if !body.contains_key("after") {
            return Err(format!("change 事件 seq={seq} 的 body 缺少 after"));
        }
        let after = body.get("after").cloned().unwrap_or(Value::Null);

        // 第二道墙：事件声称的旧值必须等于账本折叠出的当前值。
        if let Some(current) = self.objects.get(subject).and_then(|m| m.get(path)) {
            match body.get("before") {
                Some(before) if before == current => {}
                Some(before) => {
                    return Err(format!(
                        "ext.world.ReadModel.BeforeMismatch: 旧值不符（seq={seq}，{subject}#{path}）：事件称 before={before}，\
                         但账本折叠出的当前值是 {current}。账本与事件不符——**拒绝折叠**"
                    ))
                }
                None => {
                    return Err(format!(
                        "change 事件 seq={seq} 缺少 before：回滚所需信息必须当场留下"
                    ))
                }
            }
        }

        self.objects
            .entry(subject.to_string())
            .or_default()
            .insert(path.to_string(), after);
        Ok(())
    }

    /// 从**规范形式**（[`State::to_json`] 的产物）重建状态。
    ///
    /// 用途：检查点（`M08`）从缓存恢复。字段私有 ⇒ 只有本模块能构造 `State`，
    /// 于是"从快照恢复"这件事也**必须**经过这里，不能绕过。
    pub fn from_json(v: &Value) -> Result<Self, String> {
        let u64_of = |k: &str| -> Result<u64, String> {
            v.get(k)
                .and_then(Value::as_u64)
                .ok_or_else(|| format!("ext.world.ReadModel.BadState: 缺字段 `{k}`"))
        };
        let mut objects: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
        if let Some(objs) = v.get("objects") {
            let objs = objs
                .as_object()
                .ok_or_else(|| "ext.world.ReadModel.BadState: objects 不是对象".to_string())?;
            for (subject, paths) in objs {
                let paths = paths.as_object().ok_or_else(|| {
                    format!("ext.world.ReadModel.BadState: objects[{subject}] 不是对象")
                })?;
                let mut m = BTreeMap::new();
                for (p, val) in paths {
                    m.insert(p.clone(), val.clone());
                }
                objects.insert(subject.clone(), m);
            }
        }
        Ok(State {
            last_seq: u64_of("last_seq")?,
            seen: u64_of("seen")?,
            acts: u64_of("acts")?,
            notices: u64_of("notices")?,
            objects,
        })
    }

    /// 从零折叠一串事件。
    pub fn fold(events: &[Value]) -> Result<Self, String> {
        let mut s = State::new();
        for ev in events {
            s.apply(ev)?;
        }
        Ok(s)
    }

    /// **规范形式**（canonical form）：键有序，可直接逐字节比对。
    pub fn to_json(&self) -> Value {
        let mut objects = Map::new();
        for (subject, paths) in &self.objects {
            let mut m = Map::new();
            for (path, v) in paths {
                m.insert(path.clone(), v.clone());
            }
            objects.insert(subject.clone(), Value::Object(m));
        }
        json!({
            "last_seq": self.last_seq,
            "seen": self.seen,
            "acts": self.acts,
            "notices": self.notices,
            "objects": Value::Object(objects),
        })
    }

    /// 读模型的**指纹**。
    ///
    /// ⚠️ **非加密用途**：只用 FNV-1a（不引入依赖）。它回答的唯一问题是
    /// "两次折叠是不是同一个结果"，用于第三条验收测试与将来的"同源"核对。
    /// **不得**用它做安全判断。
    pub fn digest(&self) -> String {
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut h = OFFSET;
        for b in self.to_json().to_string().as_bytes() {
            h ^= u64::from(*b);
            h = h.wrapping_mul(PRIME);
        }
        format!("fnv1a64:{h:016x}")
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use crate::event;

    fn change(seq: u64, subject: &str, path: &str, before: Value, after: Value) -> Value {
        event::new_event(
            seq,
            "change",
            "world://test",
            event::change_body(subject, path, before, after),
        )
    }

    #[test]
    fn folds_contiguous_prefix_and_applies_last_write() {
        let evs = vec![
            change(1, "world://a", "n", json!(1), json!(2)),
            change(2, "world://a", "n", json!(2), json!(3)),
            event::new_event(
                3,
                "notice",
                "world://test",
                event::notice_body("muted", "world://a", json!({})),
            ),
        ];
        let s = State::fold(&evs).unwrap();
        assert_eq!(s.last_seq(), 3);
        assert_eq!(s.seen(), 3);
        assert_eq!(s.notices(), 1);
        assert_eq!(s.get("world://a", "n"), Some(&json!(3)));
    }

    #[test]
    fn refuses_seq_gap() {
        let evs = vec![change(2, "world://a", "n", json!(null), json!(1))];
        assert!(State::fold(&evs).unwrap_err().contains("不连续"));
    }

    #[test]
    fn refuses_lying_before() {
        let evs = vec![
            change(1, "world://a", "n", json!(1), json!(2)),
            change(2, "world://a", "n", json!(99), json!(3)), // 谎称旧值是 99
        ];
        assert!(State::fold(&evs).unwrap_err().contains("旧值不符"));
    }

    #[test]
    fn refuses_unknown_family() {
        let evs = vec![event::new_event(1, "guess", "world://test", json!({}))];
        assert!(State::fold(&evs).unwrap_err().contains("未知事件家族"));
    }
}
