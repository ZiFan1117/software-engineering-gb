//! 世界核心（World Core）—— **语义事件是唯一真相**。
//!
//! 本 crate 实现 `07/4-计划/04-模块清单与实现顺序.md` 的模块：
//! - [`ontology`]  —— **法律**：一条事件长什么样、什么算合法变更
//! - [`ledger`]    —— **事实**：只追加的语义事件账本（纯文本 JSON Lines）
//! - [`readmodel`] —— **状态**：`state = fold(events[0..seq])`，**派生物、可随时重算**
//! - [`gate`]      —— **门禁**：一件事现在能不能做（默认拒绝 + 按不可逆性加摩擦）
//! - [`guard`]     —— **静态防线**：法律与真相的存放处不得对被管者开放写权限
//!
//! 并且提供 [`World`] 作为**唯一的写入口**（取号 → 造事件 → 法律校验 → 门禁裁决 → 落笔）。
//!
//! 五条贯穿纪律（`07/4-计划/03` §七）：
//! 1. **一个进程**（本体/账本/读模型/运行时/门禁同进程）；
//! 2. **一个写入口**（只有 [`World::commit`] 能改世界）；
//! 3. **法律在前、落笔在后**（校验不过就绝不写账本）；
//! 4. **状态是算出来的**（不保存状态 ⇒ 不存在"状态与账本不一致"）；
//! 5. **门禁不可绕过**（决策在唯一咽喉 + 规则与真相不受被管者写入）。

pub mod carrier;
pub mod channel;
pub mod checkpoint;
pub mod delivery;
pub mod error;
pub mod event;
pub mod gate;
pub mod guard;
pub mod ledger;
pub mod ontology;
pub mod pairing;
pub mod project;
pub mod readmodel;

use gate::{Decision, Policy};
use ledger::Ledger;
use ontology::Ontology;
use readmodel::State;
use serde_json::Value;
use std::path::Path;

/// 一个最小可用的世界：**法律（本体 + 门禁策略）+ 事实（账本）**。
///
/// ⚠️ 字段**全部私有**（2026-09-26 收窄，见 `WC-RV-R2-001` **S-02 / T-02**）。
/// 原先三个字段都是 `pub`，于是任何拿到 `&mut World` 的调用方都能
/// `w.ledger.append(任意 JSON)`、甚至 `w.policy = 自备的宽松策略`——
/// "唯一写入口"当时**只是注释里的散文**，不是代码性质。
/// 现在外部只能拿到 `&` 访问器；写入只能经 [`World::commit`]。
///
/// 这不是注释里的承诺，而是**编译期事实**——下面这段代码必须**编译失败**
/// （`WC-RV-R2-001` T-02 / `DEBT-08`：此前只有散文声明，没有任何机械证据）：
///
/// ```compile_fail
/// use world_core::World;
/// fn smuggle(w: &mut World) {
///     // `ledger` 字段私有：外部拿不到可变引用，因此无法绕过 `commit`
///     let _ = &mut w.ledger;
/// }
/// ```
///
/// 另一条同类证据：`Ledger::append` 是 `pub(crate)`（外部不可见）。
#[derive(Debug)]
pub struct World {
    ontology: Ontology,
    policy: Policy,
    ledger: Ledger,
}

impl World {
    /// 打开世界：加载本体（法律之形状）→ 加载门禁策略（法律之权限）→ 打开账本（事实）。
    ///
    /// 任一步失败即**拒绝启动**（`07/4-计划/03` §五：法律不对，带病跑比不跑更危险）。
    pub fn open(
        ontology_path: &Path,
        ledger_path: &Path,
        policy_path: &Path,
    ) -> Result<Self, String> {
        Self::open_mode(ontology_path, ledger_path, policy_path, false)
    }

    /// **只读**打开世界（`P-01` 修）：账本以只读口径打开 ⇒ **不取写锁、不截断半行、
    /// 不改既有字节**。只读命令（`read` / `state` / `project` / `check`）一律走这里。
    ///
    /// 为什么重要：`REQ-F-012` 与 IC 九册不变量① 都写"只读接口不得写 L1"，
    /// 而此前只读命令会在含半行的账本上执行一次 `set_len`（实测字节从 480 变 442）。
    pub fn open_readonly(
        ontology_path: &Path,
        ledger_path: &Path,
        policy_path: &Path,
    ) -> Result<Self, String> {
        Self::open_mode(ontology_path, ledger_path, policy_path, true)
    }

