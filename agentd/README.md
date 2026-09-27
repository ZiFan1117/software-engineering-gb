# agentd —— 参考实现（05 篇"一日实施计划"的 walking skeleton）

> 一个把 Linux 现成机制组装给 agent 用的常驻系统服务：能力路由 + 快照钩子 + 结构化审计 + 完工铃。
> 本目录是 02 篇架构的最小可用实现，主干结构化，处处留策略位。

## 它现在就会干的事（本机已验证）

| 能力 | 语义 | 状态 |
|---|---|---|
| `brightness.set` (get/set) | 直写 sysfs，结构化返回，超界类型化报错 | ✅ 逻辑全绿（真硬件需 Linux） |
| `package.install` | risk=high → **先确认 → 再快照 → 后执行**，拒绝/失败全程入账 | ✅ 逻辑全绿（真 pacman 需 Linux） |
| `job.start` (start/status/list) | **完工铃**：登记长任务零轮询，结束响铃一次；登记簿落盘，断电重启标记 lost | ✅ 真执行已验证 |
| 白名单 | cap.d 之外的能力一律拒绝（`ext.agent.Denied.*`） | ✅ 已验证 |
| 审计 | 意图/动作/铃 全部结构化留痕（journald 协议优先，JSONL 回退） | ✅ 已验证 |

一条审计流水长这样（intent → action → bell）：

```
{"INTENT":"package.install.install","GRANTED_BY":"agent@user-consent-pending","OUTCOME":"admitted",...}
{"CAPABILITY":"package.install","SNAPSHOT":"/run/agentd/.pre/a-173f","OUTCOME":"ok","DURATION_MS":"4211",...}
{"MESSAGE":"job-bell","JOB_ID":"j1","OUTCOME":"done","EXIT_CODE":"0",...}
```

## 目录

```
cmd/agentd/          主程序（装配一切）
internal/capd/       能力声明解析器（声明即授权）
internal/router/     能力路由器：查表→确认→快照→转发→审计（心脏）
internal/audit/      journald 文本协议（纯 Go）+ JSONL 文件回退
internal/job/        完工铃登记簿（持久化 + 订阅）
internal/provider/   provider 契约 + backlight/package/job 内置实现
internal/server/     行分隔 JSON-RPC（varlink 语义的过渡协议）
internal/snapshot/   btrfs 快照/回滚命令编排
deploy/              agent.service（02 篇户口）+ cap.d 样例
```

## 构建 / 测试 / 运行

```bash
go build ./...        # 编译
go vet ./...          # 静态检查
go test ./... -count=1 -v   # 单元 + 端到端（烟囱式）测试，全绿

# 跑起来（Linux；Windows 冒烟见下）
sudo cp agentd /usr/bin/ && sudo systemctl enable --now agent.service
varlink call /run/agentd/agentd.sock ...   # 或直连 TCP

# Windows 开发机冒烟（无 sysfs/pacman 也应类型化报错而不是崩）
agentd.exe --cap-dir deploy/cap.d --listen 127.0.0.1:17998 \
  --audit-file %TEMP%\audit.jsonl --jobs-store %TEMP%\jobs.json
```

## 路由处置顺序（重要，02 篇流水线的实现）

```
查表（无钥匙拒绝）→ 审计意图 → [require=confirm] 用户确认（拒绝即断，不入快照）
→ [risk=high & snapshot=before-each] btrfs 快照 → 转发 provider → 审计动作（含耗时）
```

## 还没有的（对应 07 篇硌牙清单，按疼排序）

1. **能力编译器**：cap.d → 动态 Landlock/seccomp 规则集（真正的内核强制），当前靠部署期 systemd 沙箱指令兜底；
2. **桌面确认通道**：现为 CLI y/N，待接桌面通知一键确认；
3. **network provider**：修 WiFi 全场景（03 篇 demo）；
4. **试验场**：lab.spawn/commit/discard（06 篇第 5 节）；
5. **多 agent 互斥表** 与 A2A 频道（06 篇第 3 节）；
6. 真机验证清单：systemd 单元实装、journald 字段落库、btrfs 真快照/回滚、背光真硬件。
