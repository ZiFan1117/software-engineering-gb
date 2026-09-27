package server_test

// 烟囱式端到端：真起服务（TCP 回环），真过路由器（查表→审计→快照→确认→provider），
// 真 OS 动作（写 sysfs 假根、登记 job 等完工铃）。一条链跑穿，不 mock 中段。
import (
	"net"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/audit"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/capd"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/job"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/provider"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/router"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/server"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/snapshot"
)

const backlightYAML = `
capability: brightness.set
provider: backlight
verbs: [get, set]
risk: low
`

const packageYAML = `
capability: package.install
provider: package
verbs: [install]
risk: high
snapshot: before-each
require: confirm
`

const jobYAML = `
capability: job.start
provider: job
verbs: [start, status, list]
risk: low
`

// recRunner 记录外部命令（package 走这里，不碰真 pacman）。
type recRunner struct{ commands []string }

func (r *recRunner) Run(name string, args ...string) (string, error) {
	line := name
	for _, a := range args {
		line += " " + a
	}
	r.commands = append(r.commands, line)
	return "ok", nil
}

func buildTable(t *testing.T) *capd.Table {
	var caps []capd.Capability
	for _, d := range []string{backlightYAML, packageYAML, jobYAML} {
		c, err := capd.Parse([]byte(d))
		if err != nil {
			t.Fatal(err)
		}
		caps = append(caps, c)
	}
	return capd.TableFrom(caps...)
}

func fakeSysfs(t *testing.T) string {
	root := filepath.Join(t.TempDir(), "sys")
	dev := filepath.Join(root, "class", "backlight", "intel_backlight")
	if err := os.MkdirAll(dev, 0o750); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dev, "max_brightness"), []byte("100"), 0o644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dev, "brightness"), []byte("50"), 0o644); err != nil {
		t.Fatal(err)
	}
	return root
}

