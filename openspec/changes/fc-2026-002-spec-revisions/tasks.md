# Tasks

> **本 change 的范围**：只动规格文本（`openspec/changes/fc-2026-002-spec-revisions/specs/**`），
> **不动 `world-core/` 一行代码或测试，不动 `openspec/specs/**` 既有基线文件**。
>
> 第 1 组是本 change 的**全部内容**；第 2–5 组是 delta 里逐条标注的
> 「需补断言（列进 tasks）」——它们**属实施期**，触及 `world-core/tests/**`，
> **需另经 R5 批准后执行**（`proposal.md` §档位判定；`design.md` §Decisions D-1）。
> 第 6 组是跨组的整体验收。
>
> ⚠ **本件不含常设维护项**（`schema.yaml:121-122`：常设项会永久挡住 `openspec validate --archived`）。

## 1. 规格文本落地（本 change 的全部内容）

- [ ] 1.1 核对 `specs/channel-identity/spec.md`：`## MODIFIED Requirements` 1 条（标题逐字＝基线 `身份取自内核而非请求自称`，1 个原有 Scenario 一名不落）、`## ADDED Requirements` 1 条边界条文（`通道身份的实际保证与它的边界`）。**验收**：`openspec validate fc-2026-002-spec-revisions --strict` 输出 `is valid`，且 `openspec show fc-2026-002-spec-revisions --json --deltas-only` 能列出这 2 条。
- [ ] 1.2 核对 `specs/envelope-validation/spec.md`：MODIFIED 5 条（标题逐字＝基线的 5 条；`法律损坏或指向别处时拒绝启动` 的 4 个原有 Scenario 一名不落）、ADDED 1 条（`错误码契约的已知边界`）。**验收**：同上命令，且该文件的 MODIFIED 块内 `#### Scenario:` 计数 ≥ 基线同名的计数（`八个必填字段逐字段被拦`1／`不认识的事件家族被拒`1／`坏本体拒绝启动`+`本体缺失与家族为空被拒`+`策略的每一类畸形形状都被拒`+`策略为符号链接时拒绝启动，而真实文件不被误拒`共4／`连续提交的事件 id 各不相同`1／`各类失败路径的错误码可枚举`1）。
- [ ] 1.3 核对 `specs/gate-enforcement/spec.md`：MODIFIED 5 条、ADDED 2 条（`闸读不到风险等级，且载体可逆与世界可逆无互校`／`通告的闸，以及门禁不可绕过的部分实现边界`）。**验收**：同上命令；且 `## MODIFIED` 里不再出现"SHALL 对声明为不可逆的能力追加摩擦"这一无条件写法（`grep -c` 该串 = 0）。
- [ ] 1.4 核对 `specs/ledger-integrity/spec.md`：MODIFIED 6 条、ADDED 2 条（`无链账本的升级路径边界`／`承诺与证据的绑定强度`）。**验收**：同上命令；且 `摘要链检出局部篡改，并如实声明其边界` 的 `链状态位如实反映账本现实` Scenario 的 THEN 里**不再含"（v1 兼容）"**（`grep -c 'v1 兼容）'`（该条内）= 0），而 ADDED 的 `无链账本的升级路径边界` 里**含** `WC-SCMP-001-v0.1.md:2537` 与 `K-3` 字样。
- [ ] 1.5 核对 `specs/projections/spec.md`：MODIFIED 3 条、ADDED 2 条（`出厂同源命令的判据边界`／`视觉投影与读模型逐项相等`）。**验收**：同上命令；且 `两份读法同源且可当场核对` 的正文里**不再有"两份投影的首行 SHALL 逐字同形"**这一无条件写法，改为"头部四项"。
- [ ] 1.6 核对 `specs/read-model/spec.md`：MODIFIED 3 条、ADDED 2 条（`读模型是"一个真相"的检验面之一`／`带检查点路径的性能目标不在本基线的承诺范围内`）。**验收**：同上命令；且 ADDED 的 `读模型是"一个真相"的检验面之一` 里**含** `world-core/src/main.rs:28-30` 与三份受控文档的路径（`WC-UT-001-v0.1.md:54`／`WC-SRS-001-v0.1.md:921`／`WC-RTM-001.csv` 第 22 行）。
- [ ] 1.7 逐条核对 6 个 delta 的**证据行格式**：每条 Scenario 末尾都必须有 `- **证据**：`，且指向的 `path::fn` 在 `world-core/` 下真实存在，或明确写"需补断言（列进 tasks）"。**验收**：对每个 delta 抽出全部 `- **证据**：` 行，逐条 `grep` 校验测试名存在性；**未命中数必须为 0 或全部落在"需补断言"那一类**。
- [ ] 1.8 把 46 条的**对账表**随本 change 一并提交（见 `audit.md` 逐条 → 本 change 哪一件产物 → 或为什么不落），供评审席不依赖对话记录逐条复核。**验收**：46 行齐全、编号与 `audit.md` 的 C1–C6／E1–E8／G1–G6／L1–L9／P1–P7／R1–R10 一一对应、无重号无漏号。

