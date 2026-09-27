// agentd —— 系统级 agent 运行时的总装件（02 篇架构的参考实现骨架）。
// 职责装配：cap.d 能力表 + 内置 provider（backlight/package/job）+
// 结构化审计（journald 优先、文件回退）+ 快照管理 + JSON-RPC 服务。
package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"runtime"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/audit"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/capd"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/job"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/provider"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/router"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/server"
	"github.com/ZiFan1117/agent-native-os/agentd/internal/snapshot"
)

func main() {
	var (
		capDir     = flag.String("cap-dir", "/etc/agent/cap.d", "能力声明目录")
		listen     = flag.String("listen", "127.0.0.1:0", "监听地址（TCP）；或 unix:///run/agentd/agentd.sock")
		auditFile  = flag.String("audit-file", "/var/lib/agentd/audit.jsonl", "审计回退文件（journald 不可用时）")
		jobsStore  = flag.String("jobs-store", "/var/lib/agentd/jobs.json", "完工铃登记簿存储")
		agentName  = flag.String("agent-name", "agentd", "审计中标识的默认 agent 会话名")
		requireConfirm = flag.Bool("require-confirm", false, "启用 CLI 确认（生产用桌面通知）")
	)
	flag.Parse()

	log.SetPrefix("agentd: ")

	table, err := capd.LoadDir(*capDir)
	fatalIf(err, "加载能力声明")
	log.Printf("能力表已加载: %v", table.Names())

	aud := buildAuditor(*auditFile)

	reg, err := job.NewRegistry(*jobsStore, nil)
	fatalIf(err, "完工铃登记簿")
	go drainBells(reg, aud)

	snapMgr := snapshot.NewManager(snapshot.ExecRunner{})
	providers := map[string]provider.Provider{
		"backlight": provider.NewBrightness(),
		"package":   provider.NewPackage(snapMgr.Run),
		"job":       provider.NewJobs(reg),
	}

	rt := &router.Router{
		Table:     table,
		Providers: providers,
		Audit:     aud,
		Snapshots: snapMgr,
		Confirm:   cliConfirm(*requireConfirm),
		AgentName: *agentName,
	}

	ln, addr, err := listenOn(*listen)
	fatalIf(err, "监听")
	log.Printf("agentd 就绪: %s (os=%s)", addr, runtime.GOOS)
	fatalIf(server.Serve(ln, rt), "服务退出")
}

func buildAuditor(auditFile string) audit.Auditor {
	list := audit.Multi{audit.NewJournal()} // Linux+systemd 主路径
	if af, err := audit.NewFile(auditFile); err == nil {
		list = append(list, af)
	}
	return list
}

// drainBells 把完工事件写进审计——铃响即档案。
func drainBells(reg *job.Registry, aud audit.Auditor) {
	ch := reg.Subscribe()
	for b := range ch {
		_ = aud.Record(map[string]string{
			audit.FMessage: "job-bell",
			audit.FJobID:   b.JobID,
			audit.FOutcome: b.Status,
			"EXIT_CODE":    fmt.Sprintf("%d", b.ExitCode),
		})
	}
}

func cliConfirm(enabled bool) router.ConfirmFunc {
	if !enabled {
		return nil
	}
	return func(capName string, args map[string]any) bool {
		fmt.Printf("[confirm] 高危动作 %s 参数 %v，允许？(y/N) ", capName, args)
		var ans string
		_, _ = fmt.Scanln(&ans)
		return ans == "y" || ans == "Y" || ans == "yes"
	}
}

func listenOn(listen string) (net.Listener, string, error) {
	if path, ok := cutPrefix(listen, "unix://"); ok {
		if err := os.MkdirAll(filepath.Dir(path), 0o750); err != nil {
			return nil, "", err
		}
		ln, err := net.Listen("unix", path)
		return ln, path, err
	}
	ln, err := net.Listen("tcp", listen)
	if err != nil {
		return nil, "", err
	}
	return ln, ln.Addr().String(), nil
}

func cutPrefix(s, p string) (string, bool) {
	if len(s) >= len(p) && s[:len(p)] == p {
		return s[len(p):], true
	}
	return s, false
}

func fatalIf(err error, what string) {
	if err != nil {
		log.Fatalf("%s: %v", what, err)
	}
}
