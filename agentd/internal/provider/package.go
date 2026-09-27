package provider

import (
	"strings"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/snapshot"
)

// Package 包管理 provider（cap.d 中 risk=high、snapshot=before-each、require=confirm 的范例）。
// 生产执行 pacman；测试注入假 Runner 捕获命令。
type Package struct {
	// Bin 包管理器可执行名（Arch 系为 pacman）。
	Bin string
	// Runner 命令执行器；nil 时用 snapshot.ExecRunner。
	Runner snapshot.Runner
}

func NewPackage(runner snapshot.Runner) *Package {
	if runner == nil {
		runner = snapshot.ExecRunner{}
	}
	return &Package{Bin: "pacman", Runner: runner}
}

func (p *Package) Name() string { return "package" }

func (p *Package) Capabilities() []string { return []string{"package.install"} }

// Call 支持动词：install（参数 pkg）。
func (p *Package) Call(verb string, args map[string]any) (map[string]any, error) {
	if verb != "install" {
		return nil, Errf("ext.agent.Package.Unsupported", "未知动词 %q", verb)
	}
	pkg, _ := args["pkg"].(string)
	pkg = strings.TrimSpace(pkg)
	if pkg == "" || strings.ContainsAny(pkg, " ;|&$`\n") {
		return nil, Errf("ext.agent.Package.BadArgs", "非法包名 %q", pkg)
	}
	if _, err := p.run(p.Bin, "-S", "--noconfirm", pkg); err != nil {
		return nil, Errf("ext.agent.Package.Failed", "安装 %s 失败: %v", pkg, err)
	}
	return map[string]any{"installed": pkg}, nil
}

func (p *Package) run(name string, args ...string) (string, error) {
	if r, ok := p.Runner.(snapshot.Runner); ok {
		return r.Run(name, args...)
	}
	return snapshot.ExecRunner{}.Run(name, args...)
}
