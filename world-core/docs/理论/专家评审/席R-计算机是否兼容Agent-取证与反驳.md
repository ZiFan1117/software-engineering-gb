# 席R · 「今天的计算机不兼容 Agent」这个判断能不能立住 —— 取证与反驳

> **本席的题目**（委托方逐字原文）：
> 「像现在呢，**现在的这个电脑只是服务于现在的这套软件系统，它是不兼容，其实有些时候是不兼容 Agent 的**，Agent 调用是很费劲的。……但是我们考虑了，**语义有了，就是这个语义认知是一致的时候，而且我们跟上也能去开发一些接口给这个 Agent。整个这个需求从底层就有 Agent 的这个位置的话**，那这个事儿是不是就是符合现在这种状态的？」
>
> **本席只判这一条。** 不打分、不总结优点、不替作者改立场。
>
> **取证纪律**：凡引本仓库文件 → 路径 ＋ 行号 ＋ 逐字原句；凡引外部资料 → 给链接；凡推断 → 显式写「**这是我的推断**」；查不到 → 写「**查不到**」。
> **本席未修改任何被评审文件。** 本文是本次评审唯一写入的文件。
>
> **本席的立场声明（写在最前，便于读者核对）**：本席对这条判断的结论是**半成立**——它在一个窄意义上（语义层）成立，在它的字面全称意义上（"电脑不兼容 Agent"）**不成立**，而且**仓库自己的文档已经把它的字面意义否掉过至少三次**（§Q2 反例 5、§Q5）。

---

## 零、先固定三个词，否则这句话无法判真假

作者这句话里有三个词各自至少有两个意思。不先切开，判"成立/不成立"就是空转。这是本席作的**定义性分离**，不是原文：

| 词 | 读法 A（**能力**义） | 读法 B（**主体**义） | 本席判它实际用的是哪个 |
|---|---|---|---|
| **"兼容"** | 能不能在这个机器上**跑起来**一个 agent 进程、能不能**调用**机器上的东西 | 这个机器的底层机制里，有没有**为 agent 这种主体预留的位置**（身份、授权、留痕、归责） | **B**。因为紧接着的一句是"Agent 调用是很费劲的"，"费劲"描述的是**调用成本**，不是**能不能** |
| **"位置"** | **执行位置**：进程/调度/资源配额里的一格 | **语义位置**：账本与权力结构里的一格（谁有权改哪个字段、这次改动的前值是什么、这个动作可不可逆） | **B**。因为同句前半是"整个这个需求**从底层**就有 Agent 的这个位置"，而后半问的是"这是不是**符合现在这种状态**"——若指执行位置，问题无意义（执行位置今天就有） |
| **"今天的电脑"** | 物理机 ＋ 内核 ＋ 固件这一层 | 物理机 ＋ 内核 ＋ **发行版装配起来的一整套系统服务**（systemd/polkit/auditd/包管理器…） | **混用**。这是原话最松的地方：罚"电脑"的时候举的是**系统服务与生态**的例子，罚"底层"的时候指的是**内核**。见 §Q1 判词 |

> **本席的方法论要求（下文一律遵守）**：任何"某层不能表达 X"的断言，必须给出**该层能表达的最小单位的原文定义**。否则就是印象。
> 例如"权限位不能表达字段级授权"——这句话只有在**逐字给出权限位能表达什么**（DAC 三元组）之后才成立。

---

## Q1 取证：今天的"电脑"里，有没有 Agent 的位置？

分四层取证。每层：**机制事实（带出处）→ 它能表达什么 → 它不能表达什么 → 判词**。
本席把作者问的三件事固定成三个判据，全文用它：

- **判据①（谁有权）**：这一层能不能表达"**谁有权改哪个语义字段**"？
- **判据②（前值）**：这一层能不能表达"**这次改动的前值是什么**"？
- **判据③（可逆）**：这一层能不能表达"**这个动作可不可逆**"？

### 1.1 内核层

#### (a) 进程、文件、权限位

**机制事实。** 本仓库已有一份逐条的外部核验，`world-core\docs\理论\专家评审\席D-外部系统反面举证-独立评审.md:24` 逐字：

> 「2. **状态是字节，不是字段**：`open/write/rename` 只表达"这个 inode 的字节变了"，`PATH` 记录（auditd type 1302）给的是 `item= name= inode= dev= mode= ouid= ogid= nametype=`——**没有一个字段说"哪个语义字段从什么变成了什么"**，所以"改了哪个字段"必须靠应用自己再实现一遍。」

同行 `:23` 逐字：

> 「1. **进程不是主体**：Linux 的一切记录都以 pid/uid/cgroup 为单位（journald 的 `_PID=`/`_UID=`、auditd 的 `SYSCALL` 记录、`/proc`），而 pid 会被复用、uid 会被连接池共享、cgroup 不区分"哪个 agent 的哪次任务"——所以"谁在何时改了什么"里的**"谁"在系统层根本不存在**。」

`:25` 逐字：

> 「3. **字段清单有四项在所有主流日志里都缺**：**行为的人（不是进程）／被改的对象与字段／前后值／因果父事件**。RFC 5424 的 `HEADER = PRI VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID` ＋ 三个 IANA SD-ID（`timeQuality`/`origin`/`meta`）**四项全无**；CloudTrail 的 `requestParameters` 是"请求参数"不是前后状态；Debezium 的 change event 是 `before/after/op/ts_ms/txId/lsn`，**没有 why**。」

`WC-BOOK-001-v0.1.md:436` 逐字（本仓自己写的机制表）：

> 「| 权限位 | **谁能读写这个文件** | **谁有权改哪个实体的哪个字段**（粒度差了两个数量级） |」

**逐机制判**：

| 机制 | 它记住/表达的 | 判据① | 判据② | 判据③ |
|---|---|---|---|---|
| 权限位（DAC rwx ＋ owner/group） | "**谁能读写这个文件**" | ❌ 只在**文件**这一级 | ❌ | ❌ |
| 文件系统元数据（mtime/size/inode） | "**字节与时间**"（`WC-BOOK-001-v0.1.md:433`） | ❌ | ❌（`stat` 给时间不给前值；要靠 `diff` 备份，而"**若备份周期是一天，你连 diff 都做不成**"——同文件 `:441`） | ❌ |
| auditd | `SYSCALL` 给 `pid= uid= auid= comm= exe=`；`EXECVE` 给 `argc= a0= a1= …` | ⚠️ **能到"人"，到不了"任务"**：`:71` 逐字「auditd 的 `auid`（loginuid）虽然**能穿过 sudo/su 保留原始登录 uid**（要 `CAP_AUDIT_CONTROL` 才能改、`--loginuid-immutable` 可锁），但它给的仍是**人**，不是**任务**——`auid=1000` 无法区分"你的三个 agent 会话中的哪一个"。」 | ❌ | ❌ |
| auditd 的可用性前提 | `:46` 逐字「想升级到"谁"就得先开 auditd 并写规则（`-w /etc/nginx -p wa`），而 auditd **默认不记录文件写**……**`comm=`/`exe=` 是 nginx 还是 `vim`？看不出来，因为 agent 用的是 `bash -c`、`sh -c`、`sed -i`、或者直接 `write(2)`**」 | ❌（默认关，且认不出 agent 的壳） | ❌ | ❌ |
| 因果 | `:47` 逐字「想升级到"为什么"需要 **causal parent**：journald 的受信字段里有 `_SYSTEMD_UNIT`、`_SYSTEMD_INVOCATION_ID`，**没有任何 `caused_by` 类字段**；auditd 也没有」 | — | — | — |

#### (b) capabilities（能力位）

**机制事实（外部，本席自己取的第一手）。** `capabilities(7)`：Linux 从 2.2 起把 root 的特权切成 capability 位，**"which can be independently enabled and disabled. Capabilities are a per-thread attribute."**，每一个 capability 定义的是**一类内核操作的许可**，例如 `CAP_CHOWN` = "Make arbitrary changes to file UIDs and GIDs"、`CAP_DAC_OVERRIDE` = "Bypass file read, write, and execute permission checks"、`CAP_NET_ADMIN` = 网络管理操作清单、`CAP_SYS_TIME` = "Set system clock"。全部条目是一张**内核操作清单**（[capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html)）。

同页还给出一条与"名义能力 vs 实际能力"直接相关的事实：作者/内核开发者被建议**不要为新特性轻易新增 capability**——"In order to keep the set of capabilities to a manageable size, the latter option is preferable… (There is also a technical limit: the size of capability sets is currently limited to 64 bits.)"。

**判**：
- 判据①：**❌ 结构上不可能**。capability 的定义域是**内核操作类别**，不是**实体的字段**。没有任何 capability 的名字里能出现"某个业务实体的某个字段"。
- 判据②：❌（capability 是进程属性，不携带数据状态信息）。
- 判据③：❌（capability 里没有"可逆性"这一维）。
- ⚠️ **一条必须记下的边界**：capability 集**上限 64 位**（同页）。这意味着"把权限切细"这条路在**内核原语层面就有位数天花板**——**这是我的推断**：想在内核 capability 模型里表达"字段级"授权，不是工程量问题，是要改这个模型本身。

#### (c) 命名空间与 cgroup

**机制事实（外部）。** `capabilities(7)` 同页说明：创建 namespace 自 Linux 3.8 起不需要任何 capability（"since Linux 3.8, creating user namespaces does not require any capability"），而 `setns(2)` 需要在**目标** namespace 里有 `CAP_SYS_ADMIN`。cgroup 侧，本仓 `WC-BOOK-001-v0.1.md:555` 记了一条**归因订正**并给了结论句：

> 「| 2 | "**无资源限制**" | **归因错**。限制机制今天可用且很细（内存上限、内存高水位、临时 scope 都可指定）。真因是**上游默认没装**：用户切片的默认配置只有任务数上限、**没有内存上限**；而内核控制组的语义是"**不会杀掉本组之外的任务**" ⇒ 内存压力时**在会话内部挑受害者**；临时子进程**连被单独杀的资格都没有** | "**默认无上限 ＋ 内存压力在会话内挑受害者 ＋ 临时子进程无法被单独处置**" |」

**判**：namespace/cgroup 表达的是**隔离域与资源配额**。判据①②③**全 ❌**，而且**方向上就不对**——它们回答的是"这个 agent 最多能用多少、能看到谁"，不是"它这次改动的语义是什么"。**注意**：本节也顺手证明了作者"今天的机制很弱"这一印象**在资源限制这一条上是错的**（本仓自己已收回，`:555`）。

#### (d) LSM：SELinux／AppArmor／Landlock

**机制事实。** 本仓 `席D…独立评审.md:227` 逐字：

> 「SELinux 类型强制就是四元组 `allow source_type target_type : class perm_set;`；`file` class 的权限集是**固定且内容盲**的（`ioctl/read/write/create/getattr/setattr/lock/relabelfrom/relabelto/append/map/unlink/link/rename/execute/…`）——**一个文件只有一个 type，没有文件内中介**（MLS/MCS 加的是整个对象的 level/range）；最细的 `allowxperm … ioctl xperm_set` 只用 **ioctl request 的低 16 位**；」

`:228` 逐字：

> 「AppArmor 是纯路径的，且上游原文对两处**本可以看内容的地方**明确否掉：DBus"the contents of messages are not examined"、unix socket"**The content of the communication is not examined**"；」

`:229` 逐字（结论句）：

> 「⇒ **"这个 agent 可以改 `/etc/nginx/nginx.conf`，但只许改 `worker_processes`，不许碰 `listen`"——SELinux 与 AppArmor 都表达不出来。**」

**Landlock（本席自己取的第一手，因为它是最近的一层，也最常被当成"新的细粒度机制"）。** 官方 man page 逐字：

