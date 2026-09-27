# 席D · 外部系统反面举证 · 独立评审（WC-RV-EXT-001 v0.1）

> **交付状态：六组全部完成。** 最低可交付三组（第 1 组操作系统层 / 第 3 组数据与状态层 / 第 5 组权限与治理层）**均已超额完成**：
> 第 1 组给了 **7 个场景**（要求 ≥5）、第 3 组 **4 个场景**、第 5 组 **6 个场景**；第 2、4、6 组同样已完成。
>
> **评审席位**：反面举证席（系统集成与平台架构师视角，**未参与本项目**）。
> **本席的立场**：不评这个自研项目好不好；只把"**今天外面这套系统到底哪里不好用**"讲清楚、讲具体。
> **评审对象（逐份真读）**：
> `2-依据/01-现状.md`、`06-重复工作审计.md`、`07-硌牙清单与单机经济账.md`、
> `15-世界核心的组成与职责.md`（§2.4、§十）、`13-总线与通道外部参考.md`、
> `1-理论与哲学/00-理论定稿.md`（§三）、`world-core/docs/系统全景图.md`；
> 补充取证：`00-总纲.md`、`2-依据/02`、`03`、`05`、`09`、`11`、`14`、
> `world-core/docs/S0-立项/WC-FSR-001-v0.1.md`、`S1-需求/WC-SRS-001-v0.1.md`、`WC-IRS-001-v0.1.md`、`agentd/README.md`。
> **外部事实核验方法**：三路并行检索，全部**只采信一手来源**（man page / 规范正文 / 内核源码 / 官方 API 文档 / RFC）。
> 凡本席的推理而非取证，一律显式标注"**这是我的推断**"。
> **未能读到**：无（七份指定文件全部读到并有行号引用）。
> 整理日期：2026-09-27。

---

## 一、结论摘要（9 条）

1. **进程不是主体**：Linux 的一切记录都以 pid/uid/cgroup 为单位（journald 的 `_PID=`/`_UID=`、auditd 的 `SYSCALL` 记录、`/proc`），而 pid 会被复用、uid 会被连接池共享、cgroup 不区分"哪个 agent 的哪次任务"——所以"谁在何时改了什么"里的**"谁"在系统层根本不存在**。
2. **状态是字节，不是字段**：`open/write/rename` 只表达"这个 inode 的字节变了"，`PATH` 记录（auditd type 1302）给的是 `item= name= inode= dev= mode= ouid= ogid= nametype=`——**没有一个字段说"哪个语义字段从什么变成了什么"**，所以"改了哪个字段"必须靠应用自己再实现一遍。
3. **字段清单有四项在所有主流日志里都缺**：**行为的人（不是进程）／被改的对象与字段／前后值／因果父事件**。RFC 5424 的 `HEADER = PRI VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID` ＋ 三个 IANA SD-ID（`timeQuality`/`origin`/`meta`）**四项全无**；CloudTrail 的 `requestParameters` 是"请求参数"不是前后状态；Debezium 的 change event 是 `before/after/op/ts_ms/txId/lsn`，**没有 why**。
4. **"信谁"没有技术答案，只有组织答案**：日志由各子系统自己写、自己轮转、自己真空（`journalctl --vacuum-time`、Kafka `retention.ms` 默认 7 天、SQL Server CDC 默认 3 天、Okta 90 天、GitHub 审计 180 天），且都可被 root/超管关闭或改写——**没有一条跨系统的关联 ID 能把一次人类意图串起 OS＋DB＋云＋SaaS**（W3C `traceparent` 的传播按库可选，OTel 规范原文是 "MUST use no-op propagators unless explicitly configured otherwise"）。
5. **权限能被表达的粒度是"路径/标签 × 权限位"，不是"语义"**：SELinux 的 file class 权限集是固定的内容盲集合（`read/write/append/unlink/rename/…`），`allowxperm` 最细只到 ioctl 的低 16 位；AppArmor 原文承认"**the contents of messages are not examined**"；k8s RBAC 的 `PolicyRule` 只有五个成员，官方文档明说"**依赖对象具体字段的访问控制由 admission controller 处理**"，而且"admission control happens **after** authorization"——**"只许改 `worker_processes` 不许改 `listen`"这句话，今天没有任何一层说得出来**。
6. **授权天然向"粗"坍缩**：一旦某个动作必须 root（uid 0 **完全绕过 polkit**，上游源码 `/* special case: uid 0, root, is _always_ authorized for anything */`），后面所有细粒度声明同时失效；`auth_admin_keep` 还把授权按 **(action id, subject)** 缓存 300 秒且**忽略参数差异**（"even if the variables passed along with the check are different"）。
7. **"装好了"不可验证，是因为缺少"声明 vs 实际"的对账面**：包管理器知道"装了什么"，auditd 知道"执行了哪条 argv"，journald 知道"哪个进程打了日志"——**三者之间没有共同主语**（没有"这次任务 = 这一批动作"的标识），于是 agent 的自述是唯一串起来的东西。
8. **监督确实必要但不充分且在衰减（理论 §三成立），而现有机制还在给衰减"加速"**：审批在日志里只留 `{谁, 决定, 自由文本}`——GitHub 的审批对象**没有审批时间戳、不绑定审批人所见的 sha**、且**没有 `environment.approve` 审计事件**；"人在什么信息状态下点的批准"**没有任何标准记录**——所以"批准即背书"在机制上是**不可反驳**的：你连"他当时看到了什么"都拿不出来。
9. **单机 + 云端大脑把因果链一刀两断**：账本在本地，模型在云上——"为什么"（意图）产生在账本写入之前、且在**别人家的服务器里**。这是本席认为所有"记账＝可复盘"方案都必须正面回答、而七份文件里**没有一份正面回答**的问题（**这是我的推断**）。

---

## 二、六组反面举证

> 每组格式固定：**场景 → 谁受损 → 现有机制为什么接不住 → 一句话要求**。
> "一句话要求"只写**可判定的要求**，不写实现方案。

### 第 1 组 · 操作系统层：文件系统 + shell + systemd + D-Bus + 权限位

#### 场景 1.1 一个 agent 半夜改了配置，你早上只知道"坏了"
- **谁受损**：机器的所有者（人）。**损失**：服务不可用时间 ＋ 定位时间（真实成本是"你不知道该怀疑什么"，所以只能二分法试错）。
- **现有机制为什么接不住**：
  - 你唯一拿到的是**时间戳**（`stat` 的 `mtime`/`ctime`）和**内容**（`diff`）——没有**主体**，也没有**意图**；
  - 想升级到"谁"就得先开 auditd 并写规则（`-w /etc/nginx -p wa`），而 auditd **默认不记录文件写**，且 `SYSCALL` 记录给的是 `pid= uid= auid= comm= exe=`——**`comm=`/`exe=` 是 nginx 还是 `vim`？看不出来，因为 agent 用的是 `bash -c`、`sh -c`、`sed -i`、或者直接 `write(2)`**；
  - 想升级到"为什么"需要 **causal parent**：journald 的受信字段里有 `_SYSTEMD_UNIT`、`_SYSTEMD_INVOCATION_ID`，**没有任何 `caused_by` 类字段**；auditd 也没有；
  - 唯一能回答"为什么"的地方是 **LLM 的上下文（聊天记录）**，而那是最不可信、最容易丢、最先被滚掉的地方。

#### 场景 1.2 两个 agent 抢同一份配置文件（"最后写者赢，两个人各自汇报成功"）
- **谁受损**：人（丢失 A 的成果）＋ A（它以为成功）。**损失**：静默的 lost update——**不报错、不告警、几天后才以"某个功能莫名失效"的形式暴露**。
- **现有机制为什么接不住**：
  - `open(O_WRONLY)` ＋ `write()` 之间**没有任何"读—改—写"原子性**；`flock`/`fcntl` 锁是**建议锁**，谁都可以不用；`rename()` 只保证替换原子，不保证**没覆盖别人刚写的东西**；
  - 文件系统**没有 compare-and-swap / revision 原语**；`Nix` 的原子 symlink 切换只解决"整棵树切版本"，**不做字段级合并**；
  - git 能挡这一手，但**要求两边都走 git 的仪式**（add/commit/冲突解决）——而 agent 改的是 `/etc` 和 `~/.config`，**这些路径通常不在任何仓库里**；
  - **这是我的推断**：真正的缺口不是"没有锁"，而是**没有"准入"**——没人能在**动作落地之前**看见"这个动作会覆盖谁刚写的东西"。

#### 场景 1.3 agent 说"我装好了"，而你无法验证它装了什么
- **谁受损**：人。**损失**：把不可验证的自述当成事实，进而在错误的状态假设上继续决策（真正的风险敞口）。
- **现有机制为什么接不住**：
  - **包管理器账本**（pacman/dpkg 的 `-Q`/`-l`）回答"系统里现在有什么"，**不回答"是谁、因为什么把它变成这样"**；
  - **auditd 的 `EXECVE`**（type 1309）确实记 `argc= a0= a1= …`，但：① 每条 execve 上限 `MAX_EXECVE_AUDIT_LEN 7500`，超长会被**拆成多条 EXECVE 并十六进制编码**（`a1_len=` ＋ `a1[0]=`），需要人工重组；② 对 `bash -c "…"` 只记那一条字符串；③ **默认没开**；
  - **journald** 干脆没有路径/文件字段（`_CMDLINE=` 只描述"发起日志的那个进程"）；
  - 于是**"agent 的声明"与"系统的实际变化"之间没有对账面**：清单机制（Nix/Ansible 的声明式）只覆盖它自己管的那些文件，**清单外的改动（"没声明但发生了"）没有任何机制报警**。

#### 场景 1.4 坏了只能回滚整棵快照（btrfs/snapper/VM snapshot）
- **谁受损**：人。**损失**：回滚粒度＝整个 subvolume；为了修一个字段，你把**同一时间窗内所有无关变更一起退掉**，于是"回滚"本身成了新的风险来源。
- **现有机制为什么接不住**：
  - 快照是**字节一致的**，但**不是语义可分的**：它没法把"agent 改的那部分"与"同期系统的正常变更（日志、缓存、其他 agent 的成果、包更新）"分开；
  - **收据缺失**：btrfs 给你 `UUID` 和时间，**不给你"这个快照对应世界上的哪次动作"**——所以"回到出事前"要靠人回忆时间点；
  - auditd 的 `auid`（loginuid）虽然**能穿过 sudo/su 保留原始登录 uid**（要 `CAP_AUDIT_CONTROL` 才能改、`--loginuid-immutable` 可锁），但它给的仍是**人**，不是**任务**——`auid=1000` 无法区分"你的三个 agent 会话中的哪一个"。

