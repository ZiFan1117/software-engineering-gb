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
//! 4. **缺格即报错**（`REQ-F-032`）。书第五章 5.6 表 5.2 行的通过条件逐字是
//!    「每个已声明的字段至少有一份读法可读，**缺格就报错**」，当日结果逐字是「**红**。未实现」
//!    （合订本 `:737`）。本模块补的是**读模型这一侧**：一行账本少了**已声明的必填格**
//!    ⇒ 拒绝折叠，并**点名缺的那一格**（[`DeclaredCells`] ＋ [`State::apply_declared`]）。
//!    ⚠️ 它与「未知家族」**不是一回事**：不认识的家族仍旧报 `ReadModel.UnknownKind`
//!    （那是 `REQ-F-029` 对偶的另一半），两处不许互相冒充〔本 change 的 delta `REQ-F-027`／`REQ-F-029`〕。
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
use std::fmt;

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
            .ok_or_else(|| missing_cell_msg(seq, "envelope", "kind"))?;

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
        let body = match ev.get("body") {
            Some(Value::Object(m)) => m,
            Some(_) => {
                return Err(format!(
                    "ext.world.ReadModel.BadCell: change 事件 seq={seq} 的 body **不是对象**\
                     ——它不是缺格，是形状不对（读模型不猜、也不修补）"
                ))
            }
            None => return Err(missing_cell_msg(seq, "envelope", "body")),
        };
        let subject = match body.get("subject") {
            Some(Value::String(s)) => s.as_str(),
            Some(_) => {
                return Err(format!(
                    "ext.world.ReadModel.BadCell: change 事件 seq={seq} 的 body.subject **不是字符串**\
                     ——它不是缺格，是形状不对"
                ))
            }
            None => return Err(missing_cell_msg(seq, "body[change]", "subject")),
        };
        let path = match body.get("path") {
            Some(Value::String(s)) => s.as_str(),
            Some(_) => {
                return Err(format!(
                    "ext.world.ReadModel.BadCell: change 事件 seq={seq} 的 body.path **不是字符串**\
                     ——它不是缺格，是形状不对"
                ))
            }
            None => return Err(missing_cell_msg(seq, "body[change]", "path")),
        };
        if !body.contains_key("after") {
            return Err(missing_cell_msg(seq, "body[change]", "after"));
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
                None => return Err(missing_cell_msg(seq, "body[change]", "before")),
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
    ///
    /// ⚠️ 这是**无法律的折叠**：它只查读模型自己就要用的那几格（`seq`／`kind`／`change` 的四处）。
    /// `World::read_model`（`src/lib.rs`）今天走的正是这一条 ⇒ **已声明的必填格缺了，
    /// 在 `state --json` 上仍是静默通过**——这条缺口在 `tools/s1_sys_probe.sh` 的
    /// `TC-047` ⑨ 里早就登记着，逐字：「缺必填信封字段 actor 竟**被接受**（rc=$R）：
    /// 必填字段校验只在写入路径（本体校验）上，折叠层不校验」。
    /// 要合上它，走 [`State::fold_declared`]（带法律的那条路）。
    pub fn fold(events: &[Value]) -> Result<Self, String> {
        let mut s = State::new();
        for ev in events {
            s.apply(ev)?;
        }
        Ok(s)
    }

    /// **带法律的折叠**（`REQ-F-032`）：先核"已声明的必填格"（缺格即报错），再折叠。
    ///
    /// ## 这条判据管什么（逐字对书）
    ///
    /// 书第五章 5.6 表 5.2 行（合订本 `:737`）逐字：
    /// 「读的那一份每个格子都有人读得到 ｜ 每个已声明的字段至少有一份读法可读，缺格就报错
    /// ｜ 缺格即报错 ｜ **红**。未实现」。本方法把"缺格就报错"这一格补成**可执行的**：
    /// 一行账本少了[`DeclaredCells`] 里**已声明的必填格** ⇒ 返回
    /// `ext.world.ReadModel.MissingCell`，并**点名缺的那一格**（"错误可读"不是形容词：
    /// 错误里必须能读出是**哪一层**的**哪一格**，以及该层该有哪些格）。
    ///
    /// ## 与"未知家族"的分工（**不许互相冒充**）
    ///
    /// | 情形 | 判据 | 为什么不能混 |
    /// |---|---|---|
    /// | `kind` 不认识 | `ext.world.ReadModel.UnknownKind`（在 [`State::apply`] 里） | 那是**语义不认识** ⇒ 拒（`REQ-F-029` 对偶的另一半） |
    /// | `kind` 认得、但这一行少了已声明的必填格 | `ext.world.ReadModel.MissingCell`（在本方法） | 那是**该有的格没读到** ⇒ 拒；它的家族是**认得的** |
    ///
    /// 故 [`DeclaredCells::missing_cell`] 对**不认识的家族一律不看**：
    /// 让拒它的理由留在"家族不认识"那一条上，别把病因说错。
    ///
    /// ## 本层**不判**的（如实声明边界，不读作"已完备"）
    ///
    /// - **可选格**（`to`／`trace`／`params`／`payload`）不进 [`DeclaredCells`]：
    ///   本体说它们可选，"没写"是这份法律允许的形态，不是缺格；
    /// - **`concepts` 的字段**（`notice.muted`/`job.status`）归**写入侧**判
    ///   （[`crate::ontology::Ontology::check_concepts`]），读模型侧不重复判；
    /// - 读模型**不渲染**信封的 `id`／`at`／`actor`／`world`／`flags`：它们现在**被读**
    ///   （缺了即拒），但**不进入状态**（`state --json` 里读不到它们）⇒ 书那句
    ///   「每个已声明的字段至少有一份读法可读」在**必填格**这一半成立，另一半仍待补。
    pub fn apply_declared(&mut self, cells: &DeclaredCells, ev: &Value) -> Result<(), String> {
        // **空表不许上电**：没有清单 ⇒ 这条判据无从成立。若在这里默默放行，
        // "一个格都没查"与"每个格都查过了"在**读数上一样**、在**结论上相反**——
        // 那正是本项目最贵的一类错（把没做读成做到了）。与门禁那条「空策略拒绝启动」同一纪律。
        if cells.is_empty() {
            return Err(
                "ext.world.ReadModel.NoDeclaredCells: 没有可比对的**已声明格清单**（空表）——\
                 缺格判据无从成立，故拒绝折叠，而不是默默放行。\n\
                 \x20 处置：把法律以数据递进来（`DeclaredCells::new(ont.envelope_required(), ont.family_required())`）"
                    .to_string(),
            );
        }
        if let Some(m) = cells.missing_cell(ev) {
            return Err(m.to_string());
        }
        self.apply(ev)
    }

    /// 从零折叠一串事件（**带法律**；缺格即报错）。逐条等价于 [`State::apply_declared`]。
    pub fn fold_declared(cells: &DeclaredCells, events: &[Value]) -> Result<Self, String> {
        let mut s = State::new();
        for ev in events {
            s.apply_declared(cells, ev)?;
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

/// **读模型侧的"已声明格"清单**（`REQ-F-032` 判据①：「逐格可枚举」）。
///
/// 它只装**必填**格，且只装本体逐字声明的那两处：
/// `envelope.required`（出厂本体 8 项）与 `families.<家族>.required`（出厂本体 4／3／2）。
/// 可选格（`to`／`trace`／`params`／`payload`）**不进这张表**——本体说它们可选，
/// "没写"是这份法律允许的形态，不是缺格（理由见 [`State::apply_declared`] 的边界一节）。
///
/// ## 为什么它是**数据**，而不是一个"法律"类型（这一条是刻意的）
///
/// 读模型**不许**在生产代码里 `use crate::ontology::…`：`WC-MODREG-001` §2 给 `M03` 的
/// 依赖列逐字是「**无**（生产代码零出边）」，而机核层 `tools/module_graph.py` 判据②
/// 逐边核对「声明集 ≡ 真实 import 集」——读模型加一条生产边就会让它变红。
/// 故法律以**数据**递进来，依赖方向留在**装配处**（`M04` 同时依赖 `M01` 与 `M03`）：
/// 谁递 = `World::read_model` 的调用点；数据从 [`crate::ontology::Ontology`] 取
/// （`envelope_required`／`family_required`），**同一份出厂本体** ⇒ 同源。
///
/// ⚠️ 递进来的若是空表（[`DeclaredCells::is_empty`]），[`State::apply_declared`] **拒绝折叠**：
/// 没有清单＝这条判据无从成立，"一个格都没查"不许被读成"检查通过"
/// （与门禁那条「空策略拒绝启动」同一纪律）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclaredCells {
    envelope_required: Vec<String>,
    family_required: BTreeMap<String, Vec<String>>,
}

impl DeclaredCells {
    /// 用**纯数据**装配（调用方从本体取；见类型文档的"为什么是数据"）。
    pub fn new(envelope_required: Vec<String>, family_required: BTreeMap<String, Vec<String>>) -> Self {
        Self {
            envelope_required,
            family_required,
        }
    }

    /// 一个格都没有 ⇒ 本层无从判（**不许**读成"检查通过"）。
    pub fn is_empty(&self) -> bool {
        self.envelope_required.is_empty() && self.family_required.is_empty()
    }

    /// 信封已声明的必填格（原顺序）。
    pub fn envelope_required(&self) -> &[String] {
        &self.envelope_required
    }

    /// 某家族已声明的必填格（`None` ＝ 法律里没有这个家族）。
    pub fn family_required(&self, kind: &str) -> Option<&[String]> {
        self.family_required.get(kind).map(Vec::as_slice)
    }

    /// **这一行缺了哪一格？** `None` ＝ 已声明的必填格齐备。
    ///
    /// 三处**刻意不判**（判了就会把病因说错）：
    /// - **不认识的家族** ⇒ 一律返回 `None`：拒它的判据是
    ///   [`State::apply`] 的 `UnknownKind`（`REQ-F-029` 对偶的另一半），不是"缺格"；
    /// - **非对象的事件**、**非对象的 `body`** ⇒ 返回 `None`：那是**形状**问题，
    ///   由 [`State::apply`] 报（`NotAnObject` 那一族的语义），缺格判据不抢它的错；
    /// - **可选格**：它们根本不在表里（见类型文档）。
    pub fn missing_cell(&self, ev: &Value) -> Option<MissingCell> {
        let obj = ev.as_object()?;
        let seq = obj.get("seq").and_then(Value::as_u64);
        for f in &self.envelope_required {
            if !obj.contains_key(f) {
                return Some(MissingCell {
                    at: "envelope".to_string(),
                    field: f.clone(),
                    declared: self.envelope_required.join(", "),
                    seq,
                });
            }
        }
        let kind = obj.get("kind").and_then(Value::as_str).unwrap_or("");
        let req = self.family_required.get(kind)?;
        // 信纸不是对象 ⇒ 形状问题，交给 `apply`（见上文"三处刻意不判"）。
        let body = obj.get("body").and_then(Value::as_object)?;
        for f in req {
            if !body.contains_key(f) {
                return Some(MissingCell {
                    at: format!("body[{kind}]"),
                    field: f.clone(),
                    declared: req.join(", "),
                    seq,
                });
            }
        }
        None
    }
}

/// 读模型侧的**缺格**（`REQ-F-032`）：一格"该在而不在"。
///
/// 为什么不复用 `Option<&Value>` 的"没读到就是 `None`"：那正是**静默通过**的形状。
/// 缺格是一个**有名字的事实**，它必须能被打印、被点名、被断言——
/// 否则"缺格即报错"只是一句口号（书第五章 5.6 表 5.2 行判的就是这一格）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingCell {
    /// 缺在哪一层：`envelope`，或 `body[<家族>]`。
    pub at: String,
    /// 缺的那一格的名字（**错误可读**的判据就在这个字段上：不许只说"有缺格"）。
    pub field: String,
    /// 这一层**已声明的必填格**（逗号分隔）——报错要让人当场知道"该有哪些"，
    /// 否则这条错误只说了"不行"、没说"怎么办"（与 `ontology.rs` 的 `UndeclaredEntity` 同一体例）。
    pub declared: String,
    /// 这一行的 `seq`（读不到就是 `None`——那本身也是一种缺格）。
    pub seq: Option<u64>,
}

impl fmt::Display for MissingCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seq = match self.seq {
            Some(s) => s.to_string(),
            None => "?".to_string(),
        };
        write!(
            f,
            "ext.world.ReadModel.MissingCell: 缺格：{} 少了**已声明的必填格** `{}`（seq={seq}）——\
             读模型**不猜**\"没有就是空\"：已声明的格读不到，就拒绝折叠。\n\
             \x20 该层已声明的必填格：{}\n\
             \x20 处置：把这一格补进那一行再读；若它本就该是可选格，改的是**本体**（改法律＝走评审）",
            self.at, self.field, self.declared
        )
    }
}

/// **无法律折叠**那条路上的缺格文案（[`State::apply`]／[`State::apply_change`] 用）。
///
/// 与 [`MissingCell`] 同一个错误码（`ext.world.ReadModel.MissingCell`）——"缺格"这件事
/// 全项目一种说法；差别只在**能不能列出"该层已声明的必填格"**：无法律时列不出来，
/// 故这里如实写明"只查读模型自己要用的那几格"，不假装手里有一份法律。
fn missing_cell_msg(seq: u64, at: &str, field: &str) -> String {
    format!(
        "ext.world.ReadModel.MissingCell: 缺格：{at} 少了 `{field}`（seq={seq}）\
         ——（无法律折叠：读模型只查它自己要用的那几格；\"该层已声明的必填格\"要由带法律的那条路给出，\
         见 `State::apply_declared`）"
    )
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
