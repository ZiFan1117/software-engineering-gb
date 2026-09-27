#!/usr/bin/env bash
# check.sh -- the single entry point for this project.
# Agents run only this; humans run only this. No arguments needed.
#
#   ./check.sh
#
# Exit code: 0 = all good, non-zero = something failed.

set -uo pipefail
cd "$(dirname "$0")"

fail=0
hdr() { printf '\n== %s ==\n' "$1"; }

# 关键：try 不能在子 shell 里跑，否则 fail=1 会丢。
# 用子 shell 执行命令本身，但 rc 与 fail 都在当前 shell 里结算。
try() { "$@"; local rc=$?; if [ $rc -ne 0 ]; then printf '  FAILED: %s\n' "$*"; fail=1; fi; return $rc; }
in_d() { local d="$1"; shift; local rc=0; ( cd "$d" && "$@" ) || rc=$?; if [ $rc -ne 0 ]; then printf '  FAILED: (in %s) %s\n' "$d" "$*"; fail=1; fi; return $rc; }

hdr "environment"
printf '  kernel : %s\n' "$(uname -r)"
printf '  go     : %s\n' "$(go version 2>/dev/null | awk '{print $3}')"
printf '  goproxy: %s\n' "$(go env GOPROXY 2>/dev/null)"
printf '  python : %s\n' "$(python -V 2>&1)"
printf '  cwd    : %s\n' "$PWD"
printf '  files  : %s\n' "$(find . -type f -not -path './.git/*' | wc -l)"

if [ -d agentd ]; then
  hdr "agentd: build"
  in_d agentd go build ./...

  hdr "agentd: vet"
  ( cd agentd && go vet ./... 2>&1 | head -20 )

  hdr "agentd: test"
  in_d agentd go test ./...
else
  hdr "agentd"
  echo "  agentd/ not found -- skipped"
fi

hdr "result"
if [ "$fail" -eq 0 ]; then
  echo "  CHECK_OK"
  exit 0
else
  echo "  CHECK_FAIL"
  exit 1
fi