## 2. 补断言 · 通道身份与信封（依 delta 的"需补断言"标注）

- [ ] 2.1 在 `world-core/tests/contract.rs` 的 `c14` 内补一条断言：`serve_once` 路径**不读对端凭证**（例如断言 `Listener` 的 `uid` 字段在受理路径上不参与判定）。**验收**：新增断言会随"受理层改为读对端凭证"的变异**变红**（先证会红）。
- [ ] 2.2 在 `world-core/tests/cli.rs` 补一条断言：`channel bind` 对**不在身份映射里**的套接字报 `ext.world.Channel.NotConfigured` 且 `rc=2`。**验收**：`cargo test --locked --test cli` 通过；变异（删掉 `None` 分支的拒绝）⇒ 变红。
- [ ] 2.3 在 `world-core/tests/contract.rs` 的 `c02` 内补一条**反假**断言：`field == "world"` 那一轮必须靠"报出字段名"之外的方式判真（例如断言错误串里 `MissingField` 之后紧邻的字段名 token 等于 `world`），使该轮不再是 `contains` 恒真。**验收**：把 `world-core/src/ontology.rs:32` 的 `{field}` 删掉 ⇒ 该断言变红。
- [ ] 2.4 在 `world-core/tests/acceptance.rs` 的 `t5` 内补"状态未被改动"的断言（照同文件 `:87-92` 的既有写法读回读模型比对）。**验收**：`cargo test --locked --test acceptance -- t5_` 通过；变异（让被拒事件仍落笔）⇒ 变红。
- [ ] 2.5 为"策略文件缺失"与"本体软链"各补一条断言（放入 `c03`/`c09` 或新增用例）。**验收**：两类各有一条会红的断言；`world-core/tools/system_acceptance.sh --self-test` rc=0 不变。
- [ ] 2.6 为"已知无码出口"补断言：静态墙三条（符号链接／mode 位／属主）与策略版本不符，各自断言错误串**不含** `ext.world.` 前缀。**验收**：断言存在且当前为绿（它们固定的是边界，不是缺陷）；若某出口**确实**带码，该断言变红并据此改规格。
- [ ] 2.7 为 `c15` 补一条**账本路径**的错误码断言（今天 `c15` 的七条来源无一条是账本路径），并把该用例纳入出厂可跑的路径。**验收**：新增断言指向 `world-core/tests/cli.rs:155` 同族的账本码；且在 `world-core/check.sh` 里**有一条会跑到它**（见 6.2 的步骤归属订正）。

## 3. 补断言 · 门禁（含两条 P0 边界）

- [ ] 3.1 补一条断言：`world://user`（出厂 `irreversible_actors` 的唯一成员）执行不可逆能力（`ledger.compact`）⇒ **放行**，且账本中**不出现**任何 `gate.*` 通告。**验收**：断言当前为绿；变异（让白名单主体也走 `AwaitApproval`）⇒ 变红。出处：`world-core/src/gate.rs:287-293`；`world-core/docs/理论/WC-THEORY-DEFECT-001-v0.2.md:55`（`D-20`）。
- [ ] 3.2 补一条断言：保留前缀通告被拒**那条路径**写下的流水也带 `refused`（`fnv1a64:` 前缀）指纹——今天只有不可逆加摩擦路径有该断言（`c23_gate_notice_says_what_it_refused` 走的是 `:1167` 的 `gate.awaiting-approval`）。**验收**：断言存在且会红（删掉 `refused` 字段即红）。
- [ ] 3.3 补一条断言：`risk` 不参与门禁裁决（同一能力在载体清单标 `risk: high`、在策略标 `reversible: true` ⇒ 门禁按 `reversible` 放行）。**验收**：断言存在；变异（若哪天 `gate.rs` 开始读 `risk`）⇒ 红或据实改规格。
- [ ] 3.4 补一条断言：载体撤销点（`undo: before-each`）**不**被当作世界可逆的依据。**验收**：断言存在；出处 `world-core/src/carrier/mod.rs:32`。
- [ ] 3.5 为"门禁不可绕过的未做部分"（祖先链遍历、通道层、Landlock 自缚）在本 change 的 `review.md` R5 节写明**不可机核、由评审签字承担**。**验收**：`review.md`（由人填）里有该声明；本组不产出测试。

