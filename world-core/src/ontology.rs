//! 本体（法律）：加载出厂词表，并校验一条语义事件是否合法。
//!
//! 本体是「世界的法律」——它规定"一条事件长什么样、什么算合法变更"。
//! 它**不是**代码里的硬编码常量，而是**运行时装起来的纯文本文件**（出厂设置）。
//! 依 `07/2-依据/15-世界核心的组成与职责.md` §2.3：本体是"装起来的"，
//! 方式是「运行时装 + 用内容寻址钉住版本」。

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

/// 校验失败的原因。**类型化**——上层按码处理，不靠散文猜失败原因。
#[derive(Debug, PartialEq)]
pub enum Violation {
    NotAnObject,
    MissingField {
        at: String,
        field: String,
    },
    BadVersion {
        expected: u64,
        got: u64,
    },
    UnknownKind {
        kind: String,
    },
    /// **类型不符**（这一格在，但形状不对——本体声明了它该是什么类型）。
    ///
    /// 与 [`Violation::MissingField`] 分开：那是"该有的格没读到"，这是"格在、形状不对"。
    /// 两者都是**可编程判定**的类别，故按码分开、不合并成一句散文。
    BadFieldType {
        at: String,
        field: String,
        want: String,
        got: String,
    },
    /// **实体没声明过**（`concepts` 里没有它）——声明以外的东西不许落账。
    ///
    /// `known` = 本体里**已声明**的实体名（有序、逗号分隔）：报错要让人当场知道
    /// "哪些是能写的"，否则这条错误只说了"不行"，没说"怎么办"。
    UndeclaredEntity {
        entity: String,
        subject: String,
        known: String,
    },
    /// **字段没声明过**（实体声明了，但这个字段不在它的字段表里）。
    UndeclaredField {
        entity: String,
        field: String,
        subject: String,
        known: String,
    },
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
            Violation::BadFieldType {
                at,
                field,
                want,
                got,
            } => write!(
                f,
                "ext.world.Ontology.BadFieldType: {at} 的 `{field}` 类型不符（本体声明 `{want}`，实得 `{got}`）"
            ),
            Violation::UndeclaredEntity {
                entity,
                subject,
                known,
            } => write!(
                f,
                "ext.world.Ontology.UndeclaredEntity: 实体 `{entity}` 未在出厂本体里声明\
                 （ontology.json 的 concepts 段）——**声明以外的东西不许落账**。\n\
                 \x20 本次写入的 subject 是 `{subject}`；已声明的实体：{known}。\n\
                 \x20 处置：改用已声明的实体，或先在出厂本体里声明它（改本体＝改法律，走评审）"
            ),
            Violation::UndeclaredField {
                entity,
                field,
                subject,
                known,
            } => write!(
                f,
                "ext.world.Ontology.UndeclaredField: 字段 `{field}` 未在实体 `{entity}` 下声明\
                 （ontology.json 的 concepts.`{entity}`.fields）——**声明以外的字段不许落账**。\n\
                 \x20 本次写入的 subject 是 `{subject}`；实体 `{entity}` 已声明的字段：{known}。\n\
                 \x20 处置：改用已声明的字段，或先在出厂本体里声明它（改本体＝改法律，走评审）"
            ),
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
    /// **信封每个字段的类型声明**（`envelope.fields` 段：名字 → 声明串，如 `"integer  # 词表版本…"`）。
    ///
    /// 为什么单独存：`required` 只说明"这一格必须在"，**不说明它该是什么类型**。
    /// 缺了这一段，"类型不符"就无从判起——实测两处真实违规：`world` 写成字符串 `"1"`、`actor` 写成整数 `123`，
    /// 折叠层**都收下了**（`state` rc=0）。声明串**首词**即类型（`integer`／`string`／`array`／`object`／`enum(a, b)`）。
    env_types: BTreeMap<String, String>,
    families: BTreeMap<String, Family>,
    /// **实体与字段的声明**（`concepts` 段）：世界里有哪些实体、各有哪些字段。
    ///
    /// 这一段的用途（书第五章 §5.3 逐字）：「它管两件事：什么算世界里存在的东西，
    /// 什么算一次说得通的改变。」⇒ 落笔前必须查它（[`Ontology::check_concepts`]）。
    /// 在本次改动之前，这一段在 `src/` 里**零读取**（声明写在文件里，落笔时没有人读它）。
    concepts: BTreeMap<String, BTreeSet<String>>,
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
        // 信封**字段的类型声明**（见 `env_types` 的说明）。缺 `fields` 段 ⇒ 空表 ⇒ 只判"在不在"、不判类型
        // （**不许凭空发明一套类型系统**：本体没声明的，本判据不管）。
        let mut env_types = BTreeMap::new();
        if let Some(fs_obj) = envelope.get("fields").and_then(Value::as_object) {
            for (k, val) in fs_obj {
                if k.starts_with('_') {
                    continue;
                }
                if let Some(decl) = val.as_str() {
                    env_types.insert(k.clone(), decl.to_string());
                }
            }
        }

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