#### 场景 1.5 三个 agent 同时改配置/包集，互相打架而没人知道
- **谁受损**：人（要收拾不一致的状态）＋ 后到的 agent（在错误前提上继续）。
- **现有机制为什么接不住**：
  - systemd 的单元是"一个 unit 一个状态机"，**没有"同一资源上的并发写者协调"**；
  - D-Bus 有 `serial`＋`REPLY_SERIAL` 配对，**但没有资源级锁**，且它的策略语言按上游原文"**is not well-suited to finer-grained policies**"；
  - `systemd-run --scope`／`systemctl --wait` 只解决"等它结束"，**不解决"谁有权先动"**；
  - **这是我的推断**：真正的接不住在这里——**没有任何一层知道"这两个动作在语义上冲突"**，因为冲突是**内容相关**的判断（同一个字段、同一份清单），而 OS 只认识 inode 和 unit。

#### 场景 1.6 agent 起一个 build 吃光 32G，桌面会话跟着死
- **谁受损**：人。**损失**：未保存的工作 ＋ 会话里的其他进程（浏览器、编辑器）一起被杀。
- **现有机制为什么接不住**（这条要说得比 `01-现状.md` 更准）：
  - cgroup v2 的 `memory.max`/`memory.high` **确实存在且可用**（`systemd-run --scope -p MemoryMax=2G` 今天就能跑），所以不是"没有机制"；
  - 但**上游 systemd 对用户会话不设内存上限**：`units/user-.slice.d/10-defaults.conf` 只有 `TasksMax=33%`，`units/user@.service.in` 只有 `Slice=`/`Delegate=pids memory cpu`/`TasksMax=infinity`/`OOMScoreAdjust=100`，**没有 `MemoryMax`/`MemoryHigh`**；
  - 后果按 cgroup v2 原文：`memory.max` 超限就在**这个 cgroup 内**触发 OOM——"**it's not going to kill any tasks outside of this cgroup**"，于是**受害者从会话内部挑**（桌面 shell 与 build 同一 cgroup，地位平等）；
  - 真正的缓解是 `systemd-oomd`（PSI＋cgroup v2，按 `ManagedOOMMemoryPressure=kill` 只杀**后代** cgroup），但它是**按单元 opt-in**、且服务需被 enable；
  - **一句话**：机制在，**默认没装**；而 agent 的 ad-hoc 子进程**根本没有自己的 unit/cgroup**（除非被 `systemd-run` 包起来）——所以它连"可被单独杀"的资格都没有。

#### 场景 1.7 你不在场时发生的事，没人替你"看着"（监督衰减的机制侧）
- **谁受损**：人。**损失**：不是误操作，而是**"你在场也看不过来"**——监督退化为"事后抽样"。
- **现有机制为什么接不住**：
  - 系统级的"完工通知"只在**被 systemd 建模成 unit** 的东西上存在：`Type=notify`＋`sd_notify` 的 `READY=1`、`OnSuccess=`/`OnFailure=`、D-Bus 的 `JobRemoved(…, result)`、`systemctl --wait`；
  - 一个 agent 随手 `fork/exec` 的长任务**没有自己的 job**——它只是 `session-N.scope` 或 `user@UID.service` 的一个成员，上述机制**一条都不适用**；
  - 于是 agent 只剩**轮询**这一条路，而轮询的每一次都要一次模型推理（`07` 篇 §2#5 记的"10 分钟任务轮询 20 次推理"）——**监督成本被"思考成本"放大了两个数量级**；
  - journald 侧：`RateLimitIntervalSec`/`RateLimitBurst` 默认每 30 秒每服务 10000 条，**超了就丢**；FSS 要用 `journalctl --setup-keys` 生成密钥才生效（`Seal=` 默认值虽是 yes，但**无密钥时不做事**）。

**→ 一句话要求（第 1 组）**：**每一次对共享状态的改动，必须在落地之前带上"主体身份（不是 uid/pid）、目标对象的字段路径、前值、以及它所归属的那次任务"，并且这四个字段必须是系统强制产生、而不是应用自愿填写的。**

---

### 第 2 组 · 审计与日志层：syslog / journald / 云审计 / SaaS activity log

#### 场景 2.1 出了事，你把四份日志并排放在一起，仍然答不上"谁在何时改了什么、为什么"
- **谁受损**：人（调查者）。**损失**：调查时间从"查一条记录"变成"跨四个系统拼时间线"，而且**拼不出因果**。
- **现有机制为什么接不住（字段级缺陷，逐条）**：
  | 缺陷 | 具体到哪里 |
  |---|---|
  | **① 只有进程没有主体** | RFC 5424 的字段是 `PRI / VERSION / TIMESTAMP / HOSTNAME / APP-NAME / PROCID / MSGID / STRUCTURED-DATA`；IANA 只注册了 `timeQuality(tzKnown,isSynced,syncAccuracy)`、`origin(ip,enterpriseId,software,swVersion)`、`meta(sequenceId,sysUpTime,language)`——**没有"行为人"这一格**。journald 的 `_UID=`/`_AUDIT_LOGINUID=` 是**它给记录者打的戳**，而且原文承认"entries obtained via stdout or stderr of forked processes will contain credentials valid for a **parent process**"。 |
  | **② 只有时间线没有因果** | journald 受信字段全表里**没有 causal parent**；CloudTrail 事件的 `eventID`/`requestID`/`sharedEventID` 是**请求去重与关联**，不是"本条由哪条引起"；Kafka 有 `offset`，CloudEvents 有 `id`＋`source`（去重语义）——**没有一件提供因果边**。 |
  | **③ 只有进程粒度没有语义粒度** | `journalctl _PID=` 是"**记录日志的进程**"而不是"**动了东西的进程**"；auditd 的 `PATH` 记录字段是 `item= name= inode= dev= mode= ouid= ogid= rdev= nametype=NORMAL\|PARENT\|DELETE\|CREATE`——注意是 **`nametype=`**，它描述"这条路径在系统调用里扮演什么角色"，**不是"哪个字段"**。 |
  | **④ 只有"发生了什么"没有"为什么"** | 这一格**在 RFC 5424、journald、CloudTrail、M365 UAL、Google Workspace Reports、Okta System Log 里全部不存在**。M365 有 `ModifiedProperties(Name, NewValue, OldValue)`（仅管理活动）、GitHub 有事件专属的 before/after（`data.events` vs `data.events_were`）——**但没有任何一家有"理由/意图/工单号"字段**。 |
- **附：日志还能被改**：
  - journald：`--rotate`/`--vacuum-time=`/`--vacuum-size=`/`--vacuum-files=` 由 root 执行；FSS 的目的是"protect journal files from **unnoticed alteration**"——**tamper-evident ≠ tamper-proof**；
  - `chattr +a` 的 append-only 属性原文："**Only the superuser or a process possessing the `CAP_LINUX_IMMUTABLE` capability can set or clear this attribute**"，且"Setting 'a' and 'i' attributes will not affect the ability to write to already existing file descriptors"；
  - rsyslog 的 `omfile` **没有哈希链参数**（只有 opt-in 的外部签名 provider：GuardTime/KSI）；RFC 5848（Signed Syslog Messages）**确实存在**（Standards Track, 2010，注册了 `ssign`/`ssign-cert` SD-ID），但**主流实现里找不到它**（**部署广度未能核实**）；
  - CloudTrail 的日志文件完整性校验：**API `CreateTrail.EnableLogFileValidation` 默认 false**（控制台对新 trail 默认开），且默认的 Event history **不带 digest**；
  - S3 Object Lock 的 `GOVERNANCE` 模式**可以被 `s3:BypassGovernanceRetention` 权限绕过**（`x-amz-bypass-governance-retention:true`），且"Delete markers themselves are not WORM-protected, regardless of any retention period or legal hold"。

#### 场景 2.2 跨系统对不上账（本机 ＋ 数据库 ＋ 云 ＋ SaaS）
- **谁受损**：人（合规、复盘、"这单到底谁批的"）。**损失**：**在四份日志之间做人工 join**，且这个 join **不可复现**（下次保留窗一滚就再也做不出来）。
- **现有机制为什么接不住**：
  - **没有跨系统关联 ID 这种标准**。W3C `traceparent = version "-" trace-id "-" parent-id "-" trace-flags` 是**按库选择性传播**的，规范允许供应商在信任边界处"**Restart trace**"（重新生成 `trace-id`/`parent-id`/`trace-flags`）并丢弃 `tracestate`；Baggage 明确"no key or its value or properties is given semantic meaning by this specification"，且**超过 64 项 / 8192 字节可以丢**；OTel 的传播器默认是 **no-op**（"MUST use no-op propagators unless explicitly configured otherwise"）；
  - **OTel 的 span 记录的是"调用"，不是"效果"**：span 的 name 规范原文是"the most general string that identifies a **(statistically) interesting class** of Spans, rather than individual Span instances"——**没有 before/after，没有资源状态，没有行为的人**；
  - **时间窗互不相同**：CloudTrail 平均 **约 5 分钟**送达（"typically delivers logs within an average of about 5 minutes… This time is not guaranteed"）、Event history 只留 **90 天**；GitHub 审计 **180 天**（Git 事件 **7 天**、GraphQL **90–120 天**）；M365 UAL 标准 **180 天**（Premium 主线 1 年，附加可到 10 年）；Okta **90 天**（"Specifying a longer range will result in an error"）；Kafka `retention.ms` 默认 **604800000 = 7 天**；SQL Server CDC 默认 **3 天（4320 分钟）**；n8n 执行数据默认 `EXECUTIONS_DATA_PRUNE=true`、`EXECUTIONS_DATA_MAX_AGE=336`（14 天）、`PRUNE_MAX_COUNT=10000`；
  - **敏感字段会被抹掉**：CloudTrail 的 `requestParameters` 超过 100 KB **整体省略**，"truncation order" 末尾先丢 `responseElements`/`requestParameters`/`errorMessage`；`ConsoleLogin` 事件的 `"requestParameters": null`；root `ChangePassword` 是 `"requestParameters": null, "responseElements": null`；`userName` 会写成 `HIDDEN_DUE_TO_SECURITY_REASONS`；Google Workspace Reports **默认不返回敏感内容**（需 `includeSensitiveData=true`，且只对 DLP/Chat/Workspace Studio 有效）。

#### 场景 2.3 "日志里明明有，但它说的话不可靠"
- **谁受损**：人（被日志误导的调查者）。**损失**：基于不完整日志做出错误归因。
- **机制级原因**：
  - `pgaudit` 官方原文："Audit logging is **best-effort and not transactional**… **There is no guarantee that a committed transaction will have a corresponding audit log entry**"，且"**It is not possible to reliably audit superusers with pgAudit**"；
  - auditd 的压力丢日志是**设计内行为**：`-b backlog`（内核默认 64）、`-f`（默认 1=printk）、`-r rate`（默认 0），文档列出触发条件正是"transmission errors / backlog limit exceeded / out of kernel memory / rate limit exceeded"，并有一个**可被 `--reset-lost` 清零**的 lost counter；
  - journald 的 `_PID=` 在 pid 复用下**不可作为身份**；auditd 的 `a0..a3` 在 RAW 与 ENRICHED 两种 `log_format` 下**都是裸十六进制**，含义取决于 arch＋syscall number。

