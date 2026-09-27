# Spec Delta

## MODIFIED Requirements

### Requirement: 事件按序落账并可跨进程读回

> **改的是哪一类问题**：③ 证据错位（证据层级错位：规格写"新进程"，测试是**同进程** drop + reopen）
> 兼 ① 措辞写宽（`t1` 的"逐字段一致"没有被断言）。
>
> `audit.md` **L4**：原 `spec.md:23` 写「以新进程打开」，而 `world-core/tests/acceptance.rs:85-104`
> 实为同进程 drop 后 reopen。`audit.md` **L5**：`t1` 的"逐字段一致"没有被断言。
>
> **证据是哪条测试的哪个断言**：
> ① `t2` 的实际形态：`world-core/tests/acceptance.rs:85-104` 的 `t2_events_survive_restart`
>    在一个 `#[test]` 函数内的 `{ ... }` 作用域里 drop 写者、再 reopen——**不是新进程**。
> ② 真正合格的"跨进程"证据：`world-core/tools/s1_sys_probe2.sh:379-383` 逐字
>    `# ══ TC-070 · REQ-F-022 账本可重放（重启不丢）═════════════════════════`／
>    `echo; echo "── TC-070 · REQ-F-022 重启后事件全在且顺序不变 ──"`／
>    `assert_eq "① 两次**独立进程**读回的事件序列**逐字节相同**" "$R1" "$R2"`（由 `world-core/check.sh:145` 执行）。
> ③ `t1` 实际断言（`world-core/tests/acceptance.rs:69-80`）：`assert_eq!(evs.len(), 3, "应有 3 条事件")`、
>    `assert_eq!(ev["seq"], json!(i as u64 + 1), "seq 必须从 1 连续递增")`、
>    `assert_eq!(ev["world"], json!(1), "信封必须带词表版本")`、
>    `assert_eq!(evs[0]["kind"], json!("change"))`、`assert_eq!(evs[0]["body"]["before"], json!(false), ...)`、
>    `assert_eq!(evs[2]["kind"], json!("notice"))`
>    ⇒ **`actor`／`id`／`at`／`flags`／`body.subject`／`body.path`／`body.after` 一个都没比** ⇒ 需补断言（列进 tasks）。

系统 SHALL 把每一条语义事件按 `seq` 升序追加进同一本账，且该账 SHALL 以纯文本 JSON Lines 落盘，
使事件在写入进程退出后仍然可读。

**跨进程**读回 SHALL 由**独立进程**的证据承担，
SHALL NOT 以"同进程内 drop 后 reopen"充当跨进程证据。

#### Scenario: 追加后按序读回

- **WHEN** 打开世界并连续提交 3 条 `change`
- **THEN** 按序读回得到 3 条事件，`seq` 从 1 连续递增，信封带词表版本，各条 `kind` 与 `body.before` 正确
- **证据**：`tests/acceptance.rs::t1_append_then_read_back`（由 `world-core/check.sh` 第 ③ 步执行）
      —— **⚠ 本证据今天只到"字段抽样比对"**：`actor`／`id`／`at`／`flags`／`body.subject`／
      `body.path`／`body.after` 未被断言（`world-core/tests/acceptance.rs:69-80`）
      ⇒ "逐字段一致"要成立 ⇒ 需补断言（列进 tasks）。

#### Scenario: 事件在新进程里仍存在

- **WHEN** 写入若干事件后结束写入方，再打开同一本账
- **THEN** 先前写入的事件全部可读回，条数与内容不变
- **证据**：`tests/acceptance.rs::t2_events_survive_restart`（由 `world-core/check.sh` 第 ③ 步执行）
      —— **⚠ 本证据是"同进程 drop + reopen"，不是"新进程"**（`world-core/tests/acceptance.rs:85-104`）；
      "新进程"这一层由 `world-core/tools/s1_sys_probe2.sh` 的 `TC-070` 承担
      （`world-core/check.sh:145` 执行）。

### Requirement: 同一本账同时只有一个写者

