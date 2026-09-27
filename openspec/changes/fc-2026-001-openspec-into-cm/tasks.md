# Tasks

> **数值口径（2026-09-27 补）**：本文出现的「**23 条承诺／39 条需求**」是**立件时点**的读数（时点见 `design.md` 的时序表，`ef2c9a0`／`21:32:37`）。**现行权威值见 `openspec/BRIDGE.md` §七**：规格承诺 **33 条**、流程侧唯一需求号 **41 个**（该表**现算**，并给复算命令）。按 skill §八「一个事实一个权威载体」，**本件不复述现读数**；历史读数保留，因为它记录的是"当时看到什么"。

> 格式硬约束：每条形如 `- [ ] X.Y 描述`；只有 `x` 算完成。每条自带验收方式。
> **归档门禁要求全勾**；本来就做不完的常设项**不写在这里**（落 `openspec/MAINTENANCE.md`）。
> **前置**：本 change 判为 **R5**。作者已在对话中指示开工（`review.md` §七 有追认登记）；
> **`review.md` 的签字栏仍留人**。

## 1. 默认档与守卫（让这一层的规矩可机核）

- [x] 1.1 `openspec/config.yaml` 第 1 行 `schema: spec-driven` → `schema: opsx-swe-gb`
      **验收（★ 已按评审席④ 的更正改对）**：**新建一个探测 change，看它的 `.openspec.yaml` 是否钉 `opsx-swe-gb`**——
      **不得**用 `openspec status --json` 的 `defaultSchema` 判（实测它读的是 CLI 源码里的常量
      `planning-home.js:4 const REPO_DEFAULT_SCHEMA = 'spec-driven'`，与 `config.yaml` 无关；CLI 回显那句 `with schema 'spec-driven'` 同理）。
      实测：探测 change 的 `.openspec.yaml` ＝ `schema: opsx-swe-gb` ✓（探测件已删）
- [x] 1.2 新增 `world-core/tools/spec_bridge.py`（与 `specs/spec-governance/spec.md` 逐条对应）　**★ 后续追加**：⑥ 归档件的评审已签（`7c7e0b1`）、⑦ 让路登记（`415577d`），⑧⑨ 及此后各条仍继续增加——**判据条数与逐条清单一律以 `--json` 的 `passed`/`failed` 为准，本处不复述条数**
      ① 归档硬前置 ② 证据存在性（**两种形态都查**：`<path>::<fn>` 与 `<path> --self-test`）③ 默认档守卫
      ④ 编号桥覆盖 ⑤ 覆盖在册
      **验收**：`python3 world-core/tools/spec_bridge.py` 逐条列出结论。**★ 读数带时点（M11）**：本条落笔时 ＝ `5 通过 / 0 失败`，rc=0；**此后读数已多次变化（判据增加、红项随登记件状态而变）⇒ 现值一律以当场跑出的 `--json` 的 `passed`/`failed` 为准，本处不再登记任何"现在的读数"**——**验收只认"逐条列结论"这个形态，不认写死的数**
- [x] 1.3 `spec_bridge.py --self-test`：为**每条**判据各造一个反例，反例不变红即判该守卫是装饰　**现状**：**每条判据配一个反例**（反例⑤ 用改名实现、不删夹具）；**判据与反例的条数以 `--self-test` 的输出及 `--json` 的 `passed`/`failed` 为准，本处不复述**
      **验收**：`--self-test` rc=0，**每条反例**逐条打印"已红 OK"、正控（完好沙盒）全绿 ✓（**条数不写死**）
- [x] 1.4 把 `spec_bridge.py` 接进 `world-core/check.sh`（**新增第 ⑧ 步**，不改既有步骤号 ③／③b／④／⑥／⑦ 的含义）
      **验收**：`check.sh` 第 ⑧ 步可 grep 定位；`set -euo pipefail` 下该步失败即整脚本非零退出 ✓
- [x] 1.5 在 VM 内跑一次完整出厂门禁
      **验收**：`ssh world "cd /root/world/world-core && bash check.sh"` **RC=0**，结论行含「规格层守卫」；
      日志 `/tmp/after-bridge.log`；**前置**：`openspec/` 层已同步到 VM（56 件，逐件 `LOCAL = REMOTE`）✓
