# Tasks

> **来源**：`fc-2026-002-spec-revisions/tasks.md` 的组 2–6（34 条），逐条转出，未改判据。
> **每条都必须有自己的变异证明**（"改坏哪一行 ⇒ 它变红"）；没有变异点的条目**不算完成**。


## 2. 补断言 · 通道身份与信封（依 delta 的"需补断言"标注）

- [ ] 2.1 在 `world-core/tests/contract.rs` 的 `c14` 内补一条断言：`serve_once` 路径**不读对端凭证**（例如断言 `Listener` 的 `uid` 字段在受理路径上不参与判定）。**验收**：新增断言会随"受理层改为读对端凭证"的变异**变红**（先证会红）。

- [ ] 2.2 在 `world-core/tests/cli.rs` 补一条断言：`channel bind` 对**不在身份映射里**的套接字报 `ext.world.Channel.NotConfigured` 且 `rc=2`。**验收**：`cargo test --locked --test cli` 通过；变异（删掉 `None` 分支的拒绝）⇒ 变红。

- [ ] 2.3 在 `world-core/tests/contract.rs` 的 `c02` 内补一条**反假**断言：`field == "world"` 那一轮必须靠"报出字段名"之外的方式判真（例如断言错误串里 `MissingField` 之后紧邻的字段名 token 等于 `world`），使该轮不再是 `contains` 恒真。**验收**：把 `world-core/src/ontology.rs:32` 的 `{field}` 删掉 ⇒ 该断言变红。

- [ ] 2.4 在 `world-core/tests/acceptance.rs` 的 `t5` 内补"状态未被改动"的断言（照同文件 `:87-92` 的既有写法读回读模型比对）。**验收**：`cargo test --locked --test acceptance -- t5_` 通过；变异（让被拒事件仍落笔）⇒ 变红。

- [ ] 2.5 为"策略文件缺失"与"本体软链"各补一条断言（放入 `c03`/`c09` 或新增用例）。**验收**：两类各有一条会红的断言；`world-core/tools/system_acceptance.sh --self-test` rc=0 不变。

- [ ] 2.6 为"已知无码出口"补断言：静态墙三条（符号链接／mode 位／属主）与策略版本不符，各自断言错误串**不含** `ext.world.` 前缀。**验收**：断言存在且当前为绿（它们固定的是边界，不是缺陷）；若某出口**确实**带码，该断言变红并据此改规格。

- [ ] 2.7 为 `c15` 补一条**账本路径**的错误码断言（今天 `c15` 的七条来源无一条是账本路径），并把该用例纳入出厂可跑的路径。**验收**：新增断言指向 `world-core/tests/cli.rs:155` 同族的账本码；且在 `world-core/check.sh` 里**有一条会跑到它**（见 6.2 的步骤归属订正）。

## 3. 补断言 · 门禁（含两条 P0 边界）

- [ ] 3.1 补一条断言：`world://user`（出厂 `irreversible_actors` 的唯一成员）执行不可逆能力（`ledger.compact`）⇒ **放行**，且账本中**不出现**任何 `gate.*` 通告。**验收**：断言当前为绿；变异（让白名单主体也走 `AwaitApproval`）⇒ 变红。出处：`world-core/src/gate.rs:287-293`；`world-core/docs/理论/WC-THEORY-DEFECT-001-v0.2.md:55`（`D-20`）。 〔该件已按作者指示退场；解析根＝`git show bf2eae7:<原路径>`〕

- [ ] 3.2 补一条断言：保留前缀通告被拒**那条路径**写下的流水也带 `refused`（`fnv1a64:` 前缀）指纹——今天只有不可逆加摩擦路径有该断言（`c23_gate_notice_says_what_it_refused` 走的是 `:1167` 的 `gate.awaiting-approval`）。**验收**：断言存在且会红（删掉 `refused` 字段即红）。

- [ ] 3.3 **重写后**：断言 `risk` 的**实际角色**——它**不**单独决定放行/拒绝（那由 `reversible` 与 `irreversible_actors` 裁决），但**决定摩擦的轻重与拒绝流水里的等级**（`gate.friction:low/medium/high/unlisted`）。
      **为什么不是原措辞**：原条目写「`risk` 不参与门禁裁决」，而实现已改为"参与摩擦、不参与放行"（`gate.rs:574-582`）；照搬会写成一条与实现相反的断言。
      **验收**：断言存在且会红（变异：把 `:544` 的 `level` 换成常量 ⇒ 变红）

- [ ] 3.4 断言**载体撤销点（`undo: before-each`）不参与互校**、也不被当作世界可逆的依据（`src/carrier/mod.rs:32`；互校的载体侧由 `risk`/`confirm` 导出）。
      **验收**：断言存在且会红（变异：把 `gate.rs:270` 的判据换成读 `undo` ⇒ 变红）