**→ 一句话要求（第 2 组）**：**任何一条记录都必须同时可判定地回答四个问题——"哪个主体（稳定身份，非 pid/uid）／对哪个对象的哪个字段／从什么变成什么／由哪一条上游事件引起"，且这四项缺失时该记录必须被判为不合格而不是被静默接受。**

---

### 第 3 组 · 数据与状态层：数据库（事务/CDC/审计表）、git、对象存储、消息队列

#### 场景 3.1 数据库回滚了，世界没有回滚
- **谁受损**：人（对账者）＋ 下游系统。**损失**：`ROLLBACK` 把表里的行退回去了，但**已经发出的邮件、已扣的款、已写入对象存储的文件、已推给终端的消息**都留在外面——于是数据库与外部世界**永久不一致**。
- **现有机制为什么接不住**：
  - 事务的原子性**边界就是数据库**：跨系统的原子性只有 saga/补偿（应用自己写），Kafka 的 `transactional.id` ＋ `read_committed` 也只保证"同一事务的记录全可见或全不可见"，**不提供跨系统原子性与补偿**；
  - S3 侧**没有多对象事务**：`DeleteObjects` 最多 1000 个 key，但原文是"**For each key, Amazon S3 performs a delete operation and returns the result of that delete, success or failure, in the response**"（`Deleted[]`/`Error[]`）——**逐 key，不是全成全败**。

#### 场景 3.2 "状态是真相、日志是附属" → 对账困难 + 不知道信谁
- **谁受损**：人（运维/审计）＋ 任何需要"重放一遍当时发生了什么"的人。
- **为什么这个普遍做法会带来对账困难（机制层，逐条）**：
  1. **两份真相**：表里的行是真相，audit/CDC 表是附属——两者由**不同的代码路径**写，于是"行改了但审计没写"（`pgaudit` 官方承认的 best-effort 非事务性）与"审计写了但行回滚了"**同时可能发生**；
  2. **附属品可以被合法丢弃**：MySQL binlog 有 `binlog_expire_logs_seconds`；SQL Server CDC 的 change table 默认 3 天清一次；WAL 段被回收；Kafka 默认 7 天；**唯独"状态"永远在**——所以时间一长，**你能证的只有"现在是什么"，不能证"怎么变成现在这样的"**；
  3. **附属品不记"谁"**：`pgaudit` 的身份只来自 `log_line_prefix %u`（**数据库角色**）；连接池让所有业务共用一个角色 ⇒ **终端用户不可恢复**；CDC 与 temporal table **连角色都不记**（SQL Server temporal `FOR SYSTEM_TIME AS OF` 只有 `ValidFrom`/`ValidTo`）；Debezium 事件字段是 `before/after/source/op/ts_ms/ts_us/ts_ns/transaction`，`source` 里是 `txId/lsn/xmin/db/schema/table/server_id/gtid/file/pos/row/thread/query/snapshot`——**没有一个字段叫"谁"或"为什么"**；
  4. **可改写**：审计配置本身是 superuser/root 可改（`ALTER SYSTEM`、`sys.sp_cdc_disable_table`、`pg_drop_replication_slot`），于是"证据的生产者"与"证据的可能被告"是同一方——这是**机制问题，不是道德问题**；
  5. **粒度是行不是字段的语义**：SQL Server CDC 有 `__$update_mask`（列级），**git 完全没有对应物**（blob 级）。

#### 场景 3.3 git 被当成审计系统用（这是今天最普遍的误用）
- **谁受损**：人（以为"有 commit 就有责任"）。**损失**：**追责链条建立在一个可伪造的字段上**。
- **现有机制为什么接不住**：
  - **身份是自由文本**：`git commit --author=<author>` 可任意覆盖；`user.name`/`user.email` 是普通配置；签名是 **opt-in**（`commit.gpgSign` 默认 false、SSH 签名要显式 `gpg.format=ssh`），且 `--no-gpg-sign` 原文"countermand[s] both `commit.gpgSign` configuration variable, and earlier `--gpg-sign`"；
  - **没有结构化的"为什么"**：commit message 是自由文本（`--allow-empty-message` 都允许），`--trailer` 只是约定不是 schema；
  - **粒度是整 blob**：没有"哪个字段/单元格"的概念；
  - **历史可被真正抹掉**：`git reflog expire --expire-unreachable=` ＋ `git gc` 会清理不可达对象（`gc.reflogExpire` 默认 90 天、`gc.reflogExpireUnreachable` 默认 30 天），`git filter-repo` 是官方推荐的改写工具，`git reflog drop` 原文"**completely removes the reflog**"；
  - **未追踪的文件对 git 完全不存在**（`status.showUntrackedFiles` 只影响**显示**）——而 agent 改的恰恰大量是未追踪文件。

#### 场景 3.4 对象存储与队列都不适合当"世界的账本"
- **谁受损**：人。**损失**：把"能存"当成"能证"。
- **机制级原因**：
  - **S3**：`DeleteMarker` 语义让"删除"变成"再写一条"，但 Delete marker **本身不受 WORM 保护**；Object Lock 必须**先开 versioning**、保护粒度是**每个对象版本**，bucket 级 `DefaultRetention` **只对新放入的版本生效**（`PutObjectRetention` 可对已有版本补挂），一旦开启就**不能关闭**（"you can't disable Object Lock or suspend versioning for that bucket"）；而**对象级 data events 默认不开且另行计费**（"By default, trails and event data stores do not log data events. Additional charges apply for data events"）——**"谁改了这个对象"默认根本没有记录**；
  - **Kafka**：`retention.ms` 默认 7 天、`retention.bytes=-1`、`cleanup.policy` 默认 `delete`（"will discard old segments when their retention time or size limit has been reached"）；`compact` 只"**retains the latest value for each key**"——**它是每 key 最新值快照，不是历史**；tombstone 自身也会在 `delete.retention.ms`（默认 86400000）后消失；
  - **CloudEvents** 的必需属性只有 `id/source/specversion/type`，`source` 原文"the exact syntax and semantics … is defined by the event producer"——**它不是身份证明**，其余只能放 extension attribute，而 extension "**have no defined meaning in this specification**"。

**→ 一句话要求（第 3 组）**：**每一个对持久状态的变更都必须先产生一条"主体＋对象＋字段＋前后值＋因果父"的记录，该记录与状态写入必须是同一个不可分割的动作；任何"先改状态、稍后补日志"或"只记行不记字段语义"的做法必须被判为不合格。**

---

### 第 4 组 · AI/Agent 框架层（只谈机制，不谈模型能力）

#### 场景 4.1 工具调用"成功"了，但没人能回答它到底动了什么
- **谁受损**：人（要复盘的人）＋ 下一个 agent（在错误前提上继续）。
- **现有机制为什么接不住**：
  - **工具定义里没有"不可逆性"这一格**：OpenAI Responses 的 `FunctionTool` 字段是 `type/name/description/parameters/strict/output_schema/allowed_callers/async/defer_loading`；Anthropic 的 `ToolParam` 是 `name/input_schema/description/type/strict/cache_control/defer_loading/eager_input_streaming/input_examples/allowed_callers`——**两家都没有 risk / reversible / destructive / idempotent / scope 字段**；`allowed_callers` 限制的是**调用上下文**（direct vs code_execution），**不是被声明的权限边界**；
  - **MCP 有提示，但规范自己说不能信**：`ToolAnnotations` 是 `title/readOnlyHint/destructiveHint/idempotentHint/openWorldHint`，原文"all properties in ToolAnnotations are **hints**. They are not guaranteed to provide a faithful description of tool behavior"，并且"For trust & safety and security, clients **MUST** consider tool annotations to be **untrusted** unless they come from trusted servers"——**服务端可以撒谎**；
  - **MCP 明确不做协议级强制**：Tools 是"**model-controlled**"、"the protocol itself does not mandate any specific user interaction model"、人在环中是 **SHOULD**；**MUST** 级的同意只在两个特例上（一键安装本地 MCP server 配置、以及 OAuth 授权流程）。⇒ **门禁被推给 Host，而 Host 就是 agent 的 harness**——这与整个"门禁必须低于 harness"的主张正好互证。

#### 场景 4.2 重连、断线、重放：框架不知道"我落后了多少"
- **谁受损**：agent（读到过期状态）＋ 人（基于错误状态的动作）。**损失**：**静默的陈旧读**。
- **机制级原因**：
  - **MCP 没有 `seq`**：`JSONRPCRequest = {jsonrpc, id, method, params?}`，notification **连 id 都没有**；`notifications/cancelled` 原文承认取消"**MAY arrive after the request has already finished**"；
  - **MCP 的 resource 没有版本**：`Resource` 字段是 `uri/name/title/description/mimeType/annotations/size/_meta`——**没有 version/ETag/hash**；更新信号只有 `notifications/resources/updated { uri }`（**只有 URI，没有 diff**）；
  - **LangGraph 有检查点但没有外部效果**：`thread_id`＋`checkpoint_ns`＋`checkpoint_id`、每个 super-step 一个 `Checkpoint`（`channel_versions`、`pending_writes`）、`get_state_history` 可回看与 fork——**它记录的是图内状态**；文件写、DB 事务、云 API 调用**不在检查点里**（除非工具自己把结果写回状态）。**checkpoint ≠ 外部效果回滚**：删掉检查点不会删掉已上传的 S3 对象。

#### 场景 4.3 工作流引擎的执行记录看起来很像审计，但它不是
- **谁受损**：人（把执行记录当审计）。**损失**：以为"有 trace 就有责任"，实际上**没有主体、没有回滚、且有保鲜期**。
- **机制级原因**：
  - **n8n 做得比很多人以为的多，但仍不是审计**：`EXECUTIONS_DATA_SAVE_ON_ERROR` 默认 **`all`**（失败默认保存）、`SAVE_ON_SUCCESS=all`、`SAVE_ON_PROGRESS=false`、`EXECUTIONS_DATA_PRUNE=true`、`MAX_AGE=336`（14 天）、`PRUNE_MAX_COUNT=10000`——**保留窗＋可关闭**；且**没有任何节点效果的补偿机制**（记录是日志，不是补偿）；
  - **Dify** 同理：节点级 trace ＋ 可"Test With Params"重跑，**重跑不是回滚**（重跑会再产生一次副作用）；
  - **agent 记忆**：Mem0 的 v1 `PUT /v1/memories/{id}/` 可改可删（V3 改成 ADD-only 是厂商选择而**不是规范约束**）；Letta 的 memory blocks 有完整 CRUD；LangGraph 的 `BaseStore` 是 put/get/delete/search——**没有一家的 API 面里有"不可追加、带作者、带来源任务的记忆写日志"**。

