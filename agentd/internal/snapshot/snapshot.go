// Package snapshot 封装 btrfs 快照/回滚（05 篇第 5-6.5 小时）。
// risk=high 的动作前自动拍快照；rollback 恢复到动作前并在审计留一条 ROLLBACK。
// 命令经由 Runner 接口执行，测试可注入假执行器，不碰真盘。
package snapshot

import (
	"fmt"
	"path/filepath"
	"os/exec"
	"strings"
)

// Runner 抽象外部命令执行，便于测试注入。
type Runner interface {
	Run(name string, args ...string) (string, error)
}

// ExecRunner 用真实 exec 执行（生产路径）。
type ExecRunner struct{}

func (ExecRunner) Run(name string, args ...string) (string, error) {
	out, err := exec.Command(name, args...).CombinedOutput()
	return string(out), err
}

// Manager 管理动作前快照。
type Manager struct {
	// RootSubvol 是被保护的世界根（默认 /）。
	RootSubvol string
	// SnapDir 存放动作前快照（默认 /run/agentd/.pre，tmpfs，重启即清）。
	SnapDir string
	Run     Runner
}

func NewManager(runner Runner) *Manager {
	return &Manager{RootSubvol: "/", SnapDir: "/run/agentd/.pre", Run: runner}
}

func (m *Manager) snapPath(actionID string) string {
	return filepath.Join(m.SnapDir, actionID)
}

// Before 在执行高危动作前拍快照，返回快照路径。
func (m *Manager) Before(actionID string) (string, error) {
	if m == nil || m.Run == nil {
		return "", fmt.Errorf("snapshot: manager 未初始化")
	}
	if strings.TrimSpace(actionID) == "" {
		return "", fmt.Errorf("snapshot: actionID 不能为空")
	}
	_, err := m.Run.Run("btrfs", "subvolume", "snapshot", m.RootSubvol, m.snapPath(actionID))
	if err != nil {
		return "", fmt.Errorf("snapshot: %w", err)
	}
	return m.snapPath(actionID), nil
}

// Rollback 把根子卷恢复到快照时刻（02 篇第 4 节的最小实现）。
// 真实恢复需要在快照内启动（或 initramfs/单用户模式）执行；此处先完成
// 命令编排与校验，回滚执行器由部署方（initramfs 钩子）提供。
func (m *Manager) Rollback(actionID string) (string, error) {
	if m == nil || m.Run == nil {
		return "", fmt.Errorf("snapshot: manager 未初始化")
	}
	snap := m.snapPath(actionID)
	if _, err := m.Run.Run("btrfs", "subvolume", "show", snap); err != nil {
		return "", fmt.Errorf("rollback: 快照 %s 不存在: %w", snap, err)
	}
	out, err := m.Run.Run("agentd-rollback", snap, m.RootSubvol)
	if err != nil {
		return "", fmt.Errorf("rollback: %w: %s", err, strings.TrimSpace(out))
	}
	return snap, nil
}