    fn open_mode(
        ontology_path: &Path,
        ledger_path: &Path,
        policy_path: &Path,
        readonly: bool,
    ) -> Result<Self, String> {
        let ontology = Ontology::load(ontology_path)?;
        let policy = Policy::load(policy_path)?;
        let mut ledger = if readonly {
            Ledger::open_readonly(ledger_path)?
        } else {
            Ledger::open(ledger_path)?
        };
        // 摘要链：有链则核验（篡改即拒绝启动），无链则按 v1 放行（WC-CR-003 D2）。
        // ⚠️ 无链时"局部篡改不可检出"——调用方应打印警告（`check` 会打印链状态）。
        ledger.load_chain()?;

        // 词表版本一致性（2026-09-26 补，见 WC-RV-R2-001 S-17）：
        // 信封里的 `world` 由 `event::WORLD_VERSION` 写死，而本体自带 `world` 字段。
        // 两者不一致时，构造出的事件**恒被判 BadVersion**——那是自伤性故障：
        // 世界能启动、却一条事件都写不进去，而报错看起来像"事件格式错"。
        // 故在启动时刻就拒绝，把问题暴露在这里而不是每条写入上。
        if event::WORLD_VERSION != ontology.world() {
            return Err(format!(
                "ext.world.World.VersionMismatch: 事件构造器版本 {} 与本体声明的 world={} 不一致；\
                 拒绝启动（否则每条事件都会被判 BadVersion）",
                event::WORLD_VERSION,
                ontology.world()
            ));
        }

        Ok(World {
            ontology,
            policy,
            ledger,
        })
    }

    /// 只读访问本体（法律·形状）。
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// 只读访问门禁策略（法律·权限）。
    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// 只读访问账本（事实）。**拿不到可变引用**，故无法绕过 [`World::commit`]。
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// **唯一的写入口**。顺序固定、不可交换：
    ///
    /// 1. 取号 → 造事件；
    /// 2. **法律在前**：本体校验（形状不对，连门禁都不必问）；
    /// 3. **门禁裁决**：`act` 与 `change` **都必须过闸**；拒绝/加摩擦时
    ///    **不写原事件**，改记一条 `notice`（`gate.*`）——拦得住，也记得下；
    /// 4. **落笔在后**：只有前三步都通过才追加到账本。
    ///
    /// ## 为什么 `change` 也必须过闸（2026-09-26 修，见 `WC-RV-R2-001` S-01）
    ///
    /// 原实现只裁决 `act`，理由是"`change` 只是记录"。这个理由**站不住**：
    /// `change` 恰恰是**唯一真正改状态的写原语**（读模型里就是
    /// `objects[subject][path] = after`）。于是"被拒绝的动作"完全可以用一条
    /// `change` 静默达成——`actor` 与 `subject` 之间约束为零。
    /// **"门禁不可绕过"当时是假的**，现在由 [`crate::gate::Policy::authorize_write`] 补上。
    ///
    /// `notice` 仍不过闸：它是**通告**、不改状态，且门禁自己的流水就是 `notice`。
    pub fn commit(&mut self, kind: &str, actor: &str, body: Value) -> Result<Value, String> {
        self.commit_verbatim(kind, actor, body, None, None)
    }

    /// 同 [`World::commit`]，但**显式给出因果与目的地**（可选信封字段）。
    ///
    /// 为什么要这个入口（`M10` 接线，2026-09-27）：跨进程的"请求—结果"必须能配对，
    /// 而配对的依据只有两个——**同一个 `request_id`**（业务层）与 **`trace` 指回意图事件**
    /// （世界层）。此前 `trace` 在源码里**零写入路径**，于是"为什么会变成这样"
    /// 在账本里没有落点（`REQ-F-031` 记为部分实现）。
    ///
    /// 口径：
    /// - `trace`/`to` 为 `None` 或空串 ⇒ **不写该键**（不写 `null`）：没有因果与因果指向空是两件事；
    /// - `trace` **不做引用完整性校验**（`REQ-F-031` 的 v1 口径）：指向不存在的 `id` 不导致拒绝；
    /// - 除此之外与 [`World::commit`] **完全同一条路径**（取号 → 造事件 → 法律 → 门禁 → 落笔），
    ///   不存在"带因果就绕过门禁"的第二条路。
    pub fn commit_requested(
        &mut self,
        kind: &str,
        actor: &str,
        body: Value,
        trace: Option<&str>,
        to: Option<&str>,
    ) -> Result<Value, String> {
        self.commit_verbatim(kind, actor, body, trace, to)
    }

