//! 语义事件的构造。
//!
//! 唯一原语是**语义事件**（`07/2-依据/14-总线词表v0.md`），三个家族：
//! - `change` **变更**：主体 + 字段路径 + 旧值 + 新值
//! - `act`    **动作**：能力 + 动词 + 请求号（结果本身也是一条事件）
//! - `notice` **通告**：类型 + 主体（完工铃等）
//!
//! 信封字段里的 `seq` **由账本分配**，所以 [`new_event`] 要求调用者先取号
//! （见 [`crate::World::commit`]）——这是"法律要求信封含 seq"与"账本分配 seq"的接缝。

use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 出厂词表版本。**只加 flags，永不改这个数的含义**。
pub const WORLD_VERSION: u64 = 1;

/// 已定义的能力旗标。未知旗标**必须忽略**（本体纪律）。
///
/// ⚠️ 这个常量同时是**新事件的 `flags` 初值**（见 [`new_event`]）：往这里加一项，
/// **每一条事件**都会带上它。故它只装"出厂即带"的旗标（今天为空）。
/// 由某次裁决**临时**加上去的旗标（例如闸的摩擦标记）另用 [`with_flag`]，
/// 名字常量另立（[`FLAG_FRICTION`]）。
pub const FLAGS: [&str; 0] = [];

/// **摩擦旗标**（闸加在不可逆动作上的可核流水）。
///
/// 依据：书第五章 §5.5 逐字「摩擦本该挂在动作的不可逆等级上：可逆处放手，不可逆处加摩擦」。
/// 加摩擦这件事必须**在账本上留下可核的痕迹**，否则"加过摩擦"只是一句注释。
/// 形式：`gate.friction:<等级>`（等级取自载体执行清单的 `risk`；无清单 ⇒ `gate.friction:unlisted`）。
///
/// 为什么落在信封的 `flags` 上：信封的 `flags` 就是"随事件走的旗标"，
/// 且本体纪律要求**未知旗标必须忽略** ⇒ 旧读法读到它不会坏，新读法能从它核出摩擦发生过。
pub const FLAG_FRICTION: &str = "gate.friction";

/// 给一条已造好的事件补一个**能力旗标**（去重；`flags` 不是数组时**原样放过**，不静默修补）。
pub fn with_flag(ev: &mut Value, flag: &str) {
    let Some(obj) = ev.as_object_mut() else {
        return;
    };
    let entry = obj
        .entry("flags".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let Some(arr) = entry.as_array_mut() else {
        return;
    };
    if !arr.iter().any(|v| v.as_str() == Some(flag)) {
        arr.push(Value::String(flag.to_string()));
    }
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// 造一条事件（**不含 seq 的现实值以外的一切都已就位**）。
///
/// `seq` 由调用者从 [`crate::ledger::Ledger::next_seq`] 取。
pub fn new_event(seq: u64, kind: &str, actor: &str, body: Value) -> Value {
    let mut m = Map::new();
    m.insert("world".to_string(), json!(WORLD_VERSION));
    m.insert("kind".to_string(), json!(kind));
    m.insert("id".to_string(), json!(new_id()));
    m.insert("seq".to_string(), json!(seq));
    m.insert("at".to_string(), json!(unix_secs()));
    m.insert("actor".to_string(), json!(actor));
    m.insert("flags".to_string(), json!(FLAGS.to_vec()));
    m.insert("body".to_string(), body);
    Value::Object(m)
}

/// 给一条已造好的事件补上**可选**信封字段（`trace` 因果 / `to` 目的地）。
///
/// 为什么需要它（`M10` 接线，2026-09-27）：`trace` 是本体里**早就定义**的可选字段
/// （`ontology.json` 的 `envelope.optional`），但此前**没有任何写入路径会填它**——
/// 于是"为什么会变成这样"这条追问在 v1 里没有落点（`WC-FMT-001` 记为待办）。
/// 现在由"请求—结果"配对填它：**结果事件的 `trace` = 引发它的那条意图事件的 `id`**。
///
/// 口径（刻意）：
/// - `None` 即**不写该键**，不写 `null`——"没有因果"与"因果指向空"是两件事；
/// - 空串视为未给（否则会写出一个指不到任何事件的 `trace`）；
/// - 非对象的事件**原样放过**（形态问题由本体校验负责报，不在这里静默修补）。
pub fn with_trace(ev: &mut Value, trace: Option<&str>) {
    if let (Some(t), Some(obj)) = (trace, ev.as_object_mut()) {
        if !t.is_empty() {
            obj.insert("trace".to_string(), json!(t));
        }
    }
}

/// 给一条已造好的事件补上 `to`（目的地；空 = 广播）。
///
/// 与 [`with_trace`] 同口径：`None`/空串 ⇒ 不写该键。
pub fn with_to(ev: &mut Value, to: Option<&str>) {
    if let (Some(t), Some(obj)) = (to, ev.as_object_mut()) {
        if !t.is_empty() {
            obj.insert("to".to_string(), json!(t));
        }
    }
}

/// 事件身份。不引入 uuid crate：**纳秒 + 进程内计数器**已足够唯一。
pub fn new_id() -> String {
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("e{}-{}", unix_nanos(), c)
}

/// `change` 的信纸：**`before` 必带**——回滚所需的信息当场留下
/// （RFC 6902 的 patch 没有逆操作，不记旧值就回不去）。
pub fn change_body(subject: &str, path: &str, before: Value, after: Value) -> Value {
    json!({ "subject": subject, "path": path, "before": before, "after": after })
}

/// `act` 的信纸。
pub fn act_body(capability: &str, verb: &str, request_id: &str, params: Value) -> Value {
    json!({
        "capability": capability,
        "verb": verb,
        "request_id": request_id,
        "params": params
    })
}

/// `notice` 的信纸。
pub fn notice_body(kind: &str, subject: &str, payload: Value) -> Value {
    json!({ "type": kind, "subject": subject, "payload": payload })
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