- [x] 1.6 **改掉融合档里"双读＋一行指针"的写法**（改写判据以本 change 的 `boundary.md` 为准）
      **验收**：主本 `D:\Code\10-openspec-swe-gb\schemas\` ↔ 本仓 `openspec/schemas/` **七件 sha256 一致** ✓；
      检索 `双读`／`融合档` ⇒ 0 命中 ✓（`一行指针` 仅存于否定句）

## 2. 规格层（新能力进主规格）

- [ ] 2.1 `spec-governance` **在归档时**进主规格（`openspec/specs/spec-governance/spec.md` 今天尚不存在）——delta 已在本 change 内就绪；**验收**：`openspec archive fc-2026-001-openspec-into-cm --yes` 之后 `openspec list --specs` 由 6 条变 7 条，且 `validate --all --strict` 仍 rc=0
      **验收**：`openspec list --specs` 由 6 条变 **7 条**，新增条 `requirementCount` = 5
- [x] 2.2 新增 `openspec/MAINTENANCE.md`（规格层自己的维护清单）
      **验收**：文件存在；含"本件不进任何规格树"的声明；第 4.2 步移出的两条在此可检索命中 ✓
- [ ] 2.3 规格层整体校验（与 6.1 同一件事，作 2.1 的验收）
      **验收**：`openspec validate --all --strict` ⇒ 全绿且 `list --specs` 含 `spec-governance`

## 3. 编号桥

- [x] 3.1 复核**五组数**（原文误作「四组数」，2026-09-27 由 7.13 更正）：23 条承诺 / 39 条需求 / 撞号 5 / 无号 5 / 无人认领 17
      **验收（★ 已按评审席② 的 B1 改对）**：判据是"**规格树里现有** `### Requirement` 条数 ＝ 23"——
      **不再写死 23 这个数**，因为 2.1 落地后它会变 28；正确写法是"`openspec/BRIDGE.md` 覆盖规格树下**每一条**"（由判据④ 机核）✓
      实测：`WC-SRS-001` 唯一号 **39** ✓、撞号 **5** ✓、无号 **5** ✓、无人认领 **17** ✓
      （★ 另按席② 的 M1 更正：旧稿写的 `378`／`68` 两处**不可复现**，实测 `REQ-` 出现 **371** 次、RTM **66** 次；
      数值以后一律由 `openspec/BRIDGE.md` 单一承载，其他件只引用不复述）
- [x] 3.2 编号桥落 `openspec/BRIDGE.md`（**长期载体**；change 内的 `mapping.md` 降为该次的历史快照）
      + 登记三组：**无号的 5 条承诺**、**`spec-governance` 5 条无号**、**无人认领的 17 条需求**
      **验收**：判据④ 转绿 ✓（由 `specmap.json` 生成，未手抄）

## 4. 归档遗留件（把红的那一件修绿）

- [x] 4.1 为已归档 change `2026-09-27-baseline-verified-doctrine` 补 `review.md`
      **验收**：结论栏逐字写「语义层未核，已知 46 条」且**未写"通过"**；签字栏留人 ✓
- [x] 4.2 把该 change `tasks.md` §6 两条常设项移出，落 `openspec/MAINTENANCE.md`
      **验收**：原处留痕（移出说明 ＋ 原逐字 ＋ 移出人／依据／时点），**未静默删除** ✓
- [x] 4.3 归档门禁转绿
      **验收**：`openspec validate --archived` 由 `0 passed / 1 failed` 变 **`1 passed / 0 failed`** ✓

## 5. 覆盖 change（未实现的能力在册、可见、不装成已成立）

