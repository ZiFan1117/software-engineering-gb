package job

import (
	"path/filepath"
	"runtime"
	"testing"
	"time"
)

var isWindows = runtime.GOOS == "windows"

// fakeRunner：可控的假执行器。
type fakeRunner struct {
	exitCode int
	wait     time.Duration
	started  int
}

func (f *fakeRunner) Start(cmd string, args []string) (Waiter, error) {
	f.started++
	d := f.wait
	return waiterFunc(func() int {
		if d > 0 {
			time.Sleep(d)
		}
		return f.exitCode
	}), nil
}

type waiterFunc func() int

func (w waiterFunc) Wait() int { return w() }

func waitBell(t *testing.T, r *Registry) Bell {
	t.Helper()
	select {
	case b := <-r.Subscribe():
		return b
	case <-time.After(2 * time.Second):
		t.Fatal("等铃超时")
		return Bell{}
	}
}

func TestStartDoneBell(t *testing.T) {
	fr := &fakeRunner{exitCode: 0}
	r, err := NewRegistry(filepath.Join(t.TempDir(), "jobs.json"), fr)
	if err != nil {
		t.Fatal(err)
	}
	id, err := r.Start("demo", "-x")
	if err != nil {
		t.Fatal(err)
	}
	b := waitBell(t, r)
	if b.JobID != id || b.Status != StatusDone || b.ExitCode != 0 {
		t.Fatalf("铃内容错: %+v", b)
	}
	j, _ := r.Get(id)
	if j.Status != StatusDone || j.ExitCode != 0 {
		t.Fatalf("登记状态错: %+v", j)
	}
}

func TestStartFailedBell(t *testing.T) {
	fr := &fakeRunner{exitCode: 3}
	r, _ := NewRegistry(filepath.Join(t.TempDir(), "jobs.json"), fr)
	id, _ := r.Start("build")
	b := waitBell(t, r)
	if b.Status != StatusFailed || b.ExitCode != 3 {
		t.Fatalf("失败铃错: %+v", b)
	}
	_ = id
}

func TestPersistenceAndLost(t *testing.T) {
	store := filepath.Join(t.TempDir(), "jobs.json")
	fr := &fakeRunner{exitCode: 0, wait: 500 * time.Millisecond} // 模拟"干着活机器没了"
	r1, _ := NewRegistry(store, fr)
	if _, err := r1.Start("long-build"); err != nil {
		t.Fatal(err)
	}
	// 不等铃，直接"重启"：新登记簿打开同一存储。
	r2, err := NewRegistry(store, &fakeRunner{exitCode: 0})
	if err != nil {
		t.Fatal(err)
	}
	all := r2.All()
	if len(all) != 1 {
		t.Fatalf("应恢复 1 条登记，实得 %d", len(all))
	}
	if all[0].Status != StatusLost {
		t.Fatalf("未完任务重启后应标记 lost，实得 %s", all[0].Status)
	}
}

func TestRealExecCrossPlatform(t *testing.T) {
	store := filepath.Join(t.TempDir(), "jobs.json")
	r, _ := NewRegistry(store, nil) // nil = 真执行器
	cmd, arg := "sh", []string{"-c", "exit 0"}
	// Windows 上用 cmd
	if isWindows {
		cmd, arg = "cmd", []string{"/c", "exit 0"}
	}
	id, err := r.Start(cmd, arg...)
	if err != nil {
		t.Fatal(err)
	}
	b := waitBell(t, r)
	if b.JobID != id || b.Status != StatusDone {
		t.Fatalf("真执行铃错: %+v", b)
	}
}
