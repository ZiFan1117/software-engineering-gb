package provider

import (
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

// Brightness 背光 provider（02 篇第 3 节示例）。
// 直接读写 sysfs，绕过 brightnessctl 的文本输出；SysfsRoot 可注入以便测试。
type Brightness struct {
	// SysfsRoot 默认 "/sys"；测试注入假根。
	SysfsRoot string
	// Device 背光设备名，默认自动探测 /sys/class/backlight 下第一项。
	Device string
}

func NewBrightness() *Brightness { return &Brightness{SysfsRoot: "/sys"} }

func (b *Brightness) Name() string { return "backlight" }

func (b *Brightness) Capabilities() []string { return []string{"brightness.set", "brightness.get"} }

func (b *Brightness) backlightDir() (string, error) {
	root := filepath.Join(b.sysfs(), "class", "backlight")
	if b.Device != "" {
		dir := filepath.Join(root, b.Device)
		if fi, err := os.Stat(dir); err == nil && fi.IsDir() {
			return dir, nil
		}
		return "", Errf("ext.agent.Brightness.Unsupported", "背光设备 %q 不存在", b.Device)
	}
	entries, err := os.ReadDir(root)
	if err != nil {
		return "", Errf("ext.agent.Brightness.Unsupported", "此平台没有背光子系统: %v", err)
	}
	if len(entries) == 0 {
		return "", Errf("ext.agent.Brightness.Unsupported", "未找到背光设备")
	}
	return filepath.Join(root, entries[0].Name()), nil
}

func (b *Brightness) sysfs() string {
	if b.SysfsRoot == "" {
		return "/sys"
	}
	return b.SysfsRoot
}

// Call 支持动词：get / set。
func (b *Brightness) Call(verb string, args map[string]any) (map[string]any, error) {
	dir, err := b.backlightDir()
	if err != nil {
		return nil, err
	}
	switch verb {
	case "get":
		cur, err := readInt(filepath.Join(dir, "brightness"))
		if err != nil {
			return nil, Errf("ext.agent.Brightness.IO", "读 brightness: %v", err)
		}
		max, err := readInt(filepath.Join(dir, "max_brightness"))
		if err != nil {
			return nil, Errf("ext.agent.Brightness.IO", "读 max_brightness: %v", err)
		}
		return map[string]any{"level": cur, "max": max}, nil
	case "set":
		level, ok := toInt(args["level"])
		if !ok {
			return nil, Errf("ext.agent.Brightness.BadArgs", "缺少整数参数 level")
		}
		max, err := readInt(filepath.Join(dir, "max_brightness"))
		if err != nil {
			return nil, Errf("ext.agent.Brightness.IO", "读 max_brightness: %v", err)
		}
		if level < 0 || level > max {
			return nil, Errf("ext.agent.Brightness.OutOfRange", "level=%d 超出 [0,%d]", level, max)
		}
		if err := os.WriteFile(filepath.Join(dir, "brightness"), []byte(strconv.Itoa(level)), 0o644); err != nil {
			return nil, Errf("ext.agent.Brightness.IO", "写 brightness: %v", err)
		}
		return map[string]any{"newlevel": level}, nil
	default:
		return nil, Errf("ext.agent.Brightness.Unsupported", "未知动词 %q", verb)
	}
}

func readInt(path string) (int, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}
	return strconv.Atoi(strings.TrimSpace(string(data)))
}

func toInt(v any) (int, bool) {
	switch x := v.(type) {
	case float64:
		return int(x), true
	case int:
		return x, true
	case int64:
		return int(x), true
	case string:
		n, err := strconv.Atoi(x)
		if err != nil {
			return 0, false
		}
		return n, true
	default:
		return 0, false
	}
}
