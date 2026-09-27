# Spec Delta

## MODIFIED Requirements

### Requirement: 每一个写动作都过门禁

> **改的是哪一类问题**：③ 证据错位（引错了出厂步骤）。
>
> `audit.md` **G6** 后半：`openspec/specs/gate-enforcement/spec.md:19` 写「`check.sh` 步骤 ③」，
> 而 `world-core/check.sh:98` 逐字 `cargo test --locked --test acceptance -- t1_ t2_ t7_`
> ⇒ 步骤 ③ 只选跑 `t1_`/`t2_`/`t7_` 三个函数，**`t9` 不在其中**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs::t9_gate_rejects_undeclared_capability_and_records_notice`
> **今天不在 `world-core/check.sh` 的任何一步内被选跑**——`:98` 只选 `t1_ t2_ t7_`、
> `:103` 只跑 `--test contract` ⇒ 仅在全量 `cargo test` 时执行。
> 这就是原证据行"步骤 ③"的错处：它把一个**不在该步执行**的测试挂在了该步上。

系统 SHALL 在落笔之前对每一条写入做门禁裁决；未在策略中声明的能力 SHALL 被拒绝，
且拒绝 SHALL 作为一条通告落账，而不是仅返回错误。

#### Scenario: 未声明的能力被拒且留下流水

- **WHEN** 以未在策略中声明的能力提交一条 `act`
- **THEN** 提交被拒，账本中新增一条说明该次拒绝的通告
- **证据**：`tests/acceptance.rs::t9_gate_rejects_undeclared_capability_and_records_notice`
      （**今天不在 `world-core/check.sh` 任何一步内被选跑**，仅在全量 `cargo test` 时执行）

### Requirement: `act` 的效果不能靠 `change` 偷渡

> **改的是哪一类问题**：③ 证据错位（证据行未指名出厂步骤，无法核对是否被跑）。
>
> `audit.md` 未单列此条；本条改动**只为与上一条的引用口径统一**。
> `world-core/check.sh:103` 逐字 `cargo test --locked --test contract`，`c01` 在其中。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs::c01_change_is_gated_and_cannot_smuggle_an_act`
> 位于 `--test contract` 全套之内，由 `world-core/check.sh` 第 ③b 步（`:103`）执行。

系统 SHALL 对 `change` 同样执行写权限裁决；当某一主体无权执行某动作时，
它 SHALL NOT 能通过提交一条 `change` 达到同等效果。

#### Scenario: 越权者用 change 达成静音效果被拒

- **WHEN** 主人先成功写入一条 `change`；随后一个不在白名单的主体提交一条效果相同（静音）
      的 `change`
- **THEN** 第二次提交被拒（错误含"门禁拒绝写入"），状态仍为原值，
      且账本里除那条合法 `change` 外只多一条 `gate.write-rejected` 通告
- **证据**：`tests/contract.rs::c01_change_is_gated_and_cannot_smuggle_an_act`
      （由 `world-core/check.sh` 第 ③b 步执行）

### Requirement: 不可逆动作只允许白名单主体并加摩擦

> **改的是哪一类问题**：④ 与项目文档冲突（**P0 必做项**：把书稿**判红**的东西写成了绿）
> 兼 ① 措辞写宽。
>
> `audit.md` **G1**：书稿 `world-core/docs/理论/语义世界-理论书-第一版-合订.md:717` 逐字
> 「更硬的问题在摩擦的落点上。摩擦本该挂在动作的不可逆等级上：可逆处放手，不可逆处加摩擦。今天它挂在执行者的身份上。」
> —— **判红**；而原规格写「系统 SHALL 对声明为不可逆的能力追加摩擦」。
> `audit.md` **G2**：出厂配置下「只允许白名单主体」与「加摩擦」**不可能同时为真**。
>
> **证据是哪条测试的哪个断言**：
> ① 实现侧：`world-core/src/gate.rs:287-293` 逐字
>    `Some(c) if !c.reversible => { if self .irreversible_actors .iter() .any(|a| pattern_matches(a, actor)) { Decision::Allow }`
>    ⇒ **白名单内 ⇒ `Allow`（不加摩擦、不落流水）**；`:294-303` 的 `else` 分支才 `Decision::AwaitApproval`。
> ② 出厂配置：`world-core/policy.json:32` 逐字 `  "irreversible_actors": ["world://user"],`（唯一成员）；
>    同件 `:28` 逐字 `    "world://user": ["world://*"],` ⇒ 该主体同时可写任意对象；
>    `:38` 逐字 `    "allow": ["world://user", "world://core", "world://agent/*"]`。
> ③ 断言侧：`world-core/tests/acceptance.rs::t10_gate_adds_friction_for_irreversible_capability`
>    与 `world-core/tests/contract.rs::c11_irreversible_is_owner_only_and_the_refusal_does_not_lie`
>    断言的是**白名单外主体**拿到 `AwaitApproval`；
>    **没有一条断言"白名单主体执行不可逆动作时会发生什么"** ⇒ 需补断言（列进 tasks）。