func TestEndToEndChimney(t *testing.T) {
	tmp := t.TempDir()

	// 1. 审计（文件回退路径，可读回断言）
	aud, err := audit.NewFile(filepath.Join(tmp, "audit.jsonl"))
	if err != nil {
		t.Fatal(err)
	}
	// 2. 完工铃登记簿（真执行器）+ 铃响入账（对应 main.go 的 drainBells）
	reg, err := job.NewRegistry(filepath.Join(tmp, "jobs.json"), nil)
	if err != nil {
		t.Fatal(err)
	}
	drain := reg.Subscribe()
	go func() {
		for b := range drain {
			_ = aud.Record(map[string]string{
				audit.FMessage: "job-bell",
				audit.FJobID:   b.JobID,
				audit.FOutcome: b.Status,
			})
		}
	}()
	// 3. provider 组装
	brightness := provider.NewBrightness()
	brightness.SysfsRoot = fakeSysfs(t)
	runner := &recRunner{}
	providers := map[string]provider.Provider{
		"backlight": brightness,
		"package":   provider.NewPackage(runner),
		"job":       provider.NewJobs(reg),
	}
	rt := &router.Router{
		Table:     buildTable(t),
		Providers: providers,
		Audit:     aud,
		Snapshots: snapshot.NewManager(runner),
		Confirm:   func(cap string, _ map[string]any) bool { return cap != "never.allow" },
		AgentName: "e2e-agent",
	}

	// 4. 真起服务，真连
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	go server.Serve(ln, rt)
	conn, err := net.Dial("tcp", ln.Addr().String())
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()

	// ── 场景 1：低危读亮度（现状里 agent 要解析文本；这里拿结构化字段）
	res, err := server.Call(conn, provider.Request{Capability: "brightness.set", Verb: "get"})
	// JSON 往返会把数字变成 float64
	if err != nil || !res.OK || res.Data["level"] != float64(50) || res.Data["max"] != float64(100) {
		t.Fatalf("get 亮度失败: %+v err=%v", res, err)
	}

	// ── 场景 2：低危调亮度，写假 sysfs 并读回
	res, _ = server.Call(conn, provider.Request{Capability: "brightness.set", Verb: "set", Args: map[string]any{"level": 42}})
	if !res.OK || res.Data["newlevel"] != float64(42) {
		t.Fatalf("set 亮度失败: %+v", res)
	}
	got, _ := os.ReadFile(filepath.Join(brightness.SysfsRoot, "class", "backlight", "intel_backlight", "brightness"))
	if string(got) != "42" {
		t.Fatalf("sysfs 应写成 42，实得 %q", got)
	}

	// ── 场景 3：超界 → 类型化错误
	res, _ = server.Call(conn, provider.Request{Capability: "brightness.set", Verb: "set", Args: map[string]any{"level": 999}})
	if res.OK || res.ErrCode != "ext.agent.Brightness.OutOfRange" {
		t.Fatalf("超界应类型化报错: %+v", res)
	}

	// ── 场景 4：白名单外 → 拒绝
	res, _ = server.Call(conn, provider.Request{Capability: "disk.format", Verb: "format"})
	if res.OK || res.ErrCode != "ext.agent.Denied.UnknownCapability" {
		t.Fatalf("白名单外应拒绝: %+v", res)
	}

	// ── 场景 5：高危装包——先快照、后确认、再执行
	res, _ = server.Call(conn, provider.Request{Capability: "package.install", Verb: "install", Args: map[string]any{"pkg": "ripgrep"}})
	if !res.OK || res.Data["installed"] != "ripgrep" {
		t.Fatalf("高危装包失败: %+v", res)
	}
	if len(runner.commands) < 2 || !strings.HasPrefix(runner.commands[0], "btrfs subvolume snapshot") {
		t.Fatalf("动手前应先快照: %v", runner.commands)
	}
	if !containsArg(runner.commands, "pacman -S --noconfirm ripgrep") {
		t.Fatalf("应执行 pacman 安装: %v", runner.commands)
	}

	// ── 场景 6：用户拒绝 → 审计留 denied-by-user
	rt.Confirm = func(string, map[string]any) bool { return false }
	res, _ = server.Call(conn, provider.Request{Capability: "package.install", Verb: "install", Args: map[string]any{"pkg": "badpkg"}})
	if res.OK || res.ErrCode != "ext.agent.Denied.UserConsent" {
		t.Fatalf("用户拒绝应生效: %+v", res)
	}
	rt.Confirm = func(cap string, _ map[string]any) bool { return true }

	// ── 场景 7：完工铃——登记长任务，零轮询，铃响收工
	bell := reg.Subscribe()
	var cmd string
	var args []string
	if runtime.GOOS == "windows" {
		cmd, args = "cmd", []string{"/c", "ping", "-n", "2", "127.0.0.1"}
	} else {
		cmd, args = "sh", []string{"-c", "sleep 0.3"}
	}
	res, _ = server.Call(conn, provider.Request{Capability: "job.start", Verb: "start", Args: map[string]any{"cmd": cmd, "args": anySlice(args)}})
	if !res.OK {
		t.Fatalf("job.start 失败: %+v", res)
	}
	jobID := res.Data["job_id"].(string)
	select {
	case b := <-bell:
		if b.JobID != jobID || b.Status != job.StatusDone {
			t.Fatalf("铃内容错: %+v", b)
		}
	case <-time.After(10 * time.Second):
		t.Fatal("等完工铃超时——铃没响")
	}
	res, _ = server.Call(conn, provider.Request{Capability: "job.start", Verb: "status", Args: map[string]any{"id": jobID}})
	if !res.OK || res.Data["status"] != job.StatusDone {
		t.Fatalf("job.status 应为 done: %+v", res)
	}

	// ── 场景 8：审计总账——意图/动作/铃全部在案
	lines, err := aud.ReadLines()
	if err != nil {
		t.Fatal(err)
	}
	var intents, actions, denied, bells int
	for _, m := range lines {
		switch {
		case m[audit.FMessage] == "intent":
			intents++
		case m[audit.FMessage] == "action" && m[audit.FOutcome] == "ok":
			actions++
		case m[audit.FOutcome] == "denied-by-user":
			denied++
		case m[audit.FMessage] == "job-bell":
			bells++
		}
	}
	if intents < 6 || actions < 4 || denied != 1 || bells != 1 {
		t.Fatalf("审计总账不对: intents=%d actions=%d denied=%d bells=%d（共 %d 条）",
			intents, actions, denied, bells, len(lines))
	}
}

func containsArg(commands []string, want string) bool {
	for _, c := range commands {
		if c == want {
			return true
		}
	}
	return false
}

func anySlice(ss []string) []any {
	out := make([]any, 0, len(ss))
	for _, s := range ss {
		out = append(out, s)
	}
	return out
}
