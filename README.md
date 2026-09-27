# worldcore

> 这是一座「**书 → 规格 → 流程**」三层对齐的仓库：
> **书**讲"说法"这一层应当是什么（`world-core/docs/理论/`，现为**一本合订本**），
> **规格**写这台东西对外承诺什么行为（`openspec/specs/`），
> **流程**记谁在什么时候按什么规矩做的、谁签的字（`world-core/docs/`）。
> 三层冲突时的高下与"谁让"纪律见 **§二**。
>
> **仓库**：`github.com/ZiFan1117/worldcore`（private，**唯一活仓**）。
> 旧仓 `github.com/ZiFan1117/agent-native-os` **已归档**，不再维护。
>
> **本件于 2026-09-27 按实测布局重写**：旧版按 `1-理论与哲学/`、`2-依据/`、`3-备选路线/`、
> `4-计划/`、`00-总纲.md` 那套布局描述，而这些来源件**已退场**（见 **§四**）。
> 本件里的每条路径与每个数字都是**当场实测**的，命令与读数随文给出；
> 实测时点：2026-09-27。⚠️ 仓里有多个工区**并行落件**——本次撰写期间 `main` 自 `50f30ac`
> 推进到 `8bf034d`（`git log -1` 可复核）；下面的数字都在落笔前复跑过，会随并行工区变化的量
> （规格条数、勾选数）按最后一次复跑取值。

---

## 一、仓库一层有什么（实测）

> 命令：仓库根 `Get-ChildItem -Force`；版本状态取自 `git ls-files -- <路径>` 的计数。

| 一层条目 | 一句话 | 入库件数（实测） |
|---|---|---|
| **`world-core/`** | **世界核心**：Rust 实现（`src/`、`tests/`、`ontology.json`、`policy.json`）＋**流程文档**（`docs/`）＋**门禁工具**（`tools/`）＋出厂门禁 `check.sh` | 131 |
| **`openspec/`** | **规格层**：`specs/`（对外承诺）＋`changes/`（在办与归档的改动）＋`schemas/`（融合档 `opsx-swe-gb`）＋`BOOK/`（书的派生工作件）＋`BRIDGE.md`／`MAINTENANCE.md`／`config.yaml` | 63 |
| **`.github/`** | **门禁自身**：`workflows/world-core-gate.yml`（CI 八作业）＋`PULL_REQUEST_TEMPLATE.md`（PR＝一次正式评审的记录） | 2 |
| `agentd/` | **参考实现（Go）**：能力路由＋快照钩子＋结构化审计＋完工铃（见 `agentd/README.md:1`） | 28 |
| `omarchy/` | **上游源码快照**（机制参考，不兼容其生态） | **0**（不入库） |
| `omarchy-pkgs/` | 同上，包构建那一半 | **0**（不入库） |
| `refs/` | **外部参考仓快照** 4 份：`buzz`（5 361 件）／`lively.next-index`（**空目录，0 件**）／`lively4-core`（4 507 件）／`sepa`（3 423 件） | **0**（不入库） |
| `.agents/` | **AI 侧工作流技能**：OpenSpec 的 6 个 `SKILL.md` ＋ `.openspec-target` | 7 |
| `README.md` | 本件（前门） | — |
| `check.sh` | **`agentd` 的单一入口**：`go build` → `go vet` → `go test`，末尾打印 `CHECK_OK`／`CHECK_FAIL`。⚠️ 它**不跑**世界核心的门禁——那是 `world-core/check.sh`（见 **§三**） | — |
| `变更记录.md` | 2026-09-25 那次目录重排的**旧编号对照表**；它描述的正是**已退场**的布局（实测 41 处旧路径引用），保留作史料 | — |
| `.gitattributes` | 行尾与 BOM 纪律：`.sh/.rs/.json/.yml/.yaml/.py/.csv/.md` 等一律 `eol=lf`（`:8`、`:16-23`、`:33`）；`*.ps1` **必须带 UTF-8 BOM**（`:24-28`） | — |
| `.gitignore` | 把上游快照挡在版外：`omarchy/`（`:2`）、`omarchy-pkgs/`（`:3`）、`refs/`（`:4`） | — |