- [ ] 3.5 为"门禁不可绕过的未做部分"（祖先链遍历、通道层、Landlock 自缚）在本 change 的 `review.md` R5 节写明**不可机核、由评审签字承担**。**验收**：`review.md`（由人填）里有该声明；本组不产出测试。

## 4. 补断言 · 账本与证据链

- [ ] 4.1 补一条断言：在**无链（v1）账本**上做一次合法 `append` 后，账本**仍可被打开**（即 `K-3` 的修复判据）。**验收**：该断言当前**必然为红**（`world-core/src/ledger.rs:506` 的 `chained` 只读不用）⇒ 连同修复一起另立 change；断言先写、先证红。

- [ ] 4.2 补一条断言固定 `K-3` 的**当下边界**：无链账本 + 一次合法 `append` ⇒ 下次打开报 `MixedChain`。**验收**：断言存在且在修复落地前为绿（证明边界形状）。

- [ ] 4.3 补一条断言：`c21` 的"无链账本仍能打开"**只到只读为止**——即断言打开后**写入会失败**（与 4.2 同一形态的另一侧）。**验收**：断言存在；出处 `world-core/tests/contract.rs:1026`。

- [ ] 4.4 把 `t1` 的"逐字段一致"补全：逐个断言 `actor`／`id`／`at`／`flags`／`body.subject`／`body.path`／`body.after`（今天只比 `len`／`seq`／`world`／`kind`／`body.before`）。**验收**：新增断言 ≥ 7 条；变异（改 `world-core/src/event.rs` 的某个字段构造）⇒ 至少一条变红。

- [ ] 4.5 把 `t2` 的证据层级修正为**真跨进程**：把"新进程"这一层挂到 `world-core/tools/s1_sys_probe2.sh` 的 `TC-070`（`:379-383`，由 `world-core/check.sh:145` 执行），并在 `t2` 的文档注里写明它是同进程 drop + reopen。**验收**：`bash tools/s1_sys_probe2.sh` 通过；`t2` 的注释与规格一致。
      **★ 行号订正（2026-09-28，断言工区 B 实测）**：`TC-070` 在 `world-core/tools/s1_sys_probe2.sh:413-422`（原写 `:379-383` 落在 `TC-067` 里），由 `world-core/check.sh:161`（**步骤 ⑦**）执行（原写 `:145` 是步骤 ⑥ 的注释行）。⇒ **引用一律写"命令 ＋ 步骤名"，不写行号**（行号会烂）。
- [ ] 4.6 补一条断言固定"截到最后一个 `\n`"这条边界：末行是**完整合法 JSON 但缺末尾换行** ⇒ 被截掉且 `seq` 被复用。**验收**：断言存在且为绿；出处 `world-core/src/ledger.rs:276-289` 与实现自述 `:385`。

- [ ] 4.7 补一条断言固定单写者锁的**反向失效**：pid 号被复用时持有者已不存在却不会被回收。**验收**：断言存在（若不便构造，则在本 change 的 `review.md` R5 节登记为"不可机核"）；出处 `world-core/src/ledger.rs:182-184`。

- [x] 4.8 **证据链机械门禁 —— 已由 `spec_bridge.py` 判据② 承担**（2026-09-28 核）
      **实测**：`def j2_evidence`（`world-core/tools/spec_bridge.py:165`）扫 `openspec/specs/**` **＋ 所有 delta**（`openspec/changes/**/specs/**/spec.md`），判"证据行的 token 必须指向真实存在的函数/脚本"；
      `spec_bridge.py --self-test` 里两条反例逐字：`反例②a（**主规格**证据函数不存在 => 判据② 应红）：已红 OK`、`反例②b（**delta** 证据行指向不存在的函数 => 判据② 应红）：已红 OK` ⇒ **"改一个测试名 ⇒ 规格变红"当天即成立，"先证会红"亦有反例**。
      **★ 订正**：本条原引「`design.md` §排除清单第 7 条」——**该条不存在**（该节只有 2 条：不改 `src/**` 行为／不动书与规格），系**假引用**（由断言工区 B 实测发现，执行者复核成立）。**故本条不是"范围外而搁置"，而是"已由判据② 承担"。**
      **验收**：`python world-core/tools/spec_bridge.py --self-test` ⇒ rc=0 且含上列两条反例；`spec_bridge.py` 正跑判据② **[OK]**。

- [ ] 5.2 补一条断言固定"三个不等分支在命令路径上不可达"：断言 `project check` 的两份投影取自同一 `state` 与同一 `vocab`。**验收**：断言存在；出处 `world-core/src/main.rs:408-410`。

- [ ] 5.3 补一条断言：`Checkpoint::FORMAT` 版本不符 ⇒ 报 `ext.world.Checkpoint.BadFormat`（今天该分支零断言）。**验收**：断言存在且会红（删掉 `world-core/src/checkpoint.rs:94-99` 的判定即红）。