#### 场景 4.4 RAG 的"不可复现"让复盘失去意义
- **谁受损**：人（想复现"为什么它当时那么答"）。**损失**：复现失败被误判为"模型不稳定"，真因是**索引不可复现**。
- **机制级原因**：
  - **没有"（分块规则, 嵌入模型版本, 索引参数）"三元组的标准**：向量库的 collection schema 只记 `VectorParams{size, distance}`；嵌入 `model` 名是**每次请求带**的，**不被规范要求随向量持久化**；
  - **换模型的语义是"重灌"而不是"兼容"**：Qdrant 官方"Switching models requires re-embedding all vectors in your collection, which can take time"；标准做法是新建 collection ＋ 双写 ＋ alias 切换；
  - **近似检索本身不保证确定**：Qdrant FAQ 原文"**Can the same similarity search query yield different results on different machines?** Yes, due to differences in hardware configurations and parallel processing, results may vary slightly"，且 HNSW 是近似算法——**同一个 `limit=20` 与 `limit=100` 的前 20 条不保证一致**。

**→ 一句话要求（第 4 组）**：**每一个可被调用的动作，必须在被调用之前就携带机器可校验的"边界声明（作用于什么对象/字段）＋ 不可逆性等级 ＋ 补偿方式"，且该声明必须由调用方之外的一方强制生效——不得由被调用方自述、也不得由 harness 自己裁判。**

---

### 第 5 组 · 权限与治理层

#### 场景 5.1 sudoers 写了通配符，看起来锁住了，实际上没有
- **谁受损**：管理员（以为锁住了）。**损失**：**以为的最小权限其实是完全权限**。
- **机制级原因**：`sudoers(5)` 原文——`*` 匹配"any set of zero or more characters (**including white space**)"、"These are **not** regular expressions"、文件名部分的 `/` 不被通配符匹配但"**When matching the command line arguments, however, a slash does get matched by wildcards**"，并有明确警告"**Wildcards in command line arguments should be used with care.**"；安全写法只有正则（sudo ≥1.9.10，POSIX ERE、必须 `^…$`、上限 1024 字符），手册自己举的例子说明某条规则"**is impossible to express safely using wildcards**"。

#### 场景 5.2 polkit 授权被缓存 5 分钟，且缓存不区分参数
- **谁受损**：人（以为"每次都问"）。**损失**：**同一 action 的第 2..N 次调用不再需要认证**，而且**参数可以完全不同**。
- **机制级原因**：
  - `ExpirationSeconds=` 在 `[Polkitd]` 默认 **300 秒**；`auth_admin_keep` 原文"authorization checks for the same action identifier and subject will succeed… for the next brief period (e.g. five minutes) **even if the variables passed along with the check are different**"；
  - **uid 0 完全绕过 polkit**（上游 `polkitbackendinteractiveauthority.c`：`/* special case: uid 0, root, is _always_ authorized for anything */`）；
  - Subject 只有三种：`unix-process`（pidfd/pid＋uid＋start-time）、`unix-session`（session-id）、`system-bus-name`（unique name）——**"谁在做什么"永远不是 subject 的一部分**；
  - 关键补证：logind 的 `SetBrightness(subsystem, name, brightness)` 是 `SD_BUS_VTABLE_UNPRIVILEGED`，**源码里没有 `bus_verify_polkit_async*`、没有 polkit action id**（对比 `Terminate`/`Kill` 用 `org.freedesktop.login1.manage`）——**"只许调亮度"这件事今天是靠一个硬编码的窄 API 实现的，不是靠能力模型**。

#### 场景 5.3 SELinux/AppArmor 装上了，可"只改一个配置项"仍然表达不出来
- **谁受损**：管理员（以为自己表达了业务规则）。**损失**：策略看起来严格，实际是**文件级的粗闸**。
- **机制级原因**：
  - SELinux 类型强制就是四元组 `allow source_type target_type : class perm_set;`；`file` class 的权限集是**固定且内容盲**的（`ioctl/read/write/create/getattr/setattr/lock/relabelfrom/relabelto/append/map/unlink/link/rename/execute/…`）——**一个文件只有一个 type，没有文件内中介**（MLS/MCS 加的是整个对象的 level/range）；最细的 `allowxperm … ioctl xperm_set` 只用 **ioctl request 的低 16 位**；
  - AppArmor 是纯路径的，且上游原文对两处**本可以看内容的地方**明确否掉：DBus"the contents of messages are not examined"、unix socket"**The content of the communication is not examined**"；
  - ⇒ **"这个 agent 可以改 `/etc/nginx/nginx.conf`，但只许改 `worker_processes`，不许碰 `listen`"——SELinux 与 AppArmor 都表达不出来。**

#### 场景 5.4 k8s RBAC 允许了 `update`，于是它也能改 `image`
- **谁受损**：平台管理员（以为只给了"调副本数"）。**损失**：**同一个 verb 覆盖了整个对象的全部字段**，包括镜像、探针、挂载的 secret。
- **机制级原因**：
  - RBAC 的 `PolicyRule` **只有五个成员**：`verbs`/`apiGroups`/`resources`/`resourceNames`/`nonResourceURLs`——**没有 field selector、没有条件、没有 CEL**；`resourceNames` 只按对象**名字**过滤，且**不能约束 `create`/`deletecollection`**；
  - 官方文档原文："Access controls and policies that depend on specific fields of specific kinds of objects are handled by **admission controllers**"，且"Kubernetes admission control happens **after** authorization has completed"；
  - `PATCH → patch` 只映射到 verb，**patch body 不是授权输入**，所以 `resourceNames`/verb 都表达不出一个 JSON Pointer；
  - 要做字段级必须引入 **ValidatingAdmissionPolicy（CEL，v1.30 GA）** 或 ValidatingWebhook / Gatekeeper / Kyverno——**也就是承认：授权层做不了语义级，只能另加一层"内容相关的"裁判。这是本席在本仓库之外找到的、对"世界核心"主张最强的外部旁证**。

#### 场景 5.5 IAM 策略看起来已经最小化了，仍然能被提权
- **谁受损**：云账号所有者。**损失**：**以"两个各自无害的权限"组合出完整管理员**。
- **机制级原因**（AWS 官方文档原文）：
  - 启动带实例角色的实例需要 `iam:PassRole`（`RunInstances` 的 `IamInstanceProfile` 参数）；AWS 自己给出的提权叙述："imagine that user Alice has permissions only to launch Amazon EC2 instances and to work with Amazon S3 buckets, but the role she passes to an Amazon EC2 instance has permissions to work with IAM and Amazon DynamoDB. In that case, Alice might be able to launch the instance, log into it, **get temporary security credentials**, and then perform IAM or DynamoDB actions that she's not authorized for."；
  - 两条官方警示特别能说明"授权粒度的天花板"：①"Do not try to control who can pass a role by tagging the role and then using the `ResourceTag` condition key… **This approach does not have reliable results**"；②"**PassRole is not an API call.** `PassRole` is a permission, meaning **no CloudTrail logs are generated** for IAM `PassRole`"——**一次提权的关键一步在审计里根本不存在**；
  - `iam:PassedToService` 单用不成立（`ec2.amazonaws.com` 就是最终服务），必须配 ARN 条件才收口。

#### 场景 5.6 OAuth scope 说是细粒度，实际语义由资源服务器自定
- **谁受损**：用户（无法判断"我授了什么"）。**损失**：**scope 字符串不携带资源实例级或字段级约束**。
- **机制级原因**：RFC 6749 §3.3 原文"The strings are **defined by the authorization server**"；RFC 6750 §3 原文"'scope' values are implementation defined; **there is no centralized registry for them**"。更细的 **RFC 9396（Rich Authorization Requests, 2023-05）确实存在**，`authorization_details` 带 `type`（"**The AS controls the interpretation of the value of the `type` parameter**"）与 `locations/actions/datatypes/identifier/privileges`，例子能到资源实例（`"identifier":"account-14-32-32-3"`）——但**语义仍由 API 定义，支持面不均**（本席只核实到 Keycloak 有官方 RAR 包）。

**→ 一句话要求（第 5 组）**：**越界判定必须以"语义对象＋字段＋动作"为最小单位做出，并且必须由一个既不是被管者、也不是请求发起者的第三方在动作落地前强制生效；任何"授权后即不可复查"的缓存、任何"uid 0 即全通"的短路，都必须被判为不合格。**

---

### 第 6 组 · 人的一侧：注意力是瓶颈（对接理论定稿 §三）

> 理论定稿 §三 的三条（必要 / 不充分 / 反噬）本席**逐条认可**，并在外部机制里找到了它们的**直接证据**。这一组不是复述理论，而是补"为什么今天的机制在给衰减加速"。

#### 场景 6.1 依赖越深越没空监督（依赖与监督互斥）
- **谁受损**：人。**损失**：监督从"逐次审查"退化为"事后抽查"，而抽查的前提是**有可抽查的东西**。
- **机制级原因**：
  - **监督的成本单位被搞错了**：系统的通知粒度是**进程/单元**（journald `_PID=`、systemd `JobRemoved`），而人的决策粒度是**意图/字段**——中间没有翻译层，于是每一次监督都要人**自己重建上下文**；
  - **轮询把监督变成推理**：agent 想知道"完了没"，只能再推理一次（`07` §2#5）；`Type=notify`/`OnSuccess=` 这类"完工拉铃"机制**只对被建模成 unit 的东西有效**，agent 的 ad-hoc 长任务不在其内；
  - **日志的保鲜期短于人的遗忘曲线**：14 天（n8n）/ 7 天（Kafka、GitHub Git 事件）/ 3 天（SQL Server CDC）/ 90 天（Okta、CloudTrail Event history）——**你想"周末回来看看"的时候，证据可能已经先走了**。

#### 场景 6.2 24 小时后回来，答不出"发生过什么、哪些在等我批"
- **谁受损**：人。**损失**：**待批集不存在**——因为"待批"这个状态从来没被持久化成任何东西。
- **机制级原因**：
  - **v1 没有审批通道**（`world-core/docs/系统全景图.md` §五 自己写着："不可逆动作只认白名单主体；被摩擦的主体拿到的是'不要等批准，它不会来'"）——所以"待批集"在**自研侧**也还不存在；
  - **在外部侧同样不存在**：GitHub 的环境审批 API 会返回 `state`/`user{login}`/`comment`，但**审批对象上没有绑定审批人所见的 `sha`**（`sha` 来自部署记录，webhook 的 `sha` "Always populated from the check suite"），**也没有一个 `environment.approve` 审计事件**；
  - 于是"哪些在等我批"只能由**各系统的未决队列**拼：CI 的一个队列、聊天工具里的若干条消息、邮件里的若干封——**没有一处是权威的**。