> **`world-core/` 再深一层**（本件用到的三处）：`docs/`＝流程文档、`tools/`＝门禁工具与守卫、
> `src/`＋`tests/`＝Rust 实现与测试；另有 `cap.d/`（能力声明样本）、`deploy/`、`templates/`（七类模板）、
> `NOTICE.md`（许可状态）、`.scope-declaration.json`（改动范围声明）。

---

## 二、三层分工与「§〇 上位规则」

| 层 | 谁（实测） | 它只回答一个问题 | 冲突时 |
|---|---|---|---|
| **1 书（理念）** | `world-core/docs/理论/`——实测**只有 1 件**：合订本 `语义世界-理论书-第一版-合订.md`（433 843 字节／2 572 行／LF／无 BOM；卷首自述"一本把'说法'这一层讲清楚的书"，装配日期 2026-09-27） | 这一层应当是什么、今天做到几分、还剩什么没定 | **书赢** |
| **2 规格（对外承诺）** | `openspec/specs/`——实测 **6 个能力／33 条 `Requirement`**：`channel-identity` 2／`envelope-validation` 6／`gate-enforcement` 7／`ledger-integrity` 8／`projections` 5／`read-model` 5 | 这台东西对外承诺什么行为，每条由哪条会红的测试作证 | 与书冲突 ⇒ **改规格**（除非作者裁定改书） |
| **3 流程（过程证据）** | `world-core/docs/`——实测 `S0-立项` 5／`S1-需求` 5／`S2-设计` 16／`S3-骨架` 2／`S4-实现` 5／`S5-测试` 1／**`S7-交付` 0**、`评审` 9、`阶段外-待启用` 10、`demo` 1、`系统全景图.md`；**无 `S6`** | 谁在什么时候按什么规矩做的、谁签的字 | 与书或规格冲突 ⇒ **改流程文档** |

**§〇 上位规则**（源：`openspec/schemas/README.md` §〇）：**书 > 规格 > 流程**。两条硬规矩：

1. **冲突要写明"谁让"**：任何一次"不按书来"的处置，必须在件里写下**让的是哪一条、为什么让、谁批的**——
   **不写＝违规**。这条 2026-09-27 起已有机器项：`world-core/tools/spec_bridge.py` 的**判据⑦「让路登记」**。
2. **尺子是机抽的摘要，判定冲突必须回原书核逐字**：尺子＝`openspec/BOOK/理念条目.md`；
   台账＝`openspec/BOOK/冲突总账.md`（书 ↔ 规格 ↔ 流程三条边上的冲突逐条登记，带 `path:line`）。
   `openspec/BOOK/` **只放派生工作件，不放书**——**它不是书，不承担结论**。

---

## 三、怎么跑门禁

四条命令，都在**仓库根**跑。前三条已实测，读数如下（实测时点同前）：

| # | 命令 | 管什么 | 今天的读数（rc） |
|---|---|---|---|
| 1 | `openspec validate --all --strict` | **形态**：结构、每个 `Scenario` 恰好 4 个 `#`、delta 语法 | `Totals: 9 passed, 0 failed (9 items)` → **rc=0** |
| 2 | `python world-core/tools/spec_bridge.py` | **规格层守卫七条判据**（见下） | `通过 6 / 失败 1` → **rc=1**（判据⑥ 红） |
| 3 | `python world-core/tools/spec_bridge.py --self-test` | 守卫**自证会红**：七条判据逐条造反例 | 逐条"已红 OK" → **rc=0** |
| 4 | `bash world-core/check.sh` | **出厂门禁 8 步**：① 构建 → ② 骨架冒烟（必须打印 `READY`）→ ③ 三条专属验收（＋③b 契约测试）→ ④ 投影同源 → ⑤ 纯文本审计 → ⑥ 系统级验收 → ⑦ S1 验证面补建 → **⑧ 规格层守卫** | 需 `cargo` ＋ `bash`（Linux／VM 侧），本机 Windows 未实跑 |

