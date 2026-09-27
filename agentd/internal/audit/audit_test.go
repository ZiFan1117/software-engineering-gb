package audit

import (
	"io"
	"net"
	"strings"
	"testing"
	"time"
)

func TestFileRecordAndRead(t *testing.T) {
	path := t.TempDir() + "/audit.jsonl"
	f, err := NewFile(path)
	if err != nil {
		t.Fatal(err)
	}
	if err := f.Record(map[string]string{FIntent: "package.install", FOutcome: "ok"}); err != nil {
		t.Fatalf("Record: %v", err)
	}
	if err := f.Record(map[string]string{FIntent: "brightness.set", FOutcome: "failed"}); err != nil {
		t.Fatalf("Record2: %v", err)
	}
	lines, err := f.ReadLines()
	if err != nil {
		t.Fatal(err)
	}
	if len(lines) != 2 {
		t.Fatalf("应 2 条记录，实得 %d", len(lines))
	}
	if lines[0][FIntent] != "package.install" {
		t.Fatalf("第一条错: %v", lines[0])
	}
	if lines[0][Timestamp] == "" {
		t.Fatal("应自动补时间戳")
	}
}

func TestJournalWireFormat(t *testing.T) {
	var got []byte
	j := NewJournal()
	j.SocketPath = "stub"
	j.dial = func(network, addr string) (net.Conn, error) {
		return &captureConn{dst: &got}, nil
	}
	err := j.Record(map[string]string{
		FIntent:  "package.install",
		FOutcome: "ok",
		FParams:  `{"pkg":"ripgrep"}`, // JSON 无裸换行，线上一行一条
	})
	if err != nil {
		t.Fatalf("Record: %v", err)
	}
	for _, want := range []string{"INTENT=package.install", "OUTCOME=ok", `PARAMS={"pkg":"ripgrep"}`} {
		if !strings.Contains(string(got), want) {
			t.Fatalf("线上缺 %q，实得: %q", want, got)
		}
	}
}

// captureConn 只实现 Write 的最小 net.Conn，把载荷原样捕获。
type captureConn struct{ dst *[]byte }

func (c *captureConn) Write(b []byte) (int, error) { *c.dst = append(*c.dst, b...); return len(b), nil }
func (c *captureConn) Read(_ []byte) (int, error)  { return 0, io.EOF }
func (c *captureConn) Close() error                { return nil }
func (c *captureConn) LocalAddr() net.Addr         { return nil }
func (c *captureConn) RemoteAddr() net.Addr        { return nil }
func (c *captureConn) SetDeadline(_ time.Time) error {
	return nil
}
func (c *captureConn) SetReadDeadline(_ time.Time) error  { return nil }
func (c *captureConn) SetWriteDeadline(_ time.Time) error { return nil }

func TestSanitizeValue(t *testing.T) {
	if got := sanitizeValue("a\nb\rc"); got != "a b c" {
		t.Fatalf("换行应替换为空格: %q", got)
	}
}