#### 场景 6.3 "批准即背书"锁死追责——而且机制上不可反驳
- **谁受损**：人（批准者）。**损失**：他背了一个**无法举证自己当时看到了什么**的责任。
- **机制级原因**：
  - **没有任何标准记录"批准时呈现给人的信息状态"**：RFC 9396 §3 只要求授权服务器"**MUST present** the merged set of requirements"，RFC 全文里 "audit" 只在一处非规范附录出现，**没有一处要求持久化或暴露"给用户看了什么"**；
  - GitHub 的审批记录只有**身份＋自由文本 comment**——**没有审批时间戳、没有绑定制品**，而且 REST 文档把 `comment` 写成必填、UI 文档写成"Optionally, leave a comment"（**两份官方文档自相矛盾**）；
  - C2PA 2.1 只做**内容来源**（`c2pa.hash.data`/`c2pa.actions`/`c2pa.ingredient`…），**没有"批准者/UI 状态"这类构造**；
  - ⇒ **"批准疲劳"在机制上是不可探测的**：你既证不了"他没看"，也证不了"他看了"。**这正是理论 §三第 3 条（反噬）在工程上的样子。**

#### 场景 6.4 你以为在"确认"，其实在"确认一个摘要"
- **谁受损**：人。**损失**：确认的动作发生了，确认的**对象**从来没被固定下来。
- **机制级原因**：
  - 审批界面给人看的是**渲染结果**（一个页面、一段 diff、一句自然语言摘要），而日志里存的是**一个布尔**（`state: approved`）；
  - **两者之间没有任何 hash 约束**：没有"批准时那一屏内容的 hash"这个字段，所以事后无法证明"当时那一屏是什么"；
  - `polkit` 的 details 更直白：缓存判定**忽略参数差异**（原文见 5.2）——**"他批准过"与"他批准了这件事"在机制上不是同一句话**。

**→ 一句话要求（第 6 组）**：**每一次需要人介入的决定，都必须把"呈现给他的那一份信息"和"他做出的那个决定"作为同一条记录一起持久化，并且必须能机械地判定"当前待批集"与"已批集"的完备性（既不漏也不重）。**

---

## 三、我验证过的、委托方写成事实的批评

> 格式：**原文引用（带文件路径与行号）→ 判定 → 证据**。
> 判定用四档：**成立 / 部分成立 / 不成立 / 未能核实**。

