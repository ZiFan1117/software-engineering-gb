# read-model Specification

## Purpose
规定"读法可以有若干份，但说法只有一份"这一层对外可核的行为：读模型必须能由账本百分之百重算、
检查点只是缓存可以随时丢弃。**它不是"唯一"的自动化检验面**——另有**摘要链**一条独立检验
（`world-core/tests/contract.rs` 的 `c19`／`c20`／`c21`，由 `check.sh` 整跑 `--test contract` 执行）。
⚠ **不把"投影同源"算作这一层的检验面**：它**只证同源、证不了同一件事**（见 `projections` 能力，书第五章判它红）。
⚠ **措辞出处**：「**唯一**自动化检验」这句原出自**流程侧** `WC-SQAP-001-v0.1.md:536`，**不是书的原话**
（书里检索 `一个真相`／`检验面` 均 0 命中）；本能力的更正依据是 `fc-2026-002-spec-revisions` 里**待并入**的条文。

## Requirements

### Requirement: 读模型是可丢弃的缓存

> **改的是哪一类问题**：④ 与项目文档冲突（把"账本内容损坏 ⇒ 拒绝产出"这条**全称**写法
> 收窄到实测的三类）兼 ① 措辞写宽。
>
> `audit.md` **R2**：原 `spec.md:22-23` 用全称量词，而**末尾半行是静默丢弃**的，
> 且 `t3_partial_line_is_discarded` 正面背书该行为。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs::t8_read_model_refuses_broken_ledger`
> 只断言**三类**：`:298-299`（seq 从 2 开始 ⇒ 含"不连续"）、`:316-317`（旧值说谎 ⇒ 含"旧值不符"）、
> `:321-322`（未知家族 ⇒ 含"未知事件家族"）。它**不覆盖**：解析失败的行、末尾半行、摘要链不符。
> 实现侧：`world-core/src/ledger.rs:276-279` 在折叠**之前**就把文件截到最后一个 `\n`
> ⇒ 读模型**根本没有机会**看到那次截断。另 `world-core/src/main.rs:389-395` 逐字
> `let state = match w.read_model() { Ok(s) => s, Err(e) => { eprintln!("[FAIL] 读模型拒绝折叠：{e}"); return ExitCode::from(2); } };`
> ⇒ CLI 侧确实把拒绝转成 `rc=2`。

"现在是什么样" SHALL 由账本重算得出，SHALL NOT 作为与账本并行的第二份原件存在。
删掉读模型（及其持久形态）之后 SHALL 能从账本重算出完全一致的结果。

系统 SHALL NOT 声称"账本内容损坏一律拒绝产出"这一全称命题。
已实测的拒绝面 SHALL 被逐类写明：① 序号断裂、② 旧值说谎（`change` 的 `before` 与当前值不符）、
③ 未知事件家族。
**边界 SHALL 被如实声明**：末尾半行在进入折叠**之前**就已被账本层截掉
（`world-core/src/ledger.rs:276-289`），故它**不是**读模型的拒绝面；
读模型对"解析失败的行"不承担拒绝责任。

#### Scenario: 删掉读模型后重算结果一致

- **WHEN** 记录当前读模型，删除其持久形态，再由账本重算
- **THEN** 重算结果与删除前逐项一致
- **证据**：`tests/acceptance.rs::t7_read_model_is_disposable_and_reproducible`
      —— **⚠ 原证据行括注的"（`check.sh` 步骤 ③）"写法有歧义**：仓里有两个同名脚本，
      仓根 `check.sh` 完全不碰 `world-core`（`audit.md` **L8** 后半）
      ⇒ 本 change 一律写全 **`world-core/check.sh`** 并注明步骤号；
      本节该步确实执行 `t7`（`world-core/check.sh:98` 逐字
      `cargo test --locked --test acceptance -- t1_ t2_ t7_`）。

#### Scenario: 坏账本不得产出"看起来正常"的读模型

- **WHEN** 账本内容损坏（序号断裂／旧值说谎／未知事件家族）
- **THEN** 读模型拒绝产出，而不是给出部分结果
- **证据**：`tests/acceptance.rs::t8_read_model_refuses_broken_ledger`
      —— **⚠ 本证据今天只覆盖上述三类**；"末尾半行"由账本层在折叠前截掉，**不属于**本拒绝面。

### Requirement: 增量折叠等价于全量折叠

> **改的是哪一类问题**：③ 证据错位（`t8`/`c06` 用 `State::fold` 直喂向量，
> 使 `SeqGap` 检查在生产路径上不可达）。`audit.md` **R10** 判它**不是缺陷**（防线冗余），
> 但读者会误以为是端到端验证 ⇒ 本条**只补注边界**，**不改结论强度**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/contract.rs::c06_incremental_apply_equals_full_fold`
> （由 `world-core/check.sh` 第 ③b 步执行）；边界侧 `world-core/src/ledger.rs:304` 逐字
> `if seq != last + 1 {` —— 账本层**更早**拦截缺号。

