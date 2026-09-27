# Spec Delta

## MODIFIED Requirements

### Requirement: 信封的必填字段被逐字段强制

> **改的是哪一类问题**：① 措辞写宽（要求强于断言）。**不是**"能力有没有"的问题。
>
> `audit.md` **E6**：八个必填字段里，`world` 那一轮的「报出字段名」**恒真**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs:120-123` 逐字
> `assert!( msg.contains("MissingField") && msg.contains(field), "删除 \`{field}\` 应被拒且指明字段，实得: {msg}" );`
> 而错误前缀自带 `world` 二字——`world-core/src/ontology.rs:32` 逐字
> `"ext.world.Ontology.MissingField: {at} 缺少必填字段 \`{field}\`"`
> ⇒ `field == "world"` 时 `msg.contains("world")` 由前缀满足，该轮**不证明**字段名被报出。
> 其余 7 个字段不受影响。⇒ 需补断言（列进 tasks）。

系统 SHALL 对每一条写入事件逐字段校验信封必填项 `world` / `kind` / `id` / `seq` / `at` /
`actor` / `flags` / `body`；任一字段缺失时 SHALL 拒绝该事件。

系统 SHALL 另外明确声明该断言的边界：错误信息 **SHALL** 报出缺失字段名这一要求，
今天只对 `kind` / `id` / `seq` / `at` / `actor` / `flags` / `body` **七个字段**成立；
`world` 一轮由错误码前缀 `ext.world.` 满足了"含该字段名"的判据，
SHALL NOT 被读成"`world` 缺失也被指名报出"。

#### Scenario: 八个必填字段逐字段被拦

- **WHEN** 依次构造 8 条事件，每条分别删掉一个必填字段
- **THEN** 每条都被校验拒绝
- **证据**：`tests/contract.rs::c02_every_required_envelope_field_is_enforced`
      —— **⚠ `world` 一轮的"报出字段名"是恒真断言**（`world-core/src/ontology.rs:32` 的前缀自带 `world`）
      ⇒ 该轮要成为"指名报出"的证据 ⇒ 需补断言（列进 tasks）。

### Requirement: 三类话之外一律被拒

> **改的是哪一类问题**：③ 证据错位（引错了出厂步骤）兼 ① 措辞写宽（缺"状态未被改动"的断言）。
> **不是**"能力有没有"的问题——三类话之外确实被拒且不落笔。
>
> `audit.md` **E2**：被引证据的证据行写着「`check.sh` 步骤 ③」，而那一步**根本不跑 `t5`**
> （`world-core/check.sh:98` 逐字 `cargo test --locked --test acceptance -- t1_ t2_ t7_`，只选三个函数）。
> `audit.md` **E5**：`t5` 没有断言本 Scenario 的第二句「状态未被改动」。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs:158-159` 逐字
> `let e1 = w.commit("bogus", "world://user", json!({})).unwrap_err();`／
> `assert!(e1.contains("UnknownKind"), "实得: {e1}");`
> 与 `:187` 逐字 `assert_eq!(w.ledger().last_seq(), 1, "被拒的事件绝不允许落笔");`
> ——**这断言的是"不落笔"，不是"状态未被改动"**。同文件 `:87-92` 有"状态未变"的既有写法可照抄。

系统 SHALL 只接受本体声明的三个家族（变更 / 请求与结果 / 通告）；出现未知家族时 SHALL 拒绝，
SHALL NOT 猜测其含义，SHALL NOT 落笔该事件，且 SHALL NOT 改动世界状态。

#### Scenario: 不认识的事件家族被拒

- **WHEN** 以本体未声明的 `kind` 提交事件
- **THEN** 提交失败，错误串含 `UnknownKind`，且不落笔该事件（账本 `last_seq` 不前进）
- **证据**：`tests/acceptance.rs::t5_law_rejects_and_does_not_write`
      —— **⚠ 该测试今天不在 `world-core/check.sh` 的任何一步内被选跑**：
      `:98` 只选 `t1_ t2_ t7_`、`:103` 只跑 `--test contract` ⇒ 仅在全量 `cargo test` 时执行；
      且它**不断言"状态未被改动"** ⇒ 需补断言（列进 tasks）。

### Requirement: 法律损坏或指向别处时拒绝启动

