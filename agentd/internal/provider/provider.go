// Package provider 定义 provider 契约与内置实现。
// provider 是"驱动"：把一项系统能力以结构化动词暴露给路由器，
// 拿结构化数据回来——没有文本猜测（02 篇第 3 节）。
package provider

import "fmt"

// Request 是 agent 发出的一个结构化调用。
type Request struct {
	Capability string         `json:"capability"`
	Verb       string         `json:"verb"`
	Args       map[string]any `json:"args,omitempty"`
}

// Result 是结构化返回：成功带 Data，失败带类型化错误码。
type Result struct {
	OK      bool           `json:"ok"`
	Data    map[string]any `json:"data,omitempty"`
	ErrCode string         `json:"err_code,omitempty"`
	ErrMsg  string         `json:"err_msg,omitempty"`
}

// Error 是类型化错误：错误码可被 LLM 直接分支，不用从散文里猜。
type Error struct {
	Code    string
	Message string
}

func (e *Error) Error() string { return fmt.Sprintf("%s: %s", e.Code, e.Message) }

func Errf(code, format string, a ...any) *Error {
	return &Error{Code: code, Message: fmt.Sprintf(format, a...)}
}

// Provider 是能力提供者契约。
type Provider interface {
	// Name 返回 provider 名，与 cap.d 的 provider 字段对应。
	Name() string
	// Capabilities 列出它提供的能力名（用于启动期自检与文档生成）。
	Capabilities() []string
	// Call 执行一个动词。实现必须：无歧义入参、结构化返回、失败返回 *Error。
	Call(verb string, args map[string]any) (map[string]any, error)
}