        // `concepts`：世界里有哪些实体、各有哪些字段（**落笔时要查的那一段**）。
        //
        // 口径（三条，都可判真假）：
        // 1. 下划线开头的键不是实体（`_comment` 是说明文字）；
        // 2. 实体的字段表是它 `fields` 对象的**键**；值（`"bool"` / `"enum(a,b)"`）
        //    是**自由文本**的说明，不是机器 schema ⇒ 本模块不解释它，只当字段名用。
        //    理由：现在去解释它，等于替法律发明一套类型系统，而这套系统的规则
        //    在出厂本体里**没有写**；
        // 3. 缺 `concepts` 段 ⇒ 实体表为空 ⇒ **任何带实体段的写入都会被拒**
        //    （默认拒绝：没声明过的世界不该接受任何实体）。这不是"什么都不允许"的
        //    安全默认，而是"法律没写全"——它会当场表现为写入被拒。
        let mut concepts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        if let Some(cs) = v.get("concepts").and_then(Value::as_object) {
            for (entity, def) in cs {
                if entity.starts_with('_') {
                    continue;
                }
                let mut fields = BTreeSet::new();
                if let Some(fs_obj) = def.get("fields").and_then(Value::as_object) {
                    for name in fs_obj.keys() {
                        if !name.starts_with('_') {
                            fields.insert(name.clone());
                        }
                    }
                }
                concepts.insert(entity.clone(), fields);
            }
        }

        // **判据②**：扩展项**不得与核心字段重名**（`REQ-F-030`；书 §2.7）。
        // 放在装载期（而不是写入期）：重名是"法律自相矛盾"，法律不对就该拒绝启动，
        // 而不是等到某一条写入路过时才报——那时读到的是"某条事件非法"，说错了病因。
        let core_fields = core_field_names(&required, &optional);
        check_extension_names(&concepts, &core_fields)?;

