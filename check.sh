#!/usr/bin/env bash
# check.sh —— 仓根**唯一入口**：把这一次调用转发给世界核心的出厂门禁 `world-core/check.sh`。
#
# 为什么改成"转发"（2026-10-06，作者指示删除 `agentd/` 之后）：
#   本脚本此前是 **agentd（Go）的入口**（`go build` / `go vet` / `go test`）。
#   `agentd/` 已按作者裁定退场（`WC-FC-2026-005` §3.1「不再作独立组件 ⇒ 从工作树移除；旧件留 git 历史」，
#   解析根见该 change 的 `design.md` Migration Plan），于是本脚本原先那条
#   `agentd/ not found -- skipped` 分支会让它**什么都不跑、却打印 `CHECK_OK`**
#   —— 那正是本仓最忌的「空集恒真 ⇒ 判绿」。⇒ 现在它只做一件事：转发给真门禁。
#
# 用法：  ./check.sh
# 退出码：与 `world-core/check.sh` 相同（0 = 全过；非 0 = 任一步失败，阻断式）
#
# ⚠️ 射程（如实写）：宿主机（Windows）**没有 cargo、也没有 bash** ⇒ 本脚本连同
#   `world-core/check.sh` **只能在 Linux／VM 侧跑**。在哪跑、怎么同步、读数怎么取（四要素），
#   见 `world-core/docs/S5-测试/WC-ST-001-v0.1.md` §二／§四。
#   ★ 本脚本**不**替你把树同步到 VM，也**不**替你跑门禁 —— 它只是那一个入口。
set -uo pipefail
cd "$(dirname "$0")"

if [ ! -f world-core/check.sh ]; then
  echo "❌ 找不到 world-core/check.sh —— 入口断链，不许报绿" >&2
  exit 2
fi

exec bash world-core/check.sh "$@"
