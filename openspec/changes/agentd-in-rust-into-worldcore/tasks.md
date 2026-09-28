# Tasks

> **坐标口径**：本件正文里的 `path:line` 是**写下时的 as-of 坐标**；**权威定位子是
> 「命令 ＋ 用例名／步骤名」**——它们**不随行号漂移**。要复现某条证据：先按用例名
> `grep`，再读那一段。
> **每条断言类任务都必须有自己的变异证明**（"改坏哪一行 ⇒ 它变红"）；没有变异点的
> 断言类任务**不算完成**。
> **读数一律现取**：本件不写死任何数（写死必然过期）。

## 1. 逐件落地四个真缺口（**每件一笔**；每笔提交时树必须是绿的）

> 顺序固定：**实现 ＋ 会红断言 ＋ 把该件证据行的「（待补）」改回真证据**，三样在**同一笔**里。
> 为什么：判据② 会核证据行指向的测试是否真实存在——**先落证据行、后落测试**会让树变红。

- [ ] 1.1 **结构化审计留痕**：落 `world-core/src/agent/audit.rs` ＋ `world-core/tests/agent_audit.rs`（`g01`–`g04`），并把 delta 里该条 4 行证据的「（待补）」改回真证据。
      **原子**：`design.md` 原子表第 1 行。
      **断言在哪**：`world-core/tests/agent_audit.rs` 的 `g01_newline_in_value_does_not_split_the_frame`／`g02_file_fallback_is_jsonl_with_timestamp`／`g03_multi_sink_isolates_failures`／`g04_field_names_come_from_the_fixed_set`。
      **变异怎么变红**：把 `src/agent/audit.rs` 的 `sanitize()` 改成**恒返回原值**（不替换换行）⇒ `g01` 必须红（帧的行数会大于字段数）。
      **正控**：把 `Multi::record` 改成"第一个失败就 `return Err`"⇒ `g03` 必须红（证明它真的在核"每一路都调用"），而 `g02`／`g04` 仍绿。
      **验收**：VM 上 `cargo test --locked --test agent_audit` rc=0；本机 `spec_bridge.py` 回到 16/0。

- [ ] 1.2 **行分隔的结构化请求与应答**：落 `world-core/src/agent/protocol.rs` ＋ `world-core/tests/agent_protocol.rs`（`p01`–`p03`），并改回该条 3 行证据。
      **原子**：`design.md` 原子表第 2 行。
      **断言在哪**：`world-core/tests/agent_protocol.rs` 的 `p01_one_request_one_response_in_order`／`p02_bad_bytes_do_not_swallow_the_next_request`／`p03_refusals_are_structured_not_prose`。
      **变异怎么变红**：把 `serve()` 里"解析不了就写一条协议错应答 **并继续读**"改成 **`return Err`（中止）** ⇒ `p02` 必须红（坏字节之后那条好请求收不到应答）。
      **正控**：`p01` 在变异下**仍应绿**（它不经过坏字节路径）——若它一起红，说明变异打宽了。
      **验收**：VM 上 `cargo test --locked --test agent_protocol` rc=0；本机守卫 16/0。

- [ ] 1.3 **完工通告（不另立登记簿）**：落 `world-core/src/agent/completion.rs` ＋ `world-core/tests/agent_completion.rs`（`j01`–`j04`），并改回该条 4 行证据。
      **原子**：`design.md` 原子表第 3 行。
      **断言在哪**：`world-core/tests/agent_completion.rs` 的 `j01_completion_is_a_ledger_notice`／`j02_start_failure_still_leaves_a_notice`／`j03_pending_comes_from_the_ledger_not_a_registry`／`j04_reading_completion_is_idempotent`。
      **变异怎么变红**：把 `completion::pending()` 改成**读一个"登记簿"文件**来回答待办 ⇒ `j03` 必须红（把登记簿删掉后答案就变了）。
      **正控**：`j04` 在变异下**仍应绿**（幂等与登记簿无关）。
      **★ 一条硬纪律**：本模块**不许**出现"登记簿／registry／jobs.json"这类东西——
      逐字依据：`world-core/src/carrier/recover.rs` 头注「现在**那本登记簿是多余的**：
      登记簿要回答的"哪些活还没干完"，由**账本折叠**回答」。
      **验收**：VM 上 `cargo test --locked --test agent_completion` rc=0；本机守卫 16/0。

- [ ] 1.4 **动手前的载体撤销点：可观察断言**：落 `world-core/tests/agent_undo.rs`（`u01`–`u04`），并改回该条 4 行证据。
      **原子**：`design.md` 原子表第 5 行。**本件不改** `src/carrier/providers.rs`（行为已在，差的是断言）。
      **断言在哪**：`world-core/tests/agent_undo.rs` 的 `u01_undo_happens_once_and_after_the_confirmation`／`u02_undo_failure_blocks_the_action`／`u03_no_undo_policy_means_zero_undo_calls`／`u04_carrier_undo_is_not_world_rollback`。
      **变异怎么变红**：把 `src/carrier/providers.rs::execute` 里"需要撤销点则先做、做不成即拒绝"那一段**整段删掉**（只在本地做变异、验完**立刻恢复**，不许提交）⇒ `u01`／`u02` 必须红。
      **正控**：`u03` 在变异下**仍应绿**（不需要撤销的策略本来就不该调）。
      **验收**：VM 上 `cargo test --locked --test agent_undo` rc=0；本机守卫 16/0。

