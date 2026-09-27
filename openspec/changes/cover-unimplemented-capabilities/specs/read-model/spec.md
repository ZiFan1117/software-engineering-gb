# Spec Delta

## ADDED Requirements

### Requirement: 〔无号·待流程侧增补〕声明以外的字段不许落账

写入侧 SHALL 按出厂本体 `ontology.json` 的 `concepts` 校验实体与字段：**未声明的实体**与**未声明的字段** SHALL 被拒；读模型侧 SHALL 对缺格报错，SHALL NOT 静默接受。
（书第五章 5.3 实测的那条命令今天非零退出且账本 0 行：执行者是 `world-core/src/ontology.rs::check_concepts`。）

系统 SHALL 如实声明本条的两处边界，SHALL NOT 被读成全称成立：

- **裸主体不受约束**：`world://<名字>`（没有实体／实例那一段）今天**照样能落账**——留下它不是口径而是代价
  （既有出厂用例以这种形态写槽位，一律拒绝会把它们打红）。⇒ 书 §5.3 的"世界的边界由声明定"
  在**实体引用**这一半成立，在**裸主体**那一半**仍未成立**；
- **本段只管 `change`**：三家族里只有 `change` 改状态，`act` / `notice` 的信纸由家族必填项管；
  `notice.subject` 的语义是"这条通告关于谁"，不是"改了哪一格"，拿字段表去查它是查错了对象。

#### Scenario: 未声明的实体与字段都被拒

- **WHEN** 分别写入一个未声明的实体、一个未声明的字段
- **THEN** 两次都被拒、都不落笔，且错误里能读出**是哪个实体 / 哪个字段**没有声明
- **证据**：`world-core/tests/atom_declared_only.rs::b01_undeclared_entity_is_refused_and_nothing_lands`
      与 `world-core/tests/atom_declared_only.rs::b02_undeclared_field_is_refused_and_nothing_lands`
      —— 反"什么都拒"对照：`world-core/tests/atom_declared_only.rs::b03_declared_entity_and_field_still_land`；
      走真二进制的同一条命令：`world-core/tests/atom_declared_only.rs::b05_cli_append_of_an_undeclared_entity_is_refused`
      （断言 `rc=2` 且账本逐字为空）；
      登记在册的缺口：`world-core/tests/atom_declared_only.rs::b04_bare_subject_is_a_registered_gap_not_a_declared_entity`
      （裸主体今天仍可落账，它断言的是边界而不是"已做到"）。