- [x] 5.1 起草 `cover-unimplemented-capabilities`（**保持不归档**），九件写进 delta ＋ tasks
      **验收**：`openspec validate cover-unimplemented-capabilities --strict` 通过 ✓；其 `tasks.md` **存在未勾项** ✓
      （★ 按席② 的 B3 更正：九件里含 `REQ-F-023/026/027/029/030/031` ＋〔无号〕通告的闸／可逆性判定／声明以外不许落账；
      **四类质量目标 `REQ-N-005…008` 按分工判据归流程侧**，已在 proposal 里登记去向，**不得**当成"对账时丢了"）
- [x] 5.2 判据⑤ 对该 change 转绿，且**反例自证**
      **验收**：`--self-test` 的反例⑤（撤掉 cover-* change）**必须变红** ✓

## 6. 验证与取证（跨多组的整体验证）

- [x] 6.1 规格层：`openspec validate --all --strict` ⇒ **9 passed / 0 failed** ✓
      （★ 按席② 的 C1 更正：**不写"7 项全绿"**——`--all` 的射程是「全部 spec ＋ 全部未归档 change」，项数会随仓内 change 多少而变）
- [x] 6.2 归档层：`openspec validate --archived` ⇒ **1 passed / 0 failed** ✓
- [x] 6.3 **未改动的证明（★ 判据已收窄，原写『零改动』是假勾）**：`openspec/specs/**` 里**除 3 处 `## Purpose` 段外零改动**；`world-core/src/`、`world-core/tests/` **零改动**
      **验收（★ 判据＋实测，读数不许写死）**：判据＝`git diff -U0 bf2eae7 <该轮提交> -- openspec/specs` 的**每个 hunk 都落在 `## Purpose` 段内**（出现 Requirement 级 hunk 即失败）；**实测（`bf2eae7`→`457c954`，2026-09-27 复算）：`3 files changed, 11 insertions(+), 5 deletions(-)`——读数随提交变，以当场复算为准**；另 `git diff --stat fd9a892 HEAD -- world-core/src world-core/tests` **为空**
      （为什么改判据：本轮按「以书为主」更正了三处 Purpose，而原判据写的是「零改动」——**它当时已成假**。对抗席乙 把这条列为最重：一条已勾的假任务是「已知假勾进基线」的入口。）
- [x] 6.4 门禁层：VM 内 `bash check.sh` **RC=0**，含第 ⑧ 步 ✓（**该读数取自判据⑥ 落地之前**）；环境指纹见 `review.md` §五
      **★ 待重跑**：判据⑥ 落地后 `check.sh` 第 ⑧ 步会因归档件未签而 exit 1 ⇒ **签完必须重跑一次并把新读数写进 §五**
- [ ] 6.5 `review.md` 的 **R5 节签字**（人）＋ 实施完成后补 **R4 节签字**（人）——**留人**

## 7. 五席评审发现的整改（本轮新增；每条都有 `path:line` 出处）

> 来源：`fc-2026-001` 的五席独立评审（席① 分工判据／席② OpenSpec 侧／席③ 流程合规／席④ 对抗／席⑤ `opsx-swe-gb` 逐条）。

- [x] 7.1 **`boundary.md` 三处硬伤**（席① S1/S2/S3）：① `review` 与「证据行」被写进"OpenSpec 有的"——它们是**项目自加件**
      （`openspec schemas --json`：原生 `spec-driven` 只有 4 产物，无 `review`），而 `boundary.md:58` 自己说评审归流程 ⇒ **同文自相矛盾**；
      ② `boundary.md:76`「本项目**已落成** `spec_bridge.py --self-test`」——**当日不存在**（现已落成，但当时是事实错误）；
      ③ 把 `validate --archived` 称"门禁"——实测它是**归档后 lint**（`archive --yes` 在无 `review.md`、甚至 0/1 tasks 时仍 rc=0）
      **验收**：三处改正；§一 只留原生四项产物，`review`／证据行移入 §二 并注明"载体落在 change 上"
- [x] 7.2 **`boundary.md` 补漏**（席① S8/S9）：S4 交付物漏「**② 模块代码**」（`阶段流程与交付物.md:106`）、S5 漏「**覆盖率**」（`:129`）；
      H-21 归属夸大（原文只说"能指向命令或测试位置"，见 `:224`；"断言须与声明相符"出自 `评审门禁与检查单.md:272/276` 的反面清单 #5/#9）
      **验收**：逐条补齐并改正出处
