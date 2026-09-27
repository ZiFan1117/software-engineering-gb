# 规格层维护清单（MAINTENANCE）

> **本件的用途**：规格层自己的**常设维护项**落在这里——**不进任何 change 的 `tasks.md`**。
> **为什么必须移出来**：`tasks.md` 的归档门禁要求**全勾**，常设项天生做不完，
> 挂在里面会**永久挡住** `openspec validate --archived`（融合档 `schema.yaml:121-123` 逐字预言过这个形态，
> 而它真的在 `2026-09-27-baseline-verified-doctrine` 上发生了：`✗ 2 incomplete tasks (18/20 completed)`）。
> **本件不进任何规格树**，不是规格、不是 change 产物；守卫 `spec_bridge.py` 不读它。

---

## 一、从归档 change 移出的常设项

> 移出时点：`2026-09-27`（由 `fc-2026-001` 的 task 4.2 落笔）。
> 来源：`openspec/changes/archive/2026-09-27-baseline-verified-doctrine/tasks.md` §6「基线之后的维护」。
> **原处留痕**（移出说明 ＋ 原逐字），**不是静默删除**。

| # | 常设项 | 出处（原逐字） | 谁执行 | 什么时候 |
|---|---|---|---|---|
| 1 | **测试改名或 `check.sh` 步骤号变动时，同步修订对应 `证据：` 行** | 「测试改名或 `check.sh` 步骤号变动时，同步修订对应 `证据：` 行」 | 改动者本人（谁改名谁改证据行） | 与改名**同一次改动**里 |
| 2 | **VM 离线期间禁止声称基线已复验** | 「VM 离线期间禁止声称基线已复验」 | 任何写结论的人 | 持续有效 |

> **第 1 条现在有守卫兜着**：`spec_bridge.py` 判据② 会查出"证据行指向的函数不存在"并**非零退出**——
> 但它只保证**发现**，不保证**改对**；改对仍靠人。

---

## 二、规格层自己的维护规则

| # | 规则 | 落点 / 判据 |
|---|---|---|
| 1 | 规格树下**每条 Requirement** 都必须在 `openspec/BRIDGE.md` 在册（有号或显式标「无号」） | 守卫判据④ |
| 2 | 归档目录必须有非空 `review.md`，且结论栏**不许代签** | 守卫判据① ＋ `schema.yaml` 的 R5/R4 硬约定 |
| 3 | `openspec/config.yaml` 的 `schema:` 必须为 `opsx-swe-gb`（**别让它被改回默认档**——那会让每个新 change 静默退回 4 产物原生链，评审与证据守卫全部消失） | 守卫判据③ |
| 4 | 承载覆盖缺口的 `cover-*` change **必须存在且未归档**，其 `tasks.md` 保留未勾项（＝未实现的东西在册、可见、不装成已成立） | 守卫判据⑤ |
| 5 | 融合档 schema 的**主本与副本对账**：主本 `D:\Code\10-openspec-swe-gb\schemas\` ↔ 本仓 `openspec/schemas/`，**七件逐文件 sha256 一致**；主本缺任何一件即为断链 | `openspec/schemas/README.md` §五 |
| 6 | 跑守卫：`python3 world-core/tools/spec_bridge.py`；它自己也要能自证会红：`--self-test`（五条各造反例，**反例不变红即判该守卫是装饰**） | 守卫 `--self-test` |

---

## 三、谁维护

- **执行者**：任何改动规格层的人（改规格、改测试名、改 `check.sh` 步骤号的人）。
- **复核**：评审席在归档前跑一次 `python3 world-core/tools/spec_bridge.py`，把原始输出贴进该 change 的 `review.md` §五。
- **本件的更新方式**：直接改本件即可，**不需要 change**（它不是受控配置项，也不进规格树）。
