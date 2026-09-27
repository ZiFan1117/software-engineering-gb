# opsx-swe-gb · OpenSpec × 国标化流程 融合工作流

**这是什么**：一个 OpenSpec 工作流（schema），把 [OpenSpec](https://github.com/Fission-AI/OpenSpec)
的机器可校验产物链，与 `06-swe-gb` 那套国标化流程（S0–S7 / R0–R8 / 三类基线 / 变更八步）
融成**一条**链。落在 `openspec/schemas/opsx-swe-gb/`，项目级 schema 优先级最高，
`openspec new change <name> --schema opsx-swe-gb` 即可用。

**主本**：`D:\Code\10-openspec-swe-gb\schemas\opsx-swe-gb\`（改主本，再同步到这里）。

---

## 一、融合的判据：为什么是"接上"而不是"合并"

两套东西的单位不同，这是融合的支点：

| | OpenSpec | 06-swe-gb |
|---|---|---|
| 单位 | **一次改动**（change） | **一个项目 / 一个阶段** |
| 管 | 这次改动动什么、delta 怎么合、任务几步、归档到哪 | 何时可开始、何时算做完、谁签字、留什么证据、越没越界 |
| 校验器 | `openspec validate`（判**形态**） | `trace_matrix.py` / `scope_check.py` / `ci_self_check.py`（判**语义与责任**） |
| 不管 | 角色、签字、基线、追溯、覆盖率、改动范围 | 目录怎么摆、标题怎么写、delta 怎么合进主规格 |

`06-swe-gb` 的 R4/R5 **本来就是逐次触发的**（附件三第 96–97 行：R4「每个模块完成时」、
R5「每次框架变更申请时」），且它自己声明 R0–R8 编号体系属**【本仓库】工程约定、
不是国标分类**（附件三第 676、690 行）。⇒ **R4/R5 天然是 change 粒度的，接得上。**

**融点只有一个**：一个 change 目录 = 一份变更请求（CR）。同一批文件，两个校验器各读一半，
**不写两遍**。台账只留一行指针。

---

## 二、产物：同址双读

```
openspec/changes/cr-00x-<slug>/
├── .openspec.yaml        ← CLI 生成，不手改
├── proposal.md   ← CR §2 基本信息 + §3.3 变更理由（含「不改的后果」）+ 档位与敏感路径判定
├── specs/<cap>/spec.md   ← 需求增量：### Requirement: REQ-F-012 …，每条带 - **证据**：<测试实名>
├── design.md     ← R5 影响分析 + 回归范围 R-A…R-D + ≥2 方案（含不改案）+ **排除清单**
├── tasks.md      ← 实施步骤，每条自带完成判据
└── review.md     ← 【流程独有】R4/R5 评审记录与签字
```

`review.md` 不影响 `openspec validate`（它只认那四个内置产物名与 delta 语法）；
反过来 OpenSpec 也不会替你生成审批栏。

---

## 三、硬约定：两个评审档位的时机不同

这是本 schema 与普通 OpenSpec 最关键的一处设计，**有依据，不是拍脑袋**：

- **R5（框架变更评审）＝ 前置闸。** 准入是「变更申请单 + 影响分析已完成」，
  而这两样就是 `proposal.md`（含 FC-1…FC-6 触发条件与现象证据）与
  `design.md`（含 ≥2 方案对比、含不改案）。**未获 R5 批准不得进入实施。**
- **R4（模块评审）＝ 后置闸。** 准入是「模块代码已提交（含单元测试），附提交号」
  （附件三第 185 行）——**代码没写完就没什么可审的**。所以在 apply 之后、归档之前填。

因此 `review` 的 `requires` 是 `tasks`，`apply.requires` 只是 `tasks`。
**归档门禁**由两处共同承担：`openspec validate --archived`（tasks 必须全勾）
＋ 流程侧的 `spec_bridge.py`（归档目录必须有 `review.md`）。

---

## 四、实测过的门禁顺序

在一次性沙盒里跑完整条链（`openspec new change … --schema opsx-swe-gb`）：

```
START                    proposal 待造；specs/design/tasks/review 全 blocked
                         apply: state=blocked   missing=[tasks]
after proposal           1/5 ✓
after specs              2/5 ✓（tasks 转由 design 阻塞）
after design             3/5 ✓
after tasks              4/5 ✓
                         apply: state=ready              ← 四件齐了才放行
after tasks ticked       apply: state=all_done
after review             5/5 ✓
archive                  ✓ 归档为 2026-09-27-cr-001-selftest
validate --archived      ✓ 1 passed, 0 failed
```

`openspec schema validate opsx-swe-gb` → **✓ Schema 'opsx-swe-gb' is valid**；
`openspec schemas` 把它列为 `(project)`，链路 `proposal → specs → design → tasks → review`。

---

## 五、还没造的：`spec_bridge.py`

fusion 规矩里那把**归档门禁**目前**不存在**（`06-swe-gb` 也没有）。它要校验四座桥：

| 桥 | 写法 | 判什么 |
|---|---|---|
| 需求 | `### Requirement: REQ-F-012 需求名` | specs ↔ SRS ↔ RTM 三方编号一致 |
| 追溯 | RTM 第 14 列（备注）：`openspec:<capability>#<Requirement 名>` | 同上 |
| 变更 | CR 号三处一致：目录名 `cr-00x-<slug>` / `proposal.md` 首行 / `.scope-declaration.json` 的 `change_request` | 同上 |
| 证据 | `#### Scenario:` 末尾 `- **证据**：<路径>::<测试名>` | **测试实名必须存在** |

外加两条归档门禁：归档目录必须有 `review.md`（含批准人与日期）；
`git diff --diff-filter=MD -- openspec/changes/archive/` 非空即 rc=1（**归档只增不改**）。

---

## 六、为什么这把门禁必须"自己被测"

`06-swe-gb` 自己的 R5 评审记录给出了最贵的教训，逐字：

> 门禁的失败模式不是"报错太多"，而是"**该报错时不报错**"。（附件五第 418 行）

它实测出的自检盲区（评审记录第 115–117 行，三项阻断级）：给 CI 作业加 `if: false`
可停用任意门禁而自检报告 0 问题；掏空作业步骤也不被发现（只校验作业名存在）；
`continue-on-error` 的正则漏报 4 种合法写法。作者自陈"**我不能用'反向测试通过'
来论证门禁可靠**"。

⇒ 写 `spec_bridge.py` 时**必须同时写它的反例测试**：每一座桥都要有一个"改坏了就变红"的
用例。**六道桥，六道反例**，缺一不算完成。

---

## 七、使用

```powershell
# 建 change（融合档）
openspec new change cr-007-add-dark-mode --schema opsx-swe-gb

# 逐件取模板与规矩
openspec instructions proposal --change cr-007-add-dark-mode --json
openspec status   --change cr-007-add-dark-mode
openspec status   --change cr-007-add-dark-mode --json   # 看 artifacts[].requires

# 形态校验（机器判）
openspec validate cr-007-add-dark-mode --strict

# 归档后
openspec validate --archived
```

CR 号由**人**给（流程侧台账登记），agent 不自己编号。
