// Package capd parses /etc/agent/cap.d 能力声明文件（05 篇第 1-3 小时）。
// 声明即授权：每个文件声明一项能力（capability）——提供者、允许的动词、
// 风险等级、审计与快照策略、是否需要用户确认。
package capd

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// Risk 分级决定路由器的处置：low 直接放行，medium 留策略位，high 触发快照+确认。
type Risk string

const (
	RiskLow    Risk = "low"
	RiskMedium Risk = "medium"
	RiskHigh   Risk = "high"
)

const (
	SnapshotNever      = "never"
	SnapshotBeforeEach = "before-each"
	RequireConfirm     = "confirm"
)

// Capability 是一份 cap.d 声明的内存形态。
type Capability struct {
	Name     string   `yaml:"capability"`
	Provider string   `yaml:"provider"`
	Verbs    []string `yaml:"verbs"`
	Risk     Risk     `yaml:"risk"`
	Audit    string   `yaml:"audit"`
	Snapshot string   `yaml:"snapshot"`
	Require  string   `yaml:"require"`
	Sandbox  string   `yaml:"sandbox"`
}

// Allows 报告该能力是否放行指定动词。
func (c Capability) Allows(verb string) bool {
	for _, v := range c.Verbs {
		if v == verb {
			return true
		}
	}
	return false
}

func (c Capability) validate() error {
	if c.Name == "" {
		return fmt.Errorf("capability 字段不能为空")
	}
	if c.Provider == "" {
		return fmt.Errorf("%s: provider 字段不能为空", c.Name)
	}
	if len(c.Verbs) == 0 {
		return fmt.Errorf("%s: verbs 至少要有一项", c.Name)
	}
	switch c.Risk {
	case RiskLow, RiskMedium, RiskHigh:
	default:
		return fmt.Errorf("%s: 非法 risk=%q（low/medium/high）", c.Name, c.Risk)
	}
	switch c.Snapshot {
	case "", SnapshotNever, SnapshotBeforeEach:
	default:
		return fmt.Errorf("%s: 非法 snapshot=%q", c.Name, c.Snapshot)
	}
	switch c.Require {
	case "", RequireConfirm:
	default:
		return fmt.Errorf("%s: 非法 require=%q", c.Name, c.Require)
	}
	return nil
}

// Table 是全部已加载能力的索引，键为能力名。
type Table struct {
	caps map[string]Capability
}

func (t *Table) Lookup(name string) (Capability, bool) {
	c, ok := t.caps[name]
	return c, ok
}

func (t *Table) Names() []string {
	names := make([]string, 0, len(t.caps))
	for n := range t.caps {
		names = append(names, n)
	}
	return names
}

// Parse 解析单个 YAML 字节串。
func Parse(data []byte) (Capability, error) {
	var c Capability
	if err := yaml.Unmarshal(data, &c); err != nil {
		return c, fmt.Errorf("yaml 解析失败: %w", err)
	}
	if err := c.validate(); err != nil {
		return c, err
	}
	return c, nil
}

// TableFrom 用给定能力构建表（内嵌默认策略与测试用）。
func TableFrom(caps ...Capability) *Table {
	t := &Table{caps: map[string]Capability{}}
	for _, c := range caps {
		t.caps[c.Name] = c
	}
	return t
}

// LoadDir 加载目录下全部 .yaml/.yml 声明；同名能力后加载者覆盖前者。
func LoadDir(dir string) (*Table, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("读取 cap.d 目录 %s: %w", dir, err)
	}
	t := &Table{caps: map[string]Capability{}}
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		ext := strings.ToLower(filepath.Ext(e.Name()))
		if ext != ".yaml" && ext != ".yml" {
			continue
		}
		data, err := os.ReadFile(filepath.Join(dir, e.Name()))
		if err != nil {
			return nil, fmt.Errorf("读取 %s: %w", e.Name(), err)
		}
		c, err := Parse(data)
		if err != nil {
			return nil, fmt.Errorf("%s: %w", e.Name(), err)
		}
		t.caps[c.Name] = c
	}
	return t, nil
}
