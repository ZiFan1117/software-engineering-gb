# WC-ATOM-001 · 原子化编程约定（项目要求）

| 项 | 值 |
|---|---|
| 文件 | `WC-ATOM-001-v0.1.md` |
| 版本 | v0.1（2026-09-27 首次确立） |
| 状态 | **生效**——本项目**模块开发一律遵循本件** |
| 上位 | 《语义世界》（书的理念）｜本件是**工程约定**，不与书冲突；书不规定开发形态 |
| 参照物 | `D:\Code\03-atom-market\software-atom-market`（软件原子市场：`ATOMIZATION.md` ＋ `spec/atom.schema.json` ＋ `spec/detail-convention.md`） |
| 出处 | 作者 2026-09-27 指示：「原子化编程我觉得也应该作为我们那个编程的一个要求」 |

---

## 一、为什么立这一条

参照仓自己的判词（`ATOMIZATION.md:116` 逐字）：「**同一协议曾有多套判定**：校验器、目录生成器、联邦发现器、插件 `atom_validate` 各写一份等价逻辑，改一处就会漂移。现在真源唯一。」

本项目有同款风险：同一个约定在规格、流程文档、测试、工具脚本里各写一份。**原子化的目的就是把"一个意图"收成一个自包含单元，让它的契约、实现、测试同夹，且依赖与生成物都由机器断言。**

## 二、六条约定（**可机核**，逐条给判据）

| # | 约定 | 判据（怎么机核） | 对齐的参照原文 |
|---|---|---|---|
| **A-1** | **单意图原子性**：一个模块＝一个**一句话说得清**的意图；说不清的拆 | `design.md` 的原子表里每个原子必须有且只有一句 `intent`；出现并列两事（"与／和／及"）⇒ 拆。**（"≤30 字"是本项目的自定阈值，参照仓无此数——`spec/atom.schema.json` 对 `intent` 只有 `minLength: 2`，不许把它说成参照仓的要求）** | 参照仓 `software-atom-market/README.md:13` 逐字「**Single-intent atomicity / 单意图原子性** — one atom = a capability you can describe in one intent-sentence with no implementation detail; bigger → split, smaller → merge」 |
| **A-2** | **一个原子一个文件夹，四件同夹**：契约文档 ＋ 实现 ＋ 原子级测试（＋机器可读边车，若为生成物） | 每 `world-core/src/<mNN>/`（或 `src/**/<atom>/`）下：实现、`tests/` 里的原子级测试、契约条目三者齐备 | `ATOMIZATION.md:11`「一个原子一个文件夹：`<id>.atom.md` ＋ `detail.json` ＋ `impl/` ＋ `tests/`」 |
| **A-3** | **契约字段齐**：`intent / input / output / side_effects` 必写；不写副作用＝声明"无副作用" | 契约表逐列非空；`side_effects` 允许写"无"，但不许留空 | `spec/atom.schema.json` 必填 `id, layer, version, intent, description, input, output` ＋ 可选 `side_effects` |
| **A-4** | **依赖单向 DAG，且 `deps` 必须等于真实 import** | 机器断言：声明的依赖集 ≡ 代码里真实 import/use 的兄弟模块集；有向图无环 | `ATOMIZATION.md:32`「依赖是单向 DAG，且 `deps` 必须等于 `impl/` 里真实 import 的兄弟原子」 |
| **A-5** | **生成物不许手编**：改了源必须重跑生成器 | 生成物带头部声明"生成物·勿手编"；改动源后未重跑 ⇒ 生成物闸红 | `ATOMIZATION.md:80`「改实现只动 `impl/`；`detail.json` 是生成物，跑 `npm run atoms:details` 而不是手编」 |
| **A-6** | **编码一律 UTF-8 无 BOM**，且有编码闸 | 现存判据：`tools/plain_text_audit.py`（本项目已要求纯文本 UTF-8、无 NUL）；BOM 纳入同一闸 | `ATOMIZATION.md:84`「编码一律 UTF-8 无 BOM；编码闸会拦非法 UTF-8 字节」 |

> **五条机器断言的移植**（参照仓的五条 ＝ `deps == import`、`detail.json == .atom.md`、`CATALOG == 重算`、`spec schema == 代码常量`、`编码合法`）：
> 本项目对应关系见 §四 的对照表。

