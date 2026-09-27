//! 本体（法律）：加载出厂词表，并校验一条语义事件是否合法。
//!
//! 本体是「世界的法律」——它规定"一条事件长什么样、什么算合法变更"。
//! 它**不是**代码里的硬编码常量，而是**运行时装起来的纯文本文件**（出厂设置）。
//! 依 `07/2-依据/15-世界核心的组成与职责.md` §2.3：本体是"装起来的"，
//! 方式是「运行时装 + 用内容寻址钉住版本」。

use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

/// 校验失败的原因。**类型化**——上层按码处理，不靠散文猜失败原因。
#[derive(Debug, PartialEq)]
pub enum Violation {
    NotAnObject,
    MissingField { at: String, field: String },
    BadVersion { expected: u64, got: u64 },
    UnknownKind { kind: String },
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Violation::NotAnObject => {
                write!(f, "ext.world.Ontology.NotAnObject: 事件必须是 JSON 对象")
            }
            Violation::MissingField { at, field } => {
                write!(
                    f,
                    "ext.world.Ontology.MissingField: {at} 缺少必填字段 `{field}`"
                )
            }
            Violation::BadVersion { expected, got } => write!(
                f,
                "ext.world.Ontology.BadVersion: 词表版本不符（期望 {expected}，实得 {got}）"
            ),
            Violation::UnknownKind { kind } => {
                write!(f, "ext.world.Ontology.UnknownKind: 未知家族 `{kind}`")
            }
        }
    }
}

impl std::error::Error for Violation {}

#[derive(Debug)]
struct Family {
    required: Vec<String>,
    #[allow(dead_code)]
    optional: Vec<String>,
}

/// 出厂本体。加载后即为本次运行的"法律"。
#[derive(Debug)]
pub struct Ontology {
    world: u64,
    required: Vec<String>,
    optional: Vec<String>,
    families: BTreeMap<String, Family>,
    /// 词表的**内容地址**（见 [`Ontology::vocab_hash`]），加载时算好。
    vocab_hash: String,
}

impl Ontology {
    /// 从纯文本 JSON 文件加载本体。
    ///
    /// 加载失败**必须让程序拒绝启动**——法律不对，带病跑比不跑更危险
    /// （`07/4-计划/03` §五 启动顺序第 ③ 步：本体版本对不上即拒绝启动）。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("ext.world.Ontology.ReadFail: {path:?}: {e}"))?;
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| format!("ext.world.Ontology.BadJson: {path:?}: {e}"))?;

        // 静态防线（2026-09-26 补，见 WC-RV-R2-001 S-06）：词表即契约，
        // 与门禁策略、账本一样必须放在被管者不可写之处。
        // ⚠️ 顺序：**先读成功、再查权限**——否则"文件不存在"会报成
        // "无法读取…的权限"，把简单故障说成权限问题（c04 实测抓到）。
        crate::guard::assert_not_other_writable(path, "本体（法律·形状）")?;

        let world = v
            .get("world")
            .and_then(Value::as_u64)
            .ok_or_else(|| "ext.world.Ontology.NoVersion: 缺 `world` 版本号".to_string())?;

        let envelope = v
            .get("envelope")
            .ok_or_else(|| "ext.world.Ontology.NoEnvelope: 缺 `envelope`".to_string())?;
        let required = str_list(envelope, "required")?;
        let optional = str_list(envelope, "optional")?;

        let fams = v
            .get("families")
            .and_then(Value::as_object)
            .ok_or_else(|| "ext.world.Ontology.NoFamilies: 缺 `families`".to_string())?;
        let mut families = BTreeMap::new();
        for (name, def) in fams {
            // `_comment` 之类的下划线键不是家族
            if name.starts_with('_') {
                continue;
            }
            families.insert(
                name.clone(),
                Family {
                    required: str_list(def, "required")?,
                    optional: str_list(def, "optional").unwrap_or_default(),
                },
            );
        }
        if families.is_empty() {
            return Err("ext.world.Ontology.NoFamilies: 家族列表为空".to_string());
        }

        Ok(Ontology {
            world,
            required,
            optional,
            families,
            vocab_hash: vocab_hash_of(&v),
        })
    }

    pub fn world(&self) -> u64 {
        self.world
    }

    /// **词表的身份**：内容寻址（content-addressed）。
    ///
    /// 为什么需要它：两个投影必须**同源**——同一个读模型 + **同一份词表**
    /// （`WC-SRS-001` REQ-F-020）。若两边各读各的词表副本，"同源"就只是口号。
    /// 故词表本身必须可被内容寻址：内容相同 ⇒ hash 相同；内容一变 ⇒ hash 变，
    /// 于是"有人换了词表"这件事**能被检出**，而不是靠人去比对两份文件。
    ///
    /// 口径（两处刻意选择）：
    /// - 对**规范化 JSON**求值（解析后紧凑序列化、键有序），**不是**文件原始字节——
    ///   否则改一个空格就报"换词表"，那是假警报，会训练人忽略这个信号；
    /// - **剔除 `_` 开头的键**（`_comment` / `_source` 等说明文字）——
    ///   说明文字不是词表语义。改注释不该改变世界的身份。
    ///
    /// ⚠️ 用 FNV-1a 是**非加密**指纹（本项目零外部依赖，不引哈希库）。
    /// 它回答的唯一问题是"是不是同一份词表"，**不得**用于安全判断。
    pub fn vocab_hash(&self) -> &str {
        &self.vocab_hash
    }

    pub fn optional(&self) -> &[String] {
        &self.optional
    }

    pub fn known_kinds(&self) -> Vec<&str> {
        self.families.keys().map(String::as_str).collect()
    }

    /// 校验一条事件是否合法：信封必填 → 版本 → 家族存在 → 信纸必填。
    pub fn validate(&self, ev: &Value) -> Result<(), Violation> {
        let obj = ev.as_object().ok_or(Violation::NotAnObject)?;

        for f in &self.required {
            if !obj.contains_key(f) {
                return Err(Violation::MissingField {
                    at: "envelope".to_string(),
                    field: f.clone(),
                });
            }
        }

        let got = obj.get("world").and_then(Value::as_u64).unwrap_or(0);
        if got != self.world {
            return Err(Violation::BadVersion {
                expected: self.world,
                got,
            });
        }

        let kind = obj.get("kind").and_then(Value::as_str).unwrap_or("");
        let fam = self
            .families
            .get(kind)
            .ok_or_else(|| Violation::UnknownKind {
                kind: kind.to_string(),
            })?;

        let body =
            obj.get("body")
                .and_then(Value::as_object)
                .ok_or_else(|| Violation::MissingField {
                    at: "envelope".to_string(),
                    field: "body".to_string(),
                })?;
        for f in &fam.required {
            if !body.contains_key(f) {
                return Err(Violation::MissingField {
                    at: format!("body[{kind}]"),
                    field: f.clone(),
                });
            }
        }

        Ok(())
    }
}