对声明为不可逆（`reversible: false`）的能力，系统 SHALL 执行以下口径，且该口径 SHALL 被如实声明：

- 主体**在** `irreversible_actors` 白名单内 ⇒ **放行，且不追加摩擦、不产生 `gate.*` 通告**；
- 主体**不在** `irreversible_actors` 白名单内 ⇒ `AwaitApproval`（加摩擦），
  拒因 SHALL 明说 v1 **没有审批通道**、不要等批准。

摩擦的落点是**执行者的身份**，不是动作的不可逆等级。系统 SHALL NOT 被表述为
"对不可逆动作本身加摩擦"——出厂配置下这句话不成立：
按 `world-core/policy.json:32`，不可逆白名单只含 `world://user` 一个成员，
而该成员同时被 `world-core/policy.json:28` 授权写任意主体
⇒ **唯一能做不可逆动作的主体，恰是唯一完全免检的主体**。

#### Scenario: 不可逆能力触发摩擦

- **WHEN** 一个**不在** `irreversible_actors` 内的主体提交一个不可逆能力
- **THEN** 门禁加摩擦路径被触发，落一条 `gate.awaiting-approval` 通告，理由明说 v1 无审批通道
- **证据**：`tests/acceptance.rs::t10_gate_adds_friction_for_irreversible_capability`
      —— **⚠ 原文"提交一个不可逆能力"未限定主体**，会读出"任何主体都被加摩擦"；
      实测口径是**只有白名单外主体**才加摩擦（`world-core/src/gate.rs:287-293`）。

#### Scenario: 拒绝流水必须说清它拒绝了什么

- **WHEN** 一个**不在允许名单**的主体提交带保留前缀的通告
- **THEN** 被拒，且门禁写下的通告里含被拒内容的指纹字段
- **证据**：`tests/contract.rs::c23_notice_with_reserved_prefix_is_refused_for_outsiders`
      与 `tests/contract.rs::c23_gate_notice_says_what_it_refused`
      —— **⚠ 两段证据不是同一件事**（`audit.md` **G4** 的张冠李戴）：
      `world-core/tests/contract.rs:1095-1118` 的
      `c23_notice_with_reserved_prefix_is_refused_for_outsiders` **只查错误码与理由**
      （`:1106-1109` 断言 `ext.world.Gate.NoticeNotAllowed`、`:1110` 断言含"内核保留前缀"），
      **不查 `refused` 指纹**；
      而 `world-core/tests/contract.rs:1152-1175` 的 `c23_gate_notice_says_what_it_refused`
      走的是**不可逆加摩擦**路径（`:1157` 逐字 `// 触发一条真实的内核裁决流水：agent 请求不可逆动作 ⇒ 加摩擦`，
      `:1167` 逐字 `.find(|ev| ev["body"]["type"] == json!("gate.awaiting-approval"))`）。
      ⇒ "保留前缀被拒路径也要带 `refused` 指纹"这一条今天**没有断言** ⇒ 需补断言（列进 tasks）。

### Requirement: 门禁配置在被管者不可写的域

> **改的是哪一类问题**：① 措辞写宽（要求强于断言：写"属主与权限"，实现只看 mode 位，
> 且属主断言**默认不跑**）。
>
> `audit.md` **G5**。
>
> **证据是哪条测试的哪个断言**：
> ① `world-core/src/guard.rs:31` 逐字
>    `//! - 本模块只看 **mode 位**，不看**属主**。若被管者恰好就是文件属主，mode 再严也没用。`
> ② 属主断言默认不跑：`world-core/docs/S2-设计/WC-IC-001-v0.1.md:355` 逐字
>    「③ **只在部署方显式传 `--owner-uid` 时才运行**，且只在 `check` 子命令的 `World::open` **之后**执行（`src/main.rs:111-122`）——不传即不运行（`DEBT-02`：属主断言是工具、非自动）」。
> ③ `world-core/tests/contract.rs::c10_owner_assertion_detects_wrong_owner` 断言的是
>    **显式调用**该断言时它会报错，**不**断言"启动时会自动检查属主"。

系统 SHALL 在启动时检查**法律与账本文件的权限 mode 位及其所在目录的权限**；
当法律可被 group/other 写、或账本可被 group/other 写（含其所在目录）时 SHALL 拒绝启动。

系统 SHALL NOT 声称启动时会自动检查**属主**：属主断言是**部署方自证工具**，
只在显式传入 `--owner-uid` 时运行，不传即不运行。

