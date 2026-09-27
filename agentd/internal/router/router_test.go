package router

import (
	"strings"
	"testing"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/audit"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/capd"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/provider"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/snapshot"
)

// stubProvider 记录收到的调用，返回固定值。
type stubProvider struct {
	name  string
	caps  []string
	calls []string
	err   error
	data  map[string]any
}

func (s *stubProvider) Name() string                 { return s.name }
func (s *stubProvider) Capabilities() []string       { return s.caps }
func (s *stubProvider) Call(verb string, _ map[string]any) (map[string]any, error) {
	s.calls = append(s.calls, verb)
	if s.err != nil {
		return nil, s.err
	}
	return s.data, nil
}

// capTable 从 yaml 字符串构建能力表。
func capTable(t *testing.T, docs ...string) *capd.Table {
	t.Helper()
	var caps []capd.Capability
	for _, d := range docs {
		c, err := capd.Parse([]byte(d))
		if err != nil {
			t.Fatalf("capd.Parse: %v", err)
		}
		caps = append(caps, c)
	}
	return capd.TableFrom(caps...)
}

// recAudit 收集审计记录。
type recAudit struct{ records []map[string]string }

func (r *recAudit) Record(f map[string]string) error {
	r.records = append(r.records, f)
	return nil
}

func (r *recAudit) outcomes() []string {
	var out []string
	for _, m := range r.records {
		out = append(out, m[audit.FOutcome])
	}
	return out
}

const lowYAML = `
capability: brightness.set
provider: backlight
verbs: [get, set]
risk: low
`

const highYAML = `
capability: package.install
provider: package
verbs: [install]
risk: high
snapshot: before-each
require: confirm
`

func newRouter(t *testing.T, tbl *capd.Table, prov provider.Provider, a audit.Auditor, snaps *snapshot.Manager, confirm ConfirmFunc) *Router {
	return &Router{
		Table:     tbl,
		Providers: map[string]provider.Provider{prov.Name(): prov},
		Audit:     a,
		Snapshots: snaps,
		Confirm:   confirm,
	}
}

func TestUnknownCapabilityDenied(t *testing.T) {
	tbl := capTable(t, lowYAML)
	p := &stubProvider{name: "backlight", data: map[string]any{}}
	res := newRouter(t, tbl, p, &recAudit{}, nil, nil).
		Handle(provider.Request{Capability: "disk.format", Verb: "format"})
	if res.OK || res.ErrCode != "ext.agent.Denied.UnknownCapability" {
		t.Fatalf("白名单外应拒绝: %+v", res)
	}
	if len(p.calls) != 0 {
		t.Fatal("不应触达 provider")
	}
}

func TestVerbNotAllowed(t *testing.T) {
	tbl := capTable(t, lowYAML)
	p := &stubProvider{name: "backlight", data: map[string]any{}}
	res := newRouter(t, tbl, p, &recAudit{}, nil, nil).
		Handle(provider.Request{Capability: "brightness.set", Verb: "format"})
	if res.OK || res.ErrCode != "ext.agent.Denied.VerbNotAllowed" {
		t.Fatalf("未授权动词应拒绝: %+v", res)
	}
}

func TestLowRiskFlowAudits(t *testing.T) {
	tbl := capTable(t, lowYAML)
	p := &stubProvider{name: "backlight", data: map[string]any{"newlevel": 42}}
	ra := &recAudit{}
	res := newRouter(t, tbl, p, ra, nil, nil).
		Handle(provider.Request{Capability: "brightness.set", Verb: "set", Args: map[string]any{"level": 42}})
	if !res.OK || res.Data["newlevel"] != 42 {
		t.Fatalf("低危应直接放行: %+v", res)
	}
	if len(p.calls) != 1 || p.calls[0] != "set" {
		t.Fatalf("provider 调用错: %v", p.calls)
	}
	// 意图 + 动作两条审计
	if len(ra.records) != 2 {
		t.Fatalf("应 2 条审计，实得 %d: %v", len(ra.records), ra.records)
	}
	if ra.records[0][audit.FIntent] != "brightness.set.set" || ra.records[1][audit.FOutcome] != "ok" {
		t.Fatalf("审计字段错: %v", ra.records)
	}
	if ra.records[1][audit.FDurationMS] == "" {
		t.Fatal("动作审计应带耗时")
	}
}