- [ ] 5.4 补一条断言固定"CLI 的 `checkpoint resume` 走未核验续算路径、由事后比对兜底"：篡改快照内容 ⇒ 若续算与全量不一致则报 `ResumeMismatch` 且 `rc=2`。**验收**：断言存在；出处 `world-core/src/main.rs:556-562`。

- [ ] 5.5 补一条断言：CLI 用法串里列出 `checkpoint write|verify|resume` 三条子命令。**验收**：断言存在；出处 `world-core/src/main.rs:28-30` 与 `:153`。

- [ ] 5.6 **待人填**（作者／评审席）：为 `REQ-N-008`（带检查点续算 ≤ 全量重算的 1/2）在 `fc-2026-004-assertions` 的 `review.md` R5 节登记为**范围外、未实现、无断言**。
      **★ 订正（2026-09-28）**：本条原文自己写着「`review.md`（**由人填**）」——而该 change 目录下**没有 `review.md`**（只有 `.openspec.yaml`／`design.md`／`proposal.md`／`tasks.md`），且评审记录按本仓口径**由人备料与签署**。
      ⇒ **它不是 agent 能完成的活**：**保持未勾**，等人在 `review.md` 落笔后由签署人勾。**不许代填。**
## 6. 验证与取证（跨组的整体验收）

- [ ] 6.1 形态门禁：在仓库根跑 `openspec validate fc-2026-002-spec-revisions --strict`，把**原始输出**抄回。**验收**：输出为 `Change 'fc-2026-002-spec-revisions' is valid`、`rc=0`。

- [ ] 6.2 产物链状态：跑 `openspec status --change fc-2026-002-spec-revisions`，把**原始输出**抄回。**验收**：`proposal`／`specs`／`design`／`tasks` 四件为 `done`，`review` **未写**（`[ ]`）。

- [ ] 6.3 不越界取证：跑 `git status --short`（必要时加 `-uall`），把**原始输出**抄回。**验收**：改动清单里**只多出本 change 一个目录**；`world-core/` 与 `openspec/specs/**` 零改动。

- [ ] 6.4 出厂判据强度不变：在 VM `world` 内跑 `cd /root/world/world-core && bash check.sh`，抄回 rc 与结论行。**验收**：rc=0（本 change 不该改变它；若变了说明越界）。**环境**：VM `world`（Arch Linux，cargo 1.98.1）。**主机无 Rust 工具链，本项不得在主机上声称跑过。**

- [ ] 6.5 补跑 `check.sh` **不跑**的那五个用例并留档：`cargo test --locked --test acceptance -- t5_ t9_ t10_ t13_` 与 `cargo test --locked --test cli`。**验收**：两命令的原始输出留档，并写明它们是"出厂单入口之外"的证据。

- [ ] 6.6 步骤归属订正：核对 `openspec/specs/**` 与 6 个 delta 里对 `check.sh` 步骤号的引用是否与 `world-core/check.sh:98`（③ 只跑 `t1_ t2_ t7_`）、`:103`（③b 跑整个 `--test contract`）、`:133`（⑥ 系统级验收）、`:145`（⑦ `s1_sys_probe2.sh`）一致。**验收**：逐条比对表（引用处 → 实际步骤 → 是否相符），不相符处已在 delta 里订正。

- [ ] 6.7 证据行"指向不存在/不执行的用例"收口（`audit.md` 的 **E2**/**P3**，另含 `E8` 的历史记账）：把 delta 里已删的 `（check.sh 步骤 ③）` 类徒有虚名的括注逐条复核，并给出两条处置之一——① 该断言确实在出厂某一步执行 ⇒ 补上**正确**的步骤号；② 不在任何一步执行 ⇒ **删括注**并在本 change 的 `review.md` R5 节写明"该断言今天不在出厂路径上"。**验收**：6 个 delta 里 `（\`check.sh\` 步骤 …）` 形态的括注**逐条**能对上 `world-core/check.sh` 的实际行；对不上的为 0 条。**⚠ 不给 `cli05`/`t5`/`t9`/`t10`/`t13` 编造步骤号**——它们今天确实不在出厂路径上。

- [ ] 6.8 `E8`（历史记账）的处置留档：`fc-2026-001` 的 `tasks.md:6` 把"行边界"列在 `envelope-validation` 名下，而该 Requirement 实际在 `openspec/specs/ledger-integrity/spec.md:67`。**本 change 不追改 `fc-2026-001` 的产物**（它已定稿）；在本 change 的 `review.md` R5 节留一句说明该历史错记即可。**验收**：`review.md`（由人填）里有该句；本 change 的 6 个 delta 里"行边界"只出现在 `ledger-integrity`。
---