#### Scenario: 法律可被他人写时拒绝启动

- **WHEN** 本体文件的权限使他人可写
- **THEN** 世界拒绝启动
- **证据**：`tests/acceptance.rs::t11_world_refuses_to_start_when_law_is_writable_by_others`

#### Scenario: 账本可被他人写时拒绝启动

- **WHEN** 账本文件的权限使他人可写
- **THEN** 世界拒绝启动
- **证据**：`tests/acceptance.rs::t12_world_refuses_start_when_ledger_is_writable_by_others`

#### Scenario: 属主断言能识别错误属主

- **WHEN** 显式以与预期不符的属主运行属主断言
- **THEN** 断言报错而不是放行
- **证据**：`tests/contract.rs::c10_owner_assertion_detects_wrong_owner`
      —— **⚠ 它不证明"启动时会检查属主"**：不传 `--owner-uid` 即不运行
      （`world-core/docs/S2-设计/WC-IC-001-v0.1.md:355`）。

### Requirement: 运行中的世界不重读策略

> **改的是哪一类问题**：③ 证据错位（证据行未给路径，且该测试不在所标步骤内）。
> 本条**内容不改**，只修引用口径。
>
> `audit.md` 未单列此条；`world-core/policy.json:8` 逐字
> `    "策略在启动时一次读入内存，运行中不重读：磁盘上改了要重启才生效（消除『运行中改规则』的窗口）",`
> 与本条是同一件事。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs::t13_running_world_does_not_reread_policy`
> ——同样**不在** `world-core/check.sh` 的步骤 ③ 内（`:98` 只选 `t1_ t2_ t7_`）。

系统 SHALL 在启动时装载一次策略；运行期间 SHALL NOT 重新读取策略文件，
使"这一次裁决依据的是哪一版法律"是确定的。

#### Scenario: 运行中改动策略文件不影响本次运行

- **WHEN** 世界运行期间改动策略文件，再提交一个该改动本应影响的动作
- **THEN** 裁决结果与启动时装载的策略一致
- **证据**：`tests/acceptance.rs::t13_running_world_does_not_reread_policy`
      （今天不在 `world-core/check.sh` 任何一步内被选跑，仅在全量 `cargo test` 时执行）

## ADDED Requirements

### Requirement: 闸读不到风险等级，且载体可逆与世界可逆无互校

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格通篇不提此事）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `gate-enforcement` 之下，只声明**该既有能力的边界**。
> 本 change **不新增任何互校机制**，只把"无互校"这一事实写成条文。
>
> **改的是哪一类问题**：④ 与项目文档冲突（书稿判红的三条依据里，**两条规格完全沉默**）。
>
> `audit.md` **G3**：书稿 `world-core/docs/理论/语义世界-理论书-第一版-合订.md:723` 逐字
> 「这一格今天判红。红的依据落在三条上：闸读不到风险等级；摩擦挂在执行者的身份上，而那个人同时可写任意对象；
> 载体可不可逆与世界可不可逆这两处之间没有互校。」
>
> **证据是哪条测试的哪个断言**：
> ① 风险等级：`world-core/src/gate.rs:44-48` 逐字
>    `#[derive(Debug, Clone, PartialEq, Eq)]`／`pub struct Capability {`／
>    `    /// 该动作是否可逆（可逆 ⇒ 免检但留痕）。`／`    pub reversible: bool,`／`}`
>    —— **只有 `reversible` 一个字段**；`risk` 只被 `world-core/src/carrier/capd.rs:186` 读。
>    `openspec/specs/` 下 `risk`／`风险` **零命中**，`world-core/src/gate.rs` 零命中。
> ② 互校：`world-core/docs/理论/语义世界-理论书-第一版-合订.md:745` 逐字
>    「**红**。`cap.d/ledger.compact.json` 第 6 行写 `"undo": "before-each"`，`policy.json` 第 20 行写 `"reversible": false`；
>    载体撤销撤的是文件系统字节（`src/carrier/mod.rs` 第 30–34 行），与世界状态无关，而没有任何一处把两件事分开核对（2026-09-27 读配置与源码）」
> ③ **两条今天都没有断言** ⇒ 需补断言（列进 tasks）。

门禁 SHALL 只按 `reversible` 布尔值与 `irreversible_actors` 白名单裁决；
风险等级（`risk`）SHALL NOT 参与"准不准做"的判定，只由载体侧执行清单解析器读取。

载体可逆与世界可逆 SHALL 被分开判定，SHALL NOT 互相冒充：
载体撤销撤的是文件系统上的字节（工程兜底），世界可逆说的是世界状态退不退得回来；
两者之间**在 v1 没有任何一致性检查**，此边界 SHALL 被如实声明，
不得被读成"两处已互校"，也不得被读成"留了撤销点的能力在世界里也可逆"。