fn str_list(v: &Value, key: &str) -> Result<Vec<String>, String> {
    let arr = v
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("ext.world.Ontology.BadField: 缺数组字段 `{key}`"))?;
    let mut out = Vec::with_capacity(arr.len());
    for x in arr {
        out.push(
            x.as_str()
                .ok_or_else(|| format!("ext.world.Ontology.BadField: `{key}` 含非字符串项"))?
                .to_string(),
        );
    }
    Ok(out)
}

/// 递归剔除 `_` 开头的键（说明性文字不是词表语义）。
fn strip_comments(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut out = serde_json::Map::new();
            for (k, val) in m {
                if k.starts_with('_') {
                    continue;
                }
                out.insert(k.clone(), strip_comments(val));
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(strip_comments).collect()),
        other => other.clone(),
    }
}

/// 词表的内容地址（口径见 [`Ontology::vocab_hash`]）。
pub fn vocab_hash_of(raw: &Value) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let canonical = strip_comments(raw).to_string();
    let mut h = OFFSET;
    for b in canonical.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    format!("fnv1a64:{h:016x}")
}

#[cfg(test)]
mod unit {
    use super::*;
    use serde_json::json;

    #[test]
    fn vocab_hash_ignores_comments_but_detects_semantic_change() {
        let a = json!({
            "world": 1,
            "_comment": "第一版说明",
            "envelope": {"required": ["world", "kind"], "optional": []},
            "families": {"change": {"required": ["subject"], "optional": []}}
        });
        let b = json!({
            "world": 1,
            "_comment": "**改过措辞的**说明文字",
            "envelope": {"required": ["world", "kind"], "optional": []},
            "families": {"change": {"required": ["subject"], "optional": []}}
        });
        assert_eq!(vocab_hash_of(&a), vocab_hash_of(&b), "改注释不该换词表");

        let c = json!({
            "world": 1,
            "envelope": {"required": ["world", "kind"], "optional": []},
            "families": {"change": {"required": ["subject", "path"], "optional": []}}
        });
        assert_ne!(vocab_hash_of(&a), vocab_hash_of(&c), "改语义必须换词表");
    }

    #[test]
    fn vocab_hash_is_formatting_insensitive() {
        // 同一个对象，直接构造 vs 经过一次 JSON 文本往返 → 必须同 hash
        let a = json!({"world": 2, "families": {"act": {"required": ["capability"]}}});
        let round: Value = serde_json::from_str(&a.to_string()).unwrap();
        assert_eq!(vocab_hash_of(&a), vocab_hash_of(&round));
    }
}