## 4. 补断言 · 账本与证据链

- [ ] 4.1 补一条断言：在**无链（v1）账本**上做一次合法 `append` 后，账本**仍可被打开**（即 `K-3` 的修复判据）。**验收**：该断言当前**必然为红**（`world-core/src/ledger.rs:506` 的 `chained` 只读不用）⇒ 连同修复一起另立 change；断言先写、先证红。
- [ ] 4.2 补一条断言固定 `K-3` 的**当下边界**：无链账本 + 一次合法 `append` ⇒ 下次打开报 `MixedChain`。**验收**：断言存在且在修复落地前为绿（证明边界形状）。
- [ ] 4.3 补一条断言：`c21` 的"无链账本仍能打开"**只到只读为止**——即断言打开后**写入会失败**（与 4.2 同一形态的另一侧）。**验收**：断言存在；出处 `world-core/tests/contract.rs:1026`。
- [ ] 4.4 把 `t1` 的"逐字段一致"补全：逐个断言 `actor`／`id`／`at`／`flags`／`body.subject`／`body.path`／`body.after`（今天只比 `len`／`seq`／`world`／`kind`／`body.before`）。**验收**：新增断言 ≥ 7 条；变异（改 `world-core/src/event.rs` 的某个字段构造）⇒ 至少一条变红。
- [ ] 4.5 把 `t2` 的证据层级修正为**真跨进程**：把"新进程"这一层挂到 `world-core/tools/s1_sys_probe2.sh` 的 `TC-070`（`:379-383`，由 `world-core/check.sh:145` 执行），并在 `t2` 的文档注里写明它是同进程 drop + reopen。**验收**：`bash tools/s1_sys_probe2.sh` 通过；`t2` 的注释与规格一致。
- [ ] 4.6 补一条断言固定"截到最后一个 `\n`"这条边界：末行是**完整合法 JSON 但缺末尾换行** ⇒ 被截掉且 `seq` 被复用。**验收**：断言存在且为绿；出处 `world-core/src/ledger.rs:276-289` 与实现自述 `:385`。
- [ ] 4.7 补一条断言固定单写者锁的**反向失效**：pid 号被复用时持有者已不存在却不会被回收。**验收**：断言存在（若不便构造，则在本 change 的 `review.md` R5 节登记为"不可机核"）；出处 `world-core/src/ledger.rs:182-184`。
- [ ] 4.8 补建 `spec_bridge.py` 的**证据链机械门禁**：把 `openspec/**` 纳入受控清单，使"改一个测试名 ⇒ 规格变红"成立。**验收**：`spec_bridge.py --self-test` 会红（先证会红）；交付物落 `world-core/tools/**`。**⚠ 该脚本不属本 change 的写入范围**（`design.md` §排除清单第 7 条）⇒ 本任务只登记与验收，实施另立 change。

## 5. 补断言 · 投影与读模型