- [x] 7.3 **schema 与 README 一致性**（席⑤ P1-3/P1-4、席① B11）：`README.md:66` 写 `R-A…R-E`，而 `schema.yaml:86`／`templates/design.md:26`／
      `templates/review.md:52` 与流程侧权威表（`附件三:265`）**只有 R-A…R-D**
      `README.md:7` 说"项目级 schema 优先级最高"与实测相反（默认档由 `config.yaml` 决定）
      **验收**：`R-E` 删掉或改名；README 改成"默认档由 `config.yaml` 的 `schema:` 决定"；主本同步 ＋ 七件 sha256 一致
      **★ 留痕（假勾，2026-09-27 补记）**：本条原写的判据是「`R-E` **全流程 0 命中**」，**勾上它的时候这句话并不成立**——
      `openspec/changes/fc-2026-002-spec-revisions/design.md:63` 那一行的 `R-E` 仍在（且与同一行的「流程侧没有 R-E」自相矛盾），
      而 `openspec/BOOK/冲突总账.md:209` 已把 M6 记作「**已关**」⇒ **台账说已关、实际没关**。
      **发现者／时间**：**独立评审席·甲**，`2026-09-27`（逐字证据见 `openspec/BOOK/冲突总账.md:303` 第 7 条）。
      **本轮已改**：`R-E` 作为**档名**去除，"出厂判据强度不变"的内容挂到 `R-D` 之下并注明「原名 `R-E`」。
      **判据改成会红的形态**：`grep -rn 'R-E' openspec/` 的命中**只允许**是①否定句（"流程侧没有 R-E"）②历史留痕里的「原名 R-E」；**出现第三个用途即判失败**
- [x] 7.4 **README §四 第 11 行的口径**（席⑤ P0-2）：`validate --archived ✓ 1 passed, 0 failed` 是**沙盒口径**；
      活仓当天是 `0 passed / 1 failed`（那条"永久挡住"的常设项）。现已转绿（`1 passed / 0 failed`）
      **验收**：该行注明口径与时点，或改写成"活仓实测"
- [x] 7.5 **模板与 instruction 六处细部不符**（席⑤ P3-8 / 席① 丙 C1–C6）："首行写变更号"（实为第 3 行）、敏感路径 4 vs 5 条、
      影响分析 5 vs 6 项、"四栏"实列 5 项、结论取值（`驳回` 应限 R5）、tasks 分组（模板另有固定第 3 组）
      **验收**：逐处对齐，主本同步