func TestHighRiskConfirmGate(t *testing.T) {
	tbl := capTable(t, highYAML)
	p := &stubProvider{name: "package", data: map[string]any{"installed": "ripgrep"}}

	// 无确认入口 → 拒绝
	res := newRouter(t, tbl, p, &recAudit{}, nil, nil).
		Handle(provider.Request{Capability: "package.install", Verb: "install", Args: map[string]any{"pkg": "ripgrep"}})
	if res.OK || res.ErrCode != "ext.agent.Denied.ConfirmUnavailable" {
		t.Fatalf("缺确认入口应拒绝: %+v", res)
	}

	// 用户拒绝 → denied-by-user，provider 未被调用
	ra := &recAudit{}
	res = newRouter(t, tbl, p, ra, nil, func(string, map[string]any) bool { return false }).
		Handle(provider.Request{Capability: "package.install", Verb: "install"})
	if res.OK || res.ErrCode != "ext.agent.Denied.UserConsent" {
		t.Fatalf("用户拒绝应拒绝: %+v", res)
	}
	if len(p.calls) != 0 {
		t.Fatal("拒绝后不应触达 provider")
	}
	found := false
	for _, m := range ra.records {
		if m[audit.FOutcome] == "denied-by-user" {
			found = true
		}
	}
	if !found {
		t.Fatalf("拒绝应留审计: %v", ra.records)
	}

	// 用户允许 → 快照 → 放行
	ra2 := &recAudit{}
	snaps := snapshot.NewManager(&recSnapshotRunner{})
	rt := newRouter(t, tbl, p, ra2, snaps, func(string, map[string]any) bool { return true })
	res = rt.Handle(provider.Request{Capability: "package.install", Verb: "install", Args: map[string]any{"pkg": "ripgrep"}})
	if !res.OK {
		t.Fatalf("确认后应放行: %+v", res)
	}
	if len(p.calls) != 1 {
		t.Fatalf("provider 调用次数错: %v", p.calls)
	}
	snapAudited := false
	for _, m := range ra2.records {
		if strings.Contains(m[audit.FSnapshot], "a-") {
			snapAudited = true
		}
	}
	if !snapAudited {
		t.Fatalf("确认+放行路径应带快照审计: %v", ra2.records)
	}
}

func TestHighRiskSnapshotBefore(t *testing.T) {
	tbl := capTable(t, highYAML)
	p := &stubProvider{name: "package", data: map[string]any{}}
	rr := &recSnapshotRunner{}
	snaps := snapshot.NewManager(rr)
	ra := &recAudit{}
	rt := newRouter(t, tbl, p, ra, snaps, func(string, map[string]any) bool { return true })
	res := rt.Handle(provider.Request{Capability: "package.install", Verb: "install", Args: map[string]any{"pkg": "ripgrep"}})
	if !res.OK {
		t.Fatalf("应放行: %+v", res)
	}
	if len(rr.commands) != 1 || !strings.Contains(rr.commands[0], "btrfs subvolume snapshot") {
		t.Fatalf("动手前应有快照: %v", rr.commands)
	}
	var snapAudited bool
	for _, m := range ra.records {
		if strings.Contains(m[audit.FSnapshot], "a-") {
			snapAudited = true
		}
	}
	if !snapAudited {
		t.Fatalf("快照路径应入审计: %v", ra.records)
	}
}

// recSnapshotRunner 让快照命令在测试里可跑。
type recSnapshotRunner struct{ commands []string }

func (r *recSnapshotRunner) Run(name string, args ...string) (string, error) {
	r.commands = append(r.commands, name+" "+join(args))
	return "ok", nil
}

func join(ss []string) string {
	out := ""
	for i, s := range ss {
		if i > 0 {
			out += " "
		}
		out += s
	}
	return out
}

func TestProviderFailureTypedError(t *testing.T) {
	tbl := capTable(t, lowYAML)
	p := &stubProvider{name: "backlight", err: provider.Errf("ext.agent.Brightness.OutOfRange", "level 超界")}
	res := newRouter(t, tbl, p, &recAudit{}, nil, nil).
		Handle(provider.Request{Capability: "brightness.set", Verb: "set", Args: map[string]any{"level": 999}})
	if res.OK || res.ErrCode != "ext.agent.Brightness.OutOfRange" {
		t.Fatalf("类型化错误应透传: %+v", res)
	}
}
