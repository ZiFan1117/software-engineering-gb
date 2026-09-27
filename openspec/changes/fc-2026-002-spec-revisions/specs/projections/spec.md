# Spec Delta

## MODIFIED Requirements

### Requirement: 两份读法同源且可当场核对

> **改的是哪一类问题**：④ 与项目文档冲突（**P0 必做项**：把项目自评为"绿而无效／红"的能力
> 写成了"已成立"）兼 ① 措辞写宽（规格写"首行逐字一致"，证据不比对任何内容）。
>
> `audit.md` **P1**：原 `spec.md:18` 的 THEN 是「两份投影首行逐字一致（仅 `projection=` 不同）」，
> 而被引证据只断言 `rc==0` ＋ stdout 含「同源」「✅」；且两份投影**共用同一个 `state` 与 `vocab`**
> ⇒ `assert_same_source` 的三个不等分支在 `project check` 里**结构上不可达**。
> `audit.md` **P2**：原 `spec.md:10-13` 只写正向义务、一字不提边界。
>
> **证据是哪条测试的哪个断言**：
> ① 被引证据的全部内容：`world-core/tests/cli.rs:236-241` 逐字
>    `let (code, out, _) = run(&as_refs(&c));`／`assert_eq!(code, 0, "同源核对应通过");`／
>    `assert!( out.contains("同源") && out.contains("✅"), "应报告同源一致；stdout={out}" );`
>    ⇒ **它没有比对任何两份文本的内容**。
> ② "自己渲染两份、自己比"逐字：`world-core/src/main.rs:408-410` 逐字
>    `let a = language::render(&state, world, vocab);`／`let b = visual::render(&state, world, vocab);`／
>    `match project::assert_same_source(&a, &b) {` —— 两份投影由**同一个 `state`、同一个 `vocab`** 渲染。
> ③ 三个不等分支：`world-core/src/project/mod.rs:126-131`（`ha.world != hb.world`）、
>    `:132-138`（`ha.vocab != hb.vocab`）、`:139-145`（`ha.last_seq != hb.last_seq || ha.state != hb.state`）。
> ④ 项目文档判词：`world-core/docs/理论/语义世界-第五章-今天做到几分.md:108` 逐字
>    「**绿而无效**。命令跑得通（`world-core project check`），只比头部四项（`src/project/mod.rs` 第 124 行），
>    两份还是它自己渲染的（`src/main.rs` 第 394–397 行，2026-09-27 读）。按"能不能证明"判，这一格是红的」。
>    台账 `world-core/docs/理论/WC-PREFACE-LOG-001-v0.5.md:421` 逐字
>    「**绿而无效**（**只比头部四项、不比内容；且它是自己渲染两份再自己比，从未比过两份由不同一方独立生成的投影**——括注已按 §八 的更正改一致，M-25）」。

系统 SHALL 提供语言投影（给程序读）与视觉投影（给人看）两份读法；
两者 SHALL 由**同一读模型与同一词表**派生；
系统 SHALL 提供一条出厂命令当场核对两者首行的**头部四项**一致
（`world` / `vocab` / `last_seq` / `state` 指纹）。

系统 SHALL NOT 把该命令表述为"能证明两份记录说的是同一件事"。

#### Scenario: `project check` 报同源

- **WHEN** 在出厂状态下运行 `world-core project check`
- **THEN** 退出码为 0，stdout 含「同源」与「✅」
- **证据**：`tests/cli.rs::cli05_project_check_reports_same_source`
      —— **⚠ 本断言只到"命令跑通并自报同源"**，既不比对投影内容，也不涉及两份独立来源；
      该命令的判据边界见下一条 `## ADDED`。
      **⚠ 且原证据行括注的「（`check.sh` 步骤 ④）」不成立**：`world-core/check.sh` 全文
      **不跑 `--test cli`**（`:98` 只选 `t1_ t2_ t7_`、`:103` 只跑 `--test contract`）
      ⇒ `cli05` **不在出厂任何一步里执行**（`audit.md` **P3**）；已删该括注，
      并把"让它在出厂路径上可执行或被替代"登记进 `tasks.md`（第 6.6 条）。