- [x] 7.6 **模板引用路径不可解析**（席⑤ P2-6 / 席② M6）：`templates/review.md:30/46` 的 `docs/附件/附件三-评审与门禁.md`
      真身在 `D:\Code\06-swe-gb\docs\附件\`；本仓根无 `docs/`
      **验收**：改绝对路径，或在 README 声明解析根
- [x] 7.7 **`review.md` 模板缺三栏**（席③ P0-3/P0-5、席② C3/C4/C5）：**主持人／必参（含测试席位）**、
      **「技术内容已被评审并给出裁定」这条独立准出项**（H-24）、**H-14 追认表**（本 change 的 §七 是临时自造）
      ＋ `评审门禁与检查单.md:352-354` 要求的 **`证据类型`** 与 **`本档不能证明的事`** 两栏
      **验收**：五栏写进模板，主本同步
- [x] 7.8 **本 change 的 R5 准入补齐**（席③ P0-2/席⑤ P2-7）：`proposal.md` 按 **FC-1…FC-6 逐条**给"命中/未命中＋依据句"；
      `review.md` §四 第 1 条**恢复模板原文措辞**（不得改写成"档位判据成立"）
      **验收**：`FC-[1-6]` 在 proposal 里可检索到；§四 与模板逐字一致
- [ ] 7.9 **H-14 追认的合规性**（席③ P0-4）：现登记**不满足**第 6(ii) 条（批准人在场、事前授权本可做到）⇒ 按流程口径属**未授权变更**；
      且缺 `原门禁编号`／`只增强不放宽`／「阶段顺序偏离」三项
      **验收**：在 §七 如实登记为"不合规的自认"，并写明两条出路（作者追认补签／回退）
- [x] 7.10 **代裁与代勾**（席③ P1-7）：`review.md` §四有 4 处 AI 打的 `[x]`、5 处 AI 主张的"不适用"、1 处"无需通知使用方"
      **验收**：改成"建议值 ＋ 各选项代价"，勾选与"适用/不适用"判定留人；加"建议不等于是"一句
- [x] 7.11 **旧"双读"的最强载体未更新**（席④ ⑤）：`D:\Code\.agents\skills\openspec-swe-gb-fusion\SKILL.md`
      与副本 `C:\Users\DIY\.agents\skills\openspec-swe-gb-fusion\SKILL.md`（sha256 同为 `2B601A54…`）里仍写
      「同址双读」「每条 Requirement 标题带 REQ 号」——**它是"问融合怎么做"时真正被加载的那份指令**
      **验收**：两份技能文件按 `boundary.md` 更新，并纳入受控面或登记为带责任人的遗留项
- [ ] 7.12 **`openspec/schemas/**` 的门禁归属**（席③ 无主判据面 (a)）：`world-core/.scope-declaration.json` 的 `allowed` 不含
      `openspec/**`；`scope_check.py` 检索 `openspec` 0 命中；主本在仓库外且无版本控制 ⇒ 决定每个 change 产物形态的那一面**无门禁**
      **验收**：给 `openspec/schemas/**` 指定判据归属（并入判据③ 或单列一条），并写明主本的版本化或对账口径
- [x] 7.13 **本 tasks 文件自己的笔误**（席② L2）：3.1 说"四组数"却列了 5 个
      **已改**：3.1 更正为「五组数」并留痕（`2026-09-27 由 7.13 更正`）。
      **验收**：已在本轮改写时改正

      **★ 已办（2026-09-28，逐处核过 → 五处改、一处本已对齐）**：
      | # | 处 | 改前 | 改后 |
      |---|---|---|---|
      | ① | `schema.yaml` proposal 指令 | 「**首行**写变更号」✗（模板里它在**标题之下的属性表**里） | 「**变更号写在文档头的属性表里**（…**不是"首行"**——首行是标题）」 |
      | ② | 同上·敏感路径 | 指令列 **4 项** ✗ | 补齐为 **5 项**（加 `.scope-declaration.json`，与 `templates/proposal.md` 一致） |
      | ③ | `schema.yaml` design 指令·影响分析 | 指令列 **5 项** ✗ | 补齐为 **6 项**（加「需通知的使用方」，与 `templates/design.md` 一致） |
      | ④ | `schema.yaml` review 指令·必填 | 实列 **5 项**却写「**四栏**」✗ | 改「**五栏**」 |
      | ⑤ | **结论取值** | 任务书说"`驳回` 应限 R5" | **核后本已对齐** ✓：`review` 指令现取逐字已是「**R4 ＝ 通过 / 有条件通过 / 退回；R5 ＝ 批准 / 驳回**」⇒ **过时的是 `README.md` 那条"第 4 个出口待补"警告** ⇒ 已把它改成"已对齐"并说明为什么删 |
      | ⑥ | `schema.yaml` tasks 指令·分组 | 只写「`## 1.` `## 2.` 编号」✗ | 补「**最后一组固定是 `## 3. 验证与取证`**（见 `templates/tasks.md`）——集成检查只放那一组」 |
      **⚠ 验收的另一半"主本同步"：做不到，如实记** —— 主本在**仓库外且无版本控制**（本件 `7.12` 登记的同一事实）
      ⇒ 本笔只对齐了**仓内这一份**；**主本是否同步，本仓无法验证** ⇒ **不得读成"两处已一致"**。
      **下游**：`schema.yaml` 仍可解析（`yaml.safe_load` ⇒ `YAML_OK`）；`spec_bridge.py` **16/0**；`openspec validate --all --strict` **rc=0**。
