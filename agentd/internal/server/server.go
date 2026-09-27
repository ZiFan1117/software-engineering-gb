// Package server 行分隔 JSON-RPC 服务：agent 侧与 agentd 之间的 varlink 语义
// （结构化请求/结构化应答），传输用 TCP 或 unix socket。过渡协议：语义等价于
// varlink，后续可平移（02 篇第 3 节）。
package server

import (
	"bufio"
	"encoding/json"
	"io"
	"net"

	"github.com/ZiFan1117/agent-native-os/agentd/internal/provider"
)

// Handler 由 router.Router 满足。
type Handler interface {
	Handle(req provider.Request) provider.Result
}

// Serve 阻塞接受连接；每个连接串行处理（一行请求、一行应答）。
func Serve(ln net.Listener, h Handler) error {
	for {
		conn, err := ln.Accept()
		if err != nil {
			return err
		}
		go handleConn(conn, h)
	}
}

func handleConn(conn net.Conn, h Handler) {
	defer conn.Close()
	r := bufio.NewReader(conn)
	dec := json.NewDecoder(r)
	enc := json.NewEncoder(conn)
	for {
		var req provider.Request
		if err := dec.Decode(&req); err != nil {
			if err != io.EOF {
				writeErr(enc, "ext.agent.Protocol", err.Error())
			}
			return
		}
		res := h.Handle(req)
		if err := enc.Encode(res); err != nil {
			return
		}
	}
}

func writeErr(enc *json.Encoder, code, msg string) {
	_ = enc.Encode(provider.Result{OK: false, ErrCode: code, ErrMsg: msg})
}

// Call 是客户端助手：发一条请求、收一条应答。
func Call(conn net.Conn, req provider.Request) (provider.Result, error) {
	enc := json.NewEncoder(conn)
	dec := json.NewDecoder(conn)
	if err := enc.Encode(req); err != nil {
		return provider.Result{}, err
	}
	var res provider.Result
	if err := dec.Decode(&res); err != nil {
		return provider.Result{}, err
	}
	return res, nil
}