> **改的是哪一类问题**：① 措辞写宽（要求强于断言）。
>
> `audit.md` **E3**：本条的措辞比证据宽三处，**三处实现都拒，只是没有载体**。
>
> **证据是哪条测试的哪个断言**，逐条给出：
> ① **策略文件缺失**：`world-core/tests/contract.rs::c03` 的五类（`:154` 坏 JSON、`:159-164` 缺 `policy`、
>    `:166-171` 能力表为空、`:173-178` 白名单为空、`:180-185` 缺 `writes`）里**没有「缺失」**，全仓无载体；
> ② **本体字段形状非法**：`world-core/tests/contract.rs:224-227` 只测缺文件、`:229-242` 只测家族为空；
> ③ **本体软链**：`world-core/tests/contract.rs::c09` 只测**策略**软链。
> ⇒ 三处均需补断言（列进 tasks）。

本体与门禁策略 SHALL 在世界打开时装载并校验；文件缺失、家族为空、能力表为空、白名单为空、
`writes` 段缺失时 SHALL 拒绝启动；法律文件是**符号链接**时 SHALL 拒绝启动，
以免"检查的"与"真正读的"不是同一个文件。

系统 SHALL 明确声明该要求的覆盖边界：本规格 SHALL NOT 声称"不存在静默接受一个畸形策略的路径"
——**未知键与多余键一律被忽略**是设计选择，不是畸形。

#### Scenario: 坏本体拒绝启动

- **WHEN** 本体文件内容损坏
- **THEN** 打开失败
- **证据**：`tests/acceptance.rs::t6_bad_ontology_refuses_to_start`

#### Scenario: 本体缺失与家族为空被拒

- **WHEN** 本体文件不存在，或 `families` 为空
- **THEN** 装载失败（`ReadFail` / `NoFamilies`）
- **证据**：`tests/contract.rs::c04_ontology_load_rejects_missing_file_and_empty_families`

#### Scenario: 策略的每一类畸形形状都被拒

- **WHEN** 以 5 类畸形形状分别装载门禁策略（坏 JSON／缺 `policy`／能力表为空／白名单为空／缺 `writes`）
- **THEN** 每一类都被拒；**对照**：合法策略必须能加载
- **证据**：`tests/contract.rs::c03_policy_load_rejects_every_malformed_shape`
      —— **⚠ 原文"不存在静默接受一个畸形策略的路径"是全称句，与同用例的对照②冲突**：
      `world-core/tests/contract.rs:207` 逐字
      `let pol = Policy::load(&p).expect("多余键应被忽略，而不是拒载");`
      ⇒ **未知键与多余键是被忽略的**，该全称句已按事实收窄。

#### Scenario: 策略为符号链接时拒绝启动，而真实文件不被误拒

- **WHEN** 以指向别处的软链作为策略路径打开世界；对照组用真实文件
- **THEN** 软链被拒且错误信息含"符号链接"；真实文件正常打开
- **证据**：`tests/contract.rs::c09_symlinked_law_is_refused`
      —— **⚠ 本证据只覆盖"策略"软链**，本体软链无载体。

### Requirement: 事件身份在进程内唯一

> **改的是哪一类问题**：① 措辞写宽后的**范围收窄**——原文已有「同一进程内」限定，
> 本条**只补"跨重启不保证"这半边**，不改结论强度。`audit.md` 未单列此条。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs:245-248` 逐字
> 「`new_id()` = 纳秒 + 进程内计数器；计数器每次启动从 0 开始，/ 故"跨重启唯一"依赖纳秒不重复——本测试覆盖**同进程内**的唯一性。」
> 断言本体在 `:251-256`：2000 次 `new_event` 收进 `BTreeSet` 后断言无重复。

系统 SHALL 保证同一进程内分配的事件 `id` 不重复。
系统 SHALL NOT 声称跨进程或跨重启的 `id` 唯一性由本机制保证。

#### Scenario: 连续提交的事件 id 各不相同

- **WHEN** 在同一进程内连续提交多条事件
- **THEN** 所有事件的 `id` 互不相同
- **证据**：`tests/contract.rs::c05_event_ids_are_unique_within_a_process`

### Requirement: 错误携带机器可读的错误码

> **改的是哪一类问题**：③ 证据错位（最重的一处：Scenario 说"四类路径"，证据里**没有一类是账本**）
> 兼 ① 措辞写宽（法律类存在**无码出口**）。
>
> `audit.md` **E1**：`openspec/specs/envelope-validation/spec.md:78` 写「触发**法律、门禁、账本、读模型**四类失败路径」；
> 而 `c15` 的七条码来源逐条为 `world-core/tests/contract.rs:714`（本体坏 JSON）、
> `:718`（门禁未声明能力）、`:728`（门禁未授权写入）、`:738`（门禁不可逆加摩擦）、
> `:748`（法律违反信纸）、`:758`（通道坏请求）、`:767`（读模型坏账本）
> —— **没有一条账本路径**。全仓唯一的账本码断言在 `world-core/tests/cli.rs:155` 逐字
> `err.contains("ext.world.Ledger.NoChain"),`，而 `world-core/check.sh` **从不跑 `--test cli`**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs:770-776` 逐字
> `for c in &codes { assert!(has_code(c), "错误缺少 \`ext.world.<域>.<原因>\` 前缀：{c}"); }`
> 与 `assert!(unique.len() >= 6, "错误码区分度不足，只拿到 {unique:?}");`；
> 反例侧 `:779` 逐字 `assert_eq!(code_of("门禁拒绝：能力未声明"), None);`
> ——**没有账本类别**。⇒ 需补断言（列进 tasks）。