| # | 原文引用（出处） | 判定 | 证据 |
|---|---|---|---|
| 1 | "**文本接口不可靠** 输出格式随版本/locale 变；SSID 里的 `**` 转义、进度条、颜色码都会让解析跑偏。agent 出错的头号来源"（`01-现状.md:29`） | **成立**（机制层），但**表述略过头** | 机制成立：`01-现状.md:12-17` 自己画的旅程里，`brightnessctl` 返回的是 `stdout` 纯文本，agent 要"再推理一次'应该成功了'"。**过头的部分**：并非所有命令行都没有结构化出口——`nmcli` 有 `-t`/`-f` 与 `--mode json`（`01`、`03` 引用的正是 nmcli），`brightnessctl` 之外还有 logind 的 `SetBrightness`（见第 5 组 5.2）。**准确说法应为**："agent 被迫在**每个工具各自的文本方言**上反复试错，而方言没有版本、没有 schema、没有自省"——这一点**没有反例**：第 13 篇 §四 已核实**五个 agent 协议全部不满足自描述**。 |
| 2 | "**权限全有或全无**……**没有'只许调亮度'这个档位**"（`01-现状.md:30`） | **部分成立**（**这条是本席认为最需要改写的一条**） | 反例存在且是硬的：① 内核能力集是 `CAP_CHOWN 0` … `CAP_CHECKPOINT_RESTORE 40`，**确实没有 `CAP_BACKLIGHT`**——但"只许调亮度"今天**可表达**：logind 在 `org.freedesktop.login1.Session` 上提供 `SetBrightness(s subsystem, s name, u brightness)`，vtable 标 `SD_BUS_VTABLE_UNPRIVILEGED`，签名固定、范围由 `/sys/class/<subsystem>/<name>/max_brightness` 界定，官方文档明确它"allows unprivileged programs to access hardware settings in a controlled way"；② `brightnessctl` 那条 `chgrp video` ＋ `chmod g+w` 是 **brightnessctl 自带 udev 规则**（`90-brightnessctl.rules`），**不是 systemd 发的**——systemd 自己只有 `SUBSYSTEM=="backlight", TAG+="seat"`（`71-seat.rules.in`），内核默认是 root:root **0644**；③ sudoers 可以用正则精确约束参数（`^…$`，上限 1024 字符）。**所以准确的痛点是**：不是"没有档位"，而是"**档位是逐 API 硬编码的、不可组合的、且一旦要 root 就全通**"。 |
| 3 | "**无审计** shell history 可删、可绕过；agent 到底改了什么，只能翻聊天记录"（`01-现状.md:31`） | **部分成立**，且**内部自相矛盾** | "shell history 可删可绕过"**完全成立**：`HISTFILE` 未设置／为空则"the shell does not save the command history when it exits"；`set -o history` 原文"This option is on by default in **interactive** shells"⇒ `bash -c` 一行不留；`HISTSIZE` 默认 500。**但"无审计"过头**：auditd 的 `EXECVE`（type 1309）记 `argc=` ＋ `a0..aN`，`PATH`（type 1302）记文件路径，`auid` 是 loginuid **且能穿过 sudo/su 保留原始登录 uid**（要 `CAP_AUDIT_CONTROL` 才可改，`--logaudit-immutable` 可锁）；sudoers 的 EVENT LOGGING 默认记 `USER=… ; COMMAND=…`。**同一份文件第 56 行**把 journald 列为"agent 动作的可信审计"，与第 31 行"无审计"**直接冲突**（`07` 篇第 9-10 行的修订标注已经自认了这处矛盾）。**准确说法**："动作记录**存在，但默认不产生、与主体脱钩、且可被 root 抹掉**"。 |
| 4 | "**无资源限制** agent 起一个 build 吃光 32G 内存，你的桌面会话跟着死"（`01-现状.md:32`） | **部分成立**（结果成立，**归因错**） | 归因错：`01-现状.md:54` 自己把 cgroup（2007）列为"早就存在"的机制。机制今天**可用且细**：`MemoryMax=`→`memory.max`、`MemoryHigh=`→`memory.high`、`MemorySwapMax=`、`systemd-run --scope -p MemoryMax=2G`。真因是**默认不设**：上游 `units/user-.slice.d/10-defaults.conf` 只有 `TasksMax=33%`，`units/user@.service.in` 只有 `Slice=`/`Delegate=pids memory cpu`/`TasksMax=infinity`/`OOMScoreAdjust=100`，**没有 MemoryMax/MemoryHigh**；cgroup v2 原文"**it's not going to kill any tasks outside of this cgroup**"⇒ 受害者从会话内部挑。缓解靠 `systemd-oomd`（按 `ManagedOOMMemoryPressure=kill` 只杀**后代** cgroup），但**按单元 opt-in 且服务需 enable**。**这句应改写成"默认无限制 ＋ 事后 OOM 挑受害者"，而不是"无资源限制"。** |
| 5 | "**无协调** 开 3 个 agent 同时改 systemd 配置/Nix 通道？互相打架，谁也不知道谁干了什么"（`01-现状.md:33`） | **成立**（且比 01 自己写的更强） | 更强的地方在于：**连"冲突可被检测"这件事本身都不成立**。journald 受信字段里没有 causal parent；auditd 无因果边；D-Bus 有 `serial`＋`REPLY_SERIAL` 但**没有资源级锁**，且其策略语言按上游原文"**is not well-suited to finer-grained policies**"；文件系统无 CAS/revision；k8s 侧 RBAC 的 `PolicyRule` 只有五个成员，**字段级要另加 admission controller 且"happens after authorization"**。⇒ "谁也不知道谁干了什么"**成立**，并且**"系统也不知道这两个动作在语义上打架"**——因为冲突是内容相关判断。 |
| 6 | "**完工不拉铃，agent 只能反复轮询**"（`07:27`）；"**无长任务零轮询机制**"（`05:112`） | **部分成立**（"零轮询"过头） | 反向证据：`Type=notify`＋`sd_notify(3)` 的 `READY=1` 原文"Tells the service manager that service startup is finished"；`OnFailure=`（v201，进入 `failed` 时激活）／`OnSuccess=`（**v249**，进入 `inactive` 时激活）；`systemctl --wait`（v232"wait for started units to terminate again"）；`systemd-run --wait` 原文"Synchronously wait for the transient service to terminate… including… the exit code and status of the main process"；D-Bus `org.freedesktop.systemd1.Manager` 的 `JobNew(u id, o job, s unit)` / `JobRemoved(u id, o job, s unit, s result)`（需先 `Subscribe()`）。**准确说法**："完工通知**只对 systemd 建模成 unit 的东西**存在；agent 随手 `fork/exec` 的长任务**没有自己的 job**（只是 `session-N.scope`/`user@UID.service` 的成员），因此**不可订阅、只能轮询**"——这个更弱的版本**完全成立**，而且正好指向 `07:27` 的"事件网关缝合"主张。 |
| 7 | "**零件齐全三十年**（Landlock 最年轻也 5 年了），缺的从来不是技术，是那个'总装工'"（`01-现状.md:61`） | **部分成立** | 判断（缺总装）**成立且本席同意**。事实层两处不准：① "齐全三十年"——cgroup **v2** 统一层级是 2016 年前后才稳定（2007 是 cgroup v1），Landlock 是 **2021**（`01` 自己写了 2021）；② 把"零件在"当作"零件够用"漏了**真缺口**：`07` §6 自己列的"能力委托原语（跨进程 capability delegation）"**内核确实没有**，这不是总装问题。 |
| 8 | "**审计可篡改** 仓库管理员自己能涂账本"（`07:25`）；"append-only + 定期哈希链封存别处（U 盘/异机）"（`07:25`） | **成立**（**且本席建议保留 `07` 的自我修订意见**） | 支撑证据比 `07` 自己写的更充分：journald 只有 **tamper-evident**（FSS 目的是"protect journal files from **unnoticed alteration**"，且要 `journalctl --setup-keys` 生成密钥才生效、`--interval=` 默认 15min）；`journalctl --rotate/--vacuum-*` 由 root 执行；`chattr +a` 原文"**Only the superuser or a process possessing the `CAP_LINUX_IMMUTABLE` capability can set or clear this attribute**"；CloudTrail 日志文件完整性校验 **API 默认 false**；S3 Object Lock 的 `GOVERNANCE` 可被 `s3:BypassGovernanceRetention` 绕过；Kafka/CDC/n8n/Okta/GitHub **全都有保留窗**。⇒ `07` 第 9-10 行提出的降级为"**agent 之外 + 不可否认（哈希链定期封存）**"是**正确且必要的**；`02-目标架构.md:193` 那句"**不可被 agent 自己删改**"在这个降级下才站得住（因为 agent 不是 root 时成立，是 root 时不成立）。 |
| 9 | "**GPU 无配额计量** 最贵的房间没电表没门禁"（`07:24`） | **部分成立** | "没门禁"成立（`drm.memory.max` 覆盖差，`07` 自己写了"刚冒头、驱动覆盖差"）；"**没电表**"过头：**这是我的推断**，但方向明确——`nvidia-smi` 侧有 per-process 的 accounting（`nvidia-smi --query-compute-apps`）与 MIG 分区，AMD 侧有 `amdgpu` 的 fdinfo 计量（`drm-engine-*`/`drm-memory-*`），所以"计量"在有厂商驱动的场景**部分存在**，真缺口是"**配额 ＋ 抢占 ＋ 跨厂商统一语义**"。`07:24` 的措辞比 `01` 谨慎，**保留即可**。 |
| 10 | "**事件总线碎片化** 世界的声音散在七八个喇叭里"（`07:27`）；"**统一结构化事件总线**"列为要改内核的三件之一（`07:68-70`） | **成立（现象）**，且**归属裁决正确** | 现象：coredump（systemd-coredump）／udev（netlink）／inotify（**非递归、`IN_` 语义有限**）／NetworkManager（D-Bus）／journald（`_TRANSPORT=` 混合）／`audit`（`_TRANSPORT=audit`）——**每种协议一套自省方式**。`07:72-79` 的归属裁决（第 3 项应改为"内核侧事件源统一（可选、长期）"、总线属用户态世界核心）**本席认可**：把总线做进内核确实等于把"世界的协议"锚定在载体接口上。 |
| 11 | "**世界模板不可复现**……→ 绑定 NixOS/mkosi"（`07:31`） | **成立**，但**与 `06` 的"绑定发行版"支点叠加后是风险** | 现象成立（"世界装了什么无声明式标准"）；**这是我的推断**：`06:37` 把"绑定发行版（Omarchy 的现成机制）"作为三个差异化支点之一，而 `06:13-14` 的修订标注已自认该支点"**支点失效**"——如果同时又要 NixOS/mkosi 做世界模板，那就是**两个发行版假设并存**，需要显式裁决（`06` 只撤回了①，没有处理"世界模板绑 NixOS"与"绑定发行版 Omarchy"的关系）。 |
| 12 | "**网络无身份感**：'此 agent 只许访问这几个域名'内核做不到（iptables 仅 UID/cgroup 粒度）"（`07:32`） | **成立** | 机制层无争议：netfilter 的 `-m owner --uid-owner/--gid-owner` 与 cgroup match 都是 uid/cgroup 粒度；域名级只能在用户态代理/透明代理层做，且必须**强制走代理**（否则直连绕过）。`07:32` 给出的对策"强制走代理，代理层管"**方向正确**。 |
| 13 | "**旁路面**：io_uring/memfd 是安全规则经典绕过口（Google 生产已禁 io_uring）"（`07:34`） | **成立** | 机制层成立（io_uring 的 op 提交不经普通 syscall 面，seccomp 的 syscall 过滤天然抓不住；`memfd` 让对象无名可查）。"Google 生产已禁"**未能核实到一手来源**（属可核实的公开安全公告范畴，但本席本轮未取到原文）——建议在文档里标为**待核实**或补引用。对策"Landlock 规则集显式堵上"**方向正确但不够**：Landlock 管的是文件系统访问，**堵不住 io_uring 的异步提交面**，需要 seccomp 层面的 `io_uring_setup/enter/register` 拒绝——`07` 与 `02` 的 cap.d→规则集映射需要把这条写成具体 syscall 名单。 |
| 14 | "**五个 agent 协议全缺「引用（内容 hash）」……必须自造**"（`13:6,152,206-213`） | **成立** | 逐件核过的结论一致：MCP 的 `Resource` 只有 `uri/name/title/description/mimeType/annotations/size/_meta`（**无 version/hash**）；A2A 的 `Artifact` 无 hash；ACP 的 `fs/*` 只给路径；AG-UI 的 patch 是 RFC 6902（**无内容地址**）；OpenAI 的 `file_id` 是不透明 ID。⇒ "④引用全缺"**成立**，"必须自造"**成立**。 |
| 15 | "**规范自己承认** 'protocol itself cannot enforce these security principles at the protocol level'——正好反证我们'门禁必须在协议之外有强制力'的立场"（`13:75`） | **未能核实（原句）／结论成立** | 本席在 MCP 官方 2025-06-18 规范与 security best practices 里**没有找到这句原话**。找到的是等价的、更强的表述：Tools 是"**model-controlled**"、"the protocol itself does not mandate any specific user interaction model"、人在环中是 **SHOULD**；`ToolAnnotations` 原文"are **hints**… not guaranteed to provide a faithful description"、"clients **MUST** consider tool annotations to be **untrusted** unless they come from trusted servers"。⇒ **建议**把 `13:75` 的那句直接引文改成上列**真实原文**（引用纪律：`13` 篇本身以"逐页打开官方规范正文、不采信二手博客"自称，这处应当同标准处理）。 |
| 16 | "**系统全景图** §五 #2：门禁的强制力靠文件权限，不靠内核自缚……'不可绕过' = **进程边界 + 权限位**（跨 uid 时成立）"（`系统全景图.md:157`） | **成立，且这是最诚实的自我登记之一** | 与本席核到的机制一致：`WC-SRS-001` 的 `REQ-F-025` 明确写"**CLI 路径下 `actor` 仍是命令行参数，被管者可自称 `world://user` 冒充最高权主体**⇒ CLI 入口不得作为身份保证"（`src/channel.rs:5-7` 自述）；`REQ-F-026` 明确登记"通道四个边界**一个都没有**"（`read_line` 无长度上限、一次只处理一个连接、无超时、无限流）；`WC-IRS-001` §3.8.3 登记了 `IF-007` 的 fail-open 缺口（默认相对路径 ＋ 静态墙在 0777 目录下不生效）。⇒ **与本席"现有系统接不住"的结论同构**：自研版今天也主要靠**进程边界＋权限位**，尚未接 Landlock/seccomp。 |
| 17 | `13:104` 的 D-Bus 引文"**the dbus-daemon's policy language is not well-suited to finer-grained policies**… expressed in terms of D-Bus interfaces and method names, not higher-level domain-specific concepts" | **成立**（本席独立复核） | 与 `04-总纲` 及本席结论一致；**并且这是本席找到的最强外部旁证之一**：一个被广泛部署了二十年的总线，自己承认"策略语言不适合细粒度语义授权"、"语义授权应下沉到服务"。⇒ "**总线只判传输与名字，语义授权下沉服务**"（`13:190`）**成立**。 |
| 18 | "**概念重复：约 50%**……**实现重复：接近 0%**"（`06:35-36`） | **部分成立（不可机械核对）** | 事实层可用（`06` 的对照表逐一给了亲验结论）；但"50%""接近 0%"**是估计而非测量**——没有给出分母（"agent OS"候选集合的边界）与抽样规则，**任何人都无法复算**。`06:18` 已自认第 26 行的"没有第二个项目如此主张"是**普遍否定**并改为"本轮亲验未发现"——同理，这两个百分比也应标注为"**本轮亲验下的估计**"。 |
| 19 | `06:37` 三个差异化支点之③"**不新造内核模块、不建云沙箱，只用 Linux 十五年攒下的现成零件**" | **部分成立** | "用现成零件"成立；但"**十五年**"与 `01:61` 的"三十年"**互相矛盾**（同一批机制被写成两个年数），且 `07:66-70` 又列出"真要改内核的三件"——三者需统一时态与年数（`07:6-8` 的修订标注已自认这一处，**但 `06` 与 `01` 的措辞没有同步**）。 |
| 20 | `15:287` "**坏了能回滚**（不用载体快照）……今天的 OS：❌ **只有文件系统快照**" | **部分成立** | "操作系统没有语义级回滚"**成立**（机制见第 3 组）。"只有文件系统快照"**过头**：数据库有事务回滚、SQL Server 有 temporal table 的 `FOR SYSTEM_TIME AS OF`、git 有 `revert`/`reset`、k8s 有 `kubectl rollout undo`、NixOS 有 generation——**这些都是"应用/平台级"的回滚，粒度仍不是"字段"**。准确说法："**回滚只存在于各应用自己的语义里，跨应用、跨字段的回滚不存在**"。 |
| 21 | `15:291` "五条验收今天的 OS 一条都给不了，原因只有一个——**它里面没有"本体"这一层**" | **部分成立（"原因只有一个"过头）** | 五条"给不了"**本席认可**（并在第二、三、五组给了逐条机制证据）。"**原因只有一个**"是**单因论**，与文档自己的其他证据冲突：`07:23` 说天花板在"**吊销不能即时生效（seccomp 不可卸载）**"——这是**内核原语**问题；`07:66-70` 说"能力委托原语"要**改内核**；`07` §4.3 说"真·不可篡改"要**第二个地点**——这是**物理**问题。⇒ 至少还有**三个独立原因**：缺能力委托原语、缺跨进程吊销、缺异地保管。**建议改为"最承重的原因是缺本体这一层"**。 |
| 22 | `系统全景图.md:117` "外部依赖：只有一个 `serde_json`……换语言 ≒ 世界不归零" ＋ `:115` "Rust 21 个源文件 / 6591 行 + 2435 行 tests；二进制 865712 字节；CLI 22 子命令" | **成立（数字自洽，可复算）** | 数字给到了复算命令（`find src -name '*.rs' \| wc -l`、`ls -l`、`--help`），符合"现场实取"的登记纪律。**但注意与"世界核心"主张的落差**：`15:133-144` 的部件表现在仍是"❌ 无（实例/主线/管道/语言投影）"，即**账本/主线/投影这些"本体"的核心件在 `系统全景图` 里已有 M02/M06/M07，但 `15` 篇尚未回填**——**两份文件的现状表不一致**（**这是我的推断**：`15` 篇整理于 2026-09-26，`系统全景图` 取数于 2026-09-27，属"文档未同步"而非事实冲突，但对外部读者会造成"到底有没有"的歧义）。 |