> **改的是哪一类问题**：① 措辞写宽（原措辞超出实现前提，且反向失效未登记）。
>
> `audit.md` **L2**：原 `spec.md:29-30` 写无条件；实现**锁只记 pid 号**，
> 存活判据是 `/proc/<pid>` 是否存在。
>
> **证据是哪条测试的哪个断言**：
> ① 锁只写 pid：`world-core/src/ledger.rs:175` 逐字 `let _ = writeln!(f, "{}", std::process::id());`
> ② 存活判据：`world-core/src/ledger.rs:182-184` 逐字
>    `let alive = pid .map(|p| Path::new(&format!("/proc/{p}")).exists()) .unwrap_or(false);`
> ③ 断言侧：`world-core/tests/contract.rs::c08_stale_lock_is_reclaimed`、
>    `world-core/tests/contract.rs::c07_second_writer_is_refused`（由 `world-core/check.sh` 第 ③b 步执行）。
> ④ **反向失效（pid 号被复用时不回收）今天没有断言** ⇒ 需补断言（列进 tasks）。

系统 SHALL 拒绝第二个写者打开同一本账，且拒绝理由 SHALL 说明这是"单写者"约束；
持锁进程正常退出后锁 SHALL 被释放；持有者已不存在的陈旧锁 SHALL 被自动回收。

已知前提 SHALL 被如实声明：锁文件**只记录持有者的 pid 号**，其"是否存活"的判据是
`/proc/<pid>` 是否存在 ⇒ 本机制的正确性**依赖**：同机、同一 PID 命名空间、`/proc` 可读。
在共享存储或独立 PID 命名空间下，存活写者的锁会被当作陈旧锁回收，从而出现两个写者；
反之 pid 号被复用时持有者已不存在却不会被回收（假锁死）。
此边界 SHALL NOT 被读成"锁在所有形态下成立"。

#### Scenario: 第二个写者被拒且理由说清单写者

- **WHEN** 已经有一个写者打开了某本账，再用同一本体与策略打开同一本账
- **THEN** 打开失败，错误串包含 `Ledger.Locked` 与"单写者"
- **证据**：`tests/contract.rs::c07_second_writer_is_refused`（由 `world-core/check.sh` 第 ③b 步执行）

#### Scenario: 锁随第一个写者退出而释放

- **WHEN** 释放第一个写者（Drop）后再次打开同一本账
- **THEN** 打开成功
- **证据**：`tests/contract.rs::c07_second_writer_is_refused`

#### Scenario: 陈旧锁被回收且锁文件记录新持有者

- **WHEN** 锁文件内容是一个不存在的 pid（伪造崩溃遗留），再打开该账本
- **THEN** 打开成功，`last_seq` 为 0，且锁文件内容被改写为当前进程 pid
- **证据**：`tests/contract.rs::c08_stale_lock_is_reclaimed`

### Requirement: 残缺的尾部被丢弃，`seq` 空洞拒绝启动

> **改的是哪一类问题**：④ 与项目文档冲突（规格把一个更宽的、会**静默物理删除完整事件**的行为
> 写成了较窄的"丢弃解析失败的最后一行"）。
>
> `audit.md` **L3**：实现按**最后一个 `\n`** 截断（直接 `set_len` 落盘），**不看能否解析**；
> 一条**完整合法 JSON 但缺末尾换行**的行会被静默物理删除并复用 `seq`。
>
> **证据是哪条测试的哪个断言**：
> ① 截断判据：`world-core/src/ledger.rs:276-279` 逐字
>    `let keep = match raw.iter().rposition(|b| *b == b'\n') { Some(pos) => pos + 1, // 保留到最后一个换行（含） None => 0, // 一个换行都没有 ⇒ 整个文件都是半行 };`
> ② 落盘删除：`world-core/src/ledger.rs:280-289`（`set_len(keep)`）。
> ③ 这条边界由实现**自己如实写过**：`world-core/src/ledger.rs:385` 逐字
>    `/// ② 若粘连处含此前已 ack 的事件，启动时"截到最后一个 \n"会**静默删掉**它们；`
> ④ 断言侧：`world-core/tests/acceptance.rs::t3_partial_line_is_discarded` 正面背书"半行被丢弃"，
>    **但没有断言"完整合法 JSON 缺末尾换行也会被删"** ⇒ 需补断言（列进 tasks）。

