package provider

import (
	"fmt"
	"strings"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/job"
)

// Jobs 把完工铃登记簿暴露为一项能力：job.start / job.status / job.list。
// 语义：agent 登记"干完了叫我"，然后睡觉——零轮询（07 篇第 2 节第 5 条）。
type Jobs struct{ Reg *job.Registry }

func NewJobs(r *job.Registry) *Jobs { return &Jobs{Reg: r} }

func (j *Jobs) Name() string { return "job" }

func (j *Jobs) Capabilities() []string {
	return []string{"job.start", "job.status", "job.list"}
}

// Call 支持动词：start / status / list。
func (j *Jobs) Call(verb string, args map[string]any) (map[string]any, error) {
	switch verb {
	case "start":
		cmd, _ := args["cmd"].(string)
		cmd = strings.TrimSpace(cmd)
		if cmd == "" {
			return nil, Errf("ext.agent.Job.BadArgs", "缺少 cmd")
		}
		var argv []string
		if raw, ok := args["args"].([]any); ok {
			for _, a := range raw {
				if s, ok := a.(string); ok {
					argv = append(argv, s)
				}
			}
		}
		id, err := j.Reg.Start(cmd, argv...)
		if err != nil {
			return nil, Errf("ext.agent.Job.Failed", "登记失败: %v", err)
		}
		return map[string]any{"job_id": id, "hint": "订阅完工铃，勿轮询"}, nil
	case "status":
		id, _ := args["id"].(string)
		jb, ok := j.Reg.Get(id)
		if !ok {
			return nil, Errf("ext.agent.Job.NotFound", "没有登记 %q", id)
		}
		return map[string]any{
			"job_id": jb.ID, "status": jb.Status, "exit_code": jb.ExitCode,
			"cmd": jb.Cmd, "started_at": jb.StartedAt.Unix(),
		}, nil
	case "list":
		all := j.Reg.All()
		out := make([]any, 0, len(all))
		for _, jb := range all {
			out = append(out, map[string]any{
				"job_id": jb.ID, "status": jb.Status, "exit_code": jb.ExitCode, "cmd": jb.Cmd,
			})
		}
		return map[string]any{"jobs": out}, nil
	default:
		return nil, fmt.Errorf("ext.agent.Job.Unsupported: 未知动词 %q", verb)
	}
}