#### Scenario: 换词表能被检出

- **WHEN** 在词表变更后比对两份投影
- **THEN** 同源核对能报出不同源（错误串含"词表不同"）
- **证据**：`tests/acceptance.rs::t16_two_projections_are_same_source_and_vocab_change_is_detected`
      —— **⚠ 这是测试自己构造的 `lang_other`**（`world-core/tests/acceptance.rs:718`），
      **`project check` 命令走不到这个分支**（`world-core/src/main.rs:408-409` 两份投影共用同一 `state` 与 `vocab`）。

### Requirement: 语言投影与读模型逐项相等

> **改的是哪一类问题**：① 措辞写宽／自相张力（主句要求"完全相等"，括注说"只能省略"，互相不容；
> 且与 SRS 的 P0 条目口径不一）。
>
> `audit.md` **P5**：原 `spec.md:29-30` 主句"完全相等"与括注"只能省略"互相不容；
> SRS 的口径是**包含关系 P ⊆ S**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs::t14` 断言的是**集合相等**
> （解析出来的集合与 `entries()` 比对），故本规格取"完全相等"这一侧；
> SRS 的 `P ⊆ S` 口径见 `world-core/docs/S1-需求/WC-SRS-001-v0.1.md:92` 逐字
> 「| `REQ-F-024` | 投影不新增事实（只减不增） | P0 | 已实现（集成层：t14②/t15② 断言"解析回来与读模型逐项相等"，相等 ⇒ 不增；系统级判据 TC-042 待实现） |」
> ⇒ **相等强于包含**，两处不冲突；本 change 把"相等"写死，并删除会读成"允许省略"的括注。

语言投影 SHALL 由读模型派生；把投影逐行解析出的三元组集合与读模型逐项比对，
两者 SHALL **完全相等**（既不得添加状态源里没有的事实，也不得省略状态源里已有的事实）。

#### Scenario: 投影与读模型逐项相等

- **WHEN** 把语言投影逐行解析为 `(主体, 路径, 值)` 三元组，与读模型 `entries()` 比对
- **THEN** 两个集合完全相等
- **证据**：`tests/acceptance.rs::t14_language_projection_matches_read_model`

### Requirement: 视觉投影的排版是可审计契约

> **改的是哪一类问题**：① 措辞写宽（只写了 2 条排版规则，审计器实际执行 11 条契约）。
>
> `audit.md` **P6**：原 `spec.md:40-41` 只列「2 空格主体行 / 6 空格字段行」两条。
>
> **证据是哪条测试的哪个断言**：`world-core/tools/visual_layout_audit.py:17-29` 逐条列出契约 1–11，
> 逐字为 `| 1 | 首行是同源头 | 严格匹配 ...`／`| 2 | 第 2 行是标题 | 逐字 世界状态（视觉投影）`／
> `| 3 | 分隔线 | 第 3 行**只由** U+2500 组成，且长度 **= 44**`／`| 4 | 摘要行 | ...`／
> `| 5 | 收尾分隔线 | 非空状态第 5 行同为 44 个 U+2500；**空状态只有 4 行**（无收尾线）`／
> `| 6 | 主体行 | **恰好 2 个空格** ＋ world://…`／
> `| 7 | 字段行 | **恰好 6 个空格** ＋ 路径 = JSON值，且值必须能被 json.loads`／
> `| 8 | 次序 | **任何字段行不得出现在首个主体行之前**`／
> `| 9 | 封闭性 | 全部行**必须被分类**（未分类行数 = 0）`／`| 10 | 无裸控制字符 | ...`／
> `| 11 | 行尾 | 文件以 **LF** 结尾且**无 CR**`。

视觉投影 SHALL 采用**固定排版契约**（不只是缩进两条），该契约 SHALL 至少包含下列 11 条，
且该契约 SHALL 有**独立于渲染实现本身**的审计器逐条检验：

