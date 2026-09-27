# WC-RN-001 版本说明

> 这份文档管**"这一版是什么、怎么跑起来、它今天还不能做什么"**。

## 一、当前版本

产品版本号写在源码里（**唯一出处**）：`world-core/src/event.rs` 的
`pub const WORLD_VERSION: u64 = 1;`，随每条事件落账（`world` 字段）。

怎么读出来（**现取**）：
```
world-core/target/release/world state --json      # 或 VM 上同路径
# 或直接看常量：Select-String world-core\src\event.rs -Pattern WORLD_VERSION
```

## 二、出厂形态（怎么构建、怎么跑）

```
cd world-core
cargo build --locked --release
./target/release/world --help
```
- **依赖锁定**：一律 `--locked`；
- **环境**：Rust `1.98.1`（`cargo`／`rustc` 同版）；**主机没有 Rust 工具链 ⇒ 构建与测试只在 VM**（见 `WC-ST-001` §二）；
- **数据的三个落点**：本体 `ontology.json`、策略 `policy.json`、账本 `ledger.jsonl`（路径由命令行给出）；
- **出厂门禁**：`bash check.sh`（清单与判据见 `WC-AT-001`）。

## 三、今天还不能做的（**如实**）

| 限制 | 在册处 |
|---|---|
| 机核层守卫（`WC-ATOM-001` §四）**仍红**：源码面存在真环 `M04 ↔ M09`（`src/main.rs:622` × `src/channel.rs:52`）等三条真缺陷 | `openspec/BOOK/冲突总账.md`；工具输出逐条列名 |
| 一批"规格已写、断言未写"的条目：转出到 `openspec/changes/fc-2026-004-assertions/`，**未勾完** | 该 change 的 `tasks.md` |
| 一批能力**书要求了、实现未落地**（投递与应答、通道资源边界、家族演进、未知旗标、本体命名空间、`trace` 语义、通告的闸、读模型缺格…） | `openspec/changes/cover-unimplemented-capabilities/tasks.md` |
| `S5/S6/S7` 之外**没有**别的流程侧文档（文档集封闭，见 skill §二） | — |

**这些限制不是"待办的杂事"，是"这一版不能承诺的事"**——把没做到写成做到，是本项目最忌的一条。

## 四、权威在哪（同一件事只有一个出处）

| 问题 | 权威 |
|---|---|
| 世界必须怎样 | `openspec/specs/<能力>/spec.md` |
| 模块的契约（接口、依赖、不变量） | `world-core/docs/S2-设计/WC-IC-001-v0.1.md`（一册） |
| 模块号与源码的对应 | `world-core/docs/S2-设计/WC-MODREG-001-v0.1.md` |
| 规格承诺 ↔ 流程侧需求号 | `openspec/BRIDGE.md`（数值**现算**） |
| 这一轮改了什么、谁判的、谁签的 | `openspec/BOOK/冲突总账.md` |

## 五、这份文档不覆盖的

- **不写"某次发布的变更清单"**——那是 git 历史（提交信息即变更说明）；
- **不复述读数与条数**（会给命令）；
- **不承诺未来**——只写"今天是什么"。