#### Scenario: 风险等级不参与门禁裁决

- **WHEN** 同一能力在载体执行清单里标 `risk: high`、在门禁策略里标 `reversible: true`
- **THEN** 门禁按 `reversible: true` 放行，`risk` 不改变结论
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/gate.rs:44-48`（结构里没有 `risk` 字段）。

#### Scenario: 载体可逆不等于世界可逆

- **WHEN** 一项能力在载体侧留了撤销点（`undo: before-each`），而世界侧标 `reversible: false`
- **THEN** 门禁按"世界不可逆"处置，载体撤销点**不**被当作世界可逆的依据
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/carrier/mod.rs:28-34` 的对照表，其 `:32` 逐字
      `| **载体撤销**（本模块） | 文件系统的字节 | **不是世界状态**；只作工程兜底，**不得**用于满足"坏了能回滚" |`。

### Requirement: 通告的闸，以及门禁不可绕过的部分实现边界

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `gate-enforcement` 之下，只声明**该既有能力的边界**。
>
> **改的是哪一类问题**：② 措辞写窄（实现有、规格无）兼 ④ 与项目文档冲突（部分实现的边界规格未写）。
>
> `audit.md` **G6**：`world-core/src/lib.rs` 逐字登记的两句话在规格里**没有 Requirement 级条目**；
> 而 SRS 记的"部分实现"边界规格未写。
>
> **证据是哪条测试的哪个断言**：
> ① 通告的闸：`world-core/tests/contract.rs::c23_notice_with_reserved_prefix_is_refused_for_outsiders`
>    （保留前缀被拒 + 伪造通告不落笔，`:1113-1118`）与
>    `world-core/tests/contract.rs::c23_notice_from_unlisted_actor_is_refused`
>    （不在册主体被拒 `:1133-1134` + 在册主体写同样形状必须通过 `:1137-1142`）。
>    实现侧 `world-core/src/lib.rs:270` 逐字
>    `/// 此前 \`notice\` 落在 \`adjudicate\` 的 \`_ => Ok(())\` 分支里——**任何主体可以写任何通告**。`
>    与 `:280` 逐字 `/// 也就是说："世界拒绝过这件事"这句话，**任何人都能替世界说**。`
> ② 部分实现边界：`world-core/docs/S1-需求/WC-SRS-001-v0.1.md:83` 逐字
>    「| `REQ-F-015` | 门禁不可绕过 | P0 | 部分实现（见 WC-CON01-001：进程内 + 静态墙已跨 uid 实测；祖先链遍历、通道层与 Landlock 自缚未做） |」
>    ⇒ 该边界**规格里没有落点**。
> ③ 「祖先链遍历未做」这一项**不可被断言**（"未做"本身没有观测面）；其载体是流程侧登记。

通告 SHALL 过闸，两条纪律各自可单独判真假：
① 带保留前缀（`gate.`）的通告类型 SHALL 只许内核自己写，外部提交一律拒；
② 非保留前缀的通告，其 `actor` SHALL 已在主体白名单内。

门禁不可绕过 SHALL 只声明**已实测的范围**：进程内唯一写入口与静态墙（mode 位）已在跨 uid 形态下实测；
**祖先链遍历、通道层与 Landlock 自缚在 v1 未做**，此边界 SHALL 被如实声明，
SHALL NOT 被读成"门禁在所有路径上不可绕过"。

已知的不对称 SHALL 被登记：非保留前缀的通告**不检查 `writes` 授权表**、只检查主体白名单。

#### Scenario: 保留前缀通告只许内核自己写

- **WHEN** 一个外部主体提交 `type` 以 `gate.` 开头的通告
- **THEN** 被拒，错误串含 `ext.world.Gate.NoticeNotAllowed`，且伪造通告不落笔
- **证据**：`tests/contract.rs::c23_notice_with_reserved_prefix_is_refused_for_outsiders`

#### Scenario: 不在册主体不得写普通通告，在册主体必须能写

- **WHEN** 一个不在白名单的主体提交普通通告；对照组由在册主体提交同样形状的通告
- **THEN** 前者被拒（`ext.world.Gate.NoticeRejected`），后者通过
- **证据**：`tests/contract.rs::c23_notice_from_unlisted_actor_is_refused`

#### Scenario: 门禁不可绕过的未做部分被如实声明

- **WHEN** 查阅本能力的覆盖声明
- **THEN** 祖先链遍历、通道层与 Landlock 自缚三项标为**未做**
- **证据**：`world-core/docs/S1-需求/WC-SRS-001-v0.1.md:83`（`REQ-F-015` 状态列）
      —— 该项**今天没有自动化断言**（"未做"不可被断言），其载体是流程侧登记与 `review.md` 的签字。