    fn commit_verbatim(
        &mut self,
        kind: &str,
        actor: &str,
        body: Value,
        trace: Option<&str>,
        to: Option<&str>,
    ) -> Result<Value, String> {
        let seq = self.ledger.next_seq();
        let mut ev = event::new_event(seq, kind, actor, body);
        event::with_trace(&mut ev, trace);
        event::with_to(&mut ev, to);

        // 校验与裁决的顺序（2026-09-27 订正为两段式）：
        //   ① 先校验：形状不合 ⇒ 直接拒，**不写任何东西**（连流水都不写，因为流水同样要过校验）；
        //   ② 校验过了再问门禁：门禁拒绝 ⇒ **写一条 `notice` 流水**，然后拒。
        //
        // 为什么必须两段：门禁模块的文档明写"拒绝也要留痕"，而原实现用 `?` 早退，
        // 使"该不该留痕"取决于"先校验还是先裁决"的偶然顺序。两段式把这条口径变成**结构**。
        if let Err(e) = self.ontology.validate(&ev) {
            return Err(e.to_string());
        }
        let act_body = ev.get("body").cloned().unwrap_or(Value::Null);
        let friction = self.adjudicate(kind, actor, &act_body)?;

        // **摩擦落账**（书 §5.5：摩擦挂在动作的不可逆等级上）。
        //
        // 加摩擦不能只是裁决内存里的一个字段——它必须留下**可核的痕迹**，
        // 否则"加过摩擦"在账本里查不到，审计就无从谈起。落点选信封的 `flags`：
        // 本体纪律要求"未知旗标必须忽略" ⇒ 旧读法读到它不会坏，新读法能核出它。
        //
        // ⚠️ 加旗标发生在法律校验**之后**，所以加完必须**再过一遍法律**——
        // "法律在前、落笔在后"不许因为"我在法律之后又改了事件"而破。
        if let Some(f) = &friction {
            event::with_flag(&mut ev, &f.flag());
            if let Err(e) = self.ontology.validate(&ev) {
                return Err(e.to_string());
            }
        }

        self.ledger.append(ev)
    }

