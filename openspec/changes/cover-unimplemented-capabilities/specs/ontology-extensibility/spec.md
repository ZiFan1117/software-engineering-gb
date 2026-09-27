# Spec Delta

## Purpose

规定"本体怎么长大而不把核心改坏"：出厂本体是极小核心，扩展只能经命名空间加入，核心字段的含义永不因扩展而变；信封里的 `trace` 作为因果字段保留并可在入口写入。这一层今天只有草稿、没有实现，本 delta 把它建档。

## ADDED Requirements

### Requirement: REQ-F-030 本体：极小核心 ＋ 命名空间扩展

本体 SHALL 采用**极小核心 ＋ 命名空间扩展**的结构：出厂本体只定义三家族信封与最小概念集；扩展项 SHALL 落在命名空间内，**SHALL NOT 与核心字段重名**；换一份**只加扩展**的本体，同一账本的折叠结果 SHALL 不变。
（核心字段 = `envelope.required` ∪ `envelope.optional`；重名在**装载期**即拒启并点名撞上的那一项——
重名是"法律自相矛盾"，与 `load` 的其余错误同一处置。）

#### Scenario: 扩展与核心字段重名必须被拒

- **WHEN** 造一份"扩展项与核心字段重名"（例如扩展里再叫 `body`）的本体
- **THEN** 加载**被拒**（错误码 `ext.world.Ontology.CoreCollision`）并指出重名的那一项；
  **对照**：只加扩展的本体（新家族 ＋ 同名下的新概念）SHALL 照常加载
- **证据**：`world-core/tests/ontology_ext.rs::x01_an_extension_item_colliding_with_a_core_field_is_refused_at_load`

#### Scenario: 换一份只加扩展的本体，同一账本的折叠结果不变

- **WHEN** 用同一份账本，分别以出厂本体与"只加扩展"的本体读（过法律 + 折叠）
- **THEN** 两次折叠结果**逐字节相同**，且词表身份**必须已变**（否则"换本体"没发生，相等是同义反复）；
  **对照**：把**核心**语义改掉的本体读同一条旧事件 SHALL 失败
- **证据**：`world-core/tests/ontology_ext.rs::x02_a_pure_extension_keeps_the_fold_byte_identical`

### Requirement: REQ-F-031 `trace`（信封因果字段）

信封的 `trace` SHALL 保留为需求：一条事件 SHALL 能声明"它由哪一条事件引起"，且该字段 SHALL 能从**写入入口**给出。
（今天：判据 (1) 已实测成立；判据 (2)(3) 依赖透传，**端到端可执行验证面缺失**——`World::commit` 与 CLI `append` 都没有 `trace` 参数。）

#### Scenario: 带 trace 的事件可写可读

- **WHEN** 从写入入口提交一条带 `trace` 的事件，再读回
- **THEN** `trace` 原样落账且可读回；`trace` 指向一条不存在的事件时，行为有明文规定（接受或拒绝，二者择一并写死）
- **证据（待补）**：**本条尚无断言**（〔待补〕）本 change `tasks.md` 第 6 组 —— 写入入口补上之前，本要求**无法端到端验证**
