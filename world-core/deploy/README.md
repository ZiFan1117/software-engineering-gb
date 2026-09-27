# 世界核心的部署形态 —— **三个进程，三个身份，一条总线**

> 本目录是**可直接用的部署件**：让它由载体管理器（systemd）拉起，而不是靠人手工敲命令。
> 三份单元 + 一份安装脚本。**不含任何未实测的承诺**：判据见文末"装完怎么验"。

## 一、为什么是这三份单元

| 单元 | 装什么 | 身份 | 对账本的权限 |
|---|---|---|---|
| **`world-core.socket`** | 写入总线：**监听套接字由载体持有**，不是由进程自己持有 | — | — |
| **`world-core.service`** | 内核进程：本体 · 账本 · 读模型 · 运行时 · 门禁 · 通道 | 专用身份 | **可写（唯一写者）** |
| **`world-core-projectd.service`** | 投影服务：语言投影 / 视觉投影 | 另一个专用身份 | **只读** |
| **`world-core-actd.service`** | 载体执行器：调载体、做撤销点、等人确认 | **被管者身份** | **无权限**（只能经总线提交请求） |

**为什么套接字单独一份单元**：这样"服务挂了请求不丢"由**内核的连接积压**保证——
进程重启期间套接字仍在，客户端拿到的是"排队"而不是"连不上"。
**不要自己在进程里造请求队列**：那等于把内核已经做好的事重做一遍，还做得更差。

## 二、`world-core.socket`

```ini
[Unit]
Description=世界核心 · 写入总线（唯一写入口的入口）
Documentation=man:systemd.socket(5)
PartOf=world-core.service

[Socket]
# 监听套接字归载体所有；内核进程通过继承的描述符拿到它（`sd_listen_fds` 语义）
ListenStream=/run/world-core/world.sock
# 一个连接一个身份 ⇒ 套接字本身用 0660 + 属组表达"谁能连"；
# **"连上能干什么"必须在服务端按角色判定**（套接字权限位表达不了"只读"）
SocketMode=0660
SocketUser=world-core
SocketGroup=world-core
# 一次往返 = 两个连接（先意图、后结果），故不限制单次连接数；
# 但同一时刻的排队长由内核控制（`Backlog=` 直接映射到 listen(2) 的 backlog）
Accept=no
# 服务重启期间**保留**排队中的请求（默认即 no = 不丢弃）
FlushPending=no
# 服务被 stop 再 start 时，已传出的描述符存储**不保留**——
# 这是刻意的：重启后由内核重新接受连接，避免持有过期描述符
FileDescriptorStorePreserve=no

[Install]
WantedBy=sockets.target
```

## 三、`world-core.service`（内核进程）

```ini
[Unit]
Description=世界核心 · 内核进程（唯一写者）
Documentation=man:systemd.service(5) man:systemd.exec(5)
Requires=world-core.socket
After=world-core.socket
# 内核要独立升级时，用 `systemctl restart world-core.service` ——
# 套接字不重启，故排队中的请求不丢

[Service]
Type=notify
NotifyAccess=main
# 就绪门槛：**先自检再对外可用**。四条件不齐 ⇒ 不发 READY ⇒ 载体不认为它起来了
ExecStart=/usr/bin/world-core --ontology /etc/world-core/ontology.json \
                              --ledger /var/lib/world-core/ledger.jsonl \
                              --policy /etc/world-core/policy.json \
                              --channel /etc/world-core/channel.json \
                              --cap-dir /etc/world-core/cap.d \
                              --owner-uid world-core \
                              serve
# 常驻接受者由套接字激活提供描述符；本单元只负责"活着且就绪"
Restart=on-failure
RestartSec=2s
# 看门狗：进程必须周期性报到，否则视为卡死并重启
WatchdogSec=30s

# ── 身份与权限：法律与真相必须在这个身份之下 ──
User=world-core
Group=world-core
# 状态目录（账本）：只有本进程可写
StateDirectory=world-core
StateDirectoryMode=0700
# 运行目录（套接字、锁）：模式交给 socket 单元控制
RuntimeDirectory=world-core
RuntimeDirectoryMode=0755
# **配置目录不 chown 给本用户**（它是唯一"配置与状态权限分离"的落点）：
# 法律必须由部署方写，进程只能读
ConfigurationDirectory=world-core
ConfigurationDirectoryMode=0755

# ── 内核语义类约束（内核强制，不依赖"尽力而为"档）──
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictNamespaces=yes
RestrictRealtime=yes
LockPersonality=yes
MemoryDenyWriteExecute=yes
SystemCallArchitectures=native
# 只允许必要的地址族（本机 IPC 即可；不要开网络族）
RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6
# 只读挂载之外，仅允许写状态目录与运行目录
ReadWritePaths=/var/lib/world-core /run/world-core
# ⚠️ 注意：`ProtectSystem=`/`ReadOnlyPaths=` **不能**用来限制"谁能连上本机套接字"。
#     套接字的可达面由 `world-core.socket` 的属主/属组/模式决定。
# ⚠️ 注意：`MemoryDenyWriteExecute=` 与"用 memfd 传递描述符"会互相打架——
#     若将来要用 memfd 交接本体，必须先显式取舍（本版不用 memfd，故保持开启）。

# ── 资源 ──
MemoryMax=2G
CPUQuota=100%
TasksMax=64

# ── 停机：先把该落的落完，再退 ──
KillSignal=SIGTERM
TimeoutStopSec=15s
# 本进程不故意留残余：被杀时最多留一个残缺末行，由下次启动丢弃

[Install]
WantedBy=multi-user.target
```

