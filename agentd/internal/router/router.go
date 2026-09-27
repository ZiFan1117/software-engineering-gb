// Package router 能力路由器——agentd 的心脏（02 篇第 2 节）。
// 每个请求：查 cap.d → 风险分级处置（快照/确认）→ 转发 provider → 全程审计。
package router

import (
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"time"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/audit"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/capd"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/provider"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/snapshot"
)

// ConfirmFunc 高危动作的用户确认入口。返回 false 即拒绝。
// 生产实现是桌面通知一键确认；测试注入桩。
type ConfirmFunc func(cap string, args map[string]any) bool

// Router 把能力声明、提供者、审计、快照组装成一条处置流水线。
type Router struct {
	Table     *capd.Table
	Providers map[string]provider.Provider
	Audit     audit.Auditor
	Snapshots *snapshot.Manager
	Confirm   ConfirmFunc
	// AgentName 记入审计 GRANTED_BY，标识是哪个 agent 会话。
	AgentName string
	// Now 可注入时钟（测试）；nil 用 time.Now。
	Now func() time.Time
}

func deny(req provider.Request, code, msg string) provider.Result {
	return provider.Result{OK: false, ErrCode: code, ErrMsg: msg + fmt.Sprintf("（capability=%s verb=%s）", req.Capability, req.Verb)}
}

func (r *Router) providerFor(c capd.Capability) (provider.Provider, error) {
	p, ok := r.Providers[c.Provider]
	if !ok {
		return nil, fmt.Errorf("provider %q 未注册", c.Provider)
	}
	return p, nil
}

func (r *Router) now() time.Time {
	if r.Now != nil {
		return r.Now()
	}
	return time.Now()
}

func (r *Router) audit(fields map[string]string) {
	if r.Audit == nil {
		return
	}
	_ = r.Audit.Record(fields)
}

// Handle 处置一个结构化请求：这是 02 篇那条流水线的实现。
func (r *Router) Handle(req provider.Request) provider.Result {
	// 1. 查能力表：没有钥匙，直接拒绝。
	cap, ok := r.Table.Lookup(req.Capability)
	if !ok {
		return deny(req, "ext.agent.Denied.UnknownCapability", "能力不在 cap.d 白名单")
	}
	if !cap.Allows(req.Verb) {
		return deny(req, "ext.agent.Denied.VerbNotAllowed", "能力未授权此动词")
	}
	p, err := r.providerFor(cap)
	if err != nil {
		return deny(req, "ext.agent.Denied.NoProvider", err.Error())
	}

	actionID := newActionID()
	// 2. 审计意图（先记后做，审计在 agent 之外）。
	r.audit(map[string]string{
		audit.FMessage:    "intent",
		audit.FIntent:     req.Capability + "." + req.Verb,
		audit.FCapability: req.Capability,
		audit.FVerb:       req.Verb,
		audit.FParams:     marshalArgs(req.Args),
		audit.FGrantedBy:  r.grantedBy(cap),
		audit.FOutcome:    "admitted",
	})

	// 3. 需要用户确认的高危动作：先过确认，拒绝的动作不配拥有快照。
	if cap.Require == capd.RequireConfirm {
		confirm := r.Confirm
		if confirm == nil {
			return deny(req, "ext.agent.Denied.ConfirmUnavailable", "需要确认但确认入口未配置")
		}
		if !confirm(req.Capability, req.Args) {
			r.finish(req, "denied-by-user", 0, "")
			return deny(req, "ext.agent.Denied.UserConsent", "用户拒绝本次动作")
		}
	}

	// 4. risk=high 且 snapshot=before-each：动手前先拍快照。
	snapPath := ""
	if cap.Snapshot == capd.SnapshotBeforeEach && cap.Risk == capd.RiskHigh {
		if r.Snapshots == nil {
			return deny(req, "ext.agent.Denied.SnapshotUnavailable", "此能力要求快照但快照管理器未配置")
		}
		sp, err := r.Snapshots.Before(actionID)
		if err != nil {
			r.finish(req, "snapshot-failed", 0, "")
			return deny(req, "ext.agent.Denied.SnapshotFailed", err.Error())
		}
		snapPath = sp
	}

	// 5. 转发 provider，结构化返回。
	t0 := r.now()
	data, err := p.Call(req.Verb, req.Args)
	dur := r.now().Sub(t0).Milliseconds()
	outcome := "ok"
	if err != nil {
		outcome = "failed"
	}
	r.audit(map[string]string{
		audit.FMessage:    "action",
		audit.FCapability: req.Capability,
		audit.FVerb:       req.Verb,
		audit.FParams:     marshalArgs(req.Args),
		audit.FSnapshot:   snapPath,
		audit.FOutcome:    outcome,
		audit.FDurationMS: fmt.Sprintf("%d", dur),
	})
	if err != nil {
		if pe, ok := err.(*provider.Error); ok {
			return deny(req, pe.Code, pe.Message)
		}
		return deny(req, "ext.agent.ProviderFailed", err.Error())
	}
	return provider.Result{OK: true, Data: data}
}

func (r *Router) grantedBy(c capd.Capability) string {
	name := r.AgentName
	if name == "" {
		name = "agentd"
	}
	tag := "auto"
	if c.Require == capd.RequireConfirm {
		tag = "user-consent-pending"
	}
	return name + "@" + tag
}

func (r *Router) finish(req provider.Request, outcome string, durMS int64, snap string) {
	f := map[string]string{
		audit.FMessage:    "action",
		audit.FCapability: req.Capability,
		audit.FVerb:       req.Verb,
		audit.FOutcome:    outcome,
		audit.FDurationMS: fmt.Sprintf("%d", durMS),
	}
	if snap != "" {
		f[audit.FSnapshot] = snap
	}
	r.audit(f)
}

func newActionID() string {
	b := make([]byte, 4)
	_, _ = rand.Read(b)
	return "a-" + hex.EncodeToString(b)
}

func marshalArgs(args map[string]any) string {
	if len(args) == 0 {
		return "{}"
	}
	b, err := json.Marshal(args)
	if err != nil {
		return "{}"
	}
	return string(b)
}
