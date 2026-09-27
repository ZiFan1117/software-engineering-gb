package capd

import (
	"os"
	"path/filepath"
	"testing"
)

const sample = `
capability: package.install
provider: package
verbs: [install]
risk: high
audit: always
snapshot: before-each
require: confirm
`

func TestParseValid(t *testing.T) {
	c, err := Parse([]byte(sample))
	if err != nil {
		t.Fatalf("合法声明解析失败: %v", err)
	}
	if c.Name != "package.install" || c.Provider != "package" {
		t.Fatalf("字段错: %+v", c)
	}
	if c.Risk != RiskHigh || c.Snapshot != SnapshotBeforeEach || c.Require != RequireConfirm {
		t.Fatalf("策略错: %+v", c)
	}
	if !c.Allows("install") || c.Allows("remove") {
		t.Fatalf("动词判定错")
	}
}

func TestParseInvalidRisk(t *testing.T) {
	if _, err := Parse([]byte("capability: x\nprovider: p\nverbs: [a]\nrisk: godlike\n")); err == nil {
		t.Fatal("非法 risk 应当报错")
	}
	if _, err := Parse([]byte("provider: p\nverbs: [a]\n")); err == nil {
		t.Fatal("缺 capability 应当报错")
	}
}

func TestLoadDir(t *testing.T) {
	dir := t.TempDir()
	write := func(name, content string) {
		if err := os.WriteFile(filepath.Join(dir, name), []byte(content), 0o600); err != nil {
			t.Fatal(err)
		}
	}
	write("backlight.yaml", "capability: brightness.set\nprovider: backlight\nverbs: [get, set]\nrisk: low\n")
	write("package.yml", sample)
	write("notes.txt", "不是声明，跳过")

	tbl, err := LoadDir(dir)
	if err != nil {
		t.Fatalf("LoadDir: %v", err)
	}
	if _, ok := tbl.Lookup("brightness.set"); !ok {
		t.Fatal("缺 brightness.set")
	}
	if _, ok := tbl.Lookup("package.install"); !ok {
		t.Fatal("缺 package.install（.yml 也应加载）")
	}
	if n := len(tbl.Names()); n != 2 {
		t.Fatalf("应加载 2 项，实得 %d", n)
	}
}
