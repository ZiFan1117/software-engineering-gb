// Package job 完工铃（05 篇第 6.5-7.5 小时）。
// agent 登记"干完了叫我"然后睡觉；活儿结束铃响一次，带结果码与日志位置。
// 登记簿落盘持久化：断电重启后能报告"上次那个活没跑完"（07 篇第 5 节）。
package job

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// Status 生命周期：running → done / failed / lost(重启后发现未跑完)。
const (
	StatusRunning = "running"
	StatusDone    = "done"
	StatusFailed  = "failed"
	StatusLost    = "lost"
)

// Job 是一条登记记录。
type Job struct {
	ID         string    `json:"id"`
	Cmd        string    `json:"cmd"`
	Args       []string  `json:"args,omitempty"`
	Status     string    `json:"status"`
	ExitCode   int       `json:"exit_code"`
	StartedAt  time.Time `json:"started_at"`
	FinishedAt time.Time `json:"finished_at,omitempty"`
}

// Bell 是完工事件：铃只响一次，听铃人醒来处理。
type Bell struct {
	JobID    string
	Status   string
	ExitCode int
}

// Runner 抽象命令执行（测试注入）。
type Runner interface {
	Start(cmd string, args []string) (Waiter, error)
}

// Waiter 等待一个已启动进程结束。
type Waiter interface {
	Wait() int // 返回 exit code
}

// Registry 登记簿。
type Registry struct {
	mu        sync.Mutex
	jobs      map[string]*Job
	order     []string
	storePath string
	run       Runner
	nextID    int

	bellMu  sync.Mutex
	bells   []chan Bell
	pending []Bell // 已响但还没有订阅者领取的铃（防丢铃）
}

// execRunner 生产实现：os/exec。
type execRunner struct{}

func (execRunner) Start(cmd string, args []string) (Waiter, error) {
	p := execCmd(cmd, args)
	if err := p.Start(); err != nil {
		return nil, err
	}
	return &execWaiter{p}, nil
}

// NewRegistry 载入（或初始化）登记簿。重启后仍为 running 的记录标记为 lost——
// "上次那个活没跑完（机器断电）"。
func NewRegistry(storePath string, run Runner) (*Registry, error) {
	if run == nil {
		run = execRunner{}
	}
	r := &Registry{jobs: map[string]*Job{}, storePath: storePath, run: run}
	if err := os.MkdirAll(filepath.Dir(storePath), 0o750); err != nil {
		return nil, err
	}
	if data, err := os.ReadFile(storePath); err == nil {
		var loaded []*Job
		if err := json.Unmarshal(data, &loaded); err != nil {
			return nil, fmt.Errorf("job 登记簿损坏: %w", err)
		}
		for _, j := range loaded {
			if j.Status == StatusRunning {
				j.Status = StatusLost
			}
			r.jobs[j.ID] = j
			r.order = append(r.order, j.ID)
			if n := len(j.ID); n > r.nextID {
				// id 形如 j<N>，重启后续号
				var num int
				if _, err := fmt.Sscanf(j.ID, "j%d", &num); err == nil && num > r.nextID {
					r.nextID = num
				}
			}
		}
		_ = r.persistLocked()
	}
	return r, nil
}

// Start 登记并启动一个活儿，返回 job-id。
func (r *Registry) Start(cmd string, args ...string) (string, error) {
	r.mu.Lock()
	r.nextID++
	id := fmt.Sprintf("j%d", r.nextID)
	j := &Job{ID: id, Cmd: cmd, Args: args, Status: StatusRunning, StartedAt: time.Now()}
	r.jobs[id] = j
	r.order = append(r.order, id)
	if err := r.persistLocked(); err != nil {
		r.mu.Unlock()
		return "", err
	}
	run := r.run
	r.mu.Unlock()

	w, err := run.Start(cmd, args)
	if err != nil {
		r.mu.Lock()
		j.Status = StatusFailed
		j.ExitCode = -1
		j.FinishedAt = time.Now()
		perr := r.persistLocked()
		r.mu.Unlock()
		r.ring(Bell{JobID: id, Status: j.Status, ExitCode: -1})
		if perr != nil {
			return id, perr
		}
		return id, nil // 启动失败也响铃，语义：failed
	}
	go func() {
		code := w.Wait()
		r.mu.Lock()
		j.Status = StatusDone
		if code != 0 {
			j.Status = StatusFailed
		}
		j.ExitCode = code
		j.FinishedAt = time.Now()
		perr := r.persistLocked()
		r.mu.Unlock()
		_ = perr
		r.ring(Bell{JobID: id, Status: j.Status, ExitCode: code})
	}()
	return id, nil
}

// Get 查询一条登记。
func (r *Registry) Get(id string) (Job, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()
	j, ok := r.jobs[id]
	if !ok {
		return Job{}, false
	}
	return *j, true
}

// All 返回全部登记（按登记顺序）。
func (r *Registry) All() []Job {
	r.mu.Lock()
	defer r.mu.Unlock()
	out := make([]Job, 0, len(r.order))
	for _, id := range r.order {
		out = append(out, *r.jobs[id])
	}
	return out
}

// Subscribe 领一只铃。每个通道独立响铃，缓冲 16 条防阻塞。
// 若在订阅之前已经有铃响过（活儿干得快），这里会把它们补发给第一个订阅者，
// 避免"铃响在订阅之前"造成的丢铃（agent 永久挂起）。
func (r *Registry) Subscribe() <-chan Bell {
	ch := make(chan Bell, 16)
	r.bellMu.Lock()
	for _, b := range r.pending {
		select {
		case ch <- b:
		default:
		}
	}
	r.pending = nil
	r.bells = append(r.bells, ch)
	r.bellMu.Unlock()
	return ch
}

func (r *Registry) ring(b Bell) {
	r.bellMu.Lock()
	defer r.bellMu.Unlock()
	if len(r.bells) == 0 {
		// 还没有任何订阅者：先存着，等第一个订阅者来时补发（防止丢铃）
		r.pending = append(r.pending, b)
		return
	}
	for _, ch := range r.bells {
		select {
		case ch <- b:
		default: // 铃不等人：缓冲满则丢弃并保持审计为事实源
		}
	}
}

func (r *Registry) persistLocked() error {
	list := make([]*Job, 0, len(r.order))
	for _, id := range r.order {
		list = append(list, r.jobs[id])
	}
	data, err := json.MarshalIndent(list, "", " ")
	if err != nil {
		return err
	}
	tmp := r.storePath + ".tmp"
	if err := os.WriteFile(tmp, data, 0o640); err != nil {
		return err
	}
	return os.Rename(tmp, r.storePath)
}
