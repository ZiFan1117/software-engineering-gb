# opsx-swe-gb · 项目级 OpenSpec 工作流

**这是什么**：一个 OpenSpec 工作流（schema）。产物链**以 OpenSpec 为准**：`proposal → specs ∥ design → tasks → review`；
软件开发流程（国标／国际标准那套）里的**好经验**——影响分析、≥2 方案、评审与签字、需求号追溯——作为**件内栏位与附表**存在。
**不另立流程册、不设"同一批文件两处登记"，也不写"一行指针"。**

落在 `openspec/schemas/opsx-swe-gb/`，项目级 schema 优先级最高，`openspec new change <name> --schema opsx-swe-gb` 即可用。

**主本**：`D:\Code\10-openspec-swe-gb\schemas\`（**改主本，再同步到这里**）。
**主本必须同时含本 README 与 `opsx-swe-gb/` 下六个文件**——缺任何一件即为断链；两处逐文件 sha256 应一致。

---

## 一、判据：谁管什么（三条）

1. **OpenSpec 有的 → 跟 OpenSpec。**
2. **OpenSpec 没有的 → 跟流程。**
3. **两边都有的 → 形态随 OpenSpec、内容随流程。**

### 1.1 OpenSpec 有的（用它）

| 事项 | 载体 |
|---|---|
| 一次改动的产物链 | `proposal → specs ∥ design → tasks → review` |
| 「应当是什么行为」——**需求内容的机读权威载体** | `specs/<能力>/spec.md`（`### Requirement` ＋ `#### Scenario`） |
| delta 语义 | `## ADDED／MODIFIED／REMOVED／RENAMED Requirements` |
| 形态门禁 | `openspec validate`；`validate --archived`（tasks 全勾） |
| 规格基线 | `openspec/specs/`（归档时 delta 合并入册） |
| **承诺 ↔ 测试的绑定** | 每条 Scenario 末尾的 `- **证据**：<path>::<fn>` |
| 改动状态与进度 | `openspec status`／`list`／`show`／`instructions` |

### 1.2 OpenSpec 没有的（跟流程）

| 事项 | 流程侧出处 |
|---|---|
| **阶段与阶段交付物 S0–S7**：可行性研究／开发计划（含裁剪说明）／质量保证计划／配置管理计划／风险清单／SRS／接口需求／**RTM**／HLD／模块号登记表／接口契约／LLD／单元测试记录／**覆盖率报告**／集成与系统测试报告／缺陷清单／验收报告／用户手册／版本说明／**基线标签** | `06-swe-gb/docs/01-流程与阶段/阶段流程与交付物.md` |
| **评审 R0–R8**：立项／需求／框架／骨架／逐模块／框架变更／测试准出／验收／发布——**谁主持、谁必参、准入准出、三种结论形式** | `06-swe-gb/docs/02-评审与门禁/评审门禁与检查单.md` |
| 覆盖率门槛、缺陷分级与准出、回归策略 | 同上 ＋ `附件四-测试与缺陷.md` |
| 全阶段贯穿要求（**AI 不代签**／文档与代码不漂移／承诺可实测／评审意见辩证处置）与硬条款 **H-01…H-26** | `阶段流程与交付物.md` §硬条款速查 |
| 配置管理与版本化、三类基线（需求／框架／产品） | `04-配置与版本/配置管理与版本化.md` |

**这些的载体不落在本 schema 里**。但它们中间**与一次改动直接相关的那几栏**——批准人、执行者/批准者分离声明、准出判据、环境指纹——**写进 `review.md`**：`review.md` 就是 change 的第五件产物，**不需要第二本册子**。

### 1.3 两边都有、深度不同的（最容易出错的一档）

**OpenSpec 管形态，流程管内容。**

| 事项 | OpenSpec 管的（形态） | 流程管的（内容） |
|---|---|---|
| **规格合不合格** | 结构齐、每个 Scenario 恰好 4 个 `#` | **技术内容是否被评审并给出裁定**（H-24：「格式齐备」与「内容已裁定」**分列两条准出，只有前者不算过**） |
| **承诺与实现是否一致** | 证据指向的测试**存在** | **该测试断言的真是那句话**（H-21） |
| **判据可判定性** | 有 WHEN / THEN | 操作定义 ＋ 比对对象 ＋ 期望值/阈值 ＋ **一个反例**（H-03） |
| **验证面独立性** | 无 | **不得用被测实现自身的解析器**；每条判定配真的会失败的反例（H-16） |

⇒ **OpenSpec 的绿只证明"形态对"；"内容对不对"由评审与守卫脚本承担。** 这一条是这套工作流最要紧的分工，不许含糊。

---

## 二、产物：一份产物，两侧各读它需要的那半

```
openspec/changes/<change>/
├── .openspec.yaml        ← CLI 生成，不手改
├── proposal.md   ← 变更请求：为什么改（含「不改的后果」）、改什么、档位与敏感路径
├── specs/<cap>/spec.md   ← 需求增量：### Requirement: REQ-…，每条带 - **证据**：<测试实名>
├── design.md     ← 影响分析 ＋ 回归范围 R-A…R-E ＋ ≥2 方案（含不改案）＋ 排除清单
├── tasks.md      ← 实施步骤，每条自带完成判据
└── review.md     ← 评审记录与签字（R5 前置 / R4 后置）
```

`review.md` 不影响 `openspec validate`（它只认四个内置产物名与 delta 语法）；OpenSpec 也不会替你生成审批栏。

---

## 三、硬约定：两个评审档位的时机不同

- **R5（框架变更评审）＝ 前置闸。** 准入是「变更申请单 ＋ 影响分析已完成」——就是 `proposal.md` 与 `design.md`。**未获 R5 批准不得进入实施。**
- **R4（模块评审）＝ 后置闸。** 准入是「代码已提交（含单元测试）、附提交号」——**代码没写完就没什么可审的**。在 apply 之后、**归档之前**填、签字。

因此 `review` 的 `requires` 是 `tasks`，`apply.requires` 只是 `tasks`。
**归档门禁**由两处共同承担：`openspec validate --archived`（tasks 必须全勾）＋ **`spec_bridge.py`**（判据①：归档目录必须有 `review.md`；判据⑥：归档件的结论栏必须已签、批准人非空）。
⚠ **`openspec archive` 命令本身不拦**（实测：缺 `review.md`、甚至 tasks 未勾，`archive --yes` 仍 `rc=0`）⇒ **归档前/后必须跑一次守卫**，红了就回退补缺或补签。

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

---

## 五、同步规则（受控面）

主本 `D:\Code\10-openspec-swe-gb\schemas\` ↔ 本仓 `openspec/schemas/`。
**改主本，再同步过来**；两处**逐文件 sha256 一致**。

受控面＝本 README ＋ `opsx-swe-gb/` 下六个文件（`schema.yaml` ＋ 五个模板）。
**主本缺任何一件即为断链**——查法：对两处逐文件取哈希对账，缺件与不一致都要报出来。