1. 首行是同源头，严格匹配 `^#world-core projection=visual world=<int> vocab=<tok> last_seq=<int> state=<tok>$`（无多余空格）；
2. 第 2 行是标题，逐字 `世界状态（视觉投影）`；
3. 分隔线只由 `U+2500` 组成且长度 **= 44**；
4. 摘要行格式固定；空状态时为固定的一句话；
5. 收尾分隔线同为 44 个 `U+2500`，**空状态只有 4 行**（无收尾线）；
6. 主体行为**恰好 2 个空格** ＋ `world://…`；
7. 字段行为**恰好 6 个空格** ＋ `路径 = JSON值`，且值必须能被 `json.loads`；
8. **任何字段行不得出现在首个主体行之前**；
9. 全部行**必须被分类**（未分类行数 = 0）；
10. 正文不得含裸控制字符；含换行的值必须被转义；
11. 文件以 **LF** 结尾且无 CR。

#### Scenario: 排版审计对变异必报红、对期望样本必常绿

- **WHEN** 用独立审计器检验 3 份字节级期望样本，并施加 12 个变异
- **THEN** 3 份样本全部通过，12 个变异全部报红
- **证据**：`world-core/tools/visual_layout_audit.py --self-test`（由 `world-core/check.sh` 第 ⑦ 步执行）

## ADDED Requirements

### Requirement: 出厂同源命令的判据边界

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `projections` 之下，只声明**该既有能力的边界**——
> 照 `ledger-integrity` 对摘要链 `c18`「整本重写按设计检不出（边界固定）」那条的既有写法，
> **把边界本身写成 Requirement/Scenario，而不是注意事项**。
>
> **改的是哪一类问题**：④ 与项目文档冲突（**P0 必做项**）。这就是 `audit.md` 的 **P1** 与 **P2**。
>
> 项目文档判词：
> ① `world-core/docs/理论/语义世界-第五章-今天做到几分.md:108` 逐字
>    「**绿而无效**。命令跑得通（`world-core project check`），只比头部四项（`src/project/mod.rs` 第 124 行），
>    两份还是它自己渲染的（`src/main.rs` 第 394–397 行，2026-09-27 读）。按"能不能证明"判，这一格是红的」；
> ② `world-core/docs/理论/语义世界-第四章-它怎么落到机器上.md:192` 逐字（引框架 5.2 行）
>    「| 5.2 | 两份记录的一致性核对：只比头部四项，且是自己渲染自己比（**红**） | 实测台账（`src/project/mod.rs` 第 124 行；`src/main.rs` 第 394–397 行） |」；
> ③ 台账 `world-core/docs/理论/WC-PREFACE-LOG-001-v0.5.md:421`（`B-4` 行）逐字
>    「**绿而无效**（**只比头部四项、不比内容；且它是自己渲染两份再自己比，从未比过两份由不同一方独立生成的投影**——括注已按 §八 的更正改一致，M-25）」；
> ④ RTM 记该用例尚未实现：`world-core/docs/S1-需求/WC-RTM-001.csv` 第 21 行逐字
>    「…该用例尚未实现；IF-003a 的编号载体（WC-IC-001）不属本次执行员所有 ⇒ ⚠ 待人工裁定落点。」
>
> **证据是哪条测试的哪个断言**：
> ① 命令实际做的事：`world-core/src/main.rs:408-410`（两份投影共用同一个 `state` 与 `vocab`）；
> ② 判据只有四项：`world-core/src/project/mod.rs:124-145`（`assert_same_source`），
>    其文档注 `:121-123` 逐字「只比较"身份"三项（`world` / `vocab` / `last_seq` + `state` 指纹），/ **不比较排版**」；
> ③ **"该命令恒绿"这一事实今天没有断言**（它是一条负面结论） ⇒ 需补断言（列进 tasks）。

出厂同源命令 `world-core project check` 的判据边界 SHALL 被如实声明为**正式条文**：