- [ ] 1.5 把 `world-core/src/agent/mod.rs`（头注 ＋ 三行 `pub mod`）与 `world-core/src/lib.rs` 的一行 `pub mod agent;` 落在**第 1.1 笔**里（不要让它们单独成一笔：单独落会让 `cargo` 找不到 `audit.rs`）。

## 2. 文档口径改写（被三句裁定推翻的 5 件 28 处）

> 每一处都留**让路三要素**（让的是哪一条 ＋ 逐字引文／为什么让／**谁批的＝作者**）。
> 三要素**按件整体**核（守卫判据⑦），可以分布在 proposal／design／tasks 里。

- [ ] 2.1 按册 §7.15 的表格逐条复核 28 处（**不许照抄**：逐处用 `git show` 或直接读件确认它**今天还在主张**，不是已被撤回的转引）。
      **验收**：产出一张「件 → 行 → 现读逐字 → 改后逐字 → 让路三要素」的对账表（放本件 §3 的取证记录里）。
- [ ] 2.2 改 `WC-FSR-001`（3 处）。
- [ ] 2.3 改 `WC-SDP-001`（3 处，含 `K-03` 混语言风险项——该风险随裁定消失）。
- [ ] 2.4 改 `WC-SCMP-001`（8 处，含 `REF-07-02` 登记行）。
- [ ] 2.5 改 `WC-IRS-001`（10 处，含「不在本范围内：`agentd`（Go）**内部**实现」那一格）。
- [ ] 2.6 改 `WC-SQAP-001`（4 处，含「明确不覆盖：`agentd` 的内部质量」）。
- [ ] 2.7 **重跑生成链**：`python openspec/tools/gen_specmap.py` → `python openspec/tools/gen_bridge_md.py`。
      **验收**：`spec_bridge.py` 判据⑪ `[OK]`（不重跑 ⇒ 它会红）。
- [ ] 2.8 改过的每份 `.md` 跑 `python world-core/tools/table_width_audit.py <file>`。
      **验收**：无「行宽不符」。
- [ ] 2.9 在册 `openspec/BOOK/冲突总账.md` **登记一条**：生成器补丁改了什么／为什么／反向验证读数／退化保护。
      **验收**：该节逐字含两个反向验证读数（IDENTICAL／CHANGED）。

## 3. 仓根 Go 退场与连带归位

- [ ] 3.1 `agentd/`（Go，8 包）**不再作独立组件** ⇒ 从工作树移除（旧件**留 git 历史**，可回溯比对；本件 `design.md` 的 Migration Plan 给了解析根）。
      **验收**：`git show <退场前提交>:agentd/cmd/agentd/main.go` 能取到内容。
- [ ] 3.2 归位 仓根 `check.sh`（它今天是 `agentd` 的入口）。
- [ ] 3.3 归位 仓根 `README.md`。
- [ ] 3.4 归位 `.gitignore`（去掉 `agentd/` 那一条）。
- [ ] 3.5 归位 `world-core/.scope-declaration.json` 的白名单。
- [ ] 3.6 归位 `REF-07-02` 的登记（与 2.4 同一笔更省事，但**必须逐处核**）。
- [ ] 3.7 归位 CI 的 `working-directory`（`.github/workflows/world-core-gate.yml`）。
      **验收**：`gh run list --repo ZiFan1117/worldcore` 的下一笔为 success。

## 4. 验证与取证

- [ ] 4.1 VM 上取 4 件的会红读数与变异证明：`cd /root/world/world-core && CARGO_TARGET_DIR=/root/t-tc47 cargo test --locked`（先 `df -h /tmp` 确认未满）。
      **读数四要素**：命令 ＋ 原始输出 ＋ 时点 ＋ 对象（提交号）。
      ⚠ 跑度量前**先读件里记的那条命令**（本组第 4.1 条就是它），不要按惯用命令另跑一遍——实测那条会出 8 倍差。
- [ ] 4.2 VM 上跑 `bash check.sh`（既有 13 步），逐条读数入档。
- [ ] 4.3 本机三条门禁：`python world-core/tools/spec_bridge.py`、`npx --yes @fission-ai/openspec@1.13.2 validate --all --strict`、`python world-core/tools/table_width_audit.py <改过的 .md>`；另跑 `python openspec/tools/gen_specmap.py` 后的 `specmap.json` 未变（证明我没改规格树）。
- [ ] 4.4 归档后复跑 `npx --yes @fission-ai/openspec@1.13.2 validate --archived` 与 `python world-core/tools/spec_bridge.py`。
- [ ] 4.5 **独立评审席**：另派一个子智能体（要求它独立复跑、用 `git show <sha>:<path>` 读件而不读工作区、给**可逐字转录**的判定语、并列出"未核"与它自己的错）。
      **验收**：`review.md` 里逐字记下它的判定语与它列出的"未核"清单。
- [ ] 4.6 把**待人三格**（变更号／能力面挂法／`WC-IC-001` 是否纳入）与**未做项**逐条写进 `review.md` 的「本档不能证明的事」。