## 三、落到本项目的形态

| 参照仓 | 本项目 |
|---|---|
| `atoms/<id>/<id>.atom.md`（契约文档） | 规格条目（`openspec/specs/<cap>/spec.md` 的一条 Requirement）＋ `world-core/docs/S2-设计/` 的模块接口契约（`WC-IC-M*`） |
| `atoms/<id>/impl/` | `world-core/src/<mNN>/*.rs`（**唯一真源**） |
| `atoms/<id>/tests/` | `world-core/tests/*.rs` 里该原子的用例（**每条承诺都要有会红的断言**） |
| `detail.json`（生成物边车） | `WC-MODREG-001` 模块注册表（**生成物，勿手编**） |
| `deps == import` 闸 | 待建：`world-core/tools/module_graph.py --check`（**本件确立要求，实现列进 `cover-unimplemented-capabilities`**） |
| `npm run gate` 全闸 | `world-core/check.sh`（已有 8 步）＋ `spec_bridge.py`（规格层 6 条判据） |
| `_shared/CONTRACT.md` 跨原子公共契约 | `world-core/docs/S2-设计/WC-IC-001`（接口契约总册）＋ `openspec/schemas/` 的融合档公约 |

## 四、机核清单（**这一条要求必须能红**）

| # | 断言 | 今天的状态 | 落点 |
|---|---|---|---|
| 1 | 每个原子有且只有一句 `intent` | **未建** | `cover-unimplemented-capabilities` tasks **第 12 组**（本件落笔时新建；见该件 `tasks.md`） |
| 2 | 每原子的实现／测试／契约三件齐备 | 部分（测试在 `tests/*.rs`，与 src 不同夹） | 同上 |
| 3 | `deps == import` 且无环 | **在建**：`world-core/tools/module_graph.py` 已落盘（2026-09-27），裸跑 `通过 0 / 失败 3`（含"依赖图有环：M05 → M05"），`--self-test` 正控自己也失败 ⇒ **尚未可用**，**尚未入库** | 同件第 12 组 ＋ 该工具自证转绿 |
| 4 | 生成物与源一致（`WC-MODREG-001`） | 未建闸 | 同上 |
| 5 | 编码 UTF-8 无 BOM | **半建**：非法 UTF-8／NUL／控制字符**已拦**（`world-core/tools/plain_text_audit.py`）；**BOM 未拦**——实测 `audit_bytes(b"\xef\xbb\xbfhello\n")` 返回 `(True,'ok')`，该脚本只在 UTF-16LE BOM 上因"非法 UTF-8"顺带报错 ⇒ **BOM 这条今天没有执行者** | 落点：本件 §四 第 5 条改造 `plain_text_audit.py`（加一条 BOM 判据＋反例），**不进 `cover-*`**（它属工具改造，见 `WC-ATOM-001` 的原子侧落点） |

> **不写"做了"就是没做**：上表第 1–4 条今天**没有执行者**，故本件不声明它们已生效——按本项目规矩（书 L5/L6 的分界）它们是"**规格已定、只差做到**"，进覆盖 change 的 `## L5 覆盖边界`。

## 五、与"模块化编程本体论"的关系（两个参照各管一半）

| 参照 | 管什么 | 用在哪 |
|---|---|---|
| **软件原子市场**（`03-atom-market`） | **怎么切、怎么拼、怎么机核**（本件 §二 六条） | 本件＝开发形态要求 |
| **本体研究**（`04-research-ontology`：`Palantir本体-总览与构件.md` ＋ `operational-ontology` 参考实现） | **"本体"这一层该有什么**（Object／Link／**Action Type**／Functions／Interfaces／Roles／Action Log／Undo） | 喂 `REQ-F-030` 本体能力与 `concepts` 实体层；映射表见 `cover-unimplemented-capabilities/design.md` |

---

**修订记录**

| 版本 | 日期 | 改了什么 | 依据 |
|---|---|---|---|
| v0.1 | 2026-09-27 | 首次确立：六条约定 ＋ 机核清单 ＋ 两个参照的分工 | 作者指示（原子化编程作为项目要求）；参照 `software-atom-market/ATOMIZATION.md`、`spec/atom.schema.json` |