        Ok(Ontology {
            world,
            required,
            optional,
            env_types,
            families,
            concepts,
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

    /// **信封已声明的必填格**（`envelope.required`，出厂本体实测 8 项）。
    ///
    /// 用途：读模型侧的**缺格判据**（`REQ-F-032`）要按"本体已声明的格"来判，而读模型
    /// **不许** `use crate::ontology::…`——`WC-MODREG-001` §2 给 `M03` 的依赖列逐字是
    /// 「**无**（生产代码零出边）」，机核层 `tools/module_graph.py` 判据② 逐边核对
    /// 「声明集 ≡ 真实 import 集」⇒ 读模型加一条生产边就红。故本方法只交**纯数据**
    /// （`Vec<String>`）出去，由**装配处**递给读模型：
    /// `world_core::readmodel::DeclaredCells::new(ont.envelope_required(), ont.family_required())`；
    /// 依赖方向留在装配处（`M04` 同时依赖 `M01` 与 `M03`），法律与读法仍读**同一份**本体。
    pub fn envelope_required(&self) -> Vec<String> {
        self.required.clone()
    }

    /// **各家族已声明的必填格**（`families.<家族>.required`；出厂本体实测 `change` 4／`act` 3／`notice` 2）。
    ///
    /// 与 [`Ontology::envelope_required`] 同一用途（读模型侧的缺格判据）；同样只交**纯数据**
    /// （理由逐字同上：不让读模型多出一条生产依赖边）。
    /// ⚠️ 这里**不含**各家族的 `optional`：可选格"没写"是法律允许的形态，不是缺格。
    pub fn family_required(&self) -> BTreeMap<String, Vec<String>> {
        self.families
            .iter()
            .map(|(k, f)| (k.clone(), f.required.clone()))
            .collect()
    }

    pub fn known_kinds(&self) -> Vec<&str> {
        self.families.keys().map(String::as_str).collect()
    }

    /// 已声明的实体名（有序）。
    pub fn known_entities(&self) -> Vec<&str> {
        self.concepts.keys().map(String::as_str).collect()
    }

    /// 某实体已声明的字段集（`None` = 该实体**没声明过**）。
    pub fn declared_fields(&self, entity: &str) -> Option<&BTreeSet<String>> {
        self.concepts.get(entity)
    }

    /// **读一条事件的旗标**：认得的照常处理，**不认得的一律忽略**（`REQ-F-029`）。
    ///
    /// 依据（逐字）：`world-core/ontology.json:20`
    /// `"flags": "array  # 能力旗标；未知旗标必须忽略"`。
    ///
    /// 为什么把它挂在**本体**上：这条纪律是**法律**的一部分（写在出厂本体的信封字段表里），
    /// 所以"读的人按哪一半继续"这件事的入口在本体侧，与 `validate` 同源。
    ///
    /// ⚠️ 本方法**没有 `Result`**：出现不认得的旗标**不是**一种校验失败——
    /// 它是"读的人按他认得的那部分继续"。哪些算"认得"由调用方给：
    /// 出厂读法用 [`crate::event::is_factory_flag`]，认得摩擦旗标的读法另给它自己的判据。
    pub fn read_flags<'a, F>(&self, ev: &'a Value, knows: F) -> crate::event::Flags<'a>
    where
        F: Fn(&str) -> bool,
    {
        crate::event::read_flags(ev, knows)
    }

    /// 从 `subject` 取出**实体类型**：`world://<实体>/<实例…>` ⇒ `Some("<实体>")`。
    ///
    /// 裸主体（`world://<名字>`，没有实例段）⇒ `None`：它**不是**某个实体的实例引用。
    /// ⚠️ 这条口径留了一个**已登记的缺口**（见 [`Ontology::check_concepts`] 的文档）。
    pub fn entity_of(subject: &str) -> Option<&str> {
        let rest = subject.strip_prefix("world://")?;
        let (entity, id) = rest.split_once('/')?;
        if entity.is_empty() || id.is_empty() {
            None
        } else {
            Some(entity)
        }
    }

    /// **声明以外的东西不许落账**（书第五章 §5.3）：按 `concepts` 校验实体与字段。
    ///
    /// ## 查什么
    ///
    /// | 情形 | 结论 |
    /// |---|---|
    /// | `world://<未声明的实体>/<实例>` | **拒**，错误点名那个实体（`UndeclaredEntity`） |
    /// | `world://<已声明的实体>/<实例>` + 未声明的 `path` | **拒**，错误点名那个字段（`UndeclaredField`） |
    /// | `world://<已声明的实体>/<实例>` + 已声明的 `path` | 放行 |
    ///
    /// ## 只查 `change`
    ///
    /// 三家族里只有 `change` 改状态（读模型里就是 `objects[subject][path] = after`）——
    /// 所以"什么是世界里的东西"只在它这里判。`act` / `notice` 的信纸由家族必填项管；
    /// 且 `notice.subject` 的语义是"这条通告关于谁"，不是"改了哪一格"，拿字段表去查它
    /// 是查错了对象。
    ///
    /// ## 一处**已登记的缺口**（不假装已闭合）
    ///
    /// **裸主体**（`world://<名字>`，没有实例段）**不受**本段约束——`world://s` 这类槽位
    /// 今天照样能落账。留下它的原因不是口径，而是**代价**：既有出厂用例
    /// （`tests/cli.rs:100/230`、`tests/contract.rs:257` 等多处、`tests/acceptance.rs:296`）
    /// 都以这种形态写槽位，一律拒绝会把它们打红，而那些用例不许改。
    /// ⇒ 书 §5.3 要的"世界的边界由声明定"在**实体引用**这一半成立，
    /// 在**裸主体**那一半**仍未成立**（已在 `tests/atom_declared_only.rs` 里立成登记项）。
    ///
    /// ⚠️ 另一半（`world://check/probe`、`world://sys/a` 这类**带实例段**的未声明实体）
    /// 已经被拒。代价是出厂门禁脚本 `world-core/check.sh` 第 89 行与 `tools/` 下的探针
    /// 会非零退出：它们写的是"未声明的实体"，而本段正是要拦这个。
    /// 那些文件不属本次改动范围，已在交付说明里逐条列出（含建议的最小改法）。
    pub fn check_concepts(&self, kind: &str, body: &Value) -> Result<(), Violation> {
        if kind != "change" {
            return Ok(());
        }
        // 必填项缺失由家族必填检查负责报（这里不抢它的错误码）。
        let Some(subject) = body.get("subject").and_then(Value::as_str) else {
            return Ok(());
        };
        let Some(entity) = Self::entity_of(subject) else {
            return Ok(()); // 裸主体：见上文「已登记的缺口」
        };
        let known = || self.known_entities().join(", ");
        let Some(fields) = self.concepts.get(entity) else {
            return Err(Violation::UndeclaredEntity {
                entity: entity.to_string(),
                subject: subject.to_string(),
                known: known(),
            });
        };
        let Some(path) = body.get("path").and_then(Value::as_str) else {
            return Ok(());
        };
        if !fields.contains(path) {
            let declared = fields.iter().cloned().collect::<Vec<_>>().join(", ");
            return Err(Violation::UndeclaredField {
                entity: entity.to_string(),
                field: path.to_string(),
                subject: subject.to_string(),
                known: declared,
            });
        }
        Ok(())
    }

    /// 校验一条事件是否合法：信封必填 → 版本 → 家族存在 → 信纸必填 → **声明以内**。
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

        // **在不在／版本／家族／信纸必填**都问过了，最后问**类型对不对**（`TC-047 ⑧` 的修法）。
        // 为什么排在既有各道**之后**（实现时先排在前面，被既有断言当场纠正）：
        // `c02` 逐字把 `body` 设成字符串并期望 `MissingField`、`world` 写成字符串则由版本那道报 `BadVersion` ——
        // 那两条都是**既有契约**。本判据**只接手没人管的那一处**（如 `actor`＝整数），**不抢既有码**。
        self.validate_types(ev)?;

        // **最后一道**：声明以外的东西不许落账（书 §5.3）。
        // 放在形状检查之后：形状不对时先报形状（那是更基本的问题），
        // 形状对了再问"这个名字世界里有没有"——两道错的报错顺序不该靠偶然。
        self.check_concepts(kind, obj.get("body").unwrap_or(&Value::Null))?;

        Ok(())
    }