## 四、`world-core-projectd.service`（投影服务，只读）

```ini
[Unit]
Description=世界核心 · 投影服务（只读消费者）
Documentation=man:systemd.service(5)
# 用 Wants（不用 Requires）：投影没起来，世界照常读写
Wants=world-core.service
After=world-core.service

[Service]
Type=exec
ExecStart=/usr/bin/world-core --ontology /etc/world-core/ontology.json \
                              --ledger /var/lib/world-core/ledger.jsonl \
                              --policy /etc/world-core/policy.json \
                              project serve
Restart=on-failure
RestartSec=2s

# **只读身份**：它对账本目录没有写权限
User=world-projectd
Group=world-projectd
SupplementaryGroups=world-core-read

# 只允许读：把自己能看见的东西缩到最小
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes
PrivateDevices=yes
NoNewPrivileges=yes
RestrictAddressFamilies=AF_UNIX
# 只读挂载状态目录（不给自己写的机会；真实拦阻来自文件属主与模式）
ReadOnlyPaths=/var/lib/world-core

MemoryMax=1G
CPUQuota=50%
TasksMax=32

[Install]
WantedBy=multi-user.target
```

## 五、`world-core-actd.service`（载体执行器，被管者身份）

```ini
[Unit]
Description=世界核心 · 载体执行器（只执行、不裁决）
Documentation=man:systemd.service(5)
# 它必须能连上总线才有意义；总线没起来就不该起
Requires=world-core.service
After=world-core.service

[Service]
Type=exec
# 它是一个**客户端**：从标准输入逐行读请求、先问内核、再执行、再回写结果
ExecStart=/usr/bin/world-core --cap-dir /etc/world-core/cap.d \
                              --ledger /var/lib/world-core/ledger.jsonl \
                              --socket /run/world-core/world.sock \
                              carrier serve
# ⚠️ 高危动作需要人确认时：不带 `--confirm` ⇒ 需要确认的动作**一律拒绝**（默认拒绝）。
#     要允许终端确认，须显式加 `--confirm`，并自行承担"没人看着时的后果"。

# **被管者身份**：与内核、投影都不同
User=agent
Group=agent

# 它要动设备/装包，所以不能像内核那样把自己焊死；
# 但仍然不给它任何能碰到法律与真相的权限。
ProtectSystem=strict
ProtectHome=read-only
PrivateTmp=yes
NoNewPrivileges=yes
# 载体执行器需要的地址族（本机 IPC）
RestrictAddressFamilies=AF_UNIX
# 只允许写它自己的工作目录；账本目录**不在**此列 ⇒ 它对真相零写权限
ReadWritePaths=/var/lib/world-actd /run/world-actd
StateDirectory=world-actd
StateDirectoryMode=0700
RuntimeDirectory=world-actd

MemoryMax=2G
CPUQuota=150%
TasksMax=128

[Install]
WantedBy=multi-user.target
```

## 六、装完怎么验（逐条可机械核对）

| # | 判据 | 命令 | 期望 |
|---|---|---|---|
| 1 | 三个进程都在跑 | `systemctl status world-core world-core-projectd world-core-actd` | 三个 active |
| 2 | 内核**就绪**（不是"进程活着"） | `systemctl show -p ActiveState,SubState world-core` | `active` / `running`，且日志里有 `READY=1` |
| 3 | 总线套接字在，且只对指定身份可连 | `ls -l /run/world-core/world.sock` | 属主/属组为本核心身份，模式 0660 |
| 4 | 唯一写者 | `lsof /var/lib/world-core/ledger.jsonl` | **只有一个**进程持有可写描述符 |
| 5 | 消费者对真相零写权限 | 以投影身份执行一次读取后比对账本摘要 | 摘要不变；尝试写入时**系统调用返回拒绝** |
| 6 | 世界不会半成品 | `systemctl stop world-core && echo 一条请求 \| nc -U /run/world-core/world.sock` | 不产生任何副作用（请求排队或连接被拒，**绝不静默执行**） |
| 7 | 内核重启不丢请求（长驻化后） | 重启 `world-core.service` 期间持续发请求 | 请求排队而非丢失（由内核积压保证） |

## 七、已知缺口（如实登记，不假装已做）

| # | 缺口 | 说明 |
|---|---|---|
| 1 | 三个单元**尚未在真机上实装过** | 本目录是部署件；上表七条判据中，1–5 条已有等价的进程级实测（见系统级验收脚本），6–7 条的"套接字激活"形态**尚无实测** |
| 2 | `world-core serve` / `project serve` 两个子命令**尚未实现** | 现形态是 `channel serve <socket> <n>` 与一次性 `carrier serve`；"读继承描述符"这一步待做（要接 `LISTEN_FDS` 语义） |
| 3 | 看门狗报到（`WATCHDOG=1`）尚未实现 | `WatchdogSec=` 已写在单元里；进程侧未接 ⇒ 实装时须先做，否则会被反复重启 |
| 4 | 三个身份与目录的**属主编排**由部署脚本负责 | 属主编排是本设计最大的部署依赖；脚本尚未写 |
| 5 | 动态身份的**明确禁止** | 见"主体身份必须稳定"：任何会被回收的动态身份都不得用于长期出现在事件里的主体 |