**关键更正（本席主动记入，供委托方处置）**：`13:42` / `13:174` 把 DAP 的 `seq` 与 OpenAI 的 `sequence_number` 都当作"每事件一个全局单增 seq"的现成方案——**方向上成立，但有一个隐含差异**：DAP 的 `seq` 是**每条协议消息**（请求/响应/事件）一个序号，不是"每条语义事件"；OpenAI 的 `sequence_number` 是**流内单调**，断流恢复靠 `previous_response_id` 而非序号回放。⇒ `13` 篇 §五.2 的"抄 DAP `seq` + `request_seq`"在工程上仍然可用，但**"从指定 seq 回放"这一能力只有 JetStream 的 consumer 游标提供**（`13:181-183` 已写对）。此处建议在 `13` 篇补一句边界说明。

---

## 四、总问题 A / B / C

### A. 这个自研项目的理念还缺什么（外部视角：它有没有漏看现实中很重要的东西）

**A1. 缺"效果"这一层——账本记了意图与因果，没定义"副作用发生在哪里、由谁验证"。**
`14:78-90` 的 `trace` 字段处理的是"事件之间的因果"，`15:154` 说"可回滚 = 追加补偿事件"。但**外部世界的效果不会因为账本追加了一条补偿事件而消失**：S3 对象还在、邮件已发、款已扣。LangGraph 的 `checkpoint` 是同构的先例（**状态可 fork，外部效果不可 fork**）。**要求**：本体必须为每个事件定义"**效果引用**"（效果发生在哪个外部系统、以什么标识可查询、补偿动作是什么），否则"可回滚"只能覆盖世界内部的对象图。**这是我的推断。**

**A2. 缺"对账边界"——世界账本与外部系统的账怎么对上，没有条款。**
`15:126` 写"机器/Agent/人三方都只写事件"，但现实中大量状态变化**不经过世界核心**（云厂商侧、SaaS 侧、第三方 API 侧）。这些系统**要么没有账本，要么账本在别人手里、保留期还各不相同**（第 2 组场景 2.2 的时间窗表）。**要求**：必须有一份**可判定的"外部系统对账清单"**：每个外部系统要么给出"可查询的效果标识 ＋ 可拿到的时间窗"，要么被明确登记为"不可对账"，并且"不可对账"必须影响该动作的不可逆性定级。**这是我的推断。**

**A3. 缺"委托链"——权限与责任怎么沿"人 → agent → 子 agent → 第三方 SaaS"逐跳衰减，没有规范。**
`WC-SRS-001` 的 `REQ-F-025` 身份是平面白名单 `subjects.allow`（`world://user`、`world://agent/*`）；理论上"能力 = 声明即请求 ＋ 产权三档"（`00-理论定稿.md:104-116`）。但现实里的形态是**多跳委托**（人委托 agent，agent 委托子 agent，子 agent 调第三方 API），而这个形态在外部已经有一个**被验证过的失败模式**：AWS 的 `iam:PassRole` ＋ `ec2:RunInstances` 提权——**两个各自看似无害的权限组合出完整管理员**，且 AWS 官方明说"**PassRole is not an API call**… **no CloudTrail logs are generated**"。**要求**：必须能判定"**当前主体的有效权限 ⊆ 其委托者的有效权限**"，且每一次跨主体委托本身必须是一条带主体与前后的记录。**这是我的推断。**

**A4. 缺"时间语义"与"重放幂等"的显式条款。**
`14:94-95` 说 `state = fold(events[0..seq])`、`at` = Unix 秒且"顺序由 `seq` 决定"。但现实里存在：时钟回拨、NTP 跳变、跨机事件合并、**外部系统在事件落地后才产生效果**（CloudTrail 平均 **5 分钟**才送达）。⇒ 重放时"某条事件的后果到底发生了没有"**不可判定**。**要求**：必须区分"**事件写入时间**"/"**效果发生时间**"/"**外部系统确认时间**"三种时间，并定义"效果未确认的事件在重放时如何处理（不重放 / 重放前先查询 / 要求幂等键）"。**这是我的推断。**

**A5. 缺"删除与不可变"的正面条款——它现在是待定洞，而它撞的是法律。**
`15:322` 把它列为第 9 个待定洞（"真删与不可变账本冲突"）。**本席认为它被低估了**：GDPR/个保法下的删除权是**有时限的义务**，而 `15:154` 的"没有人（包括世界核心自己）可以改历史"是**结构性承诺**——两者相遇时，**必须有一方让步，且让步方式必须写在本体里**（否则就是工程现场随手决定）。**要求**：本体必须能表达"某条历史记录的内容已被销毁，但其存在性与因果位置仍可证"这种状态。**这是我的推断。**

**A6. 缺"资源账本"与"成本归属"。**
`15` 与 `14` 只记语义事件，不记资源；`07:24` 承认 GPU 无配额计量。⇒ **"谁受益/谁承担"（`00-理论定稿.md:180` 的"争夺可见"四问之两问）在资源层答不出来**：一次任务的推理成本、GPU 秒、电费**没有对应的账本字段**。**这是我的推断。**

**A7. 缺"可用性/降级"的语义。**
`07:44` 承认"断网 = 思考停"，`07:52` 承认"电脑关 = 社会睡"。但**降级期间"动作被拒绝"这件事本身要不要留痕、要不要在恢复后补记、补记的事件在 `seq` 上排在哪里**——`15` 篇没有条款。**这是我的推断**：这会直接打穿"人离开 24 小时能答出发生了什么"（因为那 24 小时里可能有大段"机器关着"）。

### B. 怎么把它丰富起来（可执行建议）

> 每条都指向**现有文件的具体落点**，并给**可判定的验收形态**。

1. **立"两类账本 ＋ 一张对账表"**（落点：`15` 篇 §五 新增一节）
   - 语义账本（现有）＋ **效果账本**（每条效果：外部系统 / 标识 / 可查询接口 / 确认时间 / 补偿动作）；
   - 一张**对账表**：`{外部系统, 是否可对账, 可查窗口, 效果标识形式, 确认延迟上界}`；
   - 判据：**每一个声明为"可回滚"的能力，其效果账本条目必须能在对账表里查到对应的可查询接口**。
2. **给每个事件加"效果引用"字段**（落点：`14` 篇信封 + `ontology.json`）
   - 形态与 `ref` 同族，但语义是"**后果**"而非"**值**"；judgement：无效果引用的事件必须被归入"**世界内部变更**"这一类，不得参与"可回滚"的声明。
3. **云侧锚点：把"为什么"锚在世界的边界上**（落点：`09` §二"锚定 vs 依赖"的应用）
   - 依据：`09:69-70` 已经推出"世界必须自己定义算力端口"，且 `09:76` 要求"降级模式"入阶段 B 验收；
   - 建议：**算力端口的请求/响应必须由世界核心封包**（世界生成 `request_id` 与意图摘要，模型侧只能回填结果），从而让"为什么"至少有**世界自己写下的那一半**；判据：**断网时，此前已提交的算力请求必须能在恢复后与账本条目一一配对**。
4. **委托链条款**（落点：`REQ-F-025` 扩写 + 本体能力模型）
   - 要求：能力声明必须带**委托者**；判据：**存在一条判据能证明"子主体的有效权限不超出其委托者"**（可机械核对的集合包含判定）。
5. **时间语义三条**（落点：`14` 篇信封）
   - `at_written` / `at_effect` / `at_confirmed`；判据：**对任意一条"效果已确认"的事件，三个时间必须可分别读出且单调性关系被强制检查**。
6. **删除权的技术前置**（落点：`15:322` 洞 9 结案）
   - 要求：账本必须能表达"**内容已销毁、存在性与因果位置仍可证**"；判据：销毁后**前缀折叠的指纹必须给出确定的不同值**（`系统全景图.md:145` 已有一条同类反假断言"前缀折叠必须给出不同指纹"，可直接沿用形态）。
7. **资源账本与"谁承担"**（落点：`07` §2#2 ＋ 本体）
   - 要求：每次动作必须能加上"资源消耗（至少：挂钟、CPU 秒、GPU 秒、token）"；判据：**能按主体汇总出成本，且汇总值与各动作明细相加相等**。
8. **降级语义**（落点：`15` 篇新增一节"世界睡着的时候"）
   - 要求：停机/断网期间的"被拒动作"必须能在恢复后被区分于"未提交动作"；判据：**恢复后的第一份读模型必须能回答"我睡了多久、睡之前有哪些未闭合的意图"**（这与 `07:62-63` 的"登记簿持久化"是同一条，应合并）。
9. **工程侧三项**（承接 `WC-SRS-001` 自己已登记的缺口，优先级最高的三条）
   - `trace` 的**写入入口**（`WC-SRS-001` `REQ-F-031` 判据 (2)(3) 阻塞于此，`World::commit` / CLI `append` 均无该参数）；
   - 通道**四边界**（`REQ-F-026`：单行上限、并发上限、读超时、限流——当前四个全红）；
   - **CLI 入口的身份不可核**（`REQ-F-025`：CLI 路径下 `actor` 仍是命令行参数）——在补上之前，"越界声明被拦下"这条验收**只在通道路径成立**。
10. **演示第四条"24 小时可答"的最小形态**（落点：`WC-SDP-001` 的 AC 表）
    - 依 `00-理论定稿.md:177-179` 的度量协议（"事后可答率须 100%"）与 `15:288` 的五条验收——今天五条里第四条**没有对应的 AC**（`WC-SDP-001:434` 只落了 AC-02 同一屏）。

### C. 反面举证的总结（三句话说服怀疑者）

1. **进程没有语义身份、状态没有字段、历史没有因果**——所以"谁在何时改了什么、为什么"这五个空位里，今天的系统只在"何时"和"哪个文件"上有答案；其余三格全靠你自己拼聊天记录与 shell history（`HISTSIZE` 默认 500、`bash -c` 一行不留、`journalctl --vacuum-time` 由 root 执行、CloudTrail 数据事件默认不开且另计费）。
2. **能表达的权限粒度是"路径/标签 × 权限位"，不是"语义"**——SELinux 的 `file` class 权限集内容盲、AppArmor 原文"the contents of messages are not examined"、k8s 官方承认字段级控制只能交给 admission controller 且"happens **after** authorization"、polkit 的 uid 0 直接短路、`auth_admin_keep` 缓存 300 秒且**忽略参数差异**——所以"管理员设了权限却仍然发生了不该发生的事"不是事故，是**设计边界**。
3. **依赖越深、监督越不可能，而今天的机制还在替你把证据弄丢**——通知粒度是进程不是意图、长任务完工不拉铃只能轮询（每次轮询一次推理）、审批记录只有"谁点了批准"而没有"他当时看到了什么"（GitHub 的审批对象连审批时间戳都没有）——所以"批准即背书"在机制上**不可反驳**：你连他当时看到的那一屏是什么都拿不出来。

---

## 五、反面清单：委托方的哪些批评是过头的或错的

> 这一节是本席的**核心交付**之一。以下按"过头程度"排序，**每条给出改法**。