    /// 门禁裁决（`act` 与 `change` **都必须过闸**），拒绝时**留痕并返回点名错误码**。
    ///
    /// 返回 `Ok(Some(摩擦))` = 这次动作**不可逆**：放行，但摩擦必须随事件落账
    /// （由 [`World::commit_verbatim`] 落到 `flags` 上）。
    ///
    /// 抽成独立函数是因为两条拒绝路径必须说同样的话、留同样的痕——
    /// 复制两份迟早漂移成两种口径。
    fn adjudicate(
        &mut self,
        kind: &str,
        actor: &str,
        body: &Value,
    ) -> Result<Option<gate::Friction>, String> {
        match kind {
            "act" => {
                let verdict = self.policy.verdict(actor, body);
                match verdict.decision {
                    // 准了：把摩擦交回给落笔那一步（不可逆 ⇒ `Some`，可逆 ⇒ `None`）。
                    Decision::Allow => Ok(verdict.friction),
                    Decision::Reject(reason) => Err(self.gate_refusal(
                        "ext.world.Gate.Rejected",
                        "gate.rejected",
                        "门禁拒绝",
                        actor,
                        body,
                        &reason,
                    )),
                    Decision::AwaitApproval(reason) => Err(self.gate_refusal(
                        "ext.world.Gate.AwaitingApproval",
                        "gate.awaiting-approval",
                        "门禁加摩擦",
                        actor,
                        body,
                        &reason,
                    )),
                }
            }
            "change" => {
                let subject = body
                    .get("subject")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                // `authorize_write` 现只返回 Allow/Reject（不可逆分级只作用于 `act`）；
                // 万一将来它返回"加摩擦"，按**默认拒绝**处理，而不是默默放行。
                match self.policy.authorize_write(actor, &subject) {
                    Decision::Allow => Ok(None),
                    Decision::Reject(reason) | Decision::AwaitApproval(reason) => Err(self
                        .gate_refusal(
                            "ext.world.Gate.WriteRejected",
                            "gate.write-rejected",
                            "门禁拒绝写入",
                            actor,
                            body,
                            &reason,
                        )),
                }
            }
            "notice" => {
                self.adjudicate_notice(actor, body)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// **通告的闸**（2026-09-27 补，`WC-THEORY-DEFECT-001` **D-13**）。
    ///
    /// ## 为什么通告也必须过闸（这条以前是缺的，而且缺得很危险）
    ///
    /// 此前 `notice` 落在 `adjudicate` 的 `_ => Ok(())` 分支里——**任何主体可以写任何通告**。
    /// 后果不是"多了几条噪声"，而是**整套审计可被一行伪造**：
    ///
    /// ```text
    /// world-core append notice \
    ///   '{"type":"gate.rejected","subject":"world://user","payload":{"reason":"伪造"}}' \
    ///   world://stranger
    /// ⇒ rc=0，账本里出现一条与内核真流水**逐字同形**的 notice
    /// ```
    ///
    /// 也就是说："世界拒绝过这件事"这句话，**任何人都能替世界说**。
    /// 审计的价值全在"这句话不是你说的，是世界说的"——这条一破，审计等于没有。
    ///
    /// ## 修法（两条纪律，各自可单独判真假）
    ///
    /// | # | 纪律 | 反例（必须被拒） |
    /// |---|---|---|
    /// | **① 保留前缀** | `gate.` 开头的通告类型**只许内核自己写**（内核的拒绝/加摩擦流水），外部提交一律拒 | 外部主体提交 `type=gate.rejected` |
    /// | **② 主体须在册** | 非保留前缀的通告，其 `actor` **必须已在主体白名单内** | 不在册的主体提交任意通告 |
    ///
    /// ## 为什么"内核的流水"不受 ① 之限
    ///
    /// 内核写 `gate.*` 流水**不是为了记录"某主体做了什么"**，而是为了记录
    /// **"世界对某次请求的裁决"**——那件事发生的位置就是内核自己。
    /// 而"内核对谁裁决过"已经由 `decide`/`authorize_write` 的**白名单与授权表**限定了；
    /// 被拒的主体虽然写不了别的，但**它的这次被拒必须留痕**（否则丢掉了审计信息）。
    /// 两件事不冲突：**外部不能冒充世界，世界必须记得拒绝过谁。**
    ///
    /// ## 一处诚实的不对称（登记，不假装已解决）
    ///
    /// 非保留前缀的通告**不检查 `writes` 授权表**，只检查主体白名单。理由：现实现里
    /// `writes` 的默认形状（`world://agent/* → []`）会让**所有** agent 通告被拒，
    /// 而那会顺手打断载体侧已有的状态通告路径。**本次只堵"可伪造审计"这个洞**，
    /// 通告要不要按主体收窄授权**属门禁强度变更，须人裁**。
    fn adjudicate_notice(&mut self, actor: &str, body: &Value) -> Result<(), String> {
        /// 保留前缀：只许内核自己写。
        const RESERVED_PREFIX: &str = "gate.";

        let notice_type = body.get("type").and_then(Value::as_str).unwrap_or("");

        if notice_type.starts_with(RESERVED_PREFIX) {
            return Err(self.gate_refusal(
                "ext.world.Gate.NoticeNotAllowed",
                "gate.notice-not-allowed",
                "门禁拒绝通告",
                actor,
                body,
                &format!(
                    "通告类型 `{notice_type}` 是**内核保留前缀**（`{RESERVED_PREFIX}`）——\
                     它只由内核自己写（记录'世界对某次请求的裁决'）。\n\
                     \x20 为什么必须保留：否则任何人都能替世界说\"我拒绝过这件事\"，\
                     而账本里的伪造流水与真流水**逐字同形、事后不可区分** ⇒ 审计等于没有。\n\
                     \x20 处置：换成你自己的通告类型（不要以 `{RESERVED_PREFIX}` 开头）"
                ),
            ));
        }

        if !self.policy.subject_allowed(actor) {
            return Err(self.gate_refusal(
                "ext.world.Gate.NoticeRejected",
                "gate.notice-rejected",
                "门禁拒绝通告",
                actor,
                body,
                &format!(
                    "主体 `{actor}` 不在门禁白名单内（policy.json 的 subjects.allow）——\
                     通告同样要过闸：世界允许谁说话，就得先在册"
                ),
            ));
        }
        Ok(())
    }

    /// 门禁拒绝的统一出口：**写流水 + 返回点名错误码**。
    ///
    /// 流水写失败**不得吞掉拒绝理由**：调用方收到的是"为什么被拒"，
    /// 流水失败作为附注（`WC-RV-R2-001` S-10）。
    ///
    /// ## 流水必须点名"它在拒绝什么"（2026-09-27 补，`D-14`）
    ///
    /// `gate.*` 前缀是**内核保留**的（见 [`World::adjudicate_notice`]），但保留只解决了
    /// "谁能写"，没解决"**写的是什么**"。实测发现的歧义：一次**外部提交**被拒时，
    /// 内核留的流水与内核**自己发起**的裁决流水**同样是 `gate.*` 类型**，
    /// 于是读账本的人分不清：
    ///
    /// - 这条 `gate.notice-rejected` 是"**内核拒绝了某个主体的通告**"（内核发起的裁决），还是
    /// - "某个主体的请求**被拒了**，这是它的流水"（对外部尝试的记录）？
    ///
    /// 修法：流水里带上**它在拒绝什么**——`refused` 字段，内容是"被拒对象的规范形式"的指纹与摘要：
    ///
    /// - 外部尝试被拒 ⇒ `refused` 指向**那条没落笔的事件**（kind / actor / body 的指纹）；
    /// - 内核自身发起的裁决（如某主体的通告被拒）⇒ `refused` 说明"无被拒事件"，
    ///   因为**没有一条外部事件被拒**，被拒的是内部那条流水本身。
    ///
    /// 这样：**流水与它记录的那次尝试之间有可复核的对应关系**，而不是"一堆同样类型的行"。
    /// ⚠️ 登记一处局限（不假装已解决）：本字段**不含被拒事件的全部内容**（只含指纹），
    /// 因此"拒绝一条超大载荷的请求"时，被拒载荷本身**不入账本**——这是刻意的
    /// （否则任何主体都能靠"提交垃圾再被拒"无限撑大账本）。
    fn gate_refusal(
        &mut self,
        code: &str,
        notice_type: &str,
        headline: &str,
        actor: &str,
        body: &Value,
        reason: &str,
    ) -> String {
        match self.record_gate_notice(notice_type, actor, body, reason) {
            Ok(()) => format!("{code}: {headline}：{reason}"),
            Err(notify_err) => format!(
                "{code}: {headline}：{reason}（⚠️ 且流水写入失败：{notify_err} —— \
                 本次拒绝**未能留痕**，调用方务必自行记录）"
            ),
        }
    }

    /// 记一条门禁流水。
    ///
    /// 走的是与 `commit` 同一套"校验 → 落笔"路径（**不绕过法律**），
    /// 只是主体取"被拦下的那个 actor"、类型是 `notice`。
    ///
    /// **流水里必须能看出"它在拒绝什么"**（`D-14`，见 [`World::gate_refusal`] 的文档）：
    /// `refused` 字段给出"被拒对象"的**规范形式指纹**，于是"内核发起的裁决"与
    /// "对外部尝试的记录"在账本里**可区分、可复核**，而不是一堆同样类型的行。
    fn record_gate_notice(
        &mut self,
        notice_type: &str,
        actor: &str,
        act_body: &Value,
        reason: &str,
    ) -> Result<(), String> {
        let seq = self.ledger.next_seq();
        // `refused` = 被拒对象的规范形式指纹。
        //
        // 口径：**指纹只覆盖"这次尝试是什么"**（kind / actor / 信纸），
        // 不覆盖载荷的全文——理由见 `gate_refusal` 的文档（防"提交垃圾再被拒"撑大账本）。
        let refused = Self::refused_digest(actor, act_body);
        let payload = serde_json::json!({
            "capability": act_body.get("capability").cloned().unwrap_or(Value::Null),
            "verb": act_body.get("verb").cloned().unwrap_or(Value::Null),
            "request_id": act_body.get("request_id").cloned().unwrap_or(Value::Null),
            // 被拒对象是谁（`change` 是它要写的那个主体，`act` 是发起者自己）。
            //
            // ⚠️ 字段名刻意**不叫 `subject`**（2026-09-27 改）：`notice` 家族的**信纸本身**
            // 就有一个必填的 `subject`（"这条通告是关于谁的"）。若流水里再放一个同名的
            // `subject`，就会出现"两个 subject、指两个不同的东西"的歧义——实测它把一条
            // 既有断言弄红了（脚本按字符串找被拒主体，结果捞到了内核的流水）。
            // 命名口径：**信纸字段名归本体；payload 内不得复用信纸字段名。**
            "refused_subject": act_body.get("subject").cloned().unwrap_or(Value::Null),
            "refused": refused,
            "reason": reason,
        });
        let ev = event::new_event(
            seq,
            "notice",
            actor,
            event::notice_body(notice_type, actor, payload),
        );
        self.ontology.validate(&ev).map_err(|e| e.to_string())?;
        self.ledger.append(ev)?;
        Ok(())
    }

    /// "被拒对象"的**规范形式指纹**（`D-14`）。
    ///
    /// 为什么需要它：`gate.*` 前缀保留只解决了"**谁能写**"，没解决"**写的是什么**"。
    /// 一次外部提交被拒时，内核留的流水与内核自己发起的裁决流水**类型相同**——
    /// 读账本的人分不清"这是内核在裁别人"还是"这是别人的尝试被裁了"。
    ///
    /// 口径（三条，都可判真假）：
    /// 1. **覆盖"这次尝试是什么"**：发起者 + 信纸的规范化 JSON（键有序）；
    /// 2. **不覆盖载荷全文**：否则任何主体都能靠"提交垃圾再被拒"无限撑大账本；
    /// 3. **可复算**：给定同一次尝试，任何人事后都能算出同一个值。
    fn refused_digest(actor: &str, body: &Value) -> String {
        // 规范化：`serde_json` 的 `Value::to_string` 对 `Map` 按键有序输出
        //（本项目全程依赖这一性质保证"同输入同字节"，见读模型与投影的确定性口径）。
        let canon = format!("{actor}\u{1}{body}");
        format!("fnv1a64:{:016x}", Self::fnv1a64(canon.as_bytes()))
    }

    /// FNV-1a 64 位（**非加密**指纹；全项目同一实现口径）。
    fn fnv1a64(bytes: &[u8]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        h
    }

    /// **从账本重算状态**（读模型入口）。
    ///
    /// 这个方法**不缓存、不写盘、不记住上次结果**——每次调用都是"把账本从头折叠一遍"。
    /// 慢是它的缺点，也是它的全部价值：**它不可能与账本不一致**。
    /// 将来若加快照（`M08`），必须先证明"从快照续算"与"从这里重算"结果相同。
    pub fn read_model(&self) -> Result<State, String> {
        State::fold(&self.ledger.read_all()?)
    }
}

/// `M09`（通道）对内核提出的窄接口：**纯转发**到 [`World::commit_requested`]。
///
/// 为什么要在这里写这个 `impl`（而不是让 `src/channel.rs` 直接 `use crate::World`）：
/// `WC-ATOM-001` §二 A-4 要求模块号依赖**单向 DAG**，而通道与运行时互相 `use` 会成环。
/// 依赖的真实方向只有一个——**运行时驱动通道**（`src/main.rs:622`）；通道需要的是
/// "谁能收下这条请求"，不是"世界长什么样"。把这条事实写成窄接口后：
/// `M09 → M04` 这条边**从源码里消失**，而**行为一字未改**（转发，不复制任何逻辑）。
///
/// ⚠️ 这里**不许**出现第二条写路径：转发目标就是 [`World::commit_requested`] 本身，
/// "取号 → 造事件 → 法律 → 门禁 → 落笔"仍然只有那一条（`M04` 的唯一写入口不变）。
impl crate::channel::RequestSink for World {
    fn commit_requested(
        &mut self,
        kind: &str,
        actor: &str,
        body: Value,
        trace: Option<&str>,
        to: Option<&str>,
    ) -> Result<Value, String> {
        World::commit_requested(self, kind, actor, body, trace, to)
    }
}