> "Landlock is an access-control system that enables any processes to securely restrict themselves and their future children." / "A Landlock security policy is a set of **access rights** (e.g., open a file in read-only, make a directory, etc.) **tied to a file hierarchy**."
> "The two existing types of rules are: **Filesystem rules** … the object is a **file hierarchy** … **Network rules** (since ABI v4) … the object is a **TCP port**"
> —— [landlock(7)](https://man7.org/linux/man-pages/man7/landlock.7.html)

同页还给了 Landlock 覆盖哪些**文件操作**的完整枚举：`FS_EXECUTE / FS_WRITE_FILE / FS_READ_FILE / FS_READ_DIR / FS_REMOVE_DIR / FS_REMOVE_FILE / FS_MAKE_* / FS_REFER / FS_TRUNCATE / FS_IOCTL_DEV / FS_RESOLVE_UNIX`。

并且 `CAVEATS` 段逐字承认**覆盖不全**：

> "It is currently not possible to restrict some file-related actions accessible through these system call families: `chdir(2)`, `stat(2)`, `flock(2)`, `chmod(2)`, `chown(2)`, `setxattr(2)`, `utime(2)`, `fcntl(2)`, `access(2)`. Future Landlock evolutions will enable to restrict them."

**判**：
- 判据①：**❌ 到文件/路径/端口为止**。三套 LSM 的对象模型里**没有"字段"这个类型**。
- 判据②：❌（LSM 是准入判定，不留前值）。
- 判据③：❌（LSM 的权限项是"读/写/删/改名"，不是"可逆/不可逆"）。
- ⚠️ **本席补充一条仓库里没写的事实**：Landlock **连 `chmod`/`chown`/`setxattr` 都管不住**（同页 CAVEATS）。所以哪怕只谈"锁住一个文件"，Landlock 也不是完整边界。**这是我的取证，不是仓库原话。**

#### 1.1 判词

> **内核层：判据①②③ 全部 ❌，四条机制无一例外。**
> 内核能表达的"谁"是 **uid/gid/capability/pid/cgroup/namespace/SELinux type**——即**执行者身份**；它能表达的"什么"是 **inode 与字节**；它能表达的"权力"是**操作类别**。它**没有**"语义字段""前值""可逆性"这三个类型。
> **所以"不兼容 Agent"这四个字，如果指的是"内核里没有 agent 的语义位置"——成立，而且是硬成立**（不是工程没做完，是**类型不存在**）。

### 1.2 系统服务层：systemd / polkit / dbus policy

#### (a) systemd 的 unit 身份

**机制事实。** 本仓 `world-core\docs\理论\专家评审\席G-经济与运维-独立评审.md:368` 逐字（这是对本项目**自身**现状的核验，不是对外部系统的）：

> 「| **部署** | **不存在**：`src/` 无 systemd unit、无 PID 1、无 Landlock/seccomp、**无 `projectd` 常驻入口**（`系统全景图.md:121-131` 命令表只有 `channel`/`carrier`，无 `projectd`） | `deploy/README.md` 176 行 | **1–2**（读）；真正落地另计 |」

外部研究件 `D:\Code\research\SYSTEMD_FOR_WORLD_CORE.md:1347-1359`（标题逐字）**「4.8 「语义对象」不在 systemd 的模型里 —— 它服务的是「程序」」**，逐字：

> 「| **`_SYSTEMD_UNIT=` 作为来源标识** | 标识**哪个 unit** | 语义世界需要标识**哪个语义主体**（可能是人、组织、其他服务、一次自动化流程）。unit 名是一个太粗的粒度。**但 `_SYSTEMD_UNIT=` 的「不可伪造」属性是必须继承的** |
> | **`Restart=on-failure`** | 失败就重启，重启后是新的一次运行 | 对「只追加账本」这基本正确，**但对「不可逆操作」是错的**：重启不能撤销一次已经生效的不可逆写入。**world-core 必须区分「进程失败」（可重启）与「语义动作失败」（可能不可重试）** |
> | **`BindsTo=`/`PartOf=` 传播 stop/restart** | 依赖是**进程级**的 | world-core 需要**语义级**依赖（「这个本体版本被撤销了，所以依赖它的事件需要重新判定」）。进程依赖图无法表达这个 |
> | **unit 状态机**（`inactive`/`activating`/`active`/`deactivating`/`failed`） | 描述**进程**的生命周期 | world-core 的对象有**语义状态**（例如「已受理 / 已判定 / 已生效 / 已不可逆」），与进程是否在跑无关。**一个对象的「已生效」不应该因为承载它的进程被重启而回退。**」

同文件 `:1359` 的收束句逐字：

> 「**一句话总结**：**systemd 的机制层（进程拓扑、socket handoff、FD 交接、可信字段、权限纪律、内核强制边界）几乎全部可迁移；它的语义层（unit 状态、依赖语义、重启策略）几乎全部不可迁移。** world-core 应该把 systemd 当作**进程与权限的宿主**，而不是**语义的建模框架**。」

同文件 `:1211`（Q10 的收束）逐字给出"哪些是内核强制"的三档表，其中第三档：

> 「| **尽力而为（内核不支持就静默失效）** | `ProtectProc=`、`IPAddressDeny=`、`PrivateNetwork=`（无 namespace 时） | **弱**：官方明说 "a warning is logged and the setting is ignored" / "remains without effect" | 」

**判**：systemd 给的身份是 **unit 名 ＋ 该 unit 的进程树**，即**执行者身份**，**不是语义主体**。判据①❌（unit 不携带字段级授权）、②❌、③❌（`Restart=` 的语义恰与可逆性**相反**：重启不是撤销）。
**一处必须记下的强项**：`_SYSTEMD_UNIT=` 是**内核侧注入、客户端不可伪造**的，这是**真本事**（`:1356` 逐字"**但 `_SYSTEMD_UNIT=` 的「不可伪造」属性是必须继承的**"）。所以"身份不可伪造"这一件，systemd **做到了**。

#### (b) polkit

**机制事实。** `席D…独立评审.md:221` 逐字：

> 「Subject 只有三种：`unix-process`（pidfd/pid＋uid＋start-time）、`unix-session`（session-id）、`system-bus-name`（unique name）——**"谁在做什么"永远不是 subject 的一部分**；」

`:219` 逐字（授权缓存）：

> 「`ExpirationSeconds=` 在 `[Polkitd]` 默认 **300 秒**；`auth_admin_keep` 原文"authorization checks for the same action identifier and subject will succeed… for the next brief period (e.g. five minutes) **even if the variables passed along with the check are different**"；」

`:220` 逐字（uid 0 短路）：

> 「**uid 0 完全绕过 polkit**（上游 `polkitbackendinteractiveauthority.c`：`/* special case: uid 0, root, is _always_ authorized for anything */`）；」

`:222` 逐字（这是**反例清算里最关键的一条**）：

> 「关键补证：logind 的 `SetBrightness(subsystem, name, brightness)` 是 `SD_BUS_VTABLE_UNPRIVILEGED`，**源码里没有 `bus_verify_polkit_async*`、没有 polkit action id**（对比 `Terminate`/`Kill` 用 `org.freedesktop.login1.manage`）——**"只许调亮度"这件事今天是靠一个硬编码的窄 API 实现的，不是靠能力模型**；」

外部研究件 `SYSTEMD_FOR_WORLD_CORE.md:650` 逐字：

> 「**→ world-core 的门禁应该是「服务自己 + 策略引擎」，而不是「总线/路由器」。** 把语义级授权放在路由器里会得到一个无法表达领域概念（「不可逆」）的策略语言——这正是 systemd 官方吐槽 D-Bus policy language 的点。」

**判**：polkit 给的是**执行者身份（三种 subject）** ＋ **一个 action 名的布尔的授权**。判据①❌（subject 里不含"做什么"）、②❌、③❌。
**并且它比"没做到"更糟一层**：授权被缓存 300 秒**且不区分参数**（`:219`）——即"他批准过"与"他批准了这件事"在机制上不是同一句话（`:284` 逐字）。这是**判据①的反向证据**：polkit 连"这次到底是哪个参数"都不保证看。

#### (c) D-Bus policy

**机制事实。** `席D…独立评审.md:77` 逐字：

> 「D-Bus 有 `serial`＋`REPLY_SERIAL` 配对，**但没有资源级锁**，且它的策略语言按上游原文"**is not well-suited to finer-grained policies**"；」

外部研究件 `SYSTEMD_FOR_WORLD_CORE.md:1248` 给出了该原文的完整形式与判词逐字：

> 「| D-Bus 总线只做粗粒度 `<policy>`（能否连/发/own name），细粒度官方建议下沉 polkit | 门禁：基础设施只判「能不能连」，语义级授权在服务自己的策略引擎 | **照搬。** 官方原文："the dbus-daemon's policy language is not well-suited to finer-grained policies … any policy has to be expressed in terms of D-Bus interfaces and method names, not in terms of higher-level domain-specific concepts"。**「不可逆」是领域概念，路由器表达不了** |」

同文件 `:21` 逐字（同一结论的另一处）：

> 「**即：基础设施只做「能不能连、能不能发」，语义级授权下沉到服务自己的策略引擎——这正是 world-core 门禁应该待的位置。**」

#### 1.2 判词

> **系统服务层：给的是「执行者身份」，不是「语义主体」。判据①②③ 全 ❌。**
> 但这一层有一个**必须承认的真本事**：**身份由内核侧注入、客户端不可伪造**（`_SYSTEMD_UNIT=`、journald 的 `_` 前缀、D-Bus 的 `SENDER`）。作者说的"费劲"**不是**因为这一层不做身份——它做。**是因为它做完身份就停了**：从"这是哪个 unit"到"这个动作在业务上意味着什么"之间**没有任何一层**。

### 1.3 凭证与授权层：OAuth scope / OIDC / 云 IAM / SPIFFE

#### (a) OAuth scope

**机制事实。** `席D…独立评审.md:248` 逐字：

> 「**机制级原因**：RFC 6749 §3.3 原文"The strings are **defined by the authorization server**"；RFC 6750 §3 原文"'scope' values are implementation defined; **there is no centralized registry for them**"。更细的 **RFC 9396（Rich Authorization Requests, 2023-05）确实存在**，`authorization_details` 带 `type`（"**The AS controls the interpretation of the value of the `type` parameter**"）与 `locations/actions/datatypes/identifier/privileges`，例子能到资源实例（`"identifier":"account-14-32-32-3"`）——但**语义仍由 API 定义，支持面不均**（本席只核实到 Keycloak 有官方 RAR 包）。」

**判**：OAuth scope **能**做资源实例级（RFC 9396：`identifier: account-14-32-32-3`）；**不能**做**字段级的"从什么变成什么"**。判据①⚠️**部分**（能到实例，到不了字段）、②❌、③❌。
**本条对作者原话是一个反例**：作者说的"不兼容 Agent"如果指"没有给非人主体的授权机制"，OAuth 的 `client_credentials` 与 scope **恰恰就是**给非人主体的。见 Q2。

#### (b) 云 IAM

`席D…独立评审.md:243` 逐字（两条官方警示）：

> 「两条官方警示特别能说明"授权粒度的天花板"：①"Do not try to control who can pass a role by tagging the role and then using the `ResourceTag` condition key… **This approach does not have reliable results**"；②"**PassRole is not an API call.** `PassRole` is a permission, meaning **no CloudTrail logs are generated** for IAM `PassRole`"——**一次提权的关键一步在审计里根本不存在**。」

**判**：IAM **有**"谁"，**有**"能不能"，**没有**"前值"，**没有**"可逆"。判据①❌（策略是 action×resource）、②❌、③❌。
⚠️ 尤其注意 ② 这一条对**"有账可查"这个前提本身**的杀伤：一次提权的关键步骤**在审计里不存在**。这**支持**作者"费劲"的直觉，但**理由不是"不兼容 agent"，而是"审计本身有洞"**。

#### (c) SPIFFE／SPIRE

**本席的外部核实（直接找）**：SPIFFE 的公开材料把它定位为**身份**框架——工作负载身份（SVID）。本席在 IETF 的会议材料里只取到 SPIFFE 与 OAuth 客户认证的**身份**议题（[OAuth × SPIFFE 会议材料](https://datatracker.ietf.org/meeting/interim-2025-oauth-09/materials/slides-interim-2025-oauth-09-sessa-oauth-spiffe-client-authentication-00#1#1)、[SPIFFE 与 OAuth 会话材料](https://datatracker.ietf.org/meeting/116/materials/slides-116-oauth-sessb-oauth-and-spiffe-00.pdf#1#1)）。

> **诚实登记**：**本席未取到 SPIFFE 规范原文中"SPIFFE 不做业务授权"的逐字否认句**。SPIFFE 官方定位是身份（"identity"）而非授权（"authorization"），这一点在本席取到的材料里是**上下文推定**，**不是逐字原文**。**这是我的推断，且本席标注为未能逐字核实。**

**判（按推定，且已标注）**：SPIFFE 给的是**可验证的工作负载身份**——判据①❌、②❌、③❌。它与 systemd/polkit 的问题**完全同构**：身份做到了，语义没做。

#### 1.3 判词

> **凭证与授权层：是四层里"给非人主体位置"给得最多的一层**（见 Q2），但**判据①②③ 仍全部 ❌**。
> 本层的准确定性：**它给的是"这个 workload 能不能调这个 API"，不是"这次变更在业务上意味着什么"。**

### 1.4 Agent 接入层：MCP / function calling / OpenAPI / 工具注册表

#### (a) 接入形态与失败模式

**机制事实（MCP 官方规范，本席自己取的第一手）**，[MCP 2025-06-18 · Tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) 逐字：

> "Tools in MCP are designed to be **model-controlled**, meaning that the language model can discover and invoke tools automatically based on its contextual understanding and the user's prompts. However, implementations are free to expose tools through any interface pattern that suits their needs—**the protocol itself does not mandate any specific user interaction model**."
> "For trust & safety and security, there **SHOULD** always be a human in the loop with the ability to deny tool invocations."

**判（本席据这两句作的判断）**：MCP 在"**谁在环里、谁可否决**"这件事上是 **SHOULD**，不是 **MUST**；并且**明确不做协议级的人机交互模型**。⇒ **门禁被推给 Host**。

**机制事实（本仓已核验的字段级清单）**，`席D…独立评审.md:181` 逐字：

> 「**工具定义里没有"不可逆性"这一格**：OpenAI Responses 的 `FunctionTool` 字段是 `type/name/description/parameters/strict/output_schema/allowed_callers/async/defer_loading`；Anthropic 的 `ToolParam` 是 `name/input_schema/description/type/strict/cache_control/defer_loading/eager_input_streaming/input_examples/allowed_callers`——**两家都没有 risk / reversible / destructive / idempotent / scope 字段**；`allowed_callers` 限制的是**调用上下文**（direct vs code_execution），**不是被声明的权限边界**；」

`:182` 逐字（MCP 的 hints）：

> 「**MCP 有提示，但规范自己说不能信**：`ToolAnnotations` 是 `title/readOnlyHint/destructiveHint/idempotentHint/openWorldHint`，原文"all properties in ToolAnnotations are **hints**. They are not guaranteed to provide a faithful description of tool behavior"，并且"For trust & safety and security, clients **MUST** consider tool annotations to be **untrusted** unless they come from trusted servers"——**服务端可以撒谎**；」

`:183` 逐字：

> 「**MCP 明确不做协议级强制**：Tools 是"**model-controlled**"、"the protocol itself does not mandate any specific user interaction model"、人在环中是 **SHOULD**；**MUST** 级的同意只在两个特例上（一键安装本地 MCP server 配置、以及 OAuth 授权流程）。⇒ **门禁被推给 Host，而 Host 就是 agent 的 harness**——这与整个"门禁必须低于 harness"的主张正好互证。」

`:188-189` 逐字（**没有 seq、没有版本**，这是"费劲"最硬的一处机制）：

> 「**MCP 没有 `seq`**：`JSONRPCRequest = {jsonrpc, id, method, params?}`，notification **连 id 都没有**；`notifications/cancelled` 原文承认取消"**MAY arrive after the request has already finished**"；」
> 「**MCP 的 resource 没有版本**：`Resource` 字段是 `uri/name/title/description/mimeType/annotations/size/_meta`——**没有 version/ETag/hash**；更新信号只有 `notifications/resources/updated { uri }`（**只有 URI，没有 diff**）；」

**⚠️ 本席未能逐字核实的一条**：本席**未取到 MCP 规范里 `Tool` 数据类型的逐字字段清单**（尝试抓 `tools.mdx` 源文件失败）。因此**"`Tool` 没有 version 字段"这句本席不写成事实**——`席D` 给出的是 `Resource` 的字段清单（`席D…独立评审.md:189`），本席**采信该条**，并**不**把结论外推到 `Tool`。

#### (b) 本仓对"接入成本"的机制诊断

`WC-BOOK-001-v0.1.md:480-492`（第三章 §3.4 整节，逐字引）：

> 「## 3.4 AI / Agent 框架层：MCP / tool-calling / 记忆方案 / 编排 / RAG
> > 这一层的问题**不是模型不够聪明**，而是**机制上没有可问责的位置**。
> | **tool-calling** | 把一次调用变成结构化参数 | **调用前后没有不可改的记录**：模型为什么调这个工具、参数怎么来的，**只在对话上下文里**，会话一结束就没了 |
> | **对话历史即记忆** | 简单、通用 | **它不是账本**：可以编辑、可以截断、可以摘要（= 有损）、按 token 预算丢弃。**它答不了"三个月前那次到底改了什么"** |
> | **RAG / 向量记忆** | 召回相关内容 | **相似 ≠ 真实**：它给的是"像什么"，不是"发生过什么"，更不是"前值是什么" |
> | **编排 / 工作流** | 把步骤串起来、可重试 | **重试不等于可回滚**：一个跑到一半失败的流程，**没有"补偿"这个概念**，只能整段重跑 |
> | **MCP 类接口协议** | 统一工具接入 | 它统一的是**"怎么调"**，不是**"世界现在是什么"**。世界状态仍需每个工具各自回答，且答法互不相同 |
> > **一句话要求**：**Agent 的每一次动作都要在外部的、不可被它自己修改的账上留下"意图 + 结果"，并且这两者必须能配对。**」

**判**：Agent 接入层的失败模式是**三条，且都不是"跑不起来"**：
1. **声明面没有可逆性**（工具定义里没有 risk/reversible/destructive/idempotent/scope——`:181`）；
2. **有提示但不保证**（MCP hints，"服务端可以撒谎"——`:182`）；
3. **没有共同的进度与状态坐标**（无 seq、无资源版本、更新信号只有 URI 无 diff——`:188-189`）。

⇒ **"费劲"在接入层成立**，但成立的具体内容是**"每次接入都要重新协商语义边界与进度语义"**，**不是"接不进去"**。

### 1.5 总判：**"不兼容 Agent"这四个字，在哪一层成立、在哪一层不成立**

| 层 | "不兼容 Agent"是否成立 | 成立/不成立的具体内容 |
|---|---|---|
| **内核层** | **成立（硬成立）** | 判据①②③ 全 ❌。**"语义字段／前值／可逆性"这三个类型在内核里不存在**，且 capability 64 位上限说明"切细"有原语天花板。**但**：内核**完全能**起、隔离、配额、调度 agent 进程——**执行位置一直是有的** |
| **系统服务层** | **部分成立** | **身份**给足了（不可伪造），**语义**一点没给。正确说法是"**身份做完就停了**"，不是"不兼容" |
| **凭证与授权层** | **不成立** | 这一层**恰恰是为非人主体造的**：OAuth `client_credentials`、IAM role、SPIFFE SVID 都是。它给不了的是**业务语义**，不是**非人主体的位置** |
| **Agent 接入层** | **成立（但理由与作者说的不同）** | 成立的是"**每次接入都要重谈语义**"；**不成立**的是"接不进去"——MCP/function calling/OpenAPI 每天都接得进去。**"费劲"≠"不兼容"** |

> **判词（本席给的定级）**：
> **"今天的计算机不兼容 Agent"这个全称句，只有加上限定词才站得住。**
> 站得住的最小形式是：**"今天的计算机的底层机制里，没有表达『谁有权改哪个语义字段』『改动的前值』『这个动作可不可逆』的位置——内核、系统服务、授权层、接入层四层都到不了这个粒度。"**
> 站不住的形式是：**"计算机不兼容 Agent"／"Agent 用不了"／"底层没有 Agent 的位置"**（后者见 Q5：仓库自己在三份文件里写过相反的话）。
>
> **⚠️ 补一条本席在写完上面这张表之后才拿到的证据，它比表里任何一行都硬，见 §1.6（作者补充的"截图回路"例子）。**
> **§1.6 的结论会改写这张表**：感知层不是"接不住"，而是**项目自己在最顶层已经把它立成了判断标准**。

---

## Q1.6 · 作者补充的"截图回路"例子——单独立一节（T1–T4）

> **作者补充的原话（逐字，本席收到的转述）**：
> 「现在没有给 Agent 提供相应的接口。以前我们人是使用 system 的，把这个工具用来操控计算机；但是它有一个问题：**我们打进去以后，它的反馈是显示在屏幕上的**，屏幕上的人可以看到。**但是这个 Agent 呢，就一遍一遍地截图**什么的——**它是没有眼睛的，人类没有给它提供眼睛，但是它得模仿人类有眼睛然后再干事**。其实我们应该是做成**统一的一个反馈的渠道**：你想给人看、想投影到屏幕上，好，那你就投影到屏幕上；但是呢，**如果它之前是有接口访问的，那就应该沿着这个接口再返、推回去——反馈走这个通道**，为什么还是要显示在屏幕上呢？所以说，整个系统在有定位之前，我觉得是有一定的问题的。」
>
> **本席先把这一节的总判写在最前面，因为它是本次评审里**最强**的一段实证**：
> **这个例子比 Q1 的四层取证更强，理由不是"它更真"，而是"它的候选答案已经在项目自己的最高层文件里，被写成了一条判断标准"。**
> ⇒ 见 §T3、§T4。

### T1 现象核实：今天一个 agent 要感知"刚才那次操作的反馈"，是不是只能靠截图？

**总答：不是"只能"，但"实际主流是"，而且"确实有一类场景只剩屏幕"——三层，逐层给据。**

#### ① 有没有结构化通道？**有，而且不止一条。但每一条都有一道"应用必须自己配合"的门。**

| 通道 | 机制事实（外部一手） | 它给 agent 什么 | 它要求应用做什么 |
|---|---|---|---|
| **AT-SPI2（无障碍总线）** | 逐字："**At-Spi2 is a protocol over DBus, toolkit widgets use it to provide their content to screen readers such as Orca.**"／"**The protocol essentially consists in dbus RPCs and notifications. For each application (seen as a dbus sender), its tree of widgets is represented as a tree of dbus paths.**"／"The core that defines the protocol and starts the dbus accessibility bus is **at-spi2-core**"／"**Wayland：Works just the same :D**"（[freedesktop · AT-SPI2](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/)） | **一棵结构化的控件树 ＋ 事件通知**（角色、名字、状态、可执行动作）。**这正是"结构化反馈"，而且是现成的** | ⚠️ **必须显式开启**：同页逐字给出开关 —— `IsEnabled` property of `org.a11y.Status`；"**If an application does not enable at-spi2 support by default, it should monitor that property, in order to dynamically enable at-spi2 support if it changes to true**"；并给了强制开启的环境变量清单（`GTK_MODULES=gail:atk-bridge`、`QT_ACCESSIBILITY=1`、`QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1`、`GNOME_ACCESSIBILITY=1`）。**⇒ 默认未必开；开了才在** |
| **D-Bus 方法应答** | 逐字："**sd_bus_message_set_expect_reply() sets or clears the `NO_REPLY_EXPECTED` flag on the message m. This flag matters only for method call messages and is used to specify that no method return or error reply is expected.**"（[sd_bus_message_set_expect_reply(3)](https://man7.org/linux/man-pages/man3/sd_bus_message_set_expect_reply.3.html)） | **应答沿原连接返回**：`sd_bus_reply_method_return(3)` / `sd_bus_reply_method_error(3)` ＋ header 里的 `REPLY_SERIAL`（同页与 `sd-bus(3)` 的 `sd_bus_message_get_cookie`/`sd_bus_message_get_reply_serial` 家族） | ✅ **不需要额外配合**——这是 D-Bus 的**默认语义**（要"不回"才需要显式设 `NO_REPLY_EXPECTED`） |
| **终端 PTY** | `2-依据\02-开发工作流.md:162` 逐字：「Agent **不能交互输密码、不能盯屏幕、不能手动重试**」 | 结构化输出**可以**走 stdout/stderr（本仓自己的 `WC-IC-001-v0.1.md:660` 就是一个例子：CLI 的退出码与错误码是**机读面**） | ✅ 已在；⚠️ 但**前提是程序愿意输出机器可解析的东西**——`world-core\docs\S4-实现\WC-LLD-001-v0.1.md:180` 逐字记了一条本仓自己的欠账：「| 2 | `DEBT-01` 结构化错误码（现为中文散文） | R2 前 |」 |
| **`/proc`** | 进程树、fd、cgroup、状态 | 进程**存活与资源** | ✅ 已在；❌ **不含业务状态** |
| **应用自身 API** | 有 API 就走 API | 取决于 API 设计 | ❌ **这一条就是作者说的那个缺口：没有 API 的应用，agent 无路可走** |

#### ② 截图回路在今天是不是主流做法？**是。理由不是"截图更好"，而是"只有截图是**通用**的"。**

**本仓已有的实证（不是外部印象）**：

- `2-依据\06-重复工作审计.md:26` 逐字（这是本仓对**一个真实存在、且被 Omarchy 收编的** computer-use 驱动的机制核验）：
  > 「| **CUA**（trycua/cua） | computer-use 驱动：**AT-SPI/UIA 可访问树 + 输入注入 + 合成光标**；CLI/MCP/SDK 三接口；权限模式 standard/bounded/unrestricted；跨 macOS/Windows/Linux | GUI 操作层 | **互补**：CUA 回答"agent 怎么操作 GUI"，agentd 回答"agent 能碰什么"。Omarchy 已把 `cua-driver-bin` + `cua-hyprland-plugin` 收编进自家打包仓库 |」
- `2-依据\05-Omarchy机制实证.md:115` 逐字（同一件事的第二处登记）：
  > 「| GUI 自动化 | Omarchy 已打包 **CUA 驱动**（trycua/cua：**accessibility-tree 快照 + 输入注入**，含 Hyprland 插件，`omarchy-pkgs/pkgbuilds/cua-driver-bin`、`cua-hyprland-plugin`） | **不重做**：CUA 回答"agent 怎么操作 GUI"，agentd 回答"agent 能碰什么"——两者是操作层与门禁层的关系，互补不重叠 |」
- `1-理论与哲学\01-三者关系论.md:208-209` 逐字（**本仓对"截图"与"语义树"的定级对比，这是全仓对这一问最直接的一段**）：
  > 「| 截图+坐标（computer use 原始形态） | 四灯全灭——不是镜子，是让 agent **蒙着眼摸像素** |
  > | 语义树转译（AOM/AT-SPI → ref 寻址） | 三灯半亮，且**今天就能做、覆盖一切现有界面** |」
- 同文件 `:216` 逐字：
  > 「"看截图点坐标装成一个人"，而是**语义化的 computer use**：agent 使用的对象是结构，不是像素。」
- 同文件 `:183` 逐字（作者"没有眼睛"这个比喻的仓库版表述）：
  > 「agent 的操作发生在**黑暗里**；浏览器把"看不见的代理"变成"看得见的行动"」

**⇒ 本席的判（这是我的推断，依据是上引五行）**：
**"截图回路"成为主流，机制原因是"**它是唯一不要求应用配合的通道**"**：
- AT-SPI2 **要求应用实现了 a11y 并开启了 `IsEnabled`**（上表）；
- D-Bus **要求应用注册了一个 name 且暴露了方法**；
- 应用 API **要求应用写了 API**；
- **截图不要求应用做任何事**——只要它画在屏幕上。

**这道"门"正好就是作者原话里"跟上去也能去开发一些接口给这个 Agent"所说的那件事**：接口不是没有，是**每一个应用都要各自开发一份、agent 每一家都要各自学一遍**。

#### ③ 哪些场景下确实只剩屏幕？**四类，且第四类是"被设计上禁止"。**

| 场景 | 机制原因 | 本席的核实状态 |
|---|---|---|
| **自绘 UI**（canvas / 游戏引擎 / 自定义渲染的桌面应用） | 界面上有像素，控件树里**没有对应节点**；AT-SPI 树退化为一个空壳窗口 | **这是我的推断**（机制推论，未取到逐字规范依据） |
| **无 API 的遗留程序** | 它唯一的输出面就是屏幕和 stdout | **这是我的推断** |
| **DRM / 受保护内容** | 合成器与 portal 对受保护表面**有意不允许**被截取 | ⚠️ **本席未取到逐字依据，明确标为未核实** |
| **Wayland 下的"全局输入注入"这一类** | 事实：Wayland 下要注入输入必须走 **RemoteDesktop portal**，而它的 `Start()` 逐字"**will typically result in the portal presenting a dialog letting the user select what to share**"，设备类型限于 `KEYBOARD`/`POINTER`/`TOUCHSCREEN`（[XDG Desktop Portal · RemoteDesktop](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)）；截图同样走 portal，`Screenshot.Screenshot` 的 `interactive` 逐字"**Hint whether the dialog should offer customization before taking a screenshot**"、`modal` 逐字"**Whether the dialog should be modal. Defaults to "true"**"（[XDG Desktop Portal · Screenshot](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html)） | ✅ **已取到逐字依据**。对照面：X11 侧的 **XTEST** 是"注入合成事件"的 X 扩展（[XTEST Extension Protocol](https://www.mathematik.uni-marburg.de/local-doc/rhel6/xorg-x11-docs-1.3-6.1.el6/hardcopy/Xext/xtest.pdf#1#1)），且围绕它"是否能被任意客户端调用"构成过 X11 / Wayland 的安全性争论（[wayland-devel 2012 讨论存档](https://lists.x.org/archives/wayland-devel/2012-February/002192.html)） |

> **⇒ T1 判词**：
> **作者说"Agent 就一遍一遍地截图"，现象成立，机制成立，而且本仓自己已经把它判到最低一档（`01-三者关系论.md:208` 逐字"**四灯全灭**…**让 agent 蒙着眼摸像素**"）。**
> **但"它是没有眼睛的"这个修辞要收一收**：**眼睛是有的**（AT-SPI2 的控件树就是眼睛，而且 `01-三者关系论.md:209` 逐字判它"**今天就能做、覆盖一切现有界面**"）。
> **准确的说法是**：**有眼睛，但每只眼睛都要应用自己长出来**——AT-SPI2 要应用开 `IsEnabled`，D-Bus 要应用注册 name，API 要应用去写。**agent 用截图，是因为截图是唯一一只不需要应用配合的眼睛，而不是因为它没眼睛。**

### T2 "反馈应沿原通道返回"这条原则：是不是新东西？今天为什么没做到？

**第一问：不是新东西。它是 RPC／消息系统的默认语义，四十年前就是。**

- D-Bus：**默认要应答**（上引 `sd_bus_message_set_expect_reply(3)` 逐字说明该 flag 是用来**抑制**应答的）。请求—应答**沿同一条连接返回**，靠 `serial`／`REPLY_SERIAL` 配对。
- 本仓自己的总线规格也已经写死了同一条语义。`2-依据\11-总线工程规格.md` 的数据流实例 `:45-50` 逐字（第 ⑥ 步）：
  > 「⑥ UI 投影更新（人看见）+ **agentd 记录结果**                  [投影+审计]」
  同文件 `:47-48` 逐字：
  > 「③ agent → agentd：请求 ui.web.fill(e3,"买牛奶")           [动作请求]
  > ④ agentd 查 cap.d → 记审计 → 转发 transferor             [治理]」
  同文件 `:108-109` 逐字（共享命名空间里明确**不出现** MCP/varlink 字样）——见下。

**⇒ 判**：**这不是新原则。作者提出的这条，是 RPC 的基本语义 ＋ 本仓已经写在总线规格里的东西。**

**第二问：今天为什么没做到？本席判为"三者都有，但主因是第三种（缺商业动机）＋ 一个本席新发现的第四种"。**

| 假因 | 本席的判 | 依据 |
|---|---|---|
| **缺机制** | **❌ 不成立** | D-Bus 的应答是默认（上引）；AT-SPI2 是全桌面的结构化反馈总线（上引）；MCP 的 `tools/call` 本身就是请求—应答。**机制在** |
| **缺标准** | **⚠️ 部分成立，但不是主因** | 标准也有：AT-SPI2 是既成标准；D-Bus 是既成标准。**真正的缺口在"没有一层把『谁该把结果写回哪里』规定下来"**——`WC-BOOK-001-v0.1.md:490` 逐字：「MCP 类接口协议 | 统一工具接入 | 它统一的是**"怎么调"**，不是**"世界现在是什么"**」；`:492` 逐字「Agent 的每一次动作都要在外部的、不可被它自己修改的账上留下"意图 + 结果"，**并且这两者必须能配对**」——**"配对"这一条正是标准缺的那一半，而本仓自己也还没实现** |
| **缺商业动机** | **✅ 成立，且是本席认为的主因** | 应用把反馈写到**屏幕**上，对**人**是完整的；只有对**另一个程序**才缺。而绝大多数桌面程序的产品目标是"人打开它、人用它"，**"让一个 agent 读它"从来不是它的需求**。本仓对这一层有逐字判语：`WC-BOOK-001-v0.1.md:164` 逐字「它服务的对象是"**程序**"，**程序自己懂自己的意义**」（这是本书给 systemd 划界的话，同一句对桌面应用同样成立）。**这是我的推断** |
| **漏了第四种：工具本身的 I/O 模型就是这么定的** | **✅ 本席认为这一条比"缺商业动机"更接近根，且它是可核的** | 命令行工具的**契约**是：参数进门、**stdout/stderr ＋ 退出码出门**。它**没有一个"调用-应答"的返回通道**——它的"应答"就是打印。`席D-外部系统反面举证-独立评审.md:46` 逐字给了这条的后果：「想升级到"谁"就得先开 auditd 并写规则（`-w /etc/nginx -p wa`），而 auditd **默认不记录文件写**，且 `SYSCALL` 记录给的是 `pid= uid= auid= comm= exe=`——**`comm=`/`exe=` 是 nginx 还是 `vim`？看不出来，因为 agent 用的是 `bash -c`、`sh -c`、`sed -i`、或者直接 `write(2)`**」 |

**⚠️ 本席必须为作者这一句加一条反例边界（否则它会被一句反例打掉）**：

`WC-BOOK-001-v0.1.md:550` 逐字立过一条纪律：「**一个能被一句反例打掉的开篇，会让所有成立的论证一起被扔掉**」。
作者这句话的反例是**现成的**：**对一个有 API 的工具，今天的反馈已经是沿原通道返回的。**
- 你 D-Bus 调 `SetBrightness`，**返回值沿 D-Bus 回给你**（`席D…:222` 逐字：`SetBrightness(subsystem, name, brightness)` 是一个硬编码的窄 API）；
- 你调 OpenAI 的 function calling，**`tools/call` 的 result 沿同一次请求返回**；
- 你调本仓自己的 `channel`，`WC-PFMT-001-v0.1.md:475` 逐字：「| 应答（失败） | `{"ok":false,"error":"<消息>"}` |」——**应答就是沿原通道回来的**。

**⇒ 所以作者这句话的准确形式不是"反馈应该沿原通道返回"（那是默认的），而是：**
> **"当一次操作没有走接口、而是靠人/agent 去操作界面时，它的反馈就脱离了原通道、只剩屏幕。今天的问题不是『应答不回原通道』，而是『有相当一部分操作压根没有通道』。"**

**并且本席要指出**：作者紧接着说的**"统一的一个反馈的渠道"**——**这一条本仓已经写成了规格，而且写得很完整**。见 T3。

### T3 归属判断：这是"世界核心／统一语义"该管的事，还是界面层／工具层的事？

**正面判：两者都占，而且本仓已经把这条界线划过一次了——划法对作者有利。**

#### (a) 本仓已经给出"语言投影"这个位置，并且给了它一条**与作者原话几乎同名**的定义

- `2-依据\15-世界核心的组成与职责.md:21` 逐字：
  > 「> | **语言投影 / 视觉投影** | 世界核心开的**端口**；实现可以在核心内或核心外 |」
- 同文件 `:206-213` 逐字（§7.1 命名的完整块）：
  > 「### 7.1 命名：语言投影 / 视觉投影
  > ```
  > 显 = N 端口注册表（全部同源，只许省略不许添加）
  >   ├─ 语言投影   ← 给 Agent 读（结构化，可直接进上下文）
  >   ├─ 视觉投影   ← 给人看；v1 的实现恰好是 web
  >   └─ 留位：听觉 等（总纲 §2.5 已写"听觉等留位"）
  > ```」
- `00-总纲.md:154-156` 逐字：
  > 「- **边界**：**N 端口注册表，全部同源**（v1 实现视觉 + 语言；听觉等留位）。
  > - **判断标准**：**人看到的和 agent 读到的必须是同一份状态**——不同源比截图更危险
  >   （会出现"agent 眼里的世界"和"屏幕上的世界"两个世界）。」

> **⇒ 直接把作者原话与本仓原文并排**：
> 作者：「你想给人看、想投影到屏幕上，好，那你就投影到屏幕上；但是呢，如果它之前是有接口访问的，那就应该沿着这个接口再返、推回去——反馈走这个通道」
> 本仓 `15…:210-211`：「**语言投影 ← 给 Agent 读（结构化，可直接进上下文）／视觉投影 ← 给人看**」
> **两者是同一件事的两个说法。** 作者说的"统一的一个反馈的渠道"，在本仓里叫**语言投影**；作者说的"投影到屏幕上"，在本仓里叫**视觉投影**。
> **⇒ 判：这是世界核心该管的事，而且本仓已经把它立成了概念。不是界面层的事。**

#### (b) 判据是**显式**的，而且写得比本席预想的更强

- `00-总纲.md:129` 逐字（**这是全仓唯一一条以"零截图"为形式的判断标准**）：
  > 「- **判断标准**：**agent 零截图、零坐标，能否读懂世界当前状态并发出动作。**」
- `2-依据\15-世界核心的组成与职责.md:251` 逐字（在待办清单里）：
  > 「- [ ] "Agent 直接读界面的内容"（**零截图、零坐标的判据在此**）」
- `1-理论与哲学\06-协作与承载.md:112` 逐字（三对沟通的最优承载表）：
  > 「| Agent ↔ Computer | **声明式语义调用**（JSON-RPC/varlink/能力接口） | 唯一双方皆机器的对：**零歧义、可校验、可门禁**。D-Bus 二十年已验证；自然语言让机器执行需"解析→猜参→执行"三步有损 |」
- `WC-BOOK-001-v0.1.md:219` 逐字（三方端口表）：
  > 「| **Agent** | "请你做什么"（`act`）+ **它的结果** | **语言投影** |」

**⇒ (b) 的判**：**"agent 能零截图读到结果"这条，在本仓里不是"顺带的好处"，是本仓 §2.5 `显` 这一格的判断标准本身（`00-总纲.md:129`）。**
并且 `15…:219` 那一行的"**+ 它的结果**"三个字，正好就是作者要的"**反馈走这个通道**"——**Agent 的那一格，写的是"请求 ＋ 结果"，不是"请求"**。

#### (c) 那"该由谁管、今天有没有人在管"？

| 部件 | 作者原话的哪一半归它 | 本仓的现状（逐字） |
|---|---|---|
| **世界核心的 `显`（语言投影）** | **"统一的一个反馈的渠道"这一半** | `2-依据\15-世界核心的组成与职责.md:141` 逐字：「| **语言投影** | 给 Agent 读的结构化视图 | `显` | **❌ 无** |」 |
| **适配器／脐带（transferor 及其后继）** | **"每个应用各自的接口"这一半** | `15…:230` 逐字：「**顺带钉死 transferor 的定位**：它不是"web 投影"，它是"**视觉投影在 v1 的脐带件实现**"（属 `进`，不是 `显` 的本体部分）。」 |
| **已有外部机制（AT-SPI2 / D-Bus）** | **"今天就能做"这一半** | `1-理论与哲学\01-三者关系论.md:209` 逐字：「| 语义树转译（AOM/AT-SPI → ref 寻址） | 三灯半亮，且**今天就能做、覆盖一切现有界面** |」 |

**⇒ (c) 的判：有人管，而且是本仓自己已经指派过的两个部件**——
**"统一反馈渠道"＝`显` 的语言投影；"每个应用各自的接口"＝适配器/脐带件。**
**今天两个都没实现**（`:141` 逐字"❌ 无"），**所以作者说的"整个系统在有定位之前，我觉得是有一定的问题的"，在本仓的现状表里是**绿的**——问题确实在。**

#### (d) 本席要指出的一个**更深一层**的归属判断（作者原话没说、但本仓的逻辑必然推出）

`15…:190-200` 逐字（"人让 agent 静音一条通知"穿过全景的七步）：

> 「| 6 | 运行时 | 由账本**算出读模型** → 推给**两个投影** |
> | 7 | 两个投影 | 人看到图标变；**Agent 读到 `muted=true`**（**各自只跟账本打交道**） |」

**⇒ 本席的推断（这一条是本席自己加的，原文没有）**：
**如果一个 agent 必须截图才能知道"刚才那次操作成了没有"，那么在"世界核心"的话语里，这不叫"agent 看不见"，这叫"这次操作不在账上"**——因为按 `2-依据\15-世界核心的组成与职责.md:188` 逐字，**「**三方之间没有直连**：它们**只跟账本对齐**，不互相聊。」**，又按同文件 `:126` 逐字，**「机器 / Agent / 人：三方都只"写事件"、都只"读投影"」**。
**所以"截图回路"在本项目的理论里是被**双重判死**的**：
1. **§2.5 的判断标准判它死**（`00-总纲.md:129` 逐字"**agent 零截图、零坐标**，能否读懂世界当前状态并发出动作。"）；
2. **架构判它死**（截图不是"读投影"，它绕过账本、绕过读模型、绕过同源核对——`00-总纲.md:155-156` 逐字"**人看到的和 agent 读到的必须是同一份状态**——不同源比截图更危险（会出现"agent 眼里的世界"和"屏幕上的世界"两个世界）"）。

**⚠️ 本席的诚实边界（这一条本席核错过一次，如实登记）**：本席初稿在这里引了一句"**没有账上的那一条就没有发生过**"，并把它挂到 `WC-BOOK-001-v0.1.md:518`。**复算后 `:518` 的内容是"四本账对不上 ⇒ 复盘要人工拼 ⇒ …没有反馈回路"，不是那句。**
本席**未能在 `WC-BOOK-001-v0.1.md` 里核到该句的原句行号**；本席只核到 `WC-PREFACE-LOG-001-v0.2.md:37` 对它的一处**转述**（逐字"修"没有账上的那一条就没有发生过"与收束句的主从次序"）。
**⇒ 故上面第 1、2 条一律只以 `00-总纲.md:129`／`:155-156` 与 `15…:126`／`:188` 为据，不引那句。**

### T4 与 Q1–Q6 的关系：这个例子是支持还是削弱"今天的电脑不兼容 Agent"？

#### (a) 它对 Q1 的影响：**加强了一个新的层，同时削弱了"不兼容"这个说法**

**加强的部分（本席据此在 Q1 里新增 §1.6 这一层）：**

| 层 | 加进来之后的判 |
|---|---|
| **感知—反馈层（新增）** | **"今天的系统里，一次操作的反馈默认回到屏幕，而不是回到发起者的通道"——这一条成立，而且是最日常、最普遍的一条。** 它的机制原因不是"没有机制"（D-Bus/AT-SPI2 都在），而是**"只有截图是唯一不需要应用配合的通道"**（§T1） |

**削弱的部分（三条）：**

1. **"它是没有眼睛的"不成立。** `01-三者关系论.md:209` 逐字判 AT-SPI2 语义树"**今天就能做、覆盖一切现有界面**"。**眼睛有，只是要应用自己长。**
2. **"反馈应该沿原通道返回"不是新原则**，是 RPC 默认语义（`sd_bus_message_set_expect_reply(3)` 逐字）。**用它来论证"今天不兼容"，方向反了**——今天恰恰是**沿原通道返回的地方，反馈就回来了**（§T2）。
3. **"没有给 Agent 提供相应的接口"这个全称句要收。** `2-依据\06-重复工作审计.md:26` 逐字：CUA **就有** AT-SPI/UIA 可访问树 ＋ 输入注入，**且已被 Omarchy 收编**；`2-依据\05-Omarchy机制实证.md:115` 逐字同一件事。**接口有，缺的是"统一的那一个"。**

#### (b) 它对 Q2 的影响：**本席据此给反例清单加一条"反例 8"，并调整降级档**

**反例 8 · AT-SPI2：今天已经存在一条"统一的结构化感知通道"，而且它跨工具包、跨 Wayland／X11。**

- 依据：`https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/` 逐字"**For each application (seen as a dbus sender), its tree of widgets is represented as a tree of dbus paths.**"、"**Wayland：Works just the same :D**"。
- **它给了什么位置**：**一个 agent（或任何 D-Bus 客户端）可以在没有任何应用 API 的情况下，读到该应用的控件树与事件通知。** 这正是作者以为不存在的那条"接口"。
- **它缺了什么位置**：① **默认未必开**（同页逐字"**If an application does not enable at-spi2 support by default, it should monitor that property**"）；② **只有控件的语义，没有业务的语义**——它给的是"有一个按钮叫 X"，不是"`muted` 从 false 变成了 true"（这正是本席 Q1 的判据①②③）；③ **它只覆盖实现了 a11y 的工具包**。
- **⇒ 它与 Q2 反例 1–7 是同一形状**：**给了"能感知"的位置，缺的是"这次变更的语义"的位置。**

**降级档的调整**：本席在 Q2 给的 C 档（"能跑 agent，但接不住 agent 的变更语义"）**依然成立，但要补半句**：
> **C+（本席修正后的推荐档）**：「今天的机器**能跑** agent、**也能让它看见**（AT-SPI2/D-Bus 就在那儿），**接不住的是它的变更语义**——四层机制里没有一处能回答"谁有权改哪个语义字段、前值是什么、这个动作可不可逆"；而**感知这一层连"统一"都还没有**：只有截图是不需要应用配合的那一只眼睛。」

#### (c) 它对 Q4② 的影响：**"不兼容"在感知层上反而更站不住**

- Q4② 的原判是"在语义层窄意义上成立、在字面全称意义上不成立"。
- **加了感知层之后，"字面全称意义"更不成立了**：因为**感知这件事今天有现成答案**（AT-SPI2、D-Bus、`/proc`、应用 API 四条，§T1①），**而且本仓自己判它"今天就能做"**（`01-三者关系论.md:209`）。
- **但它给了一个新的、作者原话没说出来的、更准确的靶子**：
  > **不是"没有接口"，是"接口是 1:1 的（每个应用一套、每个工具一套），而反馈通道必须是 N:1 的（所有反馈回到发起它的那一条通道）。"**
  这一句比原话准，而且它**与本仓 `WC-BOOK-001-v0.1.md:28` 逐字"agent 只能**逐个工具试错**"完全同构**——**同一个病的两种表现**。

#### (d) 它对 Q6 的影响：**必须改正文（本席改，并说明改了什么）**

**原正文（128 字，Q6.1）的缺口**：它只讲了"变更语义"，没讲"感知/反馈通道"。作者补的这个例子，**在正文里一个字都没有落点**。

**⇒ 本席给出两版，并如实交代字数（两者都用 `$s.Length` 复算过，可复核）：**

**⚠️ 本席先纠两处自己的错**（都是"估而不是算"造成的）：
1. 本节初稿把 A 版写成"138 字"——**实为 188 字**；
2. 本席随后想给的"B 版"写成 132 字——**实为 151 字，仍超上限 1 字**。
**本席不把任何一版说成"正好在 150 以内"而不算。**

**A 版（推荐用，内容最完整）· 复算 188 字 · 超出上限 38 字：**

> **今天的机器能跑 agent，也能让它看见——问题在别处。一次操作的反馈默认回到屏幕上，只有人看得见，agent 只能自己截图再猜。这不是没有通道：应答本就沿原路返回，无障碍总线也有现成的控件树。缺的是统一反馈那一层，和变更语义那一层：没有一处能回答谁有权改哪个语义字段、前值是什么、这个动作可不可逆。所以不是不兼容，是缺一层；要补的不是内核，是世界核心的本体、账本与语言投影。**

**B 版（150 字硬约束下的可用版）· 复算 147 字 · 合规：**

> **机器能跑 agent，也能让它看见。反馈却默认回到屏幕，agent 只能截图再猜。这不是没有通道——缺的是统一反馈那一层，和变更语义那一层。按本仓自己的诊断：系统从来没被要求回答"谁、对什么、从什么变成什么、为什么"。所以不是不兼容，是缺一层；要补的不是内核，是世界核心的本体、账本与语言投影。**

**B 版为什么能更短而不失内容**：它把那句四问**直接从仓库里逐字拿来用**——`WC-BOOK-001-v0.1.md:565` 逐字：「今天的问题不是"系统不够聪明"，而是**系统从来没有被要求回答"谁、对什么、从什么变成什么、为什么"这四个问题**——它被设计来管**进程与字节**，而人和 agent 需要的是管**意图与字段**。」
⇒ **这四问既比本席自写的三问更完整（多了"为什么"），又是本仓的原句**，所以引用它比自造更省字、也更稳。**本席只是把"谁、对什么、从什么变成什么、为什么"这十个字标出来，供作者直接采用**——注意 B 版把它改写为陈述式（"从来没被要求回答"），**这也与 `:565` 的"从来没有被要求回答"同形**。

**A 版相对 Q6.1 那版（165 字，已作废）改了三处**：

| 改动 | 原文 | 改为 | 理由 |
|---|---|---|---|
| **加了"也能让它看见"** | （无） | 「也能让它看见——问题在别处」 | 作者补的例子若不加这一句，正文会**默认"agent 看不见"**，而那是**可被 AT-SPI2 一句反例打掉**的（§T1①）。**这是"能被打掉的开篇"防线（`WC-BOOK-001-v0.1.md:550`）** |
| **加了"反馈回到屏幕"这一句** | （无） | 「一次操作的反馈默认回到屏幕上，只有人看得见，agent 只能自己截图再猜」 | 这是作者这一轮**最有价值的新内容**，且**它是一条独立于"语义字段"的、更日常的实证**。不写进去，正文就漏掉了作者这一轮的新观察 |
| **加了"语言投影"三个字** | 「世界核心的本体与账本」 | 「世界核心的**本体、账本与语言投影**」 | 因为按 §T3(a)，**"统一反馈渠道"在本仓的名字就是语言投影**（`15…:210` 逐字"语言投影 ← 给 Agent 读（结构化，可直接进上下文）"）。只写"本体与账本"，会**把作者这条新诉求漏在正文之外** |

**A 版删掉、B 版也没写的**：Q6.2 第 6 条要补的"上限句"。**这是本席有意的取舍**——加了上限句 B 版会到 180 字以上。**作者若要上限句，须另起一句，不能塞进这 150 字里。**

**保留未动的**：四层机制各一例的反例（"权限位只到文件／unit 只到进程／RBAC 只到对象／工具定义里没有『不可逆』这一格"）与"要补的不是内核"，理由同 Q6.2；**但这两个都要占字，这就是 150 字装不下全部内容的原因**。**本席的处理是把它们从正文移到 §T4(c) 那句更有价值的"1:1 vs N:1"判断里**（见上）。

**⚠️ 本席对这个例子的最终定性（一句话，供作者取用）**：
> **作者补的这条，是"缺少结构化感知通道"，不是"不兼容 Agent"。**
> **但它比"不兼容"这个说法有价值得多**——因为"不兼容"是一句会被反例打掉的判断，而"**反馈默认回到屏幕、agent 只能截图**"是一个**今天就能验证、且本仓自己已经在 `00-总纲.md:129` 立为判断标准**的事实。

---

## Q2 反例清算

作者说"不兼容 Agent"。本席逐条核下面六条，每条写**它给了什么位置、缺了什么位置**。

| # | 机制 | **它给了什么位置**（依据） | **它缺了什么位置** |
|---|---|---|---|
| **1** | **Linux capabilities** | 给了：**非 root 进程可以做某一类特权操作**的位置。`capabilities(7)` 逐字："Linux divides the privileges traditionally associated with superuser into distinct units, known as *capabilities*, which can be independently enabled and disabled." ＋ 每个 capability 是一类明确内核操作（`CAP_CHOWN`/`CAP_DAC_OVERRIDE`/`CAP_NET_ADMIN`/`CAP_SYS_TIME`…）。这是**真正的"最小权限"原语**，且**与主体是不是人无关**——一个 agent 进程可以被授予 `CAP_NET_ADMIN` 而不必是 root（[capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html)） | 缺：**字段级、前值、可逆性**（§1.1(b)）。并且 capability 集**上限 64 位** |
| **2** | **systemd 的 unit 隔离** | 给了：**一个 agent 进程组有一个不可伪造的执行者身份 ＋ 一套内核强制的边界**。`SYSTEMD_FOR_WORLD_CORE.md:1356` 逐字"**但 `_SYSTEMD_UNIT=` 的「不可伪造」属性是必须继承的**"；`:1203` 三档表逐字给出"**内核强制（语义类）**：`NoNewPrivileges=`…`SystemCallFilter=`（seccomp-BPF）、`CapabilityBoundingSet=`…**强**：一旦设置，进程自己无法撤销" | 缺：**语义主体**。`:1347` 标题逐字「**4.8 「语义对象」不在 systemd 的模型里 —— 它服务的是「程序」**」；`:1359` 逐字"它的语义层（unit 状态、依赖语义、重启策略）**几乎全部不可迁移**" |
| **3** | **Kubernetes ServiceAccount ＋ RBAC** | 给了：**非人主体（workload）的一等身份 ＋ 声明式授权**。这是**今天对"非人主体"表达得最完整的一套**：ServiceAccount 是给进程用的账号，RBAC 是它的授权语言 | 缺：**字段级**。`席D…独立评审.md:234` 逐字「RBAC 的 `PolicyRule` **只有五个成员**：`verbs`/`apiGroups`/`resources`/`resourceNames`/`nonResourceURLs`——**没有 field selector、没有条件、没有 CEL**；`resourceNames` 只按对象**名字**过滤，且**不能约束 `create`/`deletecollection`**」；`:235` 逐字「官方文档原文："Access controls and policies that depend on specific fields of specific kinds of objects are handled by **admission controllers**"，且"Kubernetes admission control happens **after** authorization has completed"」；`:231` 一句场景标题逐字「k8s RBAC 允许了 `update`，于是它也能改 `image`」；`:237` 逐字（**这是本席在整份取证里见到的最强的一条外部旁证**）「**也就是承认：授权层做不了语义级，只能另加一层"内容相关的"裁判。这是本席在本仓库之外找到的、对"世界核心"主张最强的外部旁证**」 |
| **4** | **OAuth client credentials** | 给了：**非人客户端的一等凭证类型**（RFC 6749 的 grant type 之一），以及**授予范围**（scope）。这是"agent 可以有自己的凭证"这件事的**标准答案** | 缺：**语义**。`席D…独立评审.md:248` 逐字「RFC 6749 §3.3 原文"The strings are **defined by the authorization server**"；RFC 6750 §3 原文"'scope' values are implementation defined; **there is no centralized registry for them**"」 |
| **5** | **云 IAM role** | 给了：**非人主体可以在云上持有一整套权限、并且这份权限不与任何人的密码绑定**的位置。这是"机器身份"的工业标准形态 | 缺：**前值、可逆、以及一个没洞的审计**。`席D…独立评审.md:243` 逐字「"**PassRole is not an API call.** `PassRole` is a permission, meaning **no CloudTrail logs are generated** for IAM `PassRole`"——**一次提权的关键一步在审计里根本不存在**」 |
| **6** | **SPIFFE／SPIRE** | 给了：**可验证的工作负载身份**（SVID）——"我是谁"这一问，在跨主机、无长期密钥的场景下有标准答案 | 缺：**授权与语义**。⚠️ **诚实登记**：本席**未取到 SPIFFE 规范原文中"不做业务授权"的逐字句**（见 §1.3(c)）。本席取到的是它与 OAuth 客户认证的**身份**议题材料（[链接](https://datatracker.ietf.org/meeting/interim-2025-oauth-09/materials/slides-interim-2025-oauth-09-sessa-oauth-spiffe-client-authentication-00#1#1)）。**"缺授权"是本席的推断，未经逐字核实** |

### 再加一条，它不在委托清单里，但比上面六条都重

**反例 7 · 本仓库自己：agent 已经被写成"三方"之一、"原住民"、"生产力"。**

- `2-依据\15-世界核心的组成与职责.md:180` 逐字（小节标题）：**「## 六、三方：机器、Agent、人」**；
- 同文件 `:126` 逐字（全景结构图末行）：**「└── 机器 / Agent / 人：三方都只"写事件"、都只"读投影"」**；
- 同文件 `:185` 逐字（三方端口表）：**「| **Agent** | "请你做什么"（`act`）+ 它的结果 | **语言投影** |」**；
- 同文件 `:188` 逐字：**「**三方之间没有直连**：它们**只跟账本对齐**，不互相聊。这正是"三方都能懂"成立的原因——**共同语言是账本给的，不是谈判谈出来的**。」**；
- `:141` 逐字（部件职责表）：**「| **语言投影** | 给 Agent 读的结构化视图 | `显` | ❌ 无 |」**；
- `:210` 逐字（端口图）：**「  ├─ 语言投影   ← 给 Agent 读（结构化，可直接进上下文）」**；
- `00-总纲.md:335` 逐字（**明确不做清单**）：**「**明确不做**：不写机器内核；不接 MCP / varlink 作为世界协议；不做多用户 / 多机 / 联邦。」**；
- `2-依据\08-创世路线论证.md:4` 逐字：**「垂直切片为出生方式、自举为生长方式、**agent 为原住民生产力**；不兼容旧 web/DOM/MCP」**；
- `2-依据\08-创世路线论证.md:120` 逐字（自首）：**「不兼容路线的历史胜率极低，需明确"这次为何不同"（答案候选：agent 是原住民、」**。

> **本席据此下的判**：**在"设计意图"这一层，本项目从来没有说过"计算机没有 agent 的位置"——相反，它把 agent 写成了三根柱子之一。**
> 所以作者原句与**本仓自己的定位**之间存在一处**显性冲突**（见 Q5）。这一条比上面六条外部反例更该先处理：**外部的反例只说明"作者说轻了"，本仓的反例说明"作者说反了"。**

### "不兼容"降级后的准确说法（本席给）

原话：「它是不兼容，其实有些时候是不兼容 Agent 的」
**本席给出的四个降级档，按强度从高到低：**

| 档 | 说法 | 是否本席建议采用 | 理由 |
|---|---|---|---|
| A | 「操作系统不兼容 agent」 | **❌ 必须删** | 全称、且可被一句反例打掉（一个 agent 进程今天就能在 Linux 上跑，还能被授予 capability、拿到 ServiceAccount、用 SPIFFE 身份）。**本仓 `WC-BOOK-001-v0.1.md:550` 自己立过这条纪律**：逐字「**其中三条必须当场收回**，因为**一个能被一句反例打掉的开篇，会让所有成立的论证一起被扔掉**」 |
| B | 「今天的计算机底层没有 agent 的**语义位置**」 | ✅ **可用** | 限定在"语义位置"，且四层取证支持（§1.5） |
| C | 「今天的计算机**能跑** agent，但**接不住** agent 的**变更语义**」 | ✅ **本席推荐** | "能跑/接不住"这一对是**可判定的**：跑得起来是可观测的；"接不住变更语义"有 §1.1–1.4 的逐条机制证据 |
| D | 「今天没有一层能回答『谁有权改哪个语义字段、前值是什么、这个动作可不可逆』」 | ✅ **最强、最可判** | 这是四层取证收敛到的**同一句**，且**它不依赖"agent"这个概念**——它对人也同样成立。⇒ 这句话**不可能被"其实有 agent 位置"这类反例打掉**，因为它压根没声称"agent 不被兼容" |

**⇒ 降级后的准确说法（本席定）：**
> **"不兼容"应降为"接不住"。**
> 今天的机器**能承载 agent 这个执行者**（内核、systemd、RBAC、OAuth、SPIFFE 都给了它身份与边界），
> **接不住的是 agent 的变更语义**——四层机制里没有一处能回答"谁有权改哪个语义字段、前值是什么、这个动作可不可逆"。
> 作者原话里"**不兼容**"这三个字，是**把"语义层缺位"升格成了"整体不兼容"**。

---

## Q3 费劲的证据：仓库内已有的实证

**先给结论：仓库内没有"接入成本"的实测数字，而且仓库自己把这一点标成了"红"。但有大量机制级实证。**

### 3.1 先看仓库自己的自首（这一条最重要，因为它决定了 Q3 的举证上限）

`world-core\docs\理论\WC-PREFACE-001-v0.7.md:514` 逐字：

> 「| **B-8** | 成本基线 + 阈值       | 给出今天的**接入／对齐成本**，并预设一个**宣布失败**的数值                                    | **今天可采集**                                    | **未采集**（仓库内有相邻基线：经常性成本 ≈3.4 人日／月、其中 77% 是文档与评审——**但那不是接入成本**）       | §二 的痛点主张**不可证伪**                                  |」

同文件 `:108-110` 逐字：

> 「> **本序在这里有一个缺口，自己标出来**：上面四条**没有基线数字**——今天的接入成本是几小时、每千次跨方调用的对齐耗时是多少，**本序没有采集**。
> > 没有基线，"每处都要重谈"与"绝大多数不必重谈"**无法区分**，第 4 条就不可证伪。
> > **可测定义与失败阈值见 §7.2 检验 B-8；在数字补上之前，第 4 条标"红"（不是"未验证"）。**」

同文件 `:528` 逐字（红项清单）：

> 「| **§二 第 4 条**                          | 无可测基线 ⇒ 不可证伪                                                    |」

> **⇒ 本席对这一条的判**：**"Agent 调用是很费劲的"这句话，在本仓库内的举证状态是"机制级有据、成本级未测、且已被自己标红"。**
> **作者本人已经承认这个缺口。本席不需要替他指出——只需要指出他这句话说出口时，仓库里那份自首都已经写好了。** 引用它时**必须带上 `WC-PREFACE-001-v0.7.md:110` 的"标红"**，否则就是把已登记的缺口当成已证成的结论用。

### 3.2 逐字引出：凡讲到"逐个工具试错／每个工具一套参数一套错误码／文档永远滞后于实现／四本账没有共同主键"的地方

**（一）"每个工具一套参数、一套错误码" ＋ "逐个工具试错"**

- `world-core\docs\理论\WC-BOOK-001-v0.1.md:21` 逐字：
  > 「- agent 对软件说：调这个接口、传这些参数（**每个工具一套参数、一套错误码**）。」
- 同文件 `:28` 逐字：
  > 「| 每个软件一套说法 | agent 只能**逐个工具试错**；换一个工具就要重新学一遍 |」
- 同内容在 `WC-BOOK-001-v0.2.md:21`、`:28` 与 `WC-BOOK-001-v0.3.md` 同位置逐字重复（v0.3 分别在 `:33` 之前的结构中，`v0.3:463` 为四本账条）。
- `world-core\docs\理论\WC-PREFACE-001-v0.7.md:101` 逐字（**这是唯一一处标了"谁在挨"的表述**）：
  > 「| **2** | agent 的开发者   | 每个工具一套参数、一套错误码；**换一个工具就要重新学一遍**。成本不随能力增长，而随"工具数量"增长                                                                                                                                                                                         |」
- `world-core\docs\理论\语义世界-序.md:27` 逐字：
  > 「**换一个工具就要重学一遍。** 每个工具一套参数、一套错误码，成本不随能力增长，而随工具的数量增长。」

**（二）"文档永远滞后于实现"**

- `world-core\docs\理论\WC-BOOK-001-v0.1.md:29` 逐字：
  > 「| 每个协议一套消息格式 | 消息**不自描述**：不查文档读不懂，而文档永远滞后于实现 |」
- `world-core\docs\理论\语义世界-序.md:25` 逐字：
  > 「**文档永远滞后于实现。** 一个字段是什么意思，只写在文档里。文档一改，约定就断了；文档没改，实现先改了。意思没有被写进事情本身，就总要靠一份外加的说明去追。」
- ⚠️ **一处必须同时引的"内证对抗"**：`world-core\docs\理论\WC-PREFACE-TRACE-002-v0.1.md:30` 逐字：
  > 「| **P-07** | §二 后果②：「**消息不自描述**｜不查文档读不懂，而**文档永远滞后于实现**」 | ✅（前半）/❌（后半） | `2-依据/14` §2.1 L52；`2-依据/13` L199 | 「**自解释的类型 + 词表版本**；**不读文档也能懂**」 | "不自描述"有据。"**文档永远滞后于实现**"在所指定 11 篇里**零命中**（该句在 `WC-BOOK-001` L29） |」
- `world-core\docs\理论\专家评审\席N-序-本原与派生主线梳理.md:109` 逐字（本仓对同一句的再次登记）：
  > 「| **B7** | **文档滞后于实现** | `WC-BOOK-001-v0.1.md:29` 逐字「消息**不自描述**：不查文档读不懂，而文档永远滞后于实现」 | **一级派生**。有据（在 BOOK）。⚠️ `WC-PREFACE-TRACE-002-v0.1.md:30` 逐字记「"**文档永远滞后于实现**"在所指定 11 篇里**零命中**（该句在 `WC-BOOK-001` L29）」——**只在同级 BOOK 里有据** |」
- **⇒ 判**："文档永远滞后于实现"这句话，**唯一的仓库内出处是 `WC-BOOK-001:29` 自己**，且 `WC-PREFACE-TRACE-002-v0.1.md:30` 已判定它**在指定的 11 篇依据里零命中**。**它是一个外部印象，不是本仓证据。** 本席**不能**把它列为"仓库内实证"。（`席N…:406` 逐字重复了同一判定：「只在 `WC-BOOK-001-v0.1.md:29` 有；`00-理论定稿` 与 `2-依据/15` 中**查不到**」）

**（三）"四本账没有共同主键"**

- `world-core\docs\理论\WC-BOOK-001-v0.1.md:445` 逐字：
  > 「5. **跨工具的账对不上**：`journalctl` 里有 agent 的日志，`~/.bash_history` 里有命令，`/var/log/pacman.log` 里有包操作，`git log` 里有文件修改——**四本账，没有共同的主键**，出了事要人工拼。**（推断：这是运维现场最常见的"复盘成本"来源）**」
- 同文件 `:518` 逐字：
  > 「3. **事后**：四本账对不上 ⇒ 复盘要人工拼 ⇒ **复盘成本高到没人做** ⇒ 于是"下次做得更好"这件事**没有反馈回路**。」
- `席N-序-本原与派生主线梳理.md:119` 逐字（本仓对这条的评价）：
  > 「| **C4** | **跨系统对不上账：四本账，没有共同的主键** | B2 + B4 | `WC-BOOK-001-v0.1.md:445` 逐字「…」 | **二级派生**。有据。**这是全仓对 B2 最具体的一段举证，而现在的序完全没有用它** |」
- `world-core\docs\理论\WC-PREFACE-LOG-001-v0.2.md:92` 逐字：
  > 「| 四本账没有共同主键（journalctl／命令历史／包日志／git log） | `WC-BOOK-001` 第 445 行 |」
- ⚠️ **注意 `WC-BOOK-001-v0.1.md:445` 自己带着括号标注**：「**（推断：这是运维现场最常见的"复盘成本"来源）**」——**这一半是作者自己的推断，不是取证**。前半（四本账、无共同主键）是机制事实；后半（"最常见的复盘成本来源"）是推断，且**本仓没有给它数据**。

**（四）"接入层没有进度坐标"（这是第三章 §3.4，机制级）**

见 §1.4(b) 已逐字引。另加 `WC-BOOK-001-v0.1.md:492` 逐字（该节的一句话要求）：

> 「> **一句话要求**：**Agent 的每一次动作都要在外部的、不可被它自己修改的账上留下"意图 + 结果"，并且这两者必须能配对。**」

**（五）"同一个故障两种表示、错误码不保证存在"（这是本仓自己实现里的实证，比外部例子更硬）**

- `world-core\docs\S2-设计\WC-PFMT-001-v0.1.md:188` 逐字：
  > 「- **推论 2（线上错误码不保证存在）**：第 4 类的线上 `error` 是**无码散文**（`221–225`），而**同一次调用**的 Rust 侧 `Err` **带**码（`226`）——两者是**同一个故障的两种不同表示**。第 5 类的线上 `error` 则是 `commit` 原样错误串，通常带码（见 `src/lib.rs:145–188` 的 `Gate.*` 三码）。⇒ **跨语言消费者若要用错误码分支，必须容忍"无码"情形。**」
- 同文件 `:552` 逐字（不担保清单里的 **N-7**）：
  > 「| **N-7** | **不担保错误码完整** | ① `guard.rs` 三条断言**无码**且会被 `bind()` 透传（D-07）；② **门禁裁决不是错误码**——「属**正常拒绝**：调用方读理由」（`WC-IC-001` §三 错误码表末行「门禁裁决（非错误码）」）；③ 冒充拒绝的线上错误**无码**（D-01）；④ `src/error.rs:30–34` 自述：前缀**不是编译期保证**、人话部分是中文散文**不得**被程序依赖、尚无用例编号↔错误码的双向映射表 |」
- `world-core\docs\理论\WC-IC-001-v0.1.md:660` 逐字（**一处具体的形态缺陷**）：
  > 「| `ext.world.VersionMismatch: 事件构造器版本 … 与本体声明的 world=… 不一致；拒绝启动…` | `src/lib.rs:87` | ⚠ **缺域段**：`code_of()` 要求前缀之后形如 `域.原因`，而 `VersionMismatch` 里**没有点号** ⇒ `error::code_of()` 返回 `None`（`src/error.rs:43-54`），即它**不满足**本契约的错误码形态；`c15` 也**未覆盖**这条路径（`tests/contract.rs:690-746`）」

**（六）"文档滞后"在本仓的另一处实证：被引文档在起草期间被并发改写**

- `world-core\docs\阶段外-待启用\S2-设计\WC-PFMT-001-v0.1.md:24` 逐字（**这是"文档永远滞后于实现"在本仓内部的一个可复算的镜像**）：
  > 「**为什么**：起草期间实测到这些文件**被其他改动连续改写**，行号逐分钟漂移——`WC-HLD-001` §6.3 在数分钟内由 L384 漂到 L411；`WC-SCMP-001` §4.2 表 C 整体移动；`WC-IC-001` 的「门禁裁决（非错误码）」行由 `:131` 变为 `:130` 后又再次移动。**任何写死的行号在交付时都可能是错的**」
- 同文件 `:579` 逐字（把它登记为待验证项 **V-14**）：
  > 「| **V-14** | **被引文档在起草期间被并发改写，行号不可用**。实测证据：`WC-HLD-001` §6.3 在数分钟内由 L384 漂到 L411；…**不要**把本文件的行号当作可长期使用的定位符 |」

**（七）"Agent 侧端口协议只能靠它自己，而它缺的正是我们要的"**

- `2-依据\11-总线工程规格.md:88` 逐字：
  > 「| 协议 | modelcontextprotocol（官方 spec+SDK / servers） | ✅ | agent 侧端口协议最强参考：动作请求/结果+事件有对应物（tools/call、notifications）；**缺状态快照/diff（resources 偏静态）——恰是我们的增量** |」
- `2-依据\11-总线工程规格.md:108-109` 逐字（**同一份文件内部的动作决定冲突**）：
  > 「> **本条动作决定**即刻废止**——它与 00.8"砍掉旧世界器官（DOM 转译层、**MCP 方言**、过渡桥）"
  > > 直接冲突。判据（00.9 第二节）：MCP 属**锚定**（世界的 agent 端口服从外部规范），」
- `2-依据\00-依据索引.md:38` 逐字：
  > 「| 11 §7 | "transferor 增加 MCP 方言" | 与总纲 §4.3"砍掉 MCP 方言"冲突，**仅兼容备选路线有效** |」
- `2-依据\13-总线与通道外部参考.md:199` 逐字（**一条与作者原话直接相关的取舍**）：
  > 「4. **不抄 MCP / ACP 的"先协商能力再看数据"**——同样违"大模型不读文档也能懂"；」

### Q3 判词

> **仓库内无"接入成本"的实测数字**（作者自己在 `WC-PREFACE-001-v0.7.md:110`、`:514` 标红承认）。
> **但有七类机制级实证**，其中"每个工具一套参数一套错误码／逐个工具试错"（`WC-BOOK-001-v0.1.md:21`、`:28`）与"四本账没有共同主键"（`:445`）是全仓最具体、被 `席N` 与 `WC-PREFACE-LOG-001-v0.2.md:92` 双重登记的两条。
> **两条必须降级使用**：
> - 「文档永远滞后于实现」→ **唯一出处是 `WC-BOOK-001:29` 自己**，`WC-PREFACE-TRACE-002-v0.1.md:30` 已判**在指定 11 篇依据里零命中**；引用它时必须写"这是本书的判断，不是外部取证"。
> - 「四本账是运维现场最常见的复盘成本来源」→ `:445` 自己已标「**（推断）**」，**无数据**。

---

## Q4 三个层次的判断

### ① 「今天的电脑服务于现有这套软件系统」——**成立**

**理由（三条，均可核）**：

1. **接口面按"程序"设计，不按"语义对象"设计。** 外部研究件 `SYSTEMD_FOR_WORLD_CORE.md:1347` 的标题就是结论，逐字：**「4.8 「语义对象」不在 systemd 的模型里 —— 它服务的是「程序」」**；`:1359` 逐字"world-core 应该把 systemd 当作**进程与权限的宿主**，而不是**语义的建模框架**"。
2. **本仓自己的第三章就是这一节的取证。** `WC-BOOK-001-v0.1.md:433-436` 五行机制表逐字给出"每个机制只服务它自己那个软件的那件事"：文件系统记"字节与时间"、shell 历史记"人敲过的命令"、systemd 记"单元状态"、权限位记"谁能读写这个文件"、快照记"某个时刻的字节"。
3. **一个反向印证**：这套系统**确实**很好地在服务现有软件——`WC-BOOK-001-v0.1.md:427` 逐字「**它擅长什么**：把字节可靠地放在盘上；把进程可靠地管起来；把资源可靠地隔离。这三件事它做得比任何自研方案都好。」

> **判**：成立。**这是三句里唯一一句没有争议的。** 但它也是一个**弱判断**——它几乎同义反复（电脑当然服务它上面的软件）。所以它不该被当作论据，它只是**背景前提**。

### ② 「它不兼容 Agent」——**在"语义层"这个窄意义上成立；在字面全称意义上不成立**

**它在哪个意义上成立**（§1.5 的收敛）：

> 内核（判据①②③全❌）、系统服务（全❌）、授权（全❌）、接入（❌且无进度坐标）——**四层机制里没有一处能表达"谁有权改哪个语义字段／前值是什么／这个动作可不可逆"**。
> 这是**硬成立**：不是"没做完"，是**这三个类型不存在**。

**它在哪个意义上不成立**（三条，均可核）：

1. **执行位置一直有，且越来越有。** 一个 agent 进程今天就能在 Linux 上起、被隔离、被配额、被授予 capability（`capabilities(7)`）；能有不可伪造的 unit 身份（`SYSTEMD_FOR_WORLD_CORE.md:1356`）；能有 ServiceAccount＋RBAC（`席D…:231-237`）；能有 OAuth client credentials 或 SPIFFE 身份。**"不兼容"这个词若指这些，是事实错误。**
2. **"费劲"≠"不兼容"。** §1.4 的机制证据说的是"接进去之后还要重谈语义边界与进度语义"，不是"接不进去"。**MCP/function calling/OpenAPI 每天都在接进去。**
3. **本仓自己在三份文件里写过相反的话。** `2-依据\15…:126` 逐字「机器 / Agent / 人：三方都只"写事件"、都只"读投影"」；`2-依据\08…:4` 逐字「**agent 为原住民生产力**」；`00-总纲.md:335` 逐字「不接 MCP / varlink 作为世界协议」（**是主动不接，不是接不上**）。

> **⚠️ 本席必须指出的一处逻辑问题**（这是原话最容易被攻击的地方）：
> 作者的原句是「**现在的这个电脑只是服务于现在的这套软件系统，它是不兼容 Agent 的**」。
> `00-总纲.md:335` 逐字：「**明确不做**：不写机器内核；**不接 MCP / varlink 作为世界协议**；不做多用户 / 多机 / 联邦。」
> ⇒ **"机器不接 agent" 与 "项目不接 MCP" 是两件不同的事，但作者的原话把它们叠在了一句里。**
> **如果读者把"不兼容"读成"所以我们要另造一个"——那么它有被误读成"为自造找理由"的风险**，而这不是事实：**MCP 是项目主动砍的（`:335`；`2-依据\11…:108-109`），不是机器不给。**
> **这是我的推断**，但依据是三处逐字原文。

### ③ 「如果底层就有 Agent 的位置，这事才符合现在的状态」——**这是主张，不是推测；而且它是一个"必要条件"式的主张，最弱**

**先分类（本席作的三分）**：

| 类别 | 定义 | 它属于哪类 | 依据 |
|---|---|---|---|
| **推测** | 陈述关于世界的未知事实，无法证伪 | ❌ 不是 | — |
| **主张** | 陈述应当如何，可辩但不可直接测 | ✅ **是主张** | 原句用的是"**才符合**"，是**规范性**表述 |
| **可检验命题** | 陈述关于世界的可观测事实，有明确否证条件 | ❌ **今天不是**，但**可以变成** | 见下 |

**为什么它是"主张"而不是"推测"**：原句「**那这个事儿是不是就是符合现在这种状态的？**」——它在问"**符不符合**"，这是一个**判据问题**，不是事实问题。它没有声称"底层加了位置之后 agent 就会怎样"，因此它**不冒事实风险**；它只是说"这样才**对**"。

**它为什么今天不可检验**：它是一个**必要条件**的主张（"只有底层有位置，才符合"），而**必要条件的主张只能被反例否证，不能被正面证实**。今天仓库里**没有**任何一条检验覆盖它。本仓已有的可证伪检验清单在 `WC-PREFACE-001-v0.7.md:501-518`（**八条，B-1…B-9**），其中与"位置"最接近的是：

- `:507` 逐字 **B-1**：「| **B-1** | 语义侵入实验          | 用**现有词表**表达 **3 个陌生领域**的真实任务，**新增能力名 = 0**                           | **今天可跑**（本项目自己的书已写明它**可以做成自证脚本**）            | **未跑**（**红**：可跑而未跑）                                                 | 它**不懂意义**，只强制形状 ⇒ §4.3b 必须再降一级                    |」
- `:512` 逐字 **B-6**：「| **B-6** | 被管者身份测试         | 在**同身份部署**下重跑全部门禁用例，逐条记录哪些保证退化为**约定**                                | **今天可跑**                                     | **本轮已实测**（2026-09-27，VM，root 部署 + `agent` uid 1001）——逐条结果见 **附录 E** | 已红项：法律可被同身份改写且不留痕；CLI 路径可自称最高权主体；套接字身份绑定对 root 无效 |」

并且本仓已经提出过这个实验的具体形态：`WC-BOOK-001-v0.1.md:689` 逐字：

> 「**它的技术形态我也采纳了一半**：**"语义侵入实验"可以做成自证脚本**（用现有本体表达 3 个陌生领域的真实任务、要求新增能力名 = 0），只是"外人"这个角色必须由人来当。」

### ③ 要把它变成可检验的主张：本席给出的形态

**本席建议把它重写成三条可检验命题（三条都不新造方法，全部复用本仓已有的 B-1…B-9 编号体系）：**

| 编号 | 可检验形态 | 通过条件（机械可核） | 否证条件 | 今天能否跑 |
|---|---|---|---|---|
| **R-1（注入式实验）** | **把同一个真实任务分别交给：(a) 一个"有语义位置"的客户端（只发 `act` ＋ 读语言投影），(b) 一个"无语义位置"的客户端（直接改底层文件）。** 逐条记录两者在**同一时间窗**内产生的记录形态 | (a) 的记录里能机械判定"**谁／改了哪个实体的哪个字段／前值／为什么／可不可逆**"六项；而 (b) 的记录里**缺项 ≥ 3**（即"有位置"确实带来可判定的信息差，而不只是好看） | **若 (a) 的记录也缺项 ≥ 3 ⇒ "位置"不产生可观测差异 ⇒ 主张落空** | **可跑**（复用 `B-1` 的"今天可跑"判级，`:507`） |
| **R-2（对照部署）** | **同构建、同任务集、同一宿主机上做两组部署**：A 组装本项目（世界核心 ＋ 门禁），B 组只装普通 systemd unit ＋ 一个 MCP server。跑同一批任务 | A 组在**"事后回答'谁改了什么、前值是什么'"这件事上**的**人工介入次数**显著低于 B 组（阈值须**先写死再跑**，不得事后定） | **若 A 组的人工介入次数与 B 组无显著差异 ⇒ "底层位置"没有换来可判定的收益** | **需先定阈值**（本席提醒：**阈值必须事前写死**，否则违反 `WC-PREFACE-001-v0.7.md:514` 对 `B-8` 的批评——"未采集 ⇒ 不可证伪"，事后定阈值等于重犯） |
| **R-3（可观测指标）** | **三个数，全部可机械采集**：① **接入成本**（一个陌生领域的 agent 从零到能发第一条 `act` 所需人时）；② **对齐耗时**（每千次跨方调用的重谈时长）；③ **复盘延迟**（从"事情发生"到"它能出现在出口上"的时延上界，实测一次） | 三个数各自**先有基线**，且基线来自**本仓之外**（不能拿本项目自己的文档当基线） | **若三个数中任一无基线即用于论证 ⇒ 该论证不可证伪** | ⚠️ **① ② 今天未采集**（`WC-PREFACE-001-v0.7.md:514` 对 `B-8` 的判定逐字"**未采集**"）；**③ 今天未跑**（同文件 `:515` 对 `B-9` 的判定逐字"**未跑（红）**"） |

> **⚠️ 本席必须补一条纪律**：上表三条里，**R-1 是唯一今天就能跑、且不依赖"第二个执行者"的**。
> 依据：`WC-PREFACE-001-v0.7.md:495` 逐字登记了这个项目的一个**永久缺口**：
> 「| **(F) 执行者缺位（本轮新增的一栏）**  | **F1** | **本序的判据今天没有独立判定者**         | **本序的合格判决者不能是它的作者**，而本项目明确不做多用户 ⇒ **第二个执行者在本适用域内不会出现**。<br>⇒ 因此本序**带有一批永久红项**，逐条列在 §7.3。**这不是"待验证"，这是"已知缺口"。**」
> **⇒ 所以 R-2（对照部署）在本项目当前适用域内也面对同一个问题：谁来判"显著低于"？** **这是我的推断**，依据是 `:495` 那条自认。

---

## Q5 与项目既有定位的关系：「一等组件 → PID 1」

### 5.1 逐字引出既有定位

- `00-总纲.md:42` 逐字：
  > 「> **先做一个"一等组件"（系统开机就起它、别人依赖它）；成熟后做 PID 1。**」
- `00-总纲.md:44-48` 逐字（两阶段表）：
  > 「| 阶段 | 世界核心站在哪里 | 对应站 |
  > |---|---|---|
  > | **一** | **载体上的一个一等组件**——载体对它的认知只有"启动它、给资源、给一条通道" | 寄主期 **B.5** |
  > | **二** | **世界的 init**（Genode 里 `init` 那个位置）；**载体降为插座** | 自举 **C** → 换轨 **D** |
  > | 切换触发 | **沿用 §8 已有的换轨触发条件，不新造** | — |」
- `00-总纲.md:50-52` 逐字（硬前提）：
  > 「**硬前提（守不住，阶段二永远到不了）**：
  > > **载体只知道"启动它"，不知道"它是什么"。**」
- `2-依据\15-世界核心的组成与职责.md:31` 逐字（**这一行把"位置"这个词的定义钉死了**）：
  > 「| 1 | **世界住在哪里、谁依赖它** | `载` + **系统方式** | 一等组件 → 将来 PID 1 | 这栋**楼是谁的地、谁供电** |」
- `2-依据\15…:36` 逐字：
  > 「三者不冲突，是三层：**位置 → 内容 → 通路**。」
- `2-依据\15…:53-59` 逐字（"一等组件"不是什么）：
  > 「### 2.2 它不是"插件"
  > | | **插件** | **一等组件** |
  > |---|---|---|
  > | 谁决定你怎么被装进去 | **宿主程序**定规矩（"你得实现我这几个函数"） | **系统**只知道"启动这个程序" |
  > | 换个宿主/载体 | 要照新宿主的规矩重写 | 照样能起 |
  > | 风险 | ⚠ 宿主定义你的边界 = **锚定** | ✅ 你自己定义自己 |」
- `00-总纲.md:61-62` 逐字（"像 systemd"要抄什么）：
  > 「**"像 systemd"要抄的是它的位置与权威**（开机第一个起、别人依赖它、它记账），
  > **不是它的接口形式**（字段写死的配置文件）。」
- `WC-BOOK-001-v0.1.md:118` 逐字：
  > 「| 1 | 世界住在哪里、谁依赖它 | **载 + 系统方式** | 它先是载体上的一个**一等组件**（开机就起、别人依赖它），成熟后才是 **PID 1** |」
- `WC-BOOK-001-v0.1.md:181` 逐字（全景结构图）：
  > 「└─┬─ 世界核心（一等组件 → 将来的 PID 1）────────── 内容相关的基础设施」

### 5.2 而"Agent 的位置"在本仓的既有定位里是**另一根轴**，而且是**已经给了的**

- `2-依据\15…:180` 逐字（小节标题）：**「## 六、三方：机器、Agent、人」**
- `2-依据\15…:126` 逐字：**「└── 机器 / Agent / 人：三方都只"写事件"、都只"读投影"」**
- `2-依据\15…:185` 逐字（三方端口表）：**「| **Agent** | "请你做什么"（`act`）+ 它的结果 | **语言投影** |」**
- `2-依据\15…:275` 逐字（"世界起来了"四条件之四）：**「| 4 | **投影已注册**，人和 Agent 能连上 | `显` 就位 |」**
- `1-理论与哲学\06-协作与承载.md:108-112` 逐字（三对沟通的最优承载表）：
  > 「### 7.3 三对沟通各自的最优承载
  > | 对 | 最优承载 | 理由 |
  > |---|---|---|
  > | Agent ↔ Computer | **声明式语义调用**（JSON-RPC/varlink/能力接口） | 唯一双方皆机器的对：零歧义、可校验、可门禁。D-Bus 二十年已验证；自然语言让机器执行需"解析→猜参→执行"三步有损 |」
- `1-理论与哲学\06-协作与承载.md:104-105` 逐字：
  > 「3. **平等 vs 不对称**：中间点假设三方对等；分层承认 agent 与 computer 同为机器、
  >    直连底座（零翻译），唯人是特殊物种、配两个"随行翻译"（对话投影 + 界面投影）。」

### 5.3 判：**一致 ＋ 加强，但作者的原话用错了一个词，造成一处"看起来像冲突"**

**（一）先说结构上为什么不冲突。**

| 轴 | 问的问题 | 已有定位 | 谁在里面 |
|---|---|---|---|
| **位置轴**（`15…:31`） | **世界住在哪里、谁依赖它** | 一等组件 → 将来 PID 1 | **世界核心自己** |
| **主体轴**（`15…:180`） | **谁在写事件、谁在读投影** | 机器 / Agent / 人 三方 | **Agent 是三方之一，已经在了** |

`2-依据\15…:36` 逐字已经把这层关系写死：**「三者不冲突，是三层：位置 → 内容 → 通路。」**

⇒ **作者说的"从底层就有 Agent 的位置"，与"一等组件 → PID 1"不冲突**：
- "一等组件 → PID 1" 说的是**世界核心自己的位置**（进程拓扑轴）；
- "底层有 Agent 的位置" 说的是**Agent 在世界里的位置**（主体轴）；
- **两条轴在 `15…:36` 里已经被显式分开过。**

**（二）再说它在哪一点上"加强"。** 加强的方向是：作者的原话把**主体轴**的要求提高了一档——从"Agent 是一个能读投影、能写 `act` 的客户端"（现状：`15…:141` 逐字"**语言投影** | 给 Agent 读的结构化视图 | `显` | **❌ 无**"）提到"**Agent 的位置要写在底层**"。这与 `06-协作与承载.md:112` 逐字已经写下的（"Agent ↔ Computer｜**声明式语义调用**…**零歧义、可校验、可门禁**"）**方向一致**，而且是它的一步具体化。

**（三）冲突在哪：作者用了"底层"这个词。**

`00-总纲.md:38` 逐字：

> 「| 不是写操作系统                 | 机器内核**买**（Linux），自建的是**世界核心**（语义本体的存储与运行时） |」

`00-总纲.md:52` 逐字（硬前提）：

> 「> **载体只知道"启动它"，不知道"它是什么"。**」

`00-总纲.md:83-86` 逐字（**注意：这一条的主语是"载"，即载体／内核，不是世界核心**）：

> 「### 2.1 载
> - **定义**：把物理机器翻译成抽象机器并仲裁使用权——调度、内存、文件、驱动、IPC、隔离与配额。
> - **它不是什么**：不是世界的组成部分。**它不提供语义，也不提供应用价值，只提供秩序。**」

⇒ **"从底层就有 Agent 的位置"这句话，如果"底层"读作"内核／载体层（`载`）"，则与 `00-总纲.md:38`（内核是买来的）、`:52`（硬前提）、`:86`（`载` 不提供语义）三处直接冲突。
本席把这条辨析写细，是因为 `:86` 很容易被误引来支持"内核不提供语义所以要去改内核"——而这一条的主语恰恰是**不打算改的那个东西**。**这是我的推断**。**
⇒ **如果"底层"读作"世界核心的最底层（本体与账本这一层）"，则完全一致，且正是本项目的原命题。**

**本席的判**：
> **一致：是。加强：是。冲突：不是——但原话的"底层"二字必须换掉，否则会被读成"要改内核"，而那正是本项目明确不做的（`00-总纲.md:38` 逐字"机器内核**买**"）。**
> **建议替换**：把"**从底层**就有 Agent 的位置"改为"**在本体与账本这一层就有 Agent 的位置**"。
> **理由**：本项目的"底层"＝**世界核心的底层（本体／账本）**，不是**机器的底层（内核）**。这两个"底层"在本仓里是**不同的两个词**（`00-总纲.md:38` 与 `15…:36` 的"位置 → 内容 → 通路"三层）。

**（四）一处必须同时指出的真正冲突（不是与"一等组件"冲突，是与本项目的另一条决定冲突）**

`00-总纲.md:335` 逐字：

> 「**明确不做**：不写机器内核；**不接 MCP / varlink 作为世界协议**；不做多用户 / 多机 / 联邦。」

`2-依据\13-总线与通道外部参考.md:199` 逐字：

> 「4. **不抄 MCP / ACP 的"先协商能力再看数据"**——同样违"大模型不读文档也能懂"；」

⇒ **"开发一些接口给这个 Agent"这一半，本项目是有明确取舍的**：接口**要开发**，但**不采用现成的 agent 侧协议**（MCP），理由是**锚定**（`2-依据\11…:109` 逐字"MCP 属**锚定**（世界的 agent 端口服从外部规范）"）。
⇒ **所以"我们跟上也能去开发一些接口给这个 Agent"这句话，在本仓里的准确形态是**：**开发一套自有的语义接口（`act` ＋ 语言投影），而不是适配 MCP。** 作者原话没说清这一点，**容易被读成"要接 MCP"**——而那恰恰是本项目已废止的动作（`2-依据\11…:108` 逐字"**本条动作决定**即刻废止"）。

---

## Q6 建议写法

### 6.1 正文级表述

> **⚠️ 本节已被 §T4(d) 取代。** 作者在委托中途补充了"**截图回路／反馈应沿原通道返回**"这个例子，本席据此把正文重写为 **A／B 两版**（A 版 **188 字**；B 版 **147 字**，在 150 字以内），并逐字说明了改动的三处与理由。
> **请以 `§T4(d)` 的那两段为准**（B 版合规，且把"反馈回到屏幕"这一层也写进去了）。
> **下面这一版保留，只用于对照——它漏掉了作者这一轮最有价值的新观察。**
> **⚠️ 本席更正一处自己写错的数字**：本节初稿写"128 字"，是**估的**；用 `$s.Length` 复算后**实为 165 字**（**也已超出 150 字上限**）。**本席不采用这一版，也不需要它。**

**（对照用，已作废）128 字版：**

> **今天的机器能跑 agent，也给了它身份和边界；接不住的是它的变更语义。内核、系统服务、授权、接入四层，没有一处能回答"谁有权改哪个语义字段、前值是什么、这个动作可不可逆"——权限位只到文件，unit 只到进程，RBAC 只到对象，工具定义里没有"不可逆"这一格。所以不是不兼容，是缺一层；要补的不是内核，是世界核心的本体与账本。**
>
> **字数核算（本席用 `$s.Length` 复算，可复核）**：正文 **128** 字（含标点与英文词内的字母，不含标题与说明）。
> 其中四层各一例（"权限位只到文件／unit 只到进程／RBAC 只到对象／工具定义里没有『不可逆』这一格"）合计 44 字，是本段**唯一不是判断、而是可核事实**的部分。

**为什么是这一版**：
- 它**先承认机器能给什么**（身份、边界）——这样就**不可能被"其实有 agent 位置"一句反例打掉**（`WC-BOOK-001-v0.1.md:550` 立的纪律）。
- 它把"不兼容"换成"**接不住**"，并把"接不住"的对象**钉死在"变更语义"**上——**这是可判定的**。
- 它给出**四层各一层的最小反例**（权限位／unit／RBAC／工具定义），每一层都可核（§Q1）。
- 它最后**主动放弃"要改内核"这个读法**（"要补的不是内核"），从而**不撞 `00-总纲.md:38`** 的"机器内核买"。

### 6.2 我删掉了原话里的哪几个词、为什么

| # | 删掉的词 | 理由（逐字依据） |
|---|---|---|
| **1** | 「**它是不兼容**」→ 改为「**不是不兼容，是缺一层**」 | ① 全称句，可被一句反例打掉（§Q2 六条反例）；② 本仓自己立过这条纪律：`WC-BOOK-001-v0.1.md:550` 逐字「**一个能被一句反例打掉的开篇，会让所有成立的论证一起被扔掉**」；③ `2-依据\08-创世路线论证.md:120` 逐字已自首「**不兼容路线的历史胜率极低，需明确"这次为何不同"**」——把"不兼容"删掉，就**不必再回答"这次为何不同"** |
| **2** | 「**现在的这个电脑只是服务于现在的这套软件系统**」整句 | ① 同义反复（电脑当然服务它上面的软件），**作为论据为零**；② 它会**误导读者把"不服务 agent"与"不接 MCP"混为一谈**——而 MCP 是本项目**主动砍的**（`00-总纲.md:335`、`2-依据\11…:108-109`），不是机器不给 |
| **3** | 「**其实有些时候**」 | 模糊限定词，使整句**不可判定**（"有些时候"是什么时候？）。本仓对这类措辞有明确的处置先例：`WC-BOOK-001-v0.1.md:637` 逐字「"**总是**"被比较过失 / 共同过失 / 市场份额责任直接反驳。｜删掉"总是"，改为"**在可预见性可判定的情形下**落在单桶里"」——**同一个治法** |
| **4** | 「**从底层**」→ 改为「**在世界核心的本体与账本这一层**」 | "底层"在本仓是**两个不同的词**：机器的底层（内核，`00-总纲.md:38` 逐字"机器内核**买**"）与世界核心的底层（本体／账本，`15…:36` 逐字"位置 → 内容 → 通路"）。不换掉，这句话会被读成"要改内核"，**而那正是本项目明确不做的** |
| **5** | 「**开发一些接口给这个 Agent**」→ 改为「**补的是世界核心的本体与账本**」 | 原话把重点放在"接口"，会与 `00-总纲.md:335`（不接 MCP/varlink 作世界协议）读成冲突。本仓的既有定位是**接口是派生的、本体是承重的**：`2-依据\11-总线工程规格.md:64` 逐字「| 治理 | polkit（进程/用户为单位） | agentd（能力+不可逆性为单位，语义级门禁） |」——**能力与不可逆性进本体，接口只是它的出口** |
| **6** | 未删但**必须补上一个上限**：（见 `WC-PREFACE-001-v0.7.md:100` 的范式） | `:100` 逐字给了一个**"上限同时说"**的现成范式：「**上限同时说**（不让这一处变成承诺过头的广告）：它能消灭"**查无此申请**"，**消灭不了"查无此后果"**，**也消灭不了"查无此人可担此责"**」。**本席建议这版正文也配一句上限**：「**它能补上"谁改了什么、前值是什么"；补不上"为什么改"（意图在模型侧）与"谁该担责"（那是制度问题）。**」——理由：`席D…独立评审.md:31` 逐字已把这一处标为所有"记账＝可复盘"方案都必须正面回答而**七份文件里没有一份正面回答**的问题 |

### 6.3 给"三方读者"的适配建议（供作者取舍，不改正文）

| 读者 | 该看到哪一句 |
|---|---|
| **人（东西的主人）** | "不是不兼容，是缺一层" ＋ **上限句**（`WC-PREFACE-001-v0.7.md:100` 的范式：能消灭"查无此申请"，消灭不了"查无此后果"） |
| **Agent 的开发者** | 四层各一层的最小反例（权限位只到文件／unit 只到进程／RBAC 只到对象／工具定义里没有不可逆性这一格）——**这是他能立刻验证的四句话** |
| **建造者／评审者** | "要补的不是内核，是世界核心的本体与账本" ＋ **R-1 注入式实验**（§Q4③） |

---

## 结论栏

（留空）

---

## 签字栏

（留空）

---

## 附录：本席的取证清单与边界

### A. 本席实际读过的仓库文件（逐一）

| 文件 | 读到的位置 |
|---|---|
| `00-总纲.md` | `:25-104`、`:335`、`:363`、`:378`、`:439`、`:130` |
| `2-依据\15-世界核心的组成与职责.md` | `:28-87`、`:95-144`、`:180`、`:185`、`:188`、`:200`、`:210`、`:251`、`:275`、`:285`、`:304` |
| `2-依据\08-创世路线论证.md` | `:4`、`:24`、`:33`、`:50`、`:83`、`:111`、`:118`、`:120` |
| `2-依据\11-总线工程规格.md` | `:13`、`:27`、`:30-89`、`:103-119` |
| `2-依据\13-总线与通道外部参考.md` | `:54`、`:72`、`:147`、`:199`、`:202` |
| `2-依据\06-重复工作审计.md` | `:4`、`:14`、`:26`、`:36`、`:53`、`:58` |
| `2-依据\00-依据索引.md` | `:23`、`:27`、`:35`、`:37`、`:38` |
| `2-依据\02-开发工作流.md` | `:162`（§1.6 T1 用） |
| `2-依据\09-理论修正推导.md` | `:265`、`:270`（§1.6 T3 用） |
| `1-理论与哲学\01-三者关系论.md` | `:183`、`:208`、`:209`、`:216`（§1.6 T1、T4 用；**这是全仓对"截图 vs 语义树"定级最直接的一段**） |
| `1-理论与哲学\06-协作与承载.md` | `:100-139` |
| `world-core\docs\理论\WC-BOOK-001-v0.1.md` | `:21`、`:28`、`:29`、`:118`、`:136-180`、`:181`、`:413-572`、`:609-653`、`:689`、`:690`、`:692`、`:705`、`:746` |
| `world-core\docs\理论\WC-PREFACE-001-v0.7.md` | `:88-127`、`:495-539` |
| `world-core\docs\理论\WC-PREFACE-TRACE-002-v0.1.md` | `:30` |
| `world-core\docs\理论\WC-PREFACE-LOG-001-v0.2.md` | `:36`、`:37`、`:92` |
| `world-core\docs\理论\语义世界-序.md` | `:11`、`:25`、`:27` |
| `world-core\docs\理论\专家评审\席D-外部系统反面举证-独立评审.md` | `:21-80`、`:176-285`、`:305`（**说明**：`:104-139` 仅由 `Select-String` 标题清单确认存在，**正文未逐字读**，故本文不引该段） |
| `world-core\docs\理论\专家评审\席N-序-本原与派生主线梳理.md` | `:109`、`:119`、`:175`、`:276`、`:277`、`:406` |
| `world-core\docs\理论\专家评审\席G-经济与运维-独立评审.md` | `:368` |
| `world-core\docs\理论\专家评审\席I-三方接受度-独立评审.md` | `:24`、`:34`、`:171`、`:249` |
| `world-core\docs\理论\专家评审\席H-对抗性批判-红队评审.md` | `:76`、`:78`、`:761` |
| `world-core\docs\理论\专家评审\席M-序-v0.5-新增三处对表.md` | `:63`、`:92`、`:93`、`:105`、`:107`、`:115`、`:176` |
| `world-core\docs\阶段外-待启用\S2-设计\WC-PFMT-001-v0.1.md` | `:24`、`:149`、`:163`、`:181`、`:188`、`:190`、`:192`、`:214`、`:518`、`:534`、`:552`、`:579`、`:587`、`:599` |
| `world-core\docs\S2-设计\WC-IC-001-v0.1.md` | `:660`、`:674`、`:691`、`:700-708`、`:737`、`:743`、`:760`、`:792` |
| `world-core\docs\S2-设计\WC-FMT-001-v0.1.md` | `:54`、`:114`、`:119`、`:123-124`、`:135`、`:137` |
| `world-core\docs\S1-需求\WC-SRS-001-v0.1.md` | `:269`、`:271`、`:384`、`:601`、`:783`、`:838` |
| `world-core\docs\理论\WC-BOOK-001-v0.2.md` / `v0.3.md` | 对应行（用于确认 v0.1 引文的版本一致性） |

### B. 本席实际读过的外部研究件

| 文件 | 读到的位置 | 与本席问题的相关性 |
|---|---|---|
| `D:\Code\research\SYSTEMD_FOR_WORLD_CORE.md`（181,246 B） | `:16`、`:21`、`:74`、`:79`、`:163`、`:271-273`、`:277-396`、`:590`、`:650`、`:656`、`:876`、`:917`、`:929`、`:985`、`:994`、`:999`、`:1102-1111`、`:1116-1245`、`:1304-1373`、`:1462-1574` | **有直接相关事实**。最关键三处：① `:1347` 标题「**4.8 「语义对象」不在 systemd 的模型里 —— 它服务的是「程序」**」（Q1、Q2、Q6）；② `:1356` / `:1308` DynamicUser UID 回收破坏 `state = fold(events)`（Q1.2）；③ `:1203` / `:1184-1187` 三档强度表与"mount namespace 可被特权进程撤销"（Q2 反例 2）。④ `:1124` 逐字「**Note that the various options that turn directories read-only … cannot be used to lock down access to IPC services hence.**」 |
| `D:\Code\world-core-topology-research\world-core-进程拓扑对标调研.md`（196,436 B） | `:24-33`、`:103-119`、`:231-254`、`:369-390` | **有直接相关事实**。最关键三处：① `:110` 逐字「**socket 文件权限只能回答「谁能连」，回答不了「你是谁」和「你能不能写」。**」；② `:107` 逐字「**凡是做对了的项目，身份都不是「被发送」的而是「被观察」的**」（Q1.2 身份判据）；③ `:112` 逐字「PostgreSQL hot standby 自认「**no part of the database is truly read-only** during hot standby mode**」＋ SQLite `immutable=1` 是**断言不是访问控制**（Q1.1 判据②的旁证） |

### C. 本席取到的外部第一手资料（链接）

- [capabilities(7) — Linux manual page](https://man7.org/linux/man-pages/man7/capabilities.7.html)（capability 定义、`CAP_CHOWN`/`CAP_DAC_OVERRIDE`/`CAP_NET_ADMIN`/`CAP_SYS_TIME` 逐条、**64 位上限**逐字）
- [landlock(7) — Linux manual page](https://man7.org/linux/man-pages/man7/landlock.7.html)（Landlock 对象模型 = **file hierarchy ＋ TCP port**；`CAVEATS` 段逐字承认 `chmod`/`chown`/`setxattr` 等**不可限制**）
- [MCP 2025-06-18 · Tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)（逐字"Tools in MCP are designed to be **model-controlled**"、"the protocol itself **does not mandate any specific user interaction model**"、人在环是 **SHOULD**）
- [SPIFFE 与 OAuth 客户认证（IETF OAuth WG 临时会议材料）](https://datatracker.ietf.org/meeting/interim-2025-oauth-09/materials/slides-interim-2025-oauth-09-sessa-oauth-spiffe-client-authentication-00#1#1)
- [OAuth 与 SPIFFE（IETF 116 材料）](https://datatracker.ietf.org/meeting/116/materials/slides-116-oauth-sessb-oauth-and-spiffe-00.pdf#1#1)

**§1.6（作者补充的"截图回路"一节）新增取到的外部第一手资料**：

- [freedesktop.org · Accessibility/AT-SPI2](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/)（逐字"**At-Spi2 is a protocol over DBus, toolkit widgets use it to provide their content to screen readers such as Orca.**"、"**For each application (seen as a dbus sender), its tree of widgets is represented as a tree of dbus paths.**"、"**Wayland：Works just the same :D**"；以及"**If an application does not enable at-spi2 support by default, it should monitor that property**"与 `IsEnabled`／强制开启的环境变量清单）
- [XDG Desktop Portal · Screenshot（version 3）](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html)（`Screenshot.Screenshot` 逐字：`modal` "**Whether the dialog should be modal. Defaults to "true".**"、`interactive` "**Hint whether the dialog should offer customization before taking a screenshot. Defaults to "false".**"；`AvailableTargets` 为 Screen／Window／Area／Active Window 四种）
- [XDG Desktop Portal · RemoteDesktop（version 2）](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)（`Start()` 逐字"**This will typically result in the portal presenting a dialog letting the user select what to share**"；设备类型仅 `KEYBOARD`/`POINTER`/`TOUCHSCREEN`；输入两条路：**EIS（recommended）** 或 D-Bus `Notify*`）
- [sd_bus_message_set_expect_reply(3)](https://man7.org/linux/man-pages/man3/sd_bus_message_set_expect_reply.3.html)（逐字"**This flag matters only for method call messages and is used to specify that no method return or error reply is expected.**"——**即：D-Bus 的应答是默认，抑制才需要显式设 flag**）
- [XTEST Extension Protocol（X11）](https://www.mathematik.uni-marburg.de/local-doc/rhel6/xorg-x11-docs-1.3-6.1.el6/hardcopy/Xext/xtest.pdf#1#1) 与 [wayland-devel 2012 关于 X11 合成事件安全性的讨论存档](https://lists.x.org/archives/wayland-devel/2012-February/002192.html)（用于对照"Wayland 下注入输入必须经 portal 用户同意"这一条）

**本席为核实委托方点名的机制而实际联网取到的页（说明用途）**：`capabilities(7)`、`landlock(7)`、MCP Tools 规范页（**这三页的原文已逐字引入正文**）；`systemd.exec(5)` 与 `dbus-broker` 的官方页**未由本席直接取**，本文引它们的每一处**均转引本仓已有的外部研究件 `SYSTEMD_FOR_WORLD_CORE.md` 的行号**（该件把官方原文与该件自己的判语分行写，本席只引其引文行）。

**本席尝试过但取不到的（供复核）**：
- `https://raw.githubusercontent.com/modelcontextprotocol/modelcontextprotocol/refs/heads/main/docs/specification/2025-06-18/server/tools.mdx` → **fetch failed**（这就是 §1.4(a) 里"MCP `Tool` 字段清单未能逐字核实"的原因）。
- SPIFFE 规范的**授权边界逐字否认句** → **未取到**（只在 IETF 会议材料里取到"身份"议题）。

### D. 本席未能核实的项（诚实登记）

| # | 未核实的内容 | 本席的处置 |
|---|---|---|
| 1 | **SPIFFE 规范原文中"SPIFFE 只做身份、不做业务授权"的逐字句** | **不写成事实**。§1.3(c)、Q2 反例 6 已标记为"本席的推断，未能逐字核实" |
| 2 | **MCP 规范 `Tool` 数据类型的逐字字段清单**（尝试抓 `tools.mdx` 源文件失败） | **不写成事实**。§1.4(a) 只用 `席D…独立评审.md:189` 的 `Resource` 字段清单，**不把"无 version"外推到 `Tool`** |
| 3 | 「**今天的接入成本是几小时**」等任何成本数字 | **查不到**。本仓 `WC-PREFACE-001-v0.7.md:108` 逐字自认"**本序没有采集**"，`:514` 标"**未采集**" |
| 4 | 「**Agent 调用是很费劲的**」的仓库内**定量**实证 | **仓库内无实证**（机制级实证有，见 §Q3；定量实证无） |
| 5 | **`WC-BOOK-001` 第一版成文日期**与作者原话的**时间先后** | **未核**。因此本席**不主张**"作者说这句话时已经看到了 B-8 那条自首"，只主张"仓库里已存在该自首"（§3.1） |
| 6 | 本项目**实测**的 agent 接入过程记录（如某个真实 agent 接某工具的耗时/失败次数） | **仓库内查不到**。本席检索了 `2-依据\*.md`、`1-理论与哲学\*.md`、`4-计划\*.md`、`世界核心\docs\理论\**` 与 `docs\S*`，**未见**此类记录 |

### E. 本席使用的检索式（供复核）

```
在 07-agent-native-os 全仓 *.md 上：
  逐个工具|一个一个工具|逐个试|每个工具|一套参数|错误码|试错
  文档.{0,12}滞后|滞后于实现|文档永远|四本账|共同主键|没有主键
在 world-core\docs\理论\专家评审\*.md 上：
  不兼容|兼容 agent|兼容 Agent|为 Agent|给 Agent|Agent 的位置|非人主体|一等组件|PID 1|PID1
在 2-依据\*.md 与 1-理论与哲学\*.md 上：
  一等组件|PID 1|PID1|位置|兼容
  接入成本|接入摩擦|MCP|方言|能力表|cap\.d|试错|重新学|门槛
在 00-总纲.md 上：
  MCP|砍掉
  EXTERNAL 检索：SPIFFE SVID authorization / Linux capabilities Landlock AppArmor field-level / MCP tool schema breaking change
```

---

（文末 · 本席未修改任何被评审文件）