系统 SHALL 在启动读账本时把文件**截到最后一个换行符**（只追加模型下唯一可能残缺的位置），
且当 `seq` 出现空洞时 SHALL 拒绝启动，而不是静默接受。

**边界 SHALL 被如实声明**：本判据是"**截到最后一个换行**"，**不是**"丢弃解析失败的最后一行"。
一条内容完整、合法的 JSON 事件，只要其末尾换行缺失，就会被一并截掉并在下次写入时被物理删除、
其 `seq` 被复用。此边界 SHALL NOT 被读成"只丢残缺的半行"。

#### Scenario: 半行被丢弃

- **WHEN** 账本最后一行是残缺的 JSON（或末尾换行缺失）
- **THEN** 该行被截掉，其余事件正常读回
- **证据**：`tests/acceptance.rs::t3_partial_line_is_discarded`
      —— **⚠ 实测判据是"截到最后一个 `\n`"**（`world-core/src/ledger.rs:276-279`），
      **不看该行能否解析** ⇒ "完整事件因末尾换行缺失被静默删除"这一形态 ⇒ 需补断言（列进 tasks）。

#### Scenario: `seq` 有空洞时拒绝启动

- **WHEN** 账本中存在 `seq` 不连续的事件
- **THEN** 打开失败并报出 `SeqGap`
- **证据**：`tests/acceptance.rs::t4_seq_gap_refuses_to_start`
      —— **⚠ 本条在生产路径上由账本层更早拦截**：`world-core/src/ledger.rs:304` 逐字
      `if seq != last + 1 {` ⇒ 本断言**不构成**端到端证明（防线冗余，非缺陷）。

### Requirement: 账本文件恒以行边界收尾