系统 SHALL 保证"逐条增量应用"与"从头全量折叠"得到同一个结果，
否则读模型就成了一本会漂移的第二本账。

#### Scenario: 增量与全量结果相同

- **WHEN** 对同一批事件分别做增量应用与全量折叠
- **THEN** 两者结果相同
- **证据**：`tests/contract.rs::c06_incremental_apply_equals_full_fold`（由 `world-core/check.sh` 第 ③b 步执行）
      —— **⚠ 本条是防线冗余的**：生产路径上缺号由账本层 `world-core/src/ledger.rs:304` 先拒，
      故本断言**不构成**端到端证明。

### Requirement: 检查点是缓存，可丢弃且必须自洽

> **改的是哪一类问题**：① 措辞写宽（"自洽"在规格里**无定义**、在断言里**无落点**）
> 兼 ③ 证据错位（补上"核验只查两项"与"CLI 走未核验续算路径"这两个事实）。
>
> `audit.md` **R3**：原 `spec.md:37` 标题承诺"自洽"，正文与两条 Scenario 零定义；
> `checkpoint.rs` 的 `verify` **只查 `base_seq` 与 `digest`**。
> `audit.md` **R4**：`world-core/src/main.rs:534-542` 的 `resume` 直接走
> `read_model_with_checkpoint`，其内部快路径是 `resume_unverified`。
> `audit.md` **R5**：`Checkpoint::FORMAT` 的版本不符分支存在但**零断言**。
>
> **证据是哪条测试的哪个断言**：
> ① 核验范围：`world-core/src/checkpoint.rs:122-127` 逐字
>    `pub fn verify(&self, ledger_events: &[Value]) -> Result<(), String> { let prefix: Vec<Value> = ledger_events .iter() .filter(|e| e.get("seq").and_then(Value::as_u64).unwrap_or(0) <= self.base_seq) .cloned() .collect();`
>    ⇒ 判据就是"前 `base_seq` 条重算后的指纹是否等于快照自称的 `digest`"，**无更多"自洽"定义**。
> ② 未核验路径：`world-core/src/checkpoint.rs:171-174` 逐字
>    `match checkpoint { None => State::fold(ledger_events), Some(cp) => cp.resume_unverified(ledger_events), }`
>    —— **`Some` 分支走的是 `resume_unverified`，不调 `verify`**。
> ③ CLI 侧确有一道**事后**比对：`world-core/src/main.rs:556-562` 逐字
>    `if fast.to_json() != full.to_json() { eprintln!( "ext.world.Checkpoint.ResumeMismatch: 从快照续算与全量重算结果不一致——缓存成了第二真相，拒绝使用" ); return ExitCode::from(2); }`
>    ⇒ **比对发生在"用完快照之后"**，不是使用前核验；它与 `world-core/src/checkpoint.rs:122` 的
>    `verify` 是两条不同判据。
> ④ 版本分支：`world-core/src/checkpoint.rs:94-99` 逐字
>    `if fmt != Self::FORMAT { return Err(format!( "ext.world.Checkpoint.BadFormat: 期望 {}，实得 {fmt}", Self::FORMAT )); }`
>    —— 该分支**没有任何测试触发** ⇒ 需补断言（列进 tasks）。
> ⑤ 断言侧：`world-core/tests/contract.rs::c12_checkpoint_is_a_cache_and_disposable` 与
>    `world-core/tests/contract.rs::c13_bad_checkpoints_are_refused`。

检查点 SHALL 只作为重放的加速形态存在；它 SHALL 可被丢弃而不影响正确性；
坏检查点 SHALL 被拒绝而不是被静默采用。