| # | 过头的说法（出处） | 问题 | 建议改法 |
|---|---|---|---|
| 1 | "**权限全有或全无**……**没有'只许调亮度'这个档位**"（`01:30`） | **最过头的一条**。logind 的 `SetBrightness` 就是一个"只许调亮度"的窄 API（`SD_BUS_VTABLE_UNPRIVILEGED`、签名固定、范围由 `max_brightness` 界定、官方文档明说是给非特权程序用的受控入口）。**用一个不成立的反例开篇，会让整个"能力模型缺失"的论证被一句反驳打掉。** | 改为："**档位是逐 API 硬编码的、不可组合的、没有统一语法**——今天要表达'只许调亮度'，你得指望某个守护进程恰好为它写了一个窄方法；换成'只许改 nginx 的 worker_processes'就没有任何一层能表达。" |
| 2 | "**无审计**……**只能翻聊天记录**"（`01:31`） | 与同文件第 56 行（把 journald 列为"agent 动作的可信审计"）**自相矛盾**；也与 `02:193` 的"不可被 agent 自己删改"矛盾（`07:9-10` 的修订标注已自认这处，但 `01` 未改）。 | 改为："**动作记录默认不产生（auditd 默认不记文件写）、与主体脱钩（auditd 记 uid/auid 不记任务）、且 root 可抹（journalctl --vacuum）**——所以它不能作为追责依据。" |
| 3 | "**无资源限制**"（`01:32`） | 归因错。cgroup 限制**今天可用且细**（`MemoryMax`/`MemoryHigh`/`systemd-run --scope -p`），真因是**上游默认不给用户会话设上限**（`user-.slice.d/10-defaults.conf` 只有 `TasksMax=33%`）＋ ad-hoc 子进程没有自己的 unit。 | 改为："**默认无上限 ＋ 超限后 OOM 在会话内部挑受害者**（cgroup v2 原文：不会杀本 cgroup 之外的任何任务），而 agent 的 ad-hoc 子进程**连被单独杀的资格都没有**。" |
| 4 | "**无协调** 开 3 个 agent 同时改 systemd 配置/Nix 通道"（`01:33`） | "Nix 通道"这个例子选得不好：Nix 的原子 symlink 切换**恰恰是**对这类并发比较健壮的机制（改一半不会生效）。 | 换成更硬的两个例子：**两个 agent 各自 `sed -i` 同一份 `/etc/nginx/nginx.conf`（最后写者赢、无人报错）**；**两个 agent 各自 `pacman -S` 冲突的包集（一个成功一个失败，但两份"我装好了"的报告都发出去了）**。 |
| 5 | "**无长任务零轮询机制**"（`05:112`）；"**完工不拉铃**"（`07:27`） | "**零**"过头：`Type=notify`＋`sd_notify READY=1`、`OnSuccess=`/`OnFailure=`（v249/v201）、`systemctl --wait`、D-Bus `JobRemoved`、`systemd-run --wait` 都是现成的"拉铃"。 | 改为："**完工通知只对 systemd 建模成 unit 的东西存在**；agent 随手起的 ad-hoc 长任务没有自己的 job，**不可订阅、只能轮询**。" 这个更弱的版本**完全成立**，而且**更有力**——它直接支撑 `07:27` 的"事件网关"主张。 |
| 6 | "**零件齐全三十年**"（`01:61`）；"**Linux 十五年攒下的现成零件**"（`06:37`） | 同一批机制两个年数；且 cgroup 2007 是 v1（v2 统一层级晚得多），Landlock 2021。**引用年头这种可查的硬事实出错，会削弱整篇的可信度。** | 统一改写为："**这些零件横跨 2007–2021（cgroup v1 → Landlock），最年轻的也已 5 年**"；并把 `06:37` 的"十五年"改为"**十余年**"或直接删掉年数。 |
| 7 | "**原因只有一个——它里面没有'本体'这一层**"（`15:291`，被 `WC-FSR-001:743,804` 直接引用为否决理由） | **单因论**，与文档自身证据冲突：`07:23`（seccomp 不可卸载 ⇒ 吊销无法即时生效）、`07:68`（缺**跨进程能力委托原语**，要改内核）、`07:52`（真·不可篡改要**第二个地点**）——这三条都**不是**"缺本体"能解决的。 | 改为："**最承重的原因是缺本体这一层**；此外还有三个独立的结构性原因：缺能力委托原语、缺跨进程吊销、缺异地保管。" |
| 8 | "**坏了能回滚**……今天的 OS ❌ **只有文件系统快照**"（`15:287`） | 过头：数据库事务回滚、SQL Server temporal 的 `FOR SYSTEM_TIME AS OF`、git `revert`、`kubectl rollout undo`、NixOS generation 都是"回滚"。 | 改为："**回滚只存在于各应用自己的语义里；跨应用、按字段的回滚不存在**。" |
| 9 | "**五个 agent 协议全缺**"（`13:152`，指④引用／内容 hash） | 这条**本席核实为成立**，不属于过头；但同一张表里"**自描述这一条，五个 agent 协议全部不满足**"（`13:160`）**略过头**：A2A 的 `AgentCard`（`13:79` 已亲验含 `supported_interfaces[]/capabilities/skills[]/signatures[]`）**就是**一种可下发的能力声明，只是**不是每条消息自带 schema**。 | 把"全部不满足"限定为："**全部不满足'每条消息自带 schema 版本 ＋ 语义类型 ＋ 单位/量纲'这一条**；A2A 有物种级自描述卡（`AgentCard`），但没有消息级自描述。" |
| 10 | `13:75` 的 MCP 直接引文 "protocol itself cannot enforce these security principles at the protocol level" | **本席在官方规范里未找到该原句**（`13` 篇自称"逐页打开官方规范正文、不采信二手博客"，这处应同标准处理）。 | 换成可核实的真实原文：Tools 是 "**model-controlled**"、"the protocol itself **does not mandate** any specific user interaction model"、人在环是 **SHOULD**；以及 `ToolAnnotations` 的 "are **hints**… **MUST** consider tool annotations to be **untrusted** unless they come from trusted servers"。后者**比原引文更有力**。 |
| 11 | "**审计可篡改**"（`07:25`）与 `02:193`"**不可被 agent 自己删改**" | 不是过头，是**两处互相矛盾**（`07:9-10` 自己已登记）。 | **按 `07` 的自我修订执行**：统一降为"**agent 之外 ＋ 不可否认（哈希链定期封存）**"；并把 `02:193` 的措辞一并改掉。**本席建议 `02` 篇的原文也要动，而不是只在 `07` 里加标注。** |
| 12 | "**概念重复约 50%／实现重复接近 0%**"（`06:35-36`） | 不可复算：没有分母与抽样规则；`06:18` 已把同段的普遍否定改为"本轮亲验未发现"，这两个百分比**没享受同一处置**。 | 加限定词："**本轮亲验（2026-09-23 三组检索＋浅克隆亲读）下的估计**"；并补一句"分母 = 检索命中的 N 个仓库"，让后人能复算。 |
| 13 | `06:37` 支点①"**绑定发行版（Omarchy 的现成机制）**" | `06:13-14` 已自认"**支点失效**"，但**同文件第 37 行仍把它列为三个支点之一**——修订只改了标注，没改正文主张（这与 `06:15` 自己批评的"虚报已修正"是同一类问题）。 | 把 `06:37` 的支点①改写为失效后的现状："**不依赖特定发行版；与 Omarchy 的关系是'可对接'而非'必须绑定'**"，避免 `07:31`（世界模板绑 NixOS/mkosi）与它并存造成双发行版假设。 |
| 14 | `13:42`"**全局单增 `seq`**……抄 DAP `seq` + `request_seq`" | 隐含差异未写：DAP 的 `seq` 是**每条协议消息**（含请求/响应/事件）的序号，不是"每条语义事件"；OpenAI 的 `sequence_number` 是**流内单调**，断线恢复靠 `previous_response_id`。 | 在 `13` §五.2 补一句边界："**'从指定 seq 回放'这项能力，只有 JetStream 的 consumer 游标提供；DAP/OpenAI 只提供定序，不提供回放。**"（`13:181-183` 已写对回放来自 JetStream，只需把这两处显式连起来。） |
| 15 | `13:229` 把"第三路（状态与事件底座）仍在进行"留在正文 | **不是错，但是风险**：§四 的覆盖矩阵（本篇"最承重的一张表"）**没有覆盖事件溯源/CQRS/CRDT/Matrix/JSON-LD** 这一路，而"我们的增量"结论正是建立在这张表上。 | 在 §四 表下加一行"**未覆盖**：事件溯源/CQRS、CRDT、Matrix state resolution、JSON-LD——**结论 `④引用全缺 → 必须自造` 仅在这 11 件参考件范围内成立**"。 |

### 附：本席未核实 / 未能取到的项（诚实登记）

1. **"Google 生产已禁 io_uring"**（`07:34`）——未取到一手来源，建议补引用或标注待核实。
2. **Rhino Security Labs 的 `iam:PassRole` 提权原文**——站点 403，但 **AWS 官方 IAM/EC2 文档独立陈述了同一提权**（本席已引官方原文），结论不受影响。
3. **RFC 5848（Signed Syslog）的部署广度**——未取到主流实现证据（rsyslog 官方模块索引里没有 `syslog-sign`）；"存在但没人用"**是本席推断**。
4. **NIST SP 800-53 Rev.5 AU-3/AU-3(1) 的字段清单原文**——未取到（OSCAL catalog 截断、PDF 不可用）；**本席未据此断言任何事**。
5. **发行版是否给 `user@.service` 预置 `ManagedOOMMemoryPressure=kill`**——未核实（Fedora dist-git 与 Debian 检索站均被 JS 挑战挡住）；上游 systemd 树内**没有**该文件。⇒ 第 1 组场景 1.6 的结论**只依赖上游事实**（无 `MemoryMax` ＋ cgroup v2 的"不杀本 cgroup 之外"语义）。
6. **Kantara Consent Receipt / ISO 27560**——403，未取到；"没有任何标准记录'批准时呈现给人的信息状态'"这一条**基于 RFC 9396、C2PA 2.1、GitHub 审批 API 三处的一手核验**，非基于这两份未取到的文件。

---

**本席一句话收束**：
> 今天的问题不是"系统不够聪明"，而是**系统从来没有被要求回答"谁、对什么、从什么变成什么、为什么"这四个问题**——
> 它被设计来管**进程与字节**，而人和 agent 需要的是管**意图与字段**。
> 这两者之间的那层东西，**外面确实没有**；但把它造出来之前，请先把上面第 1、2、3、5、7 条（各自的过头处）改掉——
> **因为一个可以被一句反例打掉的开篇，会让所有成立的论证一起被扔掉。**
