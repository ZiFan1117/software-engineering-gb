// Package audit 结构化审计（05 篇第 7.5 小时）。
// 每个 intent / outcome 写一条带字段的记录：优先投递 systemd-journald
// （unixgram 套接字协议，纯 Go 实现），不可用时回退到 JSON-lines 文件。
// 审计在 agent 之外——agent 自己删不掉。
package audit

import (
	"bufio"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"
)

// 审计字段约定（对应 02 篇第 5 节）。
const (
	FMessage    = "MESSAGE"
	FIntent     = "INTENT"
	FCapability = "CAPABILITY"
	FVerb       = "VERB"
	FParams     = "PARAMS"
	FSnapshot   = "SNAPSHOT"
	FGrantedBy  = "GRANTED_BY"
	FOutcome    = "OUTCOME"
	FDurationMS = "DURATION_MS"
	FJobID      = "JOB_ID"
	FSyscall    = "_SYSTEMD_UNIT" // journald 惯例：来源单元
)

// JournalSocket 是 journald 的文本协议接收套接字。
const JournalSocket = "/run/systemd/journal/socket"

// Auditor 接收一条结构化审计记录。
type Auditor interface {
	Record(fields map[string]string) error
}

// Multi 同时投递多个审计器（主路径失败不阻塞业务）。
type Multi []Auditor

func (m Multi) Record(fields map[string]string) error {
	var firstErr error
	for _, a := range m {
		if err := a.Record(fields); err != nil && firstErr == nil {
			firstErr = err
		}
	}
	return firstErr
}

// Journal 把字段以 journald 文本协议投递到本机 journald。
// 字段值中的换行按协议转义，保证一条记录就是一条日志。
type Journal struct {
	SocketPath string
	dial       func(network, addr string) (net.Conn, error)
}

func NewJournal() *Journal { return &Journal{SocketPath: JournalSocket, dial: net.Dial} }

func (j *Journal) Record(fields map[string]string) error {
	if len(fields) == 0 {
		return nil
	}
	buf := make([]byte, 0, 512)
	for k, v := range fields {
		if k == "" || v == "" {
			continue
		}
		buf = append(buf, k...)
		buf = append(buf, '=')
		buf = append(buf, sanitizeValue(v)...)
		buf = append(buf, '\n')
	}
	dial := j.dial
	if dial == nil {
		dial = net.Dial
	}
	conn, err := dial("unixgram", j.SocketPath)
	if err != nil {
		return fmt.Errorf("journald 不可达: %w", err)
	}
	defer conn.Close()
	if _, err := conn.Write(buf); err != nil {
		return fmt.Errorf("journald 写入失败: %w", err)
	}
	return nil
}

// sanitizeValue 保证一条记录占一行：值内的换行替换为空格。
// （journald 文本协议按换行分帧，值内裸换行会撕裂记录。）
func sanitizeValue(s string) string {
	return strings.Map(func(r rune) rune {
		if r == '\n' || r == '\r' {
			return ' '
		}
		return r
	}, s)
}

// File 把记录以 JSON-lines 追加到文件（每行一条），供无 systemd 环境（含开发机）回退。
type File struct {
	Path string
	mu   sync.Mutex
}

func NewFile(path string) (*File, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0o750); err != nil {
		return nil, err
	}
	return &File{Path: path}, nil
}

func (f *File) Record(fields map[string]string) error {
	if fields[Timestamp] == "" {
		fields = withTimestamp(fields)
	}
	line, err := json.Marshal(fields)
	if err != nil {
		return err
	}
	f.mu.Lock()
	defer f.mu.Unlock()
	w, err := os.OpenFile(f.Path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o640)
	if err != nil {
		return err
	}
	defer w.Close()
	if _, err := w.Write(append(line, '\n')); err != nil {
		return err
	}
	return w.Sync()
}

// ReadLines 读回全部审计记录（测试与 agentctl 查询用）。
func (f *File) ReadLines() ([]map[string]string, error) {
	fp, err := os.Open(f.Path)
	if err != nil {
		return nil, err
	}
	defer fp.Close()
	var out []map[string]string
	sc := bufio.NewScanner(fp)
	for sc.Scan() {
		var m map[string]string
		if json.Unmarshal(sc.Bytes(), &m) == nil && len(m) > 0 {
			out = append(out, m)
		}
	}
	return out, sc.Err()
}

const Timestamp = "_TIMESTAMP"

func withTimestamp(fields map[string]string) map[string]string {
	out := make(map[string]string, len(fields)+1)
	out[Timestamp] = time.Now().UTC().Format(time.RFC3339Nano)
	for k, v := range fields {
		out[k] = v
	}
	return out
}