系统 SHALL 明确声明"自洽"的可判定含义，SHALL NOT 留作未定义词：
检查点的**核验**判据只有两项——① 快照自称的 `base_seq` 能对上账本前若干条；
② 用账本重算前 `base_seq` 条得到的指纹等于快照自称的 `digest`。

系统 SHALL 另外声明：CLI 的 `checkpoint resume` 走的是**未核验**续算路径，
其"缓存未成为第二真相"由**事后**的"续算 ≡ 全量"逐字节比对承担
（不一致即 `ResumeMismatch` 拒用），**不是**由使用前的 `verify` 承担。

检查点格式版本不符时 SHALL 拒绝使用；该约束 SHALL 有断言。

#### Scenario: 检查点可丢弃

- **WHEN** 生成检查点后再丢弃它，由账本重新重放
- **THEN** 结果与使用检查点时一致
- **证据**：`tests/contract.rs::c12_checkpoint_is_a_cache_and_disposable`

#### Scenario: 坏检查点被拒

- **WHEN** 检查点内容被破坏
- **THEN** 被拒绝
- **证据**：`tests/contract.rs::c13_bad_checkpoints_are_refused`
      —— **⚠ 其覆盖面不含"格式版本不符"这一分支**（`world-core/src/checkpoint.rs:94-99`）
      ⇒ 需补断言（列进 tasks）。

### Requirement: 读模型是"一个真相"的检验面之一

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `read-model` 之下，只声明**该既有能力的边界**。
>
> **改的是哪一类问题**：④ 与项目文档冲突（Purpose 自称"唯一自动化检验所在"，与同仓文档冲突）
> 兼 ④（三份受控文档仍写「v1 CLI 从不读快照」，与已接线的实现冲突）。
>
> `audit.md` **R7**：原 `spec.md:4-5` 的 Purpose 写「这一能力是"一个真相"这条纪律的
> **唯一**自动化检验所在」，而同仓另有两条独立检验。
> `audit.md` **R1**：三份受控文档仍断言不存在检查点路径，而 `world-core/src/main.rs` 已把它接进生产 CLI。
> `audit.md` **R9**：证据链无机械门禁。
>
> **证据是哪条测试的哪个断言**：
> ① 摘要链核验是另一条独立检验：`world-core/tests/contract.rs::c19_written_ledger_carries_a_verifiable_chain`、
>    `world-core/tests/contract.rs::c20_startup_refuses_a_tampered_ledger`、
>    `world-core/tests/contract.rs::c21_is_chained_reflects_reality`、
>    `world-core/tests/cli.rs::cli02_after_append_check_reports_chained`。
> ② 投影同源核验是第三条：`world-core/tests/acceptance.rs::t16_two_projections_are_same_source_and_vocab_change_is_detected`。
> ③ 检查点 CLI **已接线**：`world-core/src/main.rs:28-30` 逐字
>    `  checkpoint write <path>    从账本重算状态并写一份检查点（缓存，非真相）`／
>    `  checkpoint verify <path>   核验检查点与账本一致（不一致即拒用）`／
>    `  checkpoint resume <path>   从检查点续算，并与全量重算逐字节比对`；
>    分发处 `world-core/src/main.rs:153` 逐字
>    `        "checkpoint" => cmd_checkpoint(&ontology, &ledger, &policy, &rest),`
>    ⇒ 与 `world-core/docs/S4-实现/WC-UT-001-v0.1.md:54` 逐字
>    「**且 v1 运行时/CLI 从不读快照**（`src/lib.rs` 只有 `pub mod checkpoint;`；`main.rs::cmd_state` 走 `World::read_model()` 全量折叠；`read_model_with_checkpoint` 调用者全在 `tests/` 内）」
>    **正面冲突**；同族冲突另见 `world-core/docs/S1-需求/WC-SRS-001-v0.1.md:921` 逐字
>    「**操作对象（唯一可核观测面）**：v1 的 CLI **不暴露检查点路径**」。
> ④ **"用法串里列出三条 checkpoint 子命令"今天没有断言** ⇒ 需补断言（列进 tasks）。

读模型 SHALL 是可丢弃的缓存，"现在是什么样"由账本重算得出。