    /// **只判类型**：本体声明过类型的信封字段，值必须符合声明。
    ///
    /// 与 [`Ontology::validate`] 分开的**理由（实测逼出来的）**：读路径此前跑的是整套 `validate`，
    /// 于是"**缺 `actor`**"这类事件被报成 `ext.world.Ontology.MissingField`（**写侧那条**），
    /// 而读侧的契约是 [`readmodel`] 的 `ext.world.ReadModel.MissingCell`（**它自己那条**，有断言在核）。
    /// ⇒ **补缺口（类型）不动别人的契约**：读路径只调本方法；写路径照旧跑整套 `validate`。
    ///
    /// 范围：只判 `self.env_types` 里**声明过**且**值在场**的字段；
    /// `enum(...)` 不进本判据（值不在枚举里由家族查找报 `UnknownKind` 并点名那个值——那是现成契约）。
    pub fn validate_types(&self, ev: &Value) -> Result<(), Violation> {
        let obj = match ev.as_object() {
            Some(o) => o,
            None => return Ok(()), // 不是对象：交给 validate 的 NotAnObject，本方法不越界
        };
        for (f, decl) in &self.env_types {
            if declared_type_word(decl).starts_with("enum(") {
                continue;
            }
            if let Some(v) = obj.get(f) {
                if !type_ok(decl, v) {
                    return Err(Violation::BadFieldType {
                        at: "envelope".to_string(),
                        field: f.clone(),
                        want: declared_type_word(decl).to_string(),
                        got: json_type_name(v).to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// **声明串的类型词**：本体里写的是 `"integer  # 词表版本；只加 flags…"` 这种形态 ⇒ 取**首词**。
///
/// `enum(change, act, notice)` 整体算一个词（首个空白之前即它）。
fn declared_type_word(decl: &str) -> &str {
    let s = decl.trim_start();
    // `enum(a, b, c)` **里面有空白** ⇒ 不能按空白取首词（那样会截成 `enum(a,`）。
    // 取到**配对的那个 `)`** 为止；没有 `)` 就退回"取首词"。
    if let Some(rest) = s.strip_prefix("enum(") {
        if let Some(i) = rest.find(')') {
            return &s[.."enum(".len() + i + 1];
        }
    }
    s.split(|c: char| c.is_whitespace()).next().unwrap_or("")
}

/// **JSON 值的类型名**（报错用；与本体声明里的词同一套写法）。
fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                "integer"
            } else {
                "number"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// **这一格的值符不符合本体声明的类型**。
///
/// 认得的类型词：`integer`／`string`／`array`／`object`／`bool`／`number`／`enum(a, b, …)`。
/// **不认得的词一律放行**（`_ => true`）——本体没声明的类型口径，本判据不替它发明。
fn type_ok(decl: &str, v: &Value) -> bool {
    match declared_type_word(decl) {
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "string" => v.is_string(),
        "array" => v.is_array(),
        "object" => v.is_object(),
        "bool" => v.is_boolean(),
        w if w.starts_with("enum(") => {
            let inner = w.trim_start_matches("enum(").trim_end_matches(')');
            match v.as_str() {
                Some(s) => inner.split(',').any(|x| x.trim() == s),
                None => false,
            }
        }
        _ => true,
    }
}

/// **核心字段名** = `envelope.required` ∪ `envelope.optional`（出厂本体逐字给出的 10 项）。
///
/// ## 为什么**只**算信封字段、不算家族信纸字段（这一条是刻意的）
///
/// 家族（`families`）**本身是可扩展的**：一份"只加扩展"的本体会新增一个家族，
/// 并在同一个名下新增一个概念——出厂门禁脚本 `tools/s1_sys_probe.sh` 的
/// `ontology-ext.json` 就是"新增家族 `audit` ＋ 新增概念 `audit.result`"。
/// 若把"全部家族的必填／可选字段"也算进核心，这份**纯加法**的本体会被判成"重名"而拒启，
/// 而 `REQ-F-030` 判据③ 恰恰要求纯加法**必须照常可读**（换一份只加扩展的本体 ⇒ 折叠结果不变）。
/// ⇒ 核心 = **信封**字段：它由本体逐字列出，不随扩展变动。
fn core_field_names(required: &[String], optional: &[String]) -> BTreeSet<String> {
    required.iter().chain(optional.iter()).cloned().collect()
}

/// **判据②**：扩展项（`concepts` 的实体名与字段名）**SHALL NOT 与核心字段重名**。
///
/// 依据（逐字）：书 §2.7「核心之外由命名空间扩展，各方在自己的空间里定义自己的概念；
/// 核心之内不取交集，也不做删减」；本 change 的 delta `REQ-F-030`：
/// 「扩展项 SHALL 落在命名空间内，**SHALL NOT 与核心字段重名**」。
///
/// 为什么必须**拒启**、而不是打个警告继续跑：重名的扩展项与核心字段**共用一个名字**，
/// 而"核心字段的含义永不因扩展而变"是这一结构的全部价值 ⇒ 一旦叠上，
/// 读的人再也分不清 `body` 指的是哪一个。**改本体＝改法律；法律自相矛盾时不许带病运行**
/// （与 `load` 的其余错误同一处置）。
///
/// ⚠️ 只查 `concepts`（实体名与字段名）：家族的信纸字段落在 `body` **里面**，
/// 与信封字段不在同一层，同名不构成"改掉核心那一格的含义"。
fn check_extension_names(
    concepts: &BTreeMap<String, BTreeSet<String>>,
    core: &BTreeSet<String>,
) -> Result<(), String> {
    for (entity, fields) in concepts {
        if core.contains(entity) {
            return Err(core_collision(
                &format!("`concepts` 的实体名 `{entity}`"),
                entity,
                core,
            ));
        }
        for f in fields {
            if core.contains(f) {
                return Err(core_collision(
                    &format!("实体 `{entity}` 的字段名 `{f}`"),
                    f,
                    core,
                ));
            }
        }
    }
    Ok(())
}

/// 重名的拒绝理由：**点名**撞上的那一项，并列出全部核心字段（否则只说了"不行"、没说"怎么办"）。
fn core_collision(at: &str, name: &str, core: &BTreeSet<String>) -> String {
    let list = core.iter().cloned().collect::<Vec<_>>().join(", ");
    format!(
        "ext.world.Ontology.CoreCollision: 扩展项与核心字段重名：{at} 与信封字段 `{name}` 同名——\
         本体采用**极小核心 ＋ 命名空间扩展**（书 §2.7 逐字「核心之外由命名空间扩展，\
         各方在自己的空间里定义自己的概念；核心之内不取交集，也不做删减」），\
         扩展项 SHALL NOT 与核心字段重名。\n\
         \x20 核心字段（`envelope.required` ∪ `envelope.optional`，共 {n} 项）：{list}\n\
         \x20 处置：把扩展项换到自己的名字上（出厂本体已声明的两格是 `notice.muted` 与 `job.status`）",
        n = core.len()
    )
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