> **第 4 条的第 ⑧ 步就是第 2 条**：`world-core/check.sh:160-169` 调 `spec_bridge.py`。
> 也就是说这条守卫**同时**在"一条命令跑通"和 CI 里执行，不是只写在文档里。

### 3.1 七条判据（`world-core/tools/spec_bridge.py`）

`openspec validate` 只判**形态**；下面七条是它的**内容侧补位**，任一不成立即非零退出：

| # | 判据 | 一句话 |
|---|---|---|
| ① | 归档硬前置 | 每个 `openspec/changes/archive/*/` 必须有非空 `review.md` |
| ② | 证据存在性 | 规格里 `- **证据**：<path>::<fn>` 的函数／脚本必须真实存在（改名即失锚） |
| ③ | 默认档守卫 | `openspec/config.yaml` 的 `schema:` 必须是 `opsx-swe-gb`（被改回默认档即失败） |
| ④ | 编号桥覆盖 | `openspec/BRIDGE.md` 必须覆盖规格树下**每一条** `Requirement`（有号或显式标「无号」） |
| ⑤ | 覆盖在册 | 至少一个 `cover-*` change **未归档**且 `tasks.md` 仍有未勾项（未实现的能力要有落点） |
| ⑥ | 归档件的评审已签 | 结论 ∈ 批准／通过／有条件通过，且批准人非空、非占位 |
| ⑦ | 让路登记 | 声明了「谁让」的件必须写全：让哪一条／为什么让／谁批的 |

> ⚠️ **第 2 行 rc=1 是已知红，不是脚本坏了**：唯一红项是判据⑥——
> `openspec/changes/archive/2026-09-27-baseline-verified-doctrine/review.md` 的结论栏是「未签」。
> 台账与处置权在 `openspec/BOOK/冲突总账.md` §五 裁-1（作者指示：**评审通过后**由执行者签署；未过不签）。
> **不要**为了变绿而注释掉它、加 `continue-on-error`、或放宽判据强度——那正是本项目记过的病。

### 3.2 CI（`.github/workflows/world-core-gate.yml`）

- **触发**：`push` 与 `pull_request`（分支 `main`、`develop`）＋ `workflow_dispatch`。
- **八个作业，全部阻断式**：`smoke`／`unit-test`／`gate-self-test`／`traceability`／`scope`／
  `openspec-validate`／`spec-bridge`／`module-graph`。任一失败不予合入。
- **无任何密钥**：只用仓库内文件与公开 CLI。OpenSpec CLI 的版本**钉死**在 `@fission-ai/openspec@1.13.2`
  （与本机实测一致；浮动版本会让"同一次提交、两个结论"）。
- ⚠️ `spec-bridge`（判据⑥）与 `module-graph` **今天是如实红**：见 §三 与 §三.3 的读数与台账。
  门禁自身也被门禁盯着——`world-core/tools/ci_self_check.py` 会扫**全仓**工作流，
  出现 `continue-on-error` 或必需作业缺失即判红。

### 3.3 待建／在建的机核

| 工具 | 是什么 | 今天的状态（实测 2026-09-27） |
|---|---|---|
| `world-core/tools/module_graph.py` | **原子化机核**（`WC-ATOM-001` §二 A-1 单意图／A-4 `deps == import` 且无环／A-2 四件同夹） | **已落到工作区但尚未入库**（`git status` = `?? world-core/tools/module_graph.py`，51 317 字节）；裸跑 `通过 0 / 失败 3`、`--self-test` 正控自己就失败 ⇒ **rc=1 两条**，红是在建状态的如实反映 |

---

## 四、旧引用怎么解析（退场件的**唯一**解析根）

**来源件已退场**：`1-理论与哲学/`、`2-依据/`、`3-备选路线/`、`4-计划/`、`00-总纲.md` 与
`world-core/docs/理论/` 的散件，已按作者指示（2026-09-27：「书只留一本合订本，其他文本可能不需要」）
从本仓移除，**其内容并入合订本** `world-core/docs/理论/语义世界-理论书-第一版-合订.md`。