① 该命令**自己渲染两份再自己比**——两份投影由**同一个读模型、同一个词表**派生
⇒ `assert_same_source` 的三个不等分支在该命令路径上**结构上不可达**，对任何输入只会报绿；
② 它**只比较头部四项**（`world` / `vocab` / `last_seq` / `state` 指纹），**不比较内容**；
③ 它**从未比过两份由不同一方独立生成的投影**。

系统 SHALL NOT 把该命令表述为"能证明两份记录说的是同一件事"。
跨来源比对（两份由不同一方独立生成）SHALL 被列为**尚未实现**，
其落点 SHALL 由人裁定，SHALL NOT 由本 change 自造编号。

#### Scenario: 出厂命令的判据只有头部四项（边界固定）

- **WHEN** 让两份投影在**内容**上不一致（例如一方少渲一半主体）、而头部四项相同
- **THEN** `project check` **仍然报绿** —— 本断言证明的是**边界**而不是实现缺陷；
      它把该命令的覆盖限定为"同一份输入下的身份一致性核对"
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/project/mod.rs:124-145`；
      文档出处为 `world-core/docs/理论/语义世界-第五章-今天做到几分.md:108`。

#### Scenario: 该命令的三个不等分支在命令路径上不可达（边界固定）

- **WHEN** 检查 `world-core/src/main.rs` 的 `project check` 分支如何构造两份投影
- **THEN** 两份投影取自**同一个 `state` 与同一个 `vocab`** ⇒ 命令路径上不可能出现"世界版本不同／
      词表不同／状态不同"三种不同源
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/main.rs:408-410`
      逐字 `let a = language::render(&state, world, vocab);`／`let b = visual::render(&state, world, vocab);`。

### Requirement: 视觉投影与读模型逐项相等

> **为什么用 ADDED**：这是一条**新的 Requirement 实体**（原规格没有这一条）。
> `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下逐字存在。
> 本节**不新增能力**：它挂在既有能力 `projections` 之下，把**已实现、已测、SRS 已有判据**
> 的行为补进规格——这是"实现有、规格无"的补落点，不是新增能力。
>
> **改的是哪一类问题**：② 措辞写窄（实现有、已测、SRS 有判据，规格里**没有任何条目**）。
>
> `audit.md` **P4**：原 `spec.md:29` 只限定「语言投影 SHALL 由读模型派生」；
> 而 `world-core/tests/acceptance.rs:674-679` 的 `t15` **实断言视觉投影逐项相等**。
>
> **证据是哪条测试的哪个断言**：`world-core/tests/acceptance.rs:674-679` 逐字
> `let parsed = visual::parse(&text).unwrap();`／
> `let expected: Vec<(String, String, serde_json::Value)> = state .entries() .map(|(s, p, v)| (s.to_string(), p.to_string(), v.clone())) .collect();`／
> `assert_eq!(parsed, expected, "视觉投影与读模型必须逐项一致");`
> —— 与 SRS `REQ-F-019` 判据④同款（`world-core/docs/S1-需求/WC-SRS-001-v0.1.md:87`）。
> **⚠ 该断言的验证面独立性有限**：它用**同一模块**的 `visual::parse()` 解析自己渲染的文本；
> 独立验证面是 `world-core/tools/visual_layout_audit.py`（不 import 任何 Rust 代码）。

视觉投影 SHALL 由读模型派生；把视觉投影解析出的 `(主体, 路径, 值)` 三元组集合与读模型逐项比对，
两者 SHALL **完全相等**（视觉投影只能改变画法，不得增删世界状态里的事实）。

#### Scenario: 视觉投影与读模型逐项相等

- **WHEN** 把视觉投影解析为 `(主体, 路径, 值)` 三元组，与读模型 `entries()` 比对
- **THEN** 两个集合完全相等
- **证据**：`tests/acceptance.rs::t15_visual_projection_is_human_readable_yet_auditable`
      —— **⚠ 本断言的解析器与渲染器同模块**（`world-core/src/project/visual.rs::parse()`）；
      独立于渲染实现的验证面由 `world-core/tools/visual_layout_audit.py` 承担。
