package snapshot

import (
	"fmt"
	"strings"
	"testing"
)

// recRunner 记录全部命令。
type recRunner struct {
	commands []string
	failOn   string // 包含该子串时返回错误
}

func (r *recRunner) Run(name string, args ...string) (string, error) {
	line := name + " " + strings.Join(args, " ")
	if r.failOn != "" && strings.Contains(line, r.failOn) {
		r.commands = append(r.commands, line+" [FAIL]")
		return "", fmt.Errorf("注入失败: %s", line)
	}
	r.commands = append(r.commands, line)
	return "ok", nil
}

func TestBeforeTakesSnapshot(t *testing.T) {
	rr := &recRunner{}
	m := NewManager(rr)
	snap, err := m.Before("a-173f")
	if err != nil {
		t.Fatalf("Before: %v", err)
	}
	if snap == "" {
		t.Fatal("应返回快照路径")
	}
	if len(rr.commands) != 1 || !strings.Contains(rr.commands[0], "btrfs subvolume snapshot / ") {
		t.Fatalf("命令错: %v", rr.commands)
	}
	if !strings.Contains(rr.commands[0], "a-173f") {
		t.Fatalf("快照名应含 actionID: %v", rr.commands[0])
	}
}

func TestBeforeFailsPropagates(t *testing.T) {
	rr := &recRunner{failOn: "snapshot"}
	m := NewManager(rr)
	if _, err := m.Before("a-1"); err == nil {
		t.Fatal("应把快照失败传上来")
	}
}

func TestRollbackValidatesSnapshotExists(t *testing.T) {
	rr := &recRunner{failOn: "subvolume show"}
	m := NewManager(rr)
	if _, err := m.Rollback("missing"); err == nil {
		t.Fatal("快照不存在时回滚应报错")
	}

	ok := &recRunner{}
	m2 := NewManager(ok)
	snap, err := m2.Rollback("a-173f")
	if err != nil {
		t.Fatalf("Rollback: %v", err)
	}
	if snap == "" || len(ok.commands) != 2 {
		t.Fatalf("回滚应先 show 校验再执行恢复: %v", ok.commands)
	}
	if !strings.Contains(ok.commands[1], "agentd-rollback") {
		t.Fatalf("第二步应调用恢复执行器: %v", ok.commands[1])
	}
}

func TestEmptyActionIDRejected(t *testing.T) {
	m := NewManager(&recRunner{})
	if _, err := m.Before("  "); err == nil {
		t.Fatal("空 actionID 应被拒绝")
	}
}