> **改的是哪一类问题**：③ 证据错位（出厂脚本同名两处，证据行只写 `check.sh` 未给路径）。
>
> `audit.md` **L8** 后半：原 `spec.md:19/25/36` 只写 `check.sh`，而仓里**有两个同名脚本**，
> 根 `check.sh` **完全不碰 world-core**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs::c22_file_always_ends_on_a_line_boundary`
> 位于 `--test contract` 全套之内，由 `world-core/check.sh` 第 ③b 步（`:103`）执行。
> 仓根 `check.sh` 全文 49 行只跑 `agentd` 的 `go build` / `go vet` / `go test`（`:28-40`），
> **没有一行涉及 `world-core`**。

系统 SHALL 保证写出的账本文件始终以换行符结束，
使"每行一条事件"这一分帧约定在任何时刻都成立。

#### Scenario: 每次写入后文件都以换行收尾

- **WHEN** 任意次提交之后检查账本文件的最后一个字节
- **THEN** 最后一个字节是换行符，不存在"半行挂在末尾"的中间态
- **证据**：`tests/contract.rs::c22_file_always_ends_on_a_line_boundary`
      （由 **`world-core/check.sh`** 第 ③b 步执行——仓根另有一个同名 `check.sh`，它不涉及 `world-core`）

### Requirement: 摘要链检出局部篡改，并如实声明其边界

> **改的是哪一类问题**：④ 与项目文档冲突（**P0 必做项**：把已知【高】级缺陷写成已成立行为）。
>
> `audit.md` **L1**：原 `spec.md:111` 把「v1 兼容」写成已成立，而项目自己把这条路径登记为
> **【高】级自杀缺陷**——`world-core/docs/S0-立项/WC-SCMP-001-v0.1.md:2537` 逐字
> 「### K-3【高】在 v1（无链）账本上做一次正常 `append` 会把世界锁死 —— 升级路径自杀」。
>
> **证据是哪条测试的哪个断言**：
> ① 「无链账本仍能打开」这一句的**全部**依据是 `world-core/tests/contract.rs:1026` 逐字
>    `assert_eq!(w3.ledger().last_seq(), 1, "无链账本仍应能打开（v1 兼容）");`
>    —— 它只断言了**打开**这一步，**没有断言打开之后能不能正常写入**。
> ② K-3 的后果链由文档逐字给出：`WC-SCMP-001-v0.1.md:2541` 逐字
>    「**后果（EV-11b）**：无链账本 → `check` 警告但 `READY`（**设计意图是兼容**）→ 一次合法 `append` 成功 → 下次打开 `Ledger.MixedChain … 拒绝使用`。」
>    同件 `:2542` 逐字「**触发场景不是攻击，是升级**：任何历史账本在升级后的第一次提交都会命中。」
> ③ **这条路径今天没有任何自动化断言** ⇒ 需补断言（列进 tasks）——
>    `MixedChain` 在仓库内只出现在 `world-core/tests/contract.rs:845`（注释）与 `:879`（`c17` 函数级）；
>    启动路径的拒绝由 `world-core/src/lib.rs:105` 逐字 `ledger.load_chain()?;` 触发，**无函数级断言**。

系统 SHALL 为每条写出的事件带上摘要链，并把链的核验接在**启动路径**上：
账本被局部改写、重排或插入时 SHALL 拒绝启动。

系统 SHALL 明确声明第一条边界：无密钥的链**不能**检出"整文件重写并重算链"，
此边界 SHALL 被测试固定，不得被表述为"防篡改"。

系统 SHALL 另外明确声明第二条边界：**无链（v1）账本不具备平滑升级路径**。
`append` **无条件**把 `chain` 插进事件，而 `load_chain` 对无链账本返回 `false` 后
**不改变写入行为** ⇒ 无链账本上做**一次合法 `append`** 即产生"部分有链、部分没有"的账本，
下次打开判 `MixedChain` 并拒绝使用。此边界 SHALL NOT 被表述为"v1 兼容"或"可平滑升级"。

#### Scenario: 改写、重排、插入中间事件均被检出

- **WHEN** 对一条合法链分别做三种操作：改中间某条的内容、交换相邻两条、按攻击者算好的链插入一条
- **THEN** 三种情形都报 `ChainMismatch`
- **证据**：`tests/contract.rs::c16_chain_detects_local_tampering`

#### Scenario: 无链与混用被显式区分，不得静默通过

- **WHEN** 对一份完全无链的账本、以及一份部分带链的账本做核验
- **THEN** 无链报 `NoChain` 并说明"不可检出"；混用报 `MixedChain` 并说明"比无链更危险"
- **证据**：`tests/contract.rs::c17_chain_distinguishes_absent_and_mixed`

#### Scenario: 篡改一行即拒绝启动

- **WHEN** 直接改写账本中间一行的 `body.after` 而保留原 `chain`，再打开世界
- **THEN** 打开失败，错误串包含 `ChainMismatch`，且理由提到"唯一真相"
- **证据**：`tests/contract.rs::c20_startup_refuses_a_tampered_ledger`

#### Scenario: 整本重写按设计检不出（边界固定）

- **WHEN** 攻击者改写一条事件的值，并把链整条重算（知道算法与创世种子）
- **THEN** 核验**通过**——这是无密钥链的极限，本断言证明的是边界而不是实现缺陷；
      同时不重算链的局部篡改仍必须被检出
- **证据**：`tests/contract.rs::c18_chain_cannot_detect_a_full_rewrite`

#### Scenario: 链状态位如实反映账本现实

- **WHEN** 分别对"新写入的带链账本""空账本""手工造的无链账本"读取链状态
- **THEN** 依次为 true / false / false，且**无链账本可以被打开**
- **证据**：`tests/contract.rs::c21_is_chained_reflects_reality`
      —— **⚠ 原文括注"（v1 兼容）"已删**：本证据只到"能打开"为止
      （`world-core/tests/contract.rs:1026`），它**不**断言"打开后还能正常写下去"；
      该边界见下一条 `## ADDED` 里的 `K-3` 条文。

