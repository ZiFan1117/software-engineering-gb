package job

import (
	"path/filepath"
	"testing"
	"time"
)

// 确定性复现"丢铃"：活儿在订阅之前就干完了，铃永远收不到。
// 不依赖调度运气——先等活儿确定结束，再订阅。
func TestLateSubscribeLosesBell(t *testing.T) {
	fr := &fakeRunner{exitCode: 0}
	r, err := NewRegistry(filepath.Join(t.TempDir(), "jobs.json"), fr)
	if err != nil {
		t.Fatal(err)
	}
	id, err := r.Start("demo", "-x")
	if err != nil {
		t.Fatal(err)
	}

	// 等活儿确定干完（铃此时已经响过）
	deadline := time.Now().Add(3 * time.Second)
	for time.Now().Before(deadline) {
		if j, ok := r.Get(id); ok && j.Status != StatusRunning {
			break
		}
		time.Sleep(2 * time.Millisecond)
	}
	j, _ := r.Get(id)
	t.Logf("活儿状态 = %s（说明铃已经响过）", j.Status)

	// 现在才订阅
	sub := r.Subscribe()
	select {
	case b := <-sub:
		t.Logf("收到铃: %+v  ->  没丢", b)
	case <-time.After(500 * time.Millisecond):
		t.Errorf("丢铃：活儿已完成(%s)，但订阅后 500ms 内收不到任何铃", j.Status)
	}
}