本能力 SHALL NOT 被表述为"一个真相"这条纪律的**唯一**自动化检验面：
同仓另有至少两条独立检验——摘要链核验（`c19`/`c20`/`c21`/`cli02`）与投影同源核对（`t16`）。

系统 SHALL 声明检查点路径**已接入生产 CLI**（`checkpoint write|verify|resume`），
SHALL NOT 被表述为"v1 CLI 从不读快照"。

证据链的机械门禁 SHALL 声明其覆盖边界：`world-core/tools/doc_integrity.py` 的受控清单里
`openspec/**` **零命中**，故"改一个测试名，规格不会变红"。
该缺口 SHALL 由 `spec_bridge.py` 的对应检查承担，SHALL NOT 被读成"证据链已被机核"。

#### Scenario: 检查点路径已接入生产 CLI

- **WHEN** 查阅 CLI 的用法串与分发分支
- **THEN** 其中列出 `checkpoint write` / `checkpoint verify` / `checkpoint resume` 三条子命令，
      且 `checkpoint` 分支在 CLI 分发表里有对应项
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/main.rs`:28-30 与 `world-core/src/main.rs`:153；
      三份仍写相反陈述的受控文档为 `world-core/docs/S4-实现/WC-UT-001-v0.1.md:54`、
      `world-core/docs/S1-需求/WC-SRS-001-v0.1.md:921`、`world-core/docs/S1-需求/WC-RTM-001.csv` 第 22 行。

#### Scenario: 证据链的机械门禁不覆盖 openspec

- **WHEN** 改动 `openspec/specs/` 下某条 Scenario 的证据行所指的测试名
- **THEN** 现有的受控文档一致性脚本**不会变红**（`openspec` 在其受控清单里零命中）
- **证据**：需补断言（列进 tasks）——`world-core/tools/doc_integrity.py` 全文检索 openspec 零命中。


### Requirement: 带检查点路径的性能目标不在本基线的承诺范围内

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `read-model` 之下，只声明**该既有能力的范围外声明**。
>
> **改的是哪一类问题**：④ 与项目文档冲突（性能目标被 SRS 记"未实现"，规格沉默 ⇒ 会被读成已成立）。
>
> `audit.md` **R8**：`REQ-N-008`（带检查点路径的性能目标）**无任何断言**。
>
> **证据是哪条测试的哪个断言**：
> ① `world-core/tests/perf.rs` 三个用例**全部 `#[ignore]`**——`:175-177` 逐字
>    `#[test]`／`#[ignore = "L3 度量：用 cargo test --release --test perf -- --ignored --nocapture 显式运行"]`／
>    `fn qg05_full_replay_of_100k_events() {`，另有 `:219`、`:284` 同样标 `#[ignore]`；
>    且**无一条测"带检查点续算"**。
> ② SRS 状态：`world-core/docs/S1-需求/WC-SRS-001-v0.1.md:112` 逐字
>    「| `REQ-N-008` | 带检查点路径的性能目标（QG-13） | P2 | 未实现（性能读数未取：v1 CLI 不暴露检查点路径，样本 ≥ 20 次的 P95 无法在 S1 取；…
> ③ RTM 同记：`world-core/docs/S1-需求/WC-RTM-001.csv` 第 40 行状态列为「未实现」。
> ⇒ **本条今天没有断言**（"未实现"本身不可被断言） ⇒ 需补断言（列进 tasks）。

系统 SHALL 明确声明**范围外**事项，SHALL NOT 被读成已成立：
带检查点路径的性能目标（`REQ-N-008`：走检查点续算 ≤ 全量重算的 1/2，样本 ≥ 20 次取 P95）
在本基线内**未实现、无断言**；`world-core/tests/perf.rs` 的全部用例默认 `#[ignore]`，
只在显式度量时运行。

#### Scenario: 性能目标不在本基线的承诺范围内

- **WHEN** 查阅本能力的覆盖声明
- **THEN** 写明 `REQ-N-008` 未实现、无断言，且 `world-core/tests/perf.rs` 三条用例默认不跑
- **证据**：`world-core/docs/S1-需求/WC-SRS-001-v0.1.md`:112（状态列"未实现"）
      —— 本条**今天没有断言**（"未实现"不可被断言），其载体是流程侧登记与本 change 的 `tasks.md` 范围声明。
