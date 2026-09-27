# Spec Delta

## ADDED Requirements

### Requirement: REQ-F-027 家族演进与向前兼容

规格 SHALL 写明家族演进的形态（新家族怎么加、旧读法怎么读新账本、新读法怎么读旧账本），并把"不认识的东西怎么办"两个方向**显式分开**：不认识的**家族** SHALL 被拒；不认识的**旗标** SHALL 被忽略。
（今天：只有"版本不符即拒收"已实现；演进与向前兼容的判据**无实现、无用例**。）

#### Scenario: 两个方向各有一条会红的断言

- **WHEN** 分别造出"不认识的家族"与"不认识的旗标"两条事件
- **THEN** 前者被**拒**且不落笔；后者被**接受**、落笔，且不影响后续折叠
- **证据（待补）**：**本条尚无断言**（〔待补〕）本 change `tasks.md` 第 3 组 —— 两条断言各自会红之前，本要求**不成立**

### Requirement: REQ-F-029 未知旗标必须忽略

系统 SHALL 忽略任何不认识的旗标，并按它认得的那些继续处理；SHALL NOT 因为出现未知旗标而拒收。
（依据逐字：出厂本体 `world-core/ontology.json:20` —— `"flags": "array  # 能力旗标；未知旗标必须忽略"`。）

本条与「未知**家族** SHALL 被拒」**对偶**，两条 SHALL NOT 互相冒充：分界线是
**不认识的语义拒绝，不认识的附加信息忽略**。「这条事件带着我不认得的旗标」SHALL NOT 被读成
「这个家族我不认识」；「这个家族我不认识」SHALL NOT 被读成「有旗标不认识」。
**第三情形**另立条文，SHALL NOT 被并入本条、也 SHALL NOT 被并入家族那一条：
扩展项与**核心字段重名** ⇒ SHALL 被拒（见本 change 的 `REQ-F-030`）——
它不属于「未知」那一类，而属于「与核心冲突」那一类。

#### Scenario: 未知旗标不影响受理

- **WHEN** 提交一条带未知旗标的事件
- **THEN** 该事件被接受、落笔，读回的 `flags` 保留原值；同一账本的折叠结果与不带该旗标时**相同**
- **证据**：`world-core/tests/ontology_ext.rs::e02_a_landed_ledger_line_with_unknown_flags_folds_byte_identically`

#### Scenario: 未知旗标与未知家族不许互相冒充

- **WHEN** 取同一条合法事件，分别只把它的 `flags` 换成未知旗标、只把它的 `kind` 换成未知家族
- **THEN** 前者被接受、落笔、折叠结果逐字节不变；后者被拒且**不落笔**——写入侧（`UnknownKind`）与折叠侧（`ReadModel.UnknownKind`）**两处都拒**，且各自点名那个家族
- **证据**：`world-core/tests/ontology_ext.rs::e03_unknown_family_is_refused_on_both_sides_of_the_dual`

