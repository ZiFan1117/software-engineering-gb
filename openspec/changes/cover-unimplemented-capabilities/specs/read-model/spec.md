# Spec Delta

## ADDED Requirements

### Requirement: 〔无号·待流程侧增补〕声明以外的字段不许落账

写入侧 SHALL 按出厂本体 `ontology.json` 的 `concepts` 校验实体与字段：**未声明的实体**与**未声明的字段** SHALL 被拒；读模型侧 SHALL 对缺格报错，SHALL NOT 静默接受。
（今天：`grep -rn 'concepts' world-core/src/` **命中 0 处** —— 声明写在文件里，落笔时没有任何人读它；书第五章 5.3 实测 `rc=0`、账本落笔、`state --json` 把它算进去，判红。）

#### Scenario: 未声明的实体与字段都被拒

- **WHEN** 分别写入一个未声明的实体、一个未声明的字段
- **THEN** 两次都被拒、都不落笔，且错误里能读出**是哪个实体 / 哪个字段**没有声明
- **证据**：〔待补〕本 change `tasks.md` 第 9 组 —— 今天两次都 `rc=0`，本要求**是红的**