> ### 旧引用一律解析到**本仓 git 历史**（退场前提交 `bf2eae7` 之前的树）——这是它们唯一的解析根。

实测（命令 → 读数）：

| 事实 | 命令 | 读数 |
|---|---|---|
| 这些路径今天**都不存在** | `Test-Path 1-理论与哲学` 等 5 条 | 全 `False` |
| 退场前它们在（共 **31 篇**） | `git ls-tree -r --name-only bf2eae7 -- <路径>` | `1-理论与哲学` 7 ／ `2-依据` 16 ／ `3-备选路线` 3 ／ `4-计划` 4（＝30 篇）＋ `00-总纲.md` **44 169 字节**（＝31 篇） |
| 理论散件也在（**79 件**） | 同上，`world-core/docs/理论` | 79——合订本入仓后，该目录今天只剩 1 件 |
| **取旧件** | `git show bf2eae7:2-依据/15-世界核心的组成与职责.md` | **rc=0**（反例 `git show bf2eae7:9-不存在的文件.md` → **rc=128**） |

`bf2eae7` = `bf2eae72b0df52f5aec0ce276a0826feac35ef13`（2026-09-27，「fix(book): 书名副其实——书不搬进来，只指原址」）。
同一口径另见 `openspec/schemas/README.md:38-40` 与 `openspec/BOOK/冲突总账.md:172`。

**仍留在文本里的旧引用怎么办**：它们不改写，按上面的解析根去取。实测（`rg`，排除 `refs/`、`omarchy*/`、
`.git/` 与本件）：今天有 **33 个文本件、共 467 处**（`rg -o` 计数）仍在引用退场路径，最多的是
`world-core/docs/理论/…合订.md`（94 处）、`变更记录.md`（41 处，它是那次重排的对照表）、
`world-core/docs/S0-立项/WC-FSR-001-v0.1.md`（32 处）。
**Rust/Go 源码注释里的 `07/2-依据/14`、`07/4-计划/03` 一类引用同理**——`07` 指旧仓的树，也走 git 历史
（`agentd/` 的 Go module 路径仍写 `github.com/ZiFan1117/agent-native-os/agentd`，实测 26 处）。

> **术语表的处置**：旧 README 有一张术语表（"载／存／传／管／显"、"声明即请求"、"语义事件是唯一真相"）。
> 这些说法在今天的合订本里**已查不到**（实测逐一 0 命中，如 `声明即请求` 0、`载、存` 0、`五件事` 0），
> 故本件**不再沿用**——沿用等于把已改的说法写回正文。要术语请读
> `openspec/BOOK/理念条目.md`（机抽的尺子），并回书核逐字。

---

## 五、先读哪一件

| 想干什么 | 读哪一件 |
|---|---|
| 想知道"这一层应当是什么" | 书：`world-core/docs/理论/语义世界-理论书-第一版-合订.md` |
| 想按条目核对、或判一处冲突 | 尺子 `openspec/BOOK/理念条目.md` ＋ 台账 `openspec/BOOK/冲突总账.md` |
| 想知道这台东西对外承诺什么 | `openspec/specs/`（6 个能力）与 `openspec/BRIDGE.md`（承诺 ↔ 流程侧需求号） |
| 想看正在改什么 | `openspec/changes/`：在办 3 件（`cover-unimplemented-capabilities` 未勾 26／`fc-2026-001-openspec-into-cm` 未勾 16·已勾 18／`fc-2026-002-spec-revisions` 未勾 42）＋归档 1 件（`2026-09-27-baseline-verified-doctrine`） |
| 想看开发形态要求 | `world-core/docs/S0-立项/WC-ATOM-001-v0.1.md`（原子化编程：六条约定＋机核清单） |
| 想知道规矩怎么定的 | `openspec/schemas/README.md`（融合档：谁管什么、产物链、评审档位）与 `openspec/MAINTENANCE.md`（规格层维护清单） |
| 要提交改动 | `.github/PULL_REQUEST_TEMPLATE.md`（PR＝评审记录）＋ 跑 §三 的门禁 |