### Requirement: 回滚是追加补偿事件，不是改写历史

> **改的是哪一类问题**：① 措辞写宽（原措辞把"系统提供回滚操作"读成了能力，
> 而系统里**没有回滚操作**）。
>
> `audit.md` **L7**：原 `spec.md:116/120` 口径与 SRS 不一致。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs:563-569` 逐字
> `// ② 回滚：追加一条**补偿事件**（旧值/新值互换）`／
> `w.commit( "change", "world://user", event::change_body(subj, "muted", json!(true), json!(false)), )`
> ⇒ **回滚动作由测试自己再提交一条互换的 `change` 完成**，不是系统提供的操作。
> SRS 的口径更准：`world-core/docs/S1-需求/WC-SRS-001-v0.1.md:257` 逐字
> 「| `REQ-F-014` | 回滚 = 追加补偿事件 | P1 | 不得修改或删除历史；**回滚通过追加一条普通 `change` 完成** |…」

系统 SHALL 以"追加一条补偿事件"的方式实现回滚，
SHALL NOT 修改或删除已经写下的事件。

系统 SHALL NOT 声称提供"回滚操作"或"回滚子命令"：v1 没有回滚 API，
回滚由调用方**再提交一条普通 `change`** 完成。

#### Scenario: 回滚后历史里两条都在

- **WHEN** 提交一条 `change` 后对其执行回滚（即再提交一条互换新旧值的 `change`）
- **THEN** 账本中同时存在原事件与补偿事件，没有任何一条被改写或删除，且状态回到原处
- **证据**：`tests/acceptance.rs::t17_rollback_is_an_appended_compensating_event`
      —— **⚠ 回滚动作由该测试自己提交，不是系统命令**；载体侧的"撤销"是另一件事：
      `world-core/src/carrier/mod.rs:32` 逐字
      `| **载体撤销**（本模块） | 文件系统的字节 | **不是世界状态**；只作工程兜底，**不得**用于满足"坏了能回滚" |`。

## ADDED Requirements

### Requirement: 无链账本的升级路径边界

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `ledger-integrity` 之下，只声明**该既有能力的边界**——
> 照本能力对摘要链"整本重写按设计检不出"那条的既有写法，**把边界本身写成 Requirement/Scenario**。
>
> **改的是哪一类问题**：④ 与项目文档冲突（**P0 必做项**）。这就是 `audit.md` **L1** 与 **L6**。
>
> 缺陷原文：`world-core/docs/S0-立项/WC-SCMP-001-v0.1.md:2537` 逐字
> 「### K-3【高】在 v1（无链）账本上做一次正常 `append` 会把世界锁死 —— 升级路径自杀」；
> `:2544` 给出最小补丁逐字「**最小补丁**：`append` 必须尊重 `self.chained`——`false` 时
> **要么继续写无链事件**（保持"整本无链"这个稳定态），**要么启动时即拒写并给 migrate 指引**；
> 两者择一，**不能继续写链**。」
>
> **证据是哪条测试的哪个断言**：
> ① **今天没有任何自动化断言**：`MixedChain` 在仓库内只出现在
>    `world-core/tests/contract.rs:845`（注释）与 `:879`（`c17` 函数级）；
>    启动路径的拒绝由 `world-core/src/lib.rs:105` 逐字 `ledger.load_chain()?;` 触发，
>    系统级证据只有手工留档（`world-core/docs/理论/专家评审/复跑-九项保证-2026-09-27-VM.md:130-140`）。 〔该件已按作者指示退场；解析根＝`git show bf2eae7:<原路径>`〕
> ② 实现侧：`world-core/src/ledger.rs:506` 逐字 `self.chained = true;`（只在有链时置位），
>    而 `world-core/src/ledger.rs:518-519` 逐字 `pub fn is_chained(&self) -> bool { self.chained }`
>    ⇒ 该状态位**只用于显示**，未参与 `append` 的写入决策。
> ⇒ **本 Requirement 的全部断言今天都不存在** ⇒ 需补断言（列进 tasks）。

