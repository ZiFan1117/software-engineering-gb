# Tasks

> 格式硬约束：每条形如 `- [ ] X.Y 描述`；只有 `x` 算完成。每条自带验收方式。
> **归档门禁要求全勾**；本来就做不完的常设项**不写在这里**（落 `openspec/MAINTENANCE.md`）。
> **前置**：本 change 判为 **R5**，按融合档「**未获 R5 批准不得进入实施**」——第 1 组起的所有条目
> **在 `review.md` 的 R5 节签"批准"之前不得动手**。

## 1. 默认档与守卫（让这一层的规矩可机核）

- [ ] 1.1 `openspec/config.yaml` 第 1 行 `schema: spec-driven` → `schema: opsx-swe-gb`
      **验收**：`openspec schemas --json` 里 `opsx-swe-gb` 在册；`openspec status --change fc-2026-001-openspec-into-cm` 的 `planningHome.defaultSchema` 变为 `opsx-swe-gb`
- [ ] 1.2 新增 `world-core/tools/spec_bridge.py`，五条判据（与 `specs/spec-governance/spec.md` 逐条对应）：
      ① 归档目录必须有 `review.md`（缺件 rc=1）② 每条 `证据：<path>::<fn>` 的函数/脚本必须真实存在
      ③ `config.yaml` 的 `schema:` 必须为 `opsx-swe-gb` ④ 编号桥映射表覆盖规格树下**全部** Requirement
      ⑤ 承载覆盖缺口的 change 存在且**未归档**
      **验收**：在 VM 内 `python3 world-core/tools/spec_bridge.py` **rc=0**；输出逐条列出五条判据的结论
- [ ] 1.3 `spec_bridge.py --self-test`：**为五条判据各造一个反例样本**，反例不变红即判该守卫是装饰并拒绝合入
      **验收**：`--self-test` rc=0，且输出里五条反例**逐条**打印"应红且已红"
- [ ] 1.4 把 `spec_bridge.py` 接进 `world-core/check.sh`（**新增一步，不改既有步骤号** ③／③b／④／⑦ 的含义）
      **验收**：新步骤可 grep 定位；该步失败时整个脚本非零退出（`set -euo pipefail` 下）
- [ ] 1.5 在 VM 内跑一次完整出厂门禁
      **验收**：`ssh world "cd /root/world/world-core && bash check.sh"` **rc=0**；原始输出留档
- [x] 1.6 **改掉融合档里"双读＋一行指针"的写法**（作者裁定：OpenSpec 那一套是唯一产物，流程只作件内栏位与附表）。
      **改写判据以本 change 的 `boundary.md`（分工边界表）为准**——OpenSpec 有的跟 OpenSpec、没有的跟流程、两边都有的"形态随 OpenSpec、内容随流程"：
      逐处修 `openspec/schemas/opsx-swe-gb/schema.yaml`（`proposal`／`specs`／`design`／`review` 四条 instruction 与顶层 `description`）
      与 `openspec/schemas/README.md`（整篇按新判据重写）
      **同步规则**：主本在 `D:\Code\10-openspec-swe-gb\schemas\`，**改主本再同步到本仓**
      **验收**：两处**七件**（`README.md` ＋ `opsx-swe-gb/` 六件）逐文件 sha256 一致 ✅ 实测 0 不一致；检索 `双读`／`融合档` ⇒ **0 命中** ✅
      **★ 追认登记**：本条目**在 R5 签字之前完成**（作者当面对话中指示）——按"未获 R5 批准不得进入实施"本不该动，故按流程 H-14 记为**追认项**（时点／指示人／回退点见 `review.md` §七），**不当作默认路径**。

## 2. 规格层（新能力进主规格）

- [ ] 2.1 `spec-governance` 进主规格：`openspec/specs/spec-governance/spec.md`
      **验收**：`openspec list --specs` 由 6 条变 **7 条**，新增条 `requirementCount` = 5
- [ ] 2.2 新增 `openspec/MAINTENANCE.md`（规格层自己的维护清单；写明它是什么、谁维护、条目怎么加）
      **验收**：文件存在且含"本件不进任何规格树"的声明；第 4.2 步移出的两条能在此检索命中
- [ ] 2.3 规格层整体校验
      **验收**：`openspec validate --all --strict` ⇒ **7 项全绿**（原 6 项 ＋ 新能力）

## 3. 编号桥

- [ ] 3.1 复核 `mapping.md` 的四组数：23 条承诺 / 39 条需求 / **撞号 5** / **无号 5** / **无人认领 17**
      **验收**：复算脚本对 `openspec/specs/**/spec.md` 抽 `### Requirement` 计数＝23；对 `WC-SRS-001` 抽 `REQ-` 唯一号计数＝39
- [ ] 3.2 在 `mapping.md` 末尾登记两组待增补项：**无号的 5 条承诺** ＋ **无人认领的 17 条需求**
      **验收**：逐条带出处行号；`spec_bridge.py` 判据④ 对此转绿

## 4. 归档遗留件（把红的那一件修绿）

- [ ] 4.1 为已归档 change `2026-09-27-baseline-verified-doctrine` 补 `review.md`
      **验收**：文件落在该归档目录内；结论栏**逐字写清"本基线语义层未核，已知 46 条（见 fc-2026-001 的 audit.md）"**，不写"已核"；签字栏留人
- [ ] 4.2 把该 change `tasks.md` 里 `## 6. 基线之后的维护` 两条常设项**移出**，落到 `openspec/MAINTENANCE.md`
      **验收**：原处留一行指向维护清单的说明（**留痕，不静默删除**）；维护清单内两条可检索命中
- [ ] 4.3 归档门禁转绿
      **验收**：`openspec validate --archived` 由 `0 passed / 1 failed` 变 **`1 passed / 0 failed`**

## 5. 覆盖 change（未实现的能力在册、可见、不装成已成立）

- [ ] 5.1 起草覆盖 change（**保持不归档**），把今天没有落点的能力写进 delta ＋ tasks：
      投递与应答、通道资源边界、家族演进与向前兼容、未知旗标必须忽略、本体命名空间扩展、`trace` 语义、
      通告的闸（`D-13`）、可逆性判定与配置互校、`concepts` 实体层
      **验收**：`openspec status` 可见；其 `tasks.md` **存在未勾项**
- [ ] 5.2 `spec_bridge.py` 判据⑤ 对该 change 转绿
      **验收**：临时移走该 change 后判据⑤ **必须变红**（反例自证），移回转绿

## 6. 验证与取证（跨多组的整体验证）

- [ ] 6.1 规格层：`openspec validate --all --strict` ⇒ **7 项全绿**（同 2.3，此处作为总验收）
- [ ] 6.2 归档层：`openspec validate --archived` ⇒ 全绿（同 4.3，此处作为总验收）
- [ ] 6.3 **未改动的证明**：`git diff --stat` 显示 **6 份既有主规格零改动**；`world-core/src/`、`world-core/tests/` **零改动**
- [ ] 6.4 门禁层：VM 内 `bash check.sh` **rc=0**（同 1.5，此处作为总验收，并附**环境指纹**）
- [ ] 6.5 `review.md` 的 **R5 节签字**（人）＋ 实施完成后补 **R4 节签字**（人）
      **验收**：两节四栏（提出人／影响分析人／批准人／日期）无空缺；分离声明已填