系统 SHALL 在失败路径上给出机器可读的错误码（如 `ext.world.*`），使调用方按码判定，
SHALL NOT 要求调用方去匹配中文散文措辞。

系统 SHALL 明确声明本条的范围边界：**并非所有失败出口今天都带码**；
已知的无码出口 SHALL 被逐条登记，SHALL NOT 被"凡失败路径都带码"这类全称句遮住。

#### Scenario: 各类失败路径的错误码可枚举

- **WHEN** 触发布局类（本体形状）、门禁类、法律类、通道类、读模型类失败路径
- **THEN** 每条错误都带稳定的错误码 `ext.world.<域>.<原因>`，且错误码区分度 ≥ 6
- **证据**：`tests/contract.rs::c15_errors_carry_machine_readable_codes`
      —— **⚠ 本证据不含任何账本路径**；且"散文式错误必须判为不符合契约"这一反例
      由同函数的 `world-core/tests/contract.rs:779` 承担。

## ADDED Requirements

### Requirement: 错误码契约的已知边界

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `envelope-validation` 之下，只声明**该既有能力的边界**。
>
> **改的是哪一类问题**：④ 与项目文档冲突（规格把"已知无码"的出口包在了全称句里）。
>
> `audit.md` **E1** 后半：实测法律类存在无码出口。项目文档**已自认**此项。
>
> **证据是哪条测试的哪个断言**：
> ① 静态墙三条断言逐条标「⚠ **无码**」：`world-core/docs/S2-设计/WC-IC-001-v0.1.md:350`（符号链接）、
>    `:351`（mode 位）、`:352`（属主断言）；
> ② 策略版本不符出口也是散文：`world-core/src/gate.rs:113-117` 逐字
>    `if version != 1 { return Err(format!( "门禁策略版本不支持：期望 1，实得 {version}（法律版本不符即拒绝启动）" )); }`
> ③ **上述四处今天都没有"它不带码"的断言** ⇒ 需补断言（列进 tasks）。

系统 SHALL 逐条登记**已知的无码失败出口**，使"凡失败路径都带码"这一全称句不成立：

- 静态墙的三条断言：拒绝符号链接、拒绝 group/other 可写（文件与目录）、属主断言未通过；
- 门禁策略版本不支持（`policy != 1`）。

上述出口报的是中文散文，**不带 `ext.world.` 前缀**。
系统 SHALL NOT 把本能力的覆盖声明写成"所有失败路径都已带码"。

#### Scenario: 静态墙三条断言无错误码（边界固定）

- **WHEN** 以符号链接作为法律路径、或以对 group/other 可写的文件作为法律路径打开世界
- **THEN** 拒绝启动，但错误串**不含** `ext.world.` 码——本断言证明的是**边界**而不是实现缺陷
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/guard.rs:43`（`assert_not_other_writable`）
      与同文件的符号链接断言；文档出处为 `world-core/docs/S2-设计/WC-IC-001-v0.1.md:350-352`，
      三行末列逐字都写「⚠ **无码**」。

#### Scenario: 策略版本不符出口无错误码（边界固定）

- **WHEN** 以 `policy` 版本号不为 1 的策略打开世界
- **THEN** 拒绝启动，错误串为散文，**不含** `ext.world.` 码
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/gate.rs:113-117`。