- [ ] 5.1 补一条断言固定 `project check` 的**判据只剩头部四项**：让两份投影在内容上不一致（一方少渲一半主体）而头部四项相同 ⇒ 命令**仍然报绿**。**验收**：断言存在且为绿（它固定的是边界）；出处 `world-core/src/project/mod.rs:124-145`。
- [ ] 5.2 补一条断言固定"三个不等分支在命令路径上不可达"：断言 `project check` 的两份投影取自同一 `state` 与同一 `vocab`。**验收**：断言存在；出处 `world-core/src/main.rs:408-410`。
- [ ] 5.3 补一条断言：`Checkpoint::FORMAT` 版本不符 ⇒ 报 `ext.world.Checkpoint.BadFormat`（今天该分支零断言）。**验收**：断言存在且会红（删掉 `world-core/src/checkpoint.rs:94-99` 的判定即红）。
- [ ] 5.4 补一条断言固定"CLI 的 `checkpoint resume` 走未核验续算路径、由事后比对兜底"：篡改快照内容 ⇒ 若续算与全量不一致则报 `ResumeMismatch` 且 `rc=2`。**验收**：断言存在；出处 `world-core/src/main.rs:556-562`。
- [ ] 5.5 补一条断言：CLI 用法串里列出 `checkpoint write|verify|resume` 三条子命令。**验收**：断言存在；出处 `world-core/src/main.rs:28-30` 与 `:153`。
- [ ] 5.6 为 `REQ-N-008`（带检查点续算 ≤ 全量重算的 1/2）在本 change 的 `review.md` R5 节登记为**范围外、未实现、无断言**，不补测试。**验收**：`review.md`（由人填）里有该范围外声明；出处 `WC-SRS-001-v0.1.md:112`、`WC-TP-001-v0.1.md:80`。

## 6. 验证与取证（跨组的整体验收）

- [ ] 6.1 形态门禁：在仓库根跑 `openspec validate fc-2026-002-spec-revisions --strict`，把**原始输出**抄回。**验收**：输出为 `Change 'fc-2026-002-spec-revisions' is valid`、`rc=0`。
- [ ] 6.2 产物链状态：跑 `openspec status --change fc-2026-002-spec-revisions`，把**原始输出**抄回。**验收**：`proposal`／`specs`／`design`／`tasks` 四件为 `done`，`review` **未写**（`[ ]`）。
- [ ] 6.3 不越界取证：跑 `git status --short`（必要时加 `-uall`），把**原始输出**抄回。**验收**：改动清单里**只多出本 change 一个目录**；`world-core/` 与 `openspec/specs/**` 零改动。
- [ ] 6.4 出厂判据强度不变：在 VM `world` 内跑 `cd /root/world/world-core && bash check.sh`，抄回 rc 与结论行。**验收**：rc=0（本 change 不该改变它；若变了说明越界）。**环境**：VM `world`（Arch Linux，cargo 1.98.1）。**主机无 Rust 工具链，本项不得在主机上声称跑过。**
- [ ] 6.5 补跑 `check.sh` **不跑**的那五个用例并留档：`cargo test --locked --test acceptance -- t5_ t9_ t10_ t13_` 与 `cargo test --locked --test cli`。**验收**：两命令的原始输出留档，并写明它们是"出厂单入口之外"的证据。
- [ ] 6.6 步骤归属订正：核对 `openspec/specs/**` 与 6 个 delta 里对 `check.sh` 步骤号的引用是否与 `world-core/check.sh:98`（③ 只跑 `t1_ t2_ t7_`）、`:103`（③b 跑整个 `--test contract`）、`:133`（⑥ 系统级验收）、`:145`（⑦ `s1_sys_probe2.sh`）一致。**验收**：逐条比对表（引用处 → 实际步骤 → 是否相符），不相符处已在 delta 里订正。
- [ ] 6.7 证据行"指向不存在/不执行的用例"收口（`audit.md` 的 **E2**/**P3**，另含 `E8` 的历史记账）：把 delta 里已删的 `（check.sh 步骤 ③）` 类徒有虚名的括注逐条复核，并给出两条处置之一——① 该断言确实在出厂某一步执行 ⇒ 补上**正确**的步骤号；② 不在任何一步执行 ⇒ **删括注**并在本 change 的 `review.md` R5 节写明"该断言今天不在出厂路径上"。**验收**：6 个 delta 里 `（\`check.sh\` 步骤 …）` 形态的括注**逐条**能对上 `world-core/check.sh` 的实际行；对不上的为 0 条。**⚠ 不给 `cli05`/`t5`/`t9`/`t10`/`t13` 编造步骤号**——它们今天确实不在出厂路径上。
- [ ] 6.8 `E8`（历史记账）的处置留档：`fc-2026-001` 的 `tasks.md:6` 把"行边界"列在 `envelope-validation` 名下，而该 Requirement 实际在 `openspec/specs/ledger-integrity/spec.md:67`。**本 change 不追改 `fc-2026-001` 的产物**（它已定稿）；在本 change 的 `review.md` R5 节留一句说明该历史错记即可。**验收**：`review.md`（由人填）里有该句；本 change 的 6 个 delta 里"行边界"只出现在 `ledger-integrity`。
