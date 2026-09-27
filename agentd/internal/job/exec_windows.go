//go:build windows

package job

import "os/exec"

func execCmd(cmd string, args []string) *exec.Cmd { return exec.Command(cmd, args...) }

type execWaiter struct{ p *exec.Cmd }

func (w *execWaiter) Wait() int {
	_ = w.p.Wait()
	return w.p.ProcessState.ExitCode()
}