系统 SHALL 保证：在**无链（v1）账本**上做一次合法 `append` 之后，
该账本 SHALL 仍可被打开——即"整本无链"是一个**稳定态**，
或者启动时即拒写并给出迁移指引。
系统 SHALL NOT 使"无链账本 → 一次合法 `append` → 下次打开被 `MixedChain` 拒绝"成为可能。

在此之前，系统 SHALL 在规格里**不声称**无链账本可平滑升级：
"v1 兼容"这一说法 SHALL 被限定为"**只读兼容**"。
此边界 SHALL 被测试固定，SHALL NOT 被表述为实现缺陷或注意事项。

#### Scenario: 无链账本上一次合法 append 后仍可打开（升级路径不自锁）

- **WHEN** 对一份无链（v1）账本做一次合法 `append`，再重新打开该账本
- **THEN** 打开成功，`last_seq` 为 2，且账本未变成"部分有链、部分没有"
- **证据**：需补断言（列进 tasks）——本条是 `K-3` 的**修复判据**，
      实现侧今天为 `world-core/src/ledger.rs:506`（`chained` 只读不用）与
      `world-core/src/lib.rs:105`（`load_chain()?;` 丢弃返回值）。
      **⚠ 且修复本身不属本 change**：本 change 只交规格文本，`world-core/` 一行不改
      （`design.md` §排除清单第 2 条）；断言先写、先证红，与修复同批另立 change。

#### Scenario: v1 兼容只到只读为止（边界固定）

- **WHEN** 在修复落地之前，对一份无链账本做一次合法 `append` 后再打开
- **THEN** 打开失败并报 `MixedChain` —— 本断言证明的是**边界的真实形状**，不是实现缺陷；
      它把"v1 兼容"限定为"**只读兼容**"，SHALL NOT 被读成"可平滑升级"
- **证据**：需补断言（列进 tasks）——缺陷出处为
      `world-core/docs/S0-立项/WC-SCMP-001-v0.1.md:2537`（`K-3`）与 `:2541`（后果链）。

### Requirement: 承诺与证据的绑定强度

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `ledger-integrity` 之下，只声明**该既有能力的边界**。
>
> **改的是哪一类问题**：① 措辞写宽（基线自己的验证口径只到"名字存在性"这一级，
> 规格未声明这一点）。
>
> `audit.md` **L9**：`fc-2026-001` 的 `tasks.md:16/17/22`、`design.md:121` 逐字把核对写成
> 「证据行指向真实存在的测试名」「6 份规格 / 40 条证据 / 0 条未命中」。
> **名字存在 ≠ 断言的真是那件事**：本轮六路审计抓到的正是这个差值
> （机械命中 40/40，而语义相符远低于此）。
>
> **证据是哪条测试的哪个断言**：**没有断言**——这正是本条要声明的边界。
> 现行的机核只到存在性一级，且 `openspec/**` 不在受控清单里：
> `world-core/tools/doc_integrity.py` 全文检索 `openspec` **零命中**。

本规格的每一条 Scenario 末尾的证据行 SHALL 指向**真实存在**的测试函数。

系统 SHALL 明确声明的边界：**证据行只到"测试名存在"这一级**，
它 SHALL NOT 被读成"该测试断言的正是本 Scenario 那句话"。
后者的判定 SHALL 由评审承担并留下签字，SHALL NOT 由机械门禁冒充。

#### Scenario: 证据行的存在性可机核、内容相符性不可机核

- **WHEN** 核对某条 Scenario 的证据行所指向的测试函数是否存在
- **THEN** 可机械判定；而当该测试函数的断言与 Scenario 说的不是同一件事时，
      **机械门禁不会变红**（这是本边界的定义）
- **证据**：需补断言（列进 tasks）——`world-core/tools/doc_integrity.py` 的受控清单里
      `openspec` 零命中；本 change 在 `tasks.md` 登记补建 `spec_bridge.py` 的该项检查。
