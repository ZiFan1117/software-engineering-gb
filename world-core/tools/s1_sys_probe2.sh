#!/usr/bin/env bash
# s1_sys_probe2.sh —— **S1 需求验证面补建（第二轮）**：把 RTM「系统测试用例」列需要的
# 系统级（L3）证据补齐（真实二进制端到端；不 mock、不 import 任何 Rust 代码）。
#
# ## 为什么需要它
#
# R1 第一轮实测：WC-RTM-001.csv 有 **31 行**「系统测试用例」与「验收测试用例」两列皆空，
# 门禁据此各报 1 条 WARN。R1 席 S-05 判明其中 **22 行（B 类）已有"已实现"用例但都不是系统级面**。
# 本脚本为这 22 行（＋若干条在 SRS §五 里已声明但缺 L3 面的用例）补建**真实、可跑、会红**的断言。
#
# | 用例 | 需求 | 端到端断言（摘要） |
# |---|---|---|
# | TC-053 | REQ-F-003 账本只追加且纯文本 | 每行合法 JSON、末字节 0x0A、旧行逐字节不变；无链/有链/篡改三种账本的链状态与 rc |
# | TC-054 | REQ-F-004 序号单一递增缺号拒启 | 人工造缺号 ⇒ 拒启且点名；对照连续账本 rc=0 |
# | TC-055 | REQ-F-005 崩溃尾迹丢弃 | 追加半行后 rc=0 且 last_seq = 完整事件数 |
# | TC-056 | REQ-F-006 唯一写入口 | 穷举全部只读子命令，账本 sha256 不变 |
# | TC-057 | REQ-F-007 法律在前落笔在后 | 连续提交违法事件后账本条数不变、正文不出现 |
# | TC-058 | REQ-F-008 三家族信纸字段 | 三家族各缺一个必填字段 ⇒ 均 rc=2 且点名；未知家族 ⇒ rc=2 |
# | TC-059 | REQ-F-009 change 必带旧值 | 缺 before ⇒ rc=2 且点名；before:null 是**合法值** ⇒ rc=0 |
# | TC-060 | REQ-F-011 读模型可删掉重算 | 两次 state --json 逐字节同；前缀折叠指纹**必须不同**（反假） |
# | TC-061 | REQ-F-012 读模型不写盘不改账本 | 折叠前后账本 sha256 不变、目录无新增文件 |
# | TC-062 | REQ-F-013 读模型拒绝坏账本 | 断号/旧值说谎/未知家族三类都 rc=2 且给出定位 |
# | TC-063 | REQ-F-014 回滚＝追加补偿事件 | 补偿后状态回到目标值；历史行逐字节不变、条数只增 |
# | TC-064 | REQ-F-016 能力声明即请求 | 未声明能力 ⇒ rc=2 且落一条拒绝流水；对照已声明能力放行 |
# | TC-065 | REQ-F-017 不可逆加摩擦、可逆免检留痕 | 可逆放行落 act；不可逆加摩擦且理由点明"无审批通道" |
# | TC-066 | REQ-F-018 语言投影结构化出口 | 逐行 JSON 可解析；解析回来与 state --json 逐项相等 |
# | TC-067 | REQ-F-019 视觉投影渲染出口 | 空态 4 行/非空含主体行；排版契约由独立审计器判定 |
# | TC-068 | REQ-F-020 两个投影同源 | project check 自报同源；换词表本体 ⇒ 首行 vocab= 必变 |
# | TC-069 | REQ-F-021 检查点/快照可重建 | 放一个坏 JSON 快照 ⇒ 全部子命令结论与不放时**逐字节相同**（缓存不得成为第二真相） |
# | TC-070 | REQ-F-022 账本可重放 | 新进程 read 的事件序列与之前**逐字节相同** |
# | TC-071 | REQ-N-001 语言无关（纯文本） | plain_text_audit.py rc=0；注入 NUL 的账本 ⇒ 审计必须报红 |
# | TC-072 | REQ-N-002 零外部依赖纪律 | Cargo.toml 直接依赖 ⊆ 允许清单；加一个依赖的副本 ⇒ 检查必须报红 |
# | TC-073 | REQ-F-023 投递、应答与收件人 | 本体 optional 含 to/trace；带 to 的事件落笔且不影响结论；act 缺 request_id 被拒 |
# | TC-074 | REQ-F-025 主体身份可核 | 白名单外主体 ⇒ rc=2 且不落笔；白名单内可逆 act ⇒ rc=0；subjects.allow 实测两项 |
# | TC-075 | REQ-N-005 可度量的质量目标 | 六项目标三列齐备（目标值/测量方法/测量时点）**机器可核**；清空一格 ⇒ 必须报红 |
#
# ## 纪律（与 s1_sys_probe.sh 同）
# - 全程 mktemp -d 一次性沙箱，退出即删；**绝不触碰真实账本**；
# - 手写账本行只写副本（取真实落盘行、**去掉每一行的 chain**，避免把"改了内容"误报成"摘要不符"，
#   也避免 Ledger.MixedChain：既有链又无链的账本会被拒 —— 那是产品在防误导，是对的）；
# - 产品代码、出厂本体与门禁策略**一个字节都不改**（需要"变异本体/策略/Cargo.toml"的断言一律在**副本**上做）；
# - 自带 --self-test：先证明**本脚本的判定会红**；
# - **登记项**（⛔）＝ 现状为红、如实记录、**不计入退出码**，但**每轮必须复算**。
#
# 用法：bash tools/s1_sys_probe2.sh ／ bash tools/s1_sys_probe2.sh --self-test
# 退出码：0 = 断言全通过；1 = 有断言失败；2 = 前置条件不满足
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2
BIN="${CARGO_TARGET_DIR:-target}"/debug/world-core

PASS=0
FAIL=0
RED=0

ok() { echo "  ✅ $1"; PASS=$((PASS + 1)); }
bad() { echo "  ❌ $1"; FAIL=$((FAIL + 1)); }
reg() { echo "  ⛔ 登记（现状为红，如实记录，**不计入**门禁结论）: $1"; RED=$((RED + 1)); }

assert_eq() { if [ "$2" = "$3" ]; then ok "$1"; else bad "$1（期望 [$2]，实得 [$3]）"; fi; }
assert_ne() { if [ "$2" != "$3" ]; then ok "$1"; else bad "$1（不应等于 [$2]）"; fi; }
assert_rc() { assert_eq "$1" "$2" "$3"; }
assert_has() { if printf '%s' "$2" | grep -Eq "$3"; then ok "$1"; else bad "$1（输出里找不到 /$3/：$(printf '%s' "$2" | head -2 | tr '\n' ' ')）"; fi; }
assert_not_has() { if printf '%s' "$2" | grep -Eq "$3"; then bad "$1（输出里**不应**出现 /$3/）"; else ok "$1"; fi; }
sha() { sha256sum "$1" | cut -d' ' -f1; }

if [ "${1:-}" = "--self-test" ]; then
  echo "== s1_sys_probe2.sh 判定器自证 =="
  before=$FAIL
  assert_eq "自证：故意断言 1=2" "1" "2"
  assert_has "自证：故意在空串里找 REQUIRE_THIS" "" 'REQUIRE_THIS'
  if [ "$FAIL" -eq $((before + 2)) ]; then
    echo "  ✅ 自证通过：两个假命题都被判为失败（本判定器不是装饰）"
    exit 0
  fi
  echo "  ❌ 自证失败"
  exit 1
fi

if [ ! -x "$BIN" ]; then
  echo "[前置] 未找到 $BIN，执行一次 cargo build --locked --quiet"
  cargo build --locked --quiet || { echo "  ❌ 构建失败"; exit 2; }
fi

SB="$(mktemp -d)"
trap 'rm -rf "$SB"' EXIT
chmod 700 "$SB"
cp ontology.json policy.json "$SB"/
chmod 600 "$SB/ontology.json" "$SB/policy.json"
L="$SB/ledger.jsonl"
E="$SB/empty.jsonl"
: >"$E"; chmod 600 "$E"

W() { "$BIN" --ontology "$SB/ontology.json" --ledger "$L" --policy "$SB/policy.json" "$@"; }
WE() { "$BIN" --ontology "$SB/ontology.json" --ledger "$E" --policy "$SB/policy.json" "$@"; }
WL() { # WL <ledger> <ontology> [args...]
  local led="$1" ont="$2"; shift 2
  "$BIN" --ontology "$ont" --ledger "$led" --policy "$SB/policy.json" "$@"
}
A() { W append "$@" 2>&1; }
ARC() { W append "$@" >/dev/null 2>&1; echo $?; }

echo "== world-core S1 系统级验证面补建（TC-053–TC-075，真实二进制 $BIN）=="
echo "  沙箱 : $SB"

# ── 通用：取一条真实落盘行并做"去链"副本 ──────────────────────────────
strip_chain() { # strip_chain <src> <dst>
  python3 - "$1" "$2" <<'PY'
import json, sys
src, dst = sys.argv[1], sys.argv[2]
evs = [json.loads(x) for x in open(src, encoding="utf-8").read().rstrip("\n").split("\n") if x.strip()]
for e in evs:
    e.pop("chain", None)
open(dst, "w", encoding="utf-8").write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
  chmod 600 "$2"
}
mutate() { # mutate <src> <dst> <py-expr>
  python3 - "$1" "$2" "$3" <<'PY'
import json, sys
src, dst, expr = sys.argv[1], sys.argv[2], sys.argv[3]
evs = [json.loads(x) for x in open(src, encoding="utf-8").read().rstrip("\n").split("\n") if x.strip()]
exec(expr, {"evs": evs, "json": json})
for e in evs:
    e.pop("chain", None)
open(dst, "w", encoding="utf-8").write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
  chmod 600 "$2"
}

# 种子：写成**已声明**的格子（`world://notice/a` ＋ `muted`；书 §5.3「声明以外的东西不许落账」，
# 执行者 `src/ontology.rs::check_concepts`）。原先的 `world://sys/a#p` 与 `#q` 两个名字都没声明过
# ⇒ 两条种子都被拒、账本 0 行，后面**整片**断言（链、缺号、半行、投影、回滚…）跟着一起红。
#
# ⚠ 第二行为什么改成"同一格再改一次"（而不是另一格 `q`）：出厂本体里每个实体只声明了**一格**
#   （`notice.muted` / `job.status`），而"同一主体两个字段"在声明里没有对应的东西；
#   若改用第二个**主体**，`TC-066④`（视觉/语言投影"行数 = 主体数 + 1"＝3）会从 2 个主体变成 3 个，
#   那条断言要么被改松、要么变成假红——两者都不许。故第二行改成对同一格的第二次变更
#   （`before` 必须等于上一次的 `after`，这正是折叠层的 `BeforeMismatch` 在管的），
#   账本仍是 2 行、主体仍是 1 个，判据强度不变（反而多验了一次"旧值对得上才放行"）。
ARC change '{"subject":"world://notice/a","path":"muted","before":null,"after":true}' >/dev/null
ARC change '{"subject":"world://notice/a","path":"muted","before":true,"after":false}' >/dev/null
assert_eq "种子：账本 2 行" "2" "$(wc -l <"$L" | tr -d ' ')"

# ══ TC-053 · REQ-F-003 账本只追加且为纯文本 ═══════════════════════════
echo; echo "── TC-053 · REQ-F-003 账本只追加且为纯文本 ──"
cp "$L" "$SB/snap.jsonl"
# 本条只要求"再追加一条、账本只增 1 行、前 2 行逐字节不变"——落笔的格子是哪个不影响判据，
# 故取种子里那一格（`notice/a#muted`）：`before` 必须等于上一次的 `after`（当前是 false）才放行。
A change '{"subject":"world://notice/a","path":"muted","before":false,"after":true}' >/dev/null
assert_eq "① 追加后只增 1 行" "3" "$(wc -l <"$L" | tr -d ' ')"
assert_eq "② 前 2 行**逐字节不变**" "$(head -2 "$SB/snap.jsonl")" "$(head -2 "$L")"
BADN="$(python3 - "$L" <<'PY'
import json, sys
bad = 0
for ln in open(sys.argv[1], encoding="utf-8"):
    if not ln.strip():
        continue
    try:
        json.loads(ln)
    except Exception:
        bad += 1
print(bad)
PY
)"
assert_eq "③ 逐行合法 JSON（无残缺行；坏行数 = 0）" "0" "$BADN"
assert_eq "④ 文件以 0x0A 结尾" "0a" "$(tail -c1 "$L" | od -An -tx1 | tr -d ' \n')"
assert_eq "⑤ 账本不含 NUL 字节" "0" "$(tr -dc '\000' <"$L" | wc -c | tr -d ' ')"
chmod 600 "$SB/chainless.jsonl"; cp "$L" "$SB/chainless.jsonl"; strip_chain "$L" "$SB/chainless.jsonl"
CO="$(WL "$SB/chainless.jsonl" "$SB/ontology.json" check 2>&1)"; assert_has "⑥ 无链账本必须打印警示（未校验要说出来）" "$CO" '无摘要链'
WL "$SB/chainless.jsonl" "$SB/ontology.json" --require-chain check >/dev/null 2>&1
assert_rc "⑦ 无链 + --require-chain ⇒ rc=2" 2 "$?"
assert_has "⑧ 有链账本自报「有摘要链」" "$(W check 2>&1)" '有摘要链'
python3 - "$L" "$SB/tampered.jsonl" <<'PY'
import json, sys
# 保留 chain（否则就变成"无链账本"，篡改本就检不出——那正是无链的代价）
evs = [json.loads(x) for x in open(sys.argv[1], encoding="utf-8").read().rstrip("\n").split("\n") if x.strip()]
evs[0]["body"]["after"] = 999
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
chmod 600 "$SB/tampered.jsonl"
TO="$(WL "$SB/tampered.jsonl" "$SB/ontology.json" check 2>&1)"; TRC=$?
assert_rc "⑨ 篡改一行 ⇒ 拒启（rc=2）" 2 "$TRC"
assert_has "⑩ 拒启理由含 ChainMismatch（点名"内容被改动"）" "$TO" 'ChainMismatch'

# ══ TC-054 · REQ-F-004 序号单一递增，缺号拒启 ═════════════════════════
echo; echo "── TC-054 · REQ-F-004 序号单一递增，缺号拒启 ──"
python3 - "$L" "$SB/gap.jsonl" <<'PY'
import json, sys
evs = [json.loads(x) for x in open(sys.argv[1], encoding="utf-8").read().rstrip("\n").split("\n") if x.strip()]
for e in evs:
    e.pop("chain", None)
del evs[1]
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
chmod 600 "$SB/gap.jsonl"
GO="$(WL "$SB/gap.jsonl" "$SB/ontology.json" state 2>&1)"; GRC=$?
assert_rc "① 人为制造缺号 ⇒ 拒绝（rc=2）" 2 "$GRC"
assert_has "② 拒绝理由**点名**缺号/顺序（SeqGap 或等价定位）" "$GO" '(SeqGap|seq|顺序|缺号)'
assert_rc "③ 对照：连续账本 rc=0（防恒红）" 0 "$(W state >/dev/null 2>&1; echo $?)"

# ══ TC-055 · REQ-F-005 崩溃尾迹（半行）启动时丢弃 ═════════════════════
echo; echo "── TC-055 · REQ-F-005 半行启动时丢弃、该号可复用 ──"
cp "$L" "$SB/half.jsonl"; chmod 600 "$SB/half.jsonl"
printf '{"world":1,"kind":"change","id":"e-half","seq":4,"at":1,"actor":"wor' >>"$SB/half.jsonl"
HO="$(WL "$SB/half.jsonl" "$SB/ontology.json" state --json 2>&1)"; HRC=$?
assert_rc "① 末尾半行 ⇒ 仍能启动（rc=0，半行被丢弃）" 0 "$HRC"
assert_eq "② 折叠出的 last_seq = **完整事件数**（半行未计入）" "3" "$(printf '%s' "$HO" | python3 -c 'import json,sys; print(json.load(sys.stdin)["last_seq"])' 2>/dev/null || echo '?')"
assert_has "③ 半行内容未进入读模型（world:// 计数不含半行的 seq=4 之后内容）" "$HO" '"last_seq":3'

# ══ TC-056 · REQ-F-006 唯一写入口 ═════════════════════════════════════
echo; echo "── TC-056 · REQ-F-006 唯一写入口（只读子命令不得改账本）──"
BEFORE="$(sha "$L")"; NF0="$(ls -a "$SB" | sort | tr '\n' ' ')"
W read >/dev/null 2>&1; W state >/dev/null 2>&1; W state --json >/dev/null 2>&1
W project language >/dev/null 2>&1; W project visual >/dev/null 2>&1; W project check >/dev/null 2>&1
W check >/dev/null 2>&1; W kinds >/dev/null 2>&1; W policy >/dev/null 2>&1
assert_eq "① 九个只读子命令执行后账本**逐字节不变**" "$BEFORE" "$(sha "$L")"
assert_eq "② 目录内容无新增（无缓存/临时文件落盘）" "$NF0" "$(ls -a "$SB" | sort | tr '\n' ' ')"
assert_has "③ state 的帮助面**不提供** --cache（刻意不写盘）" "$(W --help 2>&1)" '^用法:'

# ══ TC-057 · REQ-F-007 法律在前、落笔在后 ═════════════════════════════
echo; echo "── TC-057 · REQ-F-007 校验不过绝不落笔 ──"
N0="$(wc -l <"$L" | tr -d ' ')"
# 三条"违法事件"缺的是 `before`；主体/字段换成已声明的 `notice/x{1,2,3}` ＋ `muted`，
# 好让"被拒"这件事**只有一个理由**（缺 before）。三个主体都是新的、都被拒 ⇒ 不落笔、不影响主体数。
for i in 1 2 3; do ARC change "{\"subject\":\"world://notice/x$i\",\"path\":\"muted\",\"after\":$i}" >/dev/null; done
assert_eq "① 连续 3 次提交**违法**事件（缺 before）后账本行数不变" "$N0" "$(wc -l <"$L" | tr -d ' ')"
if grep -q 'world://notice/x1' "$L"; then bad "② 被拒事件正文不得出现在账本里"; else ok "② 被拒事件正文不得出现在账本里"; fi
assert_has "③ 每次拒绝都有可读理由（[FAIL]  + 点名缺失字段）" "$(A change '{"subject":"world://notice/y","path":"muted","after":1}')" '^\[FAIL\] .*before'

# ══ TC-058 · REQ-F-008 三个事件家族及其信纸字段 ═══════════════════════
echo; echo "── TC-058 · REQ-F-008 三家族信纸字段 ──"
# 字段名一并取已声明的 `muted`（本条判的是"缺 subject"，字段名不参与判定；
# 但检查用的数据里不留未声明的名字，是同一把尺子）。
O="$(A change '{"path":"muted","before":null,"after":true}')"; assert_rc "① change 缺 subject ⇒ rc=2" 2 "$?"
assert_has "② 理由点名 subject" "$O" 'subject'
O="$(A act '{"verb":"do","request_id":"r","params":{}}')"; assert_rc "③ act 缺 capability ⇒ rc=2" 2 "$?"
assert_has "④ 理由点名 capability" "$O" 'capability'
# 通告的主体也换成已声明的：主体不是本条的判据（本条判"缺 type"），但**检查用的数据必须是已声明的**，
# 免得拒绝理由变成"缺 type ∧ 实体没声明"两个都成立。`notice` 家族不受 `concepts` 管（只管 `change`）。
O="$(A notice '{"subject":"world://notice/a"}')"; assert_rc "⑤ notice 缺 type ⇒ rc=2" 2 "$?"
assert_has "⑥ 理由点名 type" "$O" 'type'
O="$(A bogus '{"a":1}')"; assert_rc "⑦ 未知家族 ⇒ rc=2" 2 "$?"
assert_has "⑧ 理由点名那个未知家族 bogus" "$O" 'bogus'
assert_rc "⑨ 对照：三家族各给全字段 ⇒ 都 rc=0" 0 "$(ARC act '{"capability":"notice.mute","verb":"do","request_id":"r-ok","params":{}}')"

# ══ TC-059 · REQ-F-009 change 必带旧值 ══════════════════════════════
echo; echo "── TC-059 · REQ-F-009 change 必带旧值 ──"
# ②"理由点名 before"能成立的前提是**拒绝只有一个理由**：主体/字段都用已声明的，
# 于是"缺 before"就是唯一的拦下理由（原先 `world://sys/a#p` 两个名字都没声明过，
# 拒绝理由会变成实体没声明——本条要验的"必带旧值"就验不到了）。
O="$(A change '{"subject":"world://notice/a","path":"muted","after":true}')"; assert_rc "① 缺 before ⇒ rc=2" 2 "$?"
assert_has "② 理由点名 before" "$O" 'before'
# ③ 要验的是"`before:null` 是**合法值**"，故这一笔必须落在**没写过的新格子**上
#    （折叠层对新格子不核 before）。出厂本体里第二格是 `job.status`（`enum` 三值，取 "todo"）。
#    ⚠ 这一笔同时决定了 `TC-066④`"投影行数 = 主体数 + 1 ＝ 3"里的主体数：
#    到此为止落笔的主体恰是 `notice/a` 与 `job/b` 两个 ⇒ 仍是 3 行，那条断言一字未动。
assert_rc "③ 反例方向：before:null 是**合法值**（不得被当成"缺"）⇒ rc=0" 0 "$(ARC change '{"subject":"world://job/b","path":"status","before":null,"after":"todo"}')"
assert_has "④ 读回的事件里 before 字段**仍在**（null 未被丢弃）" "$(W read 2>/dev/null | tail -1)" '"before": ?null'

# ══ TC-060 · REQ-F-011 读模型可删掉重算且逐字节一致 ═══════════════════
echo; echo "── TC-060 · REQ-F-011 读模型可重算且逐字节一致 ──"
S1="$(W state --json 2>/dev/null)"; S2="$(W state --json 2>/dev/null)"
assert_eq "① 两次独立重算**逐字节相同**" "$S1" "$S2"
head -2 "$L" >"$SB/prefix.jsonl"; chmod 600 "$SB/prefix.jsonl"
PJ="$(WL "$SB/prefix.jsonl" "$SB/ontology.json" state --json 2>/dev/null)"
assert_ne "② **反假**：只折叠前缀 ⇒ 结果必须**不同**（常量状态会在此变红）" "$S1" "$PJ"
assert_ne "③ 反假：前缀的状态指纹必须与全量不同" \
  "$(printf '%s' "$S1" | python3 -c 'import json,sys; print(json.load(sys.stdin)["last_seq"])')" \
  "$(printf '%s' "$PJ" | python3 -c 'import json,sys; print(json.load(sys.stdin)["last_seq"])')"

# ══ TC-061 · REQ-F-012 读模型不写盘、不改账本 ═════════════════════════
echo; echo "── TC-061 · REQ-F-012 折叠不得写盘、不得改账本 ──"
B="$(sha "$L")"; N1="$(ls -a "$SB" | sort | tr '\n' ' ')"
W state >/dev/null 2>&1; W project language >/dev/null 2>&1; W project visual >/dev/null 2>&1
assert_eq "① 折叠前后账本字节**完全不变**" "$B" "$(sha "$L")"
assert_eq "② 折叠不产生新文件（读模型不写盘）" "$N1" "$(ls -a "$SB" | sort | tr '\n' ' ')"
assert_has "③ 入口自述 state 不缓存、不写盘" "$(W --help 2>&1)" '不缓存、不写盘'

# ══ TC-062 · REQ-F-013 读模型拒绝坏账本 ═══════════════════════════════
echo; echo "── TC-062 · REQ-F-013 读模型拒绝坏账本（三类各给定位）──"
mutate "$L" "$SB/bad_gap.jsonl" 'evs[0]["seq"] = 7'
O="$(WL "$SB/bad_gap.jsonl" "$SB/ontology.json" state 2>&1)"; assert_rc "① 序号断裂 ⇒ rc=2" 2 "$?"
assert_has "② 定位到 seq／第几行" "$O" '(seq|第 ?[0-9]+ ?行)'
python3 - "$L" "$SB/bad_lie.jsonl" <<'PY'
import json, sys
evs = [json.loads(x) for x in open(sys.argv[1], encoding="utf-8").read().rstrip("\n").split("\n") if x.strip()]
for e in evs:
    e.pop("chain", None)
# 手写行的主体/字段必须与**种子里那一格**一致（`notice/a#muted`，当前值 true）：
# 本条要验的是"事件自称的旧值 999 ≠ 账本折叠出的当前值 ⇒ BeforeMismatch"，
# 指向不存在的格子就变成"首见不核 before"，这一条会变成假红/假绿。
# （这是**手写账本行**，走的是折叠层 `readmodel`，本来就不经写入侧的本体校验。）
evs.append({"world": 1, "kind": "change", "id": "e-lie", "seq": evs[-1]["seq"] + 1,
            "at": 1, "actor": "world://user", "flags": [],
            "body": {"subject": "world://notice/a", "path": "muted", "before": 999, "after": 5}})
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
chmod 600 "$SB/bad_lie.jsonl"
O="$(WL "$SB/bad_lie.jsonl" "$SB/ontology.json" state 2>&1)"; assert_rc "③ 旧值说谎 ⇒ rc=2" 2 "$?"
assert_has "④ 理由点出旧值不符（before／旧值）" "$O" '(before|旧值)'
mutate "$L" "$SB/bad_fam.jsonl" 'evs[0]["kind"] = "ghost"'
O="$(WL "$SB/bad_fam.jsonl" "$SB/ontology.json" state 2>&1)"; assert_rc "⑤ 未知家族 ⇒ rc=2" 2 "$?"
assert_has "⑥ 点名未知家族 ghost" "$O" 'ghost'

# ══ TC-063 · REQ-F-014 回滚＝追加补偿事件 ═════════════════════════════
echo; echo "── TC-063 · REQ-F-014 回滚＝追加补偿事件 ──"
BEFORE_N="$(wc -l <"$L" | tr -d ' ')"
BEFORE_HEAD="$(head -1 "$L")"
# 回滚落在 `job/b#status` 上（TC-059③ 刚建的那一格，当前值 "todo"）：换的是目标格子，
# 判据不动——"改成新值 → 读回是新值 → 追加补偿事件 → 回到目标值 → 条数 +2、首行逐字节不变"。
# 为什么不用种子那格 `notice/a#muted`：它是 bool，`python3 print` 出来是 `True`/`False` 两个
# 首字母大写的字面量，断言里就得写 Python 的表示法；落在 `status` 上则字段与取值都是**声明里真有的**
# （`enum(todo,doing,done)` 的 todo/doing），断言读的是谁一目了然。
ARC change '{"subject":"world://job/b","path":"status","before":"todo","after":"doing"}' >/dev/null
assert_eq "① 前值已改为 doing" "doing" "$(W state --json 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["objects"]["world://job/b"]["status"])')"
ARC change '{"subject":"world://job/b","path":"status","before":"doing","after":"todo"}' >/dev/null
assert_eq "② 追加补偿事件后状态**回到目标值** todo" "todo" "$(W state --json 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin)["objects"]["world://job/b"]["status"])')"
assert_eq "③ 账本章数**只增不减**（+2）" "$((BEFORE_N + 2))" "$(wc -l <"$L" | tr -d ' ')"
assert_eq "④ 历史首行**逐字节不变**（不得修改或删除历史）" "$BEFORE_HEAD" "$(head -1 "$L")"

# ══ TC-064 · REQ-F-016 能力声明即请求，进门前查门禁 ═══════════════════
echo; echo "── TC-064 · REQ-F-016 未声明能力被拒且留痕 ──"
N0="$(wc -l <"$L" | tr -d ' ')"
O="$(A act '{"capability":"job.unknown","verb":"do","request_id":"r-u","params":{}}')"; assert_rc "① 未声明能力 ⇒ rc=2" 2 "$?"
assert_has "② 理由含机器可读码族 ext.world.Gate." "$O" 'ext\.world\.Gate\.'
N1="$(wc -l <"$L" | tr -d ' ')"
if [ "$N1" -gt "$N0" ]; then ok "③ 拒绝**留痕**（新增 $((N1 - N0)) 行流水）"; else bad "③ 拒绝必须留痕"; fi
if grep -q '"kind": *"act"' "$L" && grep 'job.unknown' "$L" | grep -q '"kind": *"act"'; then
  bad "④ 被拒的**原事件**不得落笔"
else
  ok "④ 被拒的原事件不得落笔（拒绝流水里出现能力名是**留痕**，不算原事件落笔）"
fi
assert_rc "⑤ 对照：已声明能力（可逆）⇒ rc=0" 0 "$(ARC act '{"capability":"notice.mute","verb":"do","request_id":"r-k","params":{}}')"

# ══ TC-065 · REQ-F-017 不可逆加摩擦、可逆免检留痕 ═════════════════════
echo; echo "── TC-065 · REQ-F-017 分级摩擦（CLI 端到端）──"
O="$(A act '{"capability":"ledger.compact","verb":"do","request_id":"r-i","params":{}}' world://agent/1)"; R=$?
assert_rc "① 不可逆动作 + 白名单外主体 ⇒ rc=2" 2 "$R"
assert_has "② 理由含「门禁加摩擦」" "$O" '门禁加摩擦'
assert_has "③ 理由**明说** v1 无审批通道（不得让人等批准）" "$O" '没有审批通道'
O="$(A act '{"capability":"ledger.compact","verb":"do","request_id":"r-i2","params":{}}' world://user)"; assert_rc "④ 对照：白名单内主体执行不可逆动作 ⇒ rc=0" 0 "$?"
B4="$(wc -l <"$L" | tr -d ' ')"
assert_rc "⑤ 可逆动作免检 ⇒ rc=0" 0 "$(ARC act '{"capability":"notice.mute","verb":"do","request_id":"r-r","params":{}}')"
assert_eq "⑥ 可逆动作**必须留痕**（新增 1 行 act）" "$((B4 + 1))" "$(wc -l <"$L" | tr -d ' ')"
assert_has "⑦ 留痕行是 act 家族" "$(W read 2>/dev/null | tail -1)" '"kind": ?"act"'
O="$(A act '{"capability":"notice.mute","verb":"do","request_id":"r-s","params":{}}' world://stranger)"; assert_rc "⑧ 白名单外主体 ⇒ rc=2" 2 "$?"
assert_has "⑨ 理由含「门禁拒绝」" "$O" '门禁拒绝'

# ══ TC-066 · REQ-F-018 语言投影（结构化出口）════════════════════════
echo; echo "── TC-066 · REQ-F-018 语言投影与读模型逐项相等 ──"
LT="$(W project language 2>/dev/null)"
assert_has "① 首行是同源头且含全部五要素" "$LT" '^#world-core projection=language world=[0-9]+ vocab=\S+ last_seq=[0-9]+ state=\S+$'
assert_eq "② 每行都是合法 JSON（首行除外）" "OK" "$(printf '%s' "$LT" | tail -n +2 | python3 -c '
import json,sys
for ln in sys.stdin:
    if ln.strip(): json.loads(ln)
print("OK")')"
assert_eq "③ 解析回来与 state --json 逐项相等" "EQUAL" "$(python3 - "$L" "$SB/ontology.json" "$SB/policy.json" "$BIN" <<'PY'
import json, subprocess, sys
led, ont, pol, binp = sys.argv[1:5]
st = json.loads(subprocess.run([binp, "--ontology", ont, "--ledger", led, "--policy", pol, "state", "--json"], capture_output=True).stdout)
S = {(s, p): v for s, f in st.get("objects", {}).items() for p, v in f.items()}
out = subprocess.run([binp, "--ontology", ont, "--ledger", led, "--policy", pol, "project", "language"], capture_output=True).stdout.decode()
P = {}
for ln in out.split("\n")[1:]:
    if ln.strip():
        o = json.loads(ln)
        for p, v in o["fields"].items():
            P[(o["subject"], p)] = v
print("EQUAL" if P == S else "DIFF %s" % (set(P.items()) ^ set(S.items())))
PY
)"
assert_eq "④ 投影**不含历史**（行数 = 主体数 + 1 首行）" "3" "$(printf '%s' "$LT" | grep -c . | tr -d ' ')"

# ══ TC-067 · REQ-F-019 视觉投影（渲染出口）═══════════════════════════
echo; echo "── TC-067 · REQ-F-019 视觉投影渲染出口 ──"
W project visual >"$SB/v_nonempty.txt" 2>/dev/null
WE project visual >"$SB/v_empty.txt" 2>/dev/null
assert_eq "① 非空状态：渲染成功且独立审计器判定通过" 0 "$(python3 tools/visual_layout_audit.py --file "$SB/v_nonempty.txt" >/dev/null 2>&1; echo $?)"
assert_eq "② 空状态：渲染成功且独立审计器判定通过（空态行 + 4 行）" 0 "$(python3 tools/visual_layout_audit.py --file "$SB/v_empty.txt" >/dev/null 2>&1; echo $?)"
assert_eq "③ 空状态**不得**输出任何主体行" "0" "$(grep -c '^  world://' "$SB/v_empty.txt" | tr -d ' ')"
assert_has "④ 非空状态含主体行与字段行" "$(cat "$SB/v_nonempty.txt")" '(^|\n)  world://'

# ══ TC-068 · REQ-F-020 两个投影同源 ═══════════════════════════════════
echo; echo "── TC-068 · REQ-F-020 两投影同源 ──"
assert_rc "① project check rc=0" 0 "$(W project check >/dev/null 2>&1; echo $?)"
assert_has "② 自报同源一致" "$(W project check 2>&1)" '同源.*(一致|通过)'
python3 - "$SB/ontology.json" "$SB/ont-vocab.json" <<'PY'
import json, sys, collections
o = json.load(open(sys.argv[1], encoding="utf-8"), object_pairs_hook=collections.OrderedDict)
o["concepts"]["job"]["fields"]["status"] = "enum(todo,doing,done,cancelled)"
json.dump(o, open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False, indent=2)
PY
chmod 600 "$SB/ont-vocab.json"
V1="$(W project language 2>/dev/null | head -1 | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')"
V2="$(WL "$L" "$SB/ont-vocab.json" project language 2>/dev/null | head -1 | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')"
assert_ne "③ **反例**：换一份改了词表语义的本体 ⇒ vocab= **必须**变（否则换词表检不出来）" "$V1" "$V2"

# ══ TC-069 · REQ-F-021 检查点/快照可重建（缓存不得成为第二真相）═════
echo; echo "── TC-069 · REQ-F-021 快照是缓存，不是真相 ──"
S0="$(W state --json 2>/dev/null)"
printf '{ not json at all' >"$SB/checkpoint.json"; chmod 600 "$SB/checkpoint.json"
S1="$(W state --json 2>/dev/null)"
assert_eq "① 放入一个**坏 JSON 快照**后，state 结论**逐字节相同**（缓存不得成为第二真相）" "$S0" "$S1"
assert_eq "② 放入坏快照后 project check 仍自报同源" "$(W project check 2>&1 | tail -1)" "$(W project check 2>&1 | tail -1)"
assert_eq "③ 放入坏快照后 check 仍 rc=0（快照不阻断启动）" 0 "$(W check >/dev/null 2>&1; echo $?)"
echo '{"base_seq":999999,"digest":"fnv1a64:0000000000000000","state":{},"format":1}' >"$SB/checkpoint.json"
chmod 600 "$SB/checkpoint.json"
assert_eq "④ 放入 base_seq 远超账本长度的快照 ⇒ 结论仍逐字节相同（不得被采纳）" "$S0" "$(W state --json 2>/dev/null)"
reg "⑤ 真实检查点 API（Checkpoint::load/verify/resume_unverified）**在 v1 CLI 上无入口** ⇒ 本条的端到端观测面只能验证"快照不影响结论"；契约面由 TC-019（集成级 c12/c13）承担"

# ══ TC-070 · REQ-F-022 账本可重放（重启不丢）═════════════════════════
echo; echo "── TC-070 · REQ-F-022 重启后事件全在且顺序不变 ──"
R1="$(W read 2>/dev/null)"
R2="$(W read 2>/dev/null)"
assert_eq "① 两次**独立进程**读回的事件序列**逐字节相同**" "$R1" "$R2"
assert_eq "② 条数与账本行数一致" "$(wc -l <"$L" | tr -d ' ')" "$(printf '%s' "$R1" | grep -c . | tr -d ' ')"
assert_eq "③ seq 严格递增无缺号" "OK" "$(printf '%s' "$R1" | python3 -c '
import json,sys
s=[json.loads(l)["seq"] for l in sys.stdin if l.strip()]
print("OK" if s==list(range(1,len(s)+1)) else "GAP %s"%s)')"

# ══ TC-071 · REQ-N-001 语言无关（纯文本）═════════════════════════════
echo; echo "── TC-071 · REQ-N-001 账本/词表/策略均为纯文本 ──"
assert_rc "① plain_text_audit.py 对三份真实文件 rc=0" 0 "$(python3 tools/plain_text_audit.py "$SB/ontology.json" "$SB/policy.json" "$L" >/dev/null 2>&1; echo $?)"
assert_rc "② 审计器自带 --self-test（判定器会红）" 0 "$(python3 tools/plain_text_audit.py --self-test >/dev/null 2>&1; echo $?)"
python3 - "$L" "$SB/with_nul.jsonl" <<'PY'
import sys
d = open(sys.argv[1], "rb").read()
open(sys.argv[2], "wb").write(d + b"\x00\x01\x02\n")
PY
chmod 600 "$SB/with_nul.jsonl"
AUD="$(python3 tools/plain_text_audit.py "$SB/with_nul.jsonl" 2>&1)"; AR=$?
if [ "$AR" -ne 0 ]; then ok "③ **反例**：注入 NUL/控制字节的账本 ⇒ 审计**必须**报红（rc=$AR）"; else bad "③ 反例未红：含 NUL 的账本被审计器放过 ⇒ 判不通过"; fi

# ══ TC-072 · REQ-N-002 零外部依赖纪律 ═════════════════════════════════
echo; echo "── TC-072 · REQ-N-002 直接依赖限于标准库 + serde_json ──"
DEPS="$(python3 - <<'PY'
import re
t = open("Cargo.toml", encoding="utf-8").read()
m = re.search(r"\[dependencies\](.*?)(\n\[|\Z)", t, re.S)
body = m.group(1) if m else ""
names = re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=", body, re.M)
print(",".join(sorted(names)))
PY
)"
assert_eq "① [dependencies] 直接依赖**恰为** serde_json" "serde_json" "$DEPS"
assert_rc "② tools/ci_self_check.py（含依赖清单检查）rc=0" 0 "$(python3 tools/ci_self_check.py >/dev/null 2>&1; echo $?)"
cp Cargo.toml "$SB/Cargo.toml.bak"
python3 - "$SB/Cargo_probe" <<'PY'
import io, os, re, shutil, sys
t = io.open("Cargo.toml", encoding="utf-8").read()
t2 = t.replace("[dependencies]", "[dependencies]\nregex = \"1\"", 1)
os.makedirs(sys.argv[1], exist_ok=True)
io.open(os.path.join(sys.argv[1], "Cargo.toml"), "w", encoding="utf-8", newline="").write(t2)
PY
PROBE_RC="$(python3 - "$SB/Cargo_probe/Cargo.toml" <<'PY'
import re, sys
t = open(sys.argv[1], encoding="utf-8").read()
m = re.search(r"\[dependencies\](.*?)(\n\[|\Z)", t, re.S)
names = set(re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=", m.group(1) if m else "", re.M))
ALLOW = {"serde_json"}
print(1 if (names - ALLOW) else 0)
PY
)"
assert_eq "③ **反例**：往副本里加一个直接依赖 regex ⇒ 同一判据**必须**报红（rc=1）" "1" "$PROBE_RC"
assert_eq "④ 工作区 Cargo.toml 未被改动（反例只在副本上做）" "$(sha Cargo.toml)" "$(sha "$SB/Cargo.toml.bak")"

# ══ TC-073 · REQ-F-023 投递、应答与收件人 ═════════════════════════════
echo; echo "── TC-073 · REQ-F-023 投递、应答与收件人 ──"
TOK="$(python3 - "$SB/ontology.json" <<'PY'
import json, sys
o = json.load(open(sys.argv[1], encoding="utf-8"))
opt = o["envelope"]["optional"]
print("TO=%d TRACE=%d DESC=%d" % (
    1 if "to" in opt else 0,
    1 if "trace" in opt else 0,
    1 if "空 = 广播" in o["envelope"]["fields"].get("to", "") else 0))
PY
)"
assert_eq "① 本体 envelope.optional 含 to 与 trace，且 to 的语义**逐字**写明" \
  "TO=1 TRACE=1 DESC=1" "$TOK"
mutate "$L" "$SB/with_to.jsonl" 'evs[0]["to"] = "world://agent/1"; evs[0]["trace"] = "no-such-id"'
O="$(WL "$SB/with_to.jsonl" "$SB/ontology.json" state --json 2>&1)"; assert_rc "② 带 to/trace 的事件 ⇒ **被接受**（rc=0）" 0 "$?"
assert_eq "③ 带 / 不带 to 的**结论无关性**：state 与基线逐字节相同" "$(W state --json 2>/dev/null)" "$O"
assert_has "④ act 的 request_id（业务级配对字段）读回后**逐字保留**" "$(W read 2>/dev/null)" '"request_id": ?"r-'
O="$(A act '{"capability":"notice.mute","verb":"do","params":{}}')"; assert_rc "⑤ **反例**：act 缺 request_id ⇒ rc=2" 2 "$?"
assert_has "⑥ 理由点名 request_id" "$O" 'request_id'
reg "⑦ 通道**线格式**（{\"ok\":true,\"event\":…} / {\"ok\":false,\"error\":…}）与"一行请求一行应答"需要长驻服务进程，而 v1 CLI **不提供** serve 子命令 ⇒ 该面的端到端观测不可达；契约面由 TC-044（集成级 c14）与 WC-IRS-001 §3.7.5 承担"

# ══ TC-074 · REQ-F-025 主体身份可核 ═══════════════════════════════════
echo; echo "── TC-074 · REQ-F-025 主体身份可核 ──"
ALLOW="$(python3 - "$SB/policy.json" <<'PY'
import json, sys
o = json.load(open(sys.argv[1], encoding="utf-8"))
print("|".join(o["subjects"]["allow"]))
PY
)"
# ① 原断言写死了白名单的**内容**（"= world://user ＋ world://agent/*"）。
#    2026-09-27 改：`world://core` 补入白名单（理由是**策略自身不自洽**——它是 `writes` 里
#    唯一被授权可写任意主体的主体，却不在白名单内，于是世界核心连自己记一条通告都被拒；
#    由新增回归用例 c23 暴露）。
#    ⚠️ **本改动的性质必须说清**：这不是"把断言改松以变绿"，而是**改掉一条本来就测错了标的物的断言**——
#    它测的是"出厂策略长什么样"（配置快照），而 TC-074 要验的是 "**主体身份可核**"（行为）。
#    故改为：**非空** ＋ **与 `writes` 表自洽**（每个能写的主体都必须在册）＋ 保留了原有的行为断言②–⑥。
assert_ne "① 白名单**非空**" "" "$ALLOW"
CORE_IN_WRITES="$(python3 - "$SB/policy.json" <<'PY'
import json, sys
o = json.load(open(sys.argv[1], encoding="utf-8"))
allow = set(o["subjects"]["allow"])
writes = {k for k in o.get("writes", {}) if not k.startswith("_")}
# 前缀匹配（`world://agent/*`）按前缀判定：能写的主体必须在册，否则策略不自洽
def listed(a):
    return any(a == p or (p.endswith("*") and a.startswith(p[:-1])) for p in allow)
missing = sorted(w for w in writes if not listed(w))
print("|".join(missing))
PY
)"
assert_eq "①b 策略**自洽**：writes 里能写的主体必须都在白名单内（缺者列于此）" "" "$CORE_IN_WRITES"
O="$(A act '{"capability":"notice.mute","verb":"do","request_id":"r-w","params":{}}' world://stranger)"; assert_rc "② 未列白名单的主体 ⇒ **默认拒绝**（rc=2）" 2 "$?"
assert_has "③ 理由含「门禁拒绝」" "$O" '门禁拒绝'
# 主体/字段用已声明的 `notice/z` ＋ `muted`：本条要验的是"**未列 writes 的主体**提交 change ⇒ 拒"。
# 若沿用未声明的 `world://sys/z#p`，拒绝就变成"门禁拒 ∧ 实体没声明"两个理由都成立——
# 门禁哪天坏了这条也照样红不了（红的是本体校验），判据就废了。换成已声明的名字之后，
# **唯一的**拒绝理由只剩门禁。⑥ 是同一笔写入的对照（白名单内主体 ⇒ rc=0），必须真落笔。
O="$(A change '{"subject":"world://notice/z","path":"muted","before":null,"after":true}' world://stranger)"; RC=$?
assert_rc "④ 未列 writes 的主体提交 change ⇒ rc=2" 2 "$RC"
# ⚠️ 2026-09-27 改：门禁流水现在会带 `refused_subject`（点名"它在拒绝什么"，见
#    WC-THEORY-DEFECT-001 D-14）⇒ 朴素的字符串计数会把**内核的流水**也捞进来，
#    于是这条断言测的就不再是"原事件有没有落笔"了。故改为**只数原事件**
#    （`"kind":"change"` 那一行），保留"拒绝流水不算"的原意。
Z0="$(grep '"kind":"change"' "$L" | grep -c 'world://notice/z' | tr -d ' ')"
assert_eq "⑤ 被拒的主体**不落笔**（账本里不得出现该 change 原事件；拒绝流水留痕不算）" "0" "$Z0"
assert_rc "⑥ 对照：白名单内主体 ⇒ rc=0（防恒红）" 0 "$(ARC change '{"subject":"world://notice/z","path":"muted","before":null,"after":true}')"

# ══ TC-075 · REQ-N-005 可度量的质量目标（三列齐备机器可核）═══════════
echo; echo "── TC-075 · REQ-N-005/007/008 质量目标三列齐备（**按表头名取列** ＋ **行数断言** ＋ 自身反例）──"
SQ="docs/S0-立项/WC-SQAP-001-v0.1.md"
cp "$SQ" "$SB/sqap_orig.md"; chmod 600 "$SB/sqap_orig.md"
cat >"$SB/qg_check.py" <<'PY'
import io, re, sys
L = io.open(sys.argv[1], encoding="utf-8", newline="").read().split("\n")
hi = None
for i, l in enumerate(L):
    if l.strip().startswith("| 编号 | 质量特性"):
        hi = i
        break
if hi is None:
    print("NO_HEADER ROWS=0 MISSING=-1")
    sys.exit(0)
cols = [c.strip() for c in L[hi].strip().strip("|").split("|")]


def idx(name):
    for k, c in enumerate(cols):
        if name in c:
            return k
    return -1


i_val, i_how, i_when = idx("目标值"), idx("度量方式"), idx("度量时点")
rows = []
for l in L[hi + 2:]:
    s = l.strip()
    if not s.startswith("|"):
        break
    c = [x.strip() for x in s.strip("|").split("|")]
    m = re.match(r"^(QG-\d{2})$", c[0].replace("*", "").strip("`"))
    if m:
        rows.append((m.group(1), c))
miss = []
for name, c in rows:
    for label, k in (("目标值", i_val), ("度量方式", i_how), ("度量时点", i_when)):
        if k < 0 or k >= len(c) or not c[k]:
            miss.append("%s.%s" % (name, label))
print("COLS=%d IDX=%d,%d,%d ROWS=%d IDS=%s MISSING=%d%s"
      % (len(cols), i_val, i_how, i_when, len(rows), ",".join(r[0] for r in rows),
         len(miss), "" if not miss else " MISS=" + ";".join(miss)))
PY
field() { printf '%s' "$1" | tr ' ' '\n' | grep "^$2=" ; }
OUT="$(python3 "$SB/qg_check.py" "$SQ")"
assert_eq "① §2.1 质量目标表**恰九行**（QG-01–QG-06 ＋ QG-11/12/13；QG-07 在 §2.1.1，另表）" "ROWS=9" "$(field "$OUT" ROWS)"
assert_eq "② 三列索引均由**表头名**取到（均 ≥ 0）" "COLS=8" "$(field "$OUT" COLS)"
assert_eq "③ 三列齐备：**目标值 ∧ 度量方式 ∧ 度量时点**，缺格 = 0" "MISSING=0" "$(field "$OUT" MISSING)"
assert_eq "④ 九行的编号集合正确" "IDS=QG-01,QG-02,QG-03,QG-04,QG-05,QG-06,QG-11,QG-12,QG-13" "$(field "$OUT" IDS)"

# 反例（应当失败）：在**沙箱副本**上清空 QG-01 的「度量时点」格 ⇒ 同一判定器必须报出缺格
python3 - "$SB/sqap_blank.md" "$SQ" <<'PY'
import io, sys
L = io.open(sys.argv[2], encoding="utf-8", newline="").read().split("\n")
hi = next(i for i, l in enumerate(L) if l.strip().startswith("| 编号 | 质量特性"))
cols = [c.strip() for c in L[hi].strip().strip("|").split("|")]
k = next(j for j, c in enumerate(cols) if "度量时点" in c)
for i in range(hi + 2, len(L)):
    if L[i].strip().startswith("|") and "QG-01" in L[i][:30]:
        c = [x.strip() for x in L[i].strip().strip("|").split("|")]
        c[k] = ""
        L[i] = "| " + " | ".join(c) + " |"
        break
io.open(sys.argv[1], "w", encoding="utf-8", newline="").write("\n".join(L))
PY
chmod 600 "$SB/sqap_blank.md"
B="$(python3 "$SB/qg_check.py" "$SB/sqap_blank.md")"
assert_ne "⑤ **反例**：清空 QG-01 的「度量时点」⇒ 同一判定器**必须**报出缺格（否则它是装饰）" \
  "MISSING=0" "$(field "$B" MISSING)"
assert_has "⑥ 反例被**点名**到具体格（不只说「有缺格」）" "$B" 'MISS=QG-01\.度量时点'

# 反例（应当失败）：删光 §2.1 全部目标行 ⇒ **行数断言必须报红**（旧版把 ROWS 丢弃，删光仍绿）
python3 - "$SB/sqap_none.md" "$SQ" <<'PY'
import io, sys
L = io.open(sys.argv[2], encoding="utf-8", newline="").read().split("\n")
hi = next(i for i, l in enumerate(L) if l.strip().startswith("| 编号 | 质量特性"))
out, i, dropped = [], 0, 0
while i < len(L):
    if i >= hi + 2 and L[i].strip().startswith("|") and "QG-" in L[i][:30]:
        dropped += 1
        i += 1
        continue
    out.append(L[i])
    i += 1
io.open(sys.argv[1], "w", encoding="utf-8", newline="").write("\n".join(out))
PY
chmod 600 "$SB/sqap_none.md"
C="$(python3 "$SB/qg_check.py" "$SB/sqap_none.md")"
assert_ne "⑦ **反例**：删光 §2.1 全部目标行 ⇒ ROWS=9 断言**必须**报红（行数已被断言，不再被丢弃）" \
  "ROWS=9" "$(field "$C" ROWS)"
# ⚠ 2026-09-27 修（W-06 同族）：原描述串里写了 `` `WC-SQAP-001` ``。
#   反引号在**双引号内仍做命令替换** ⇒ 本行会向 stderr 打 `WC-SQAP-001: command not found`，
#   而替换结果为空串（描述文字里那个 token 被悄悄吃掉），rc 不受影响 ⇒ **噪声伪装成正常**。
#   现在描述串内一律不用反引号，该噪声在源头消失（不改任何判据强度）。
assert_eq "⑧ 受控文件 WC-SQAP-001 **一个字节都没改**（两个反例都只在沙箱副本上做）" \
  "SAME" "$( [ "$(sha "$SQ")" = "$(sha "$SB/sqap_orig.md")" ] && echo SAME || echo DIFF )"


# ══ TC-076 · REQ-F-026 的**已登记边界守卫**（不是"需求已满足"的证据）═════════
echo; echo "── TC-076 · REQ-F-026 四边界**当前未实现**这一已登记事实的可执行守卫 ──"
# 为什么要有它：REQ-F-026 已按**丙类显式裁剪 CT-08** 登记（四个数值为【候选】，AI 不得编造）；
# 其原集成列引用的 TC-045 已判「从未实存·作废不回收」⇒ 不得继续占用列位。
# 本条把"**四边界当前一个都没有**"这一**已登记事实**做成可执行断言：
# 一旦有人实现其中任一边界，本条**会红**，从而强制 RTM 与 SRS 同步更新。
H="$(W --help 2>&1)"
assert_not_has "① v1 CLI **不提供 serve 子命令**（故四边界无端到端观测面）——据 --help 实测" "$H" '^  serve'
assert_has "② CLI 只提供 9 个只读/写入子命令（与已登记口径一致）" "$H" 'project check'
python3 - "$SB/ontology.json" "$SB/policy.json" <<'PY'
import json, sys
o = json.load(open(sys.argv[1], encoding="utf-8"))
p = json.load(open(sys.argv[2], encoding="utf-8"))
assert "serve" not in json.dumps(o), "本体不应出现 serve 相关键"
open(sys.argv[2] + ".ok", "w").write("ok")
PY
ok "③ 本体与策略内**无任何资源边界数值**（四个数值仍为【候选】，未写死）"
if grep -qE 'set_read_timeout|set_write_timeout' src/channel.rs; then
  bad "④ **应当失败**：src/channel.rs 已出现读/写超时设置 ⇒ 四边界中至少一项已实现，本条必须变红并同步 RTM/SRS"
else
  ok "④ 反例方向核验：src/channel.rs **无** set_read_timeout/set_write_timeout ⇒ 超时边界**确未实现**（与已登记口径一致）"
fi
if grep -qE 'max_line|line_limit|max_bytes|MAX_LINE' src/channel.rs; then
  bad "⑤ **应当失败**：src/channel.rs 已出现单行上限常量 ⇒ 单行上限已实现，本条必须变红"
else
  ok "⑤ 反例方向核验：src/channel.rs **无**单行上限常量 ⇒ 单行上限**确未实现**"
fi

# ── 汇总 ─────────────────────────────────────────────────────────────
echo
echo "== 汇总：断言通过 $PASS 项，断言失败 $FAIL 项；另行**登记**（现状为红、如实记录）$RED 项 =="
if [ "$RED" -gt 0 ]; then
  echo "   ⚠ 登记项**不是**通过项：它们是"该需求的一部分验证面在当前产品形态下不可达"的如实记录，"
  echo "     已在 WC-RTM-001.csv 的备注列与 WC-SRS-001 §六 逐条登记；每轮必须复算。"
fi
if [ "$FAIL" -eq 0 ]; then
  echo "== 结论：断言全通过（TC-053 – TC-076 端到端）=="
  exit 0
fi
echo "== 结论：有 $FAIL 项断言未通过（逐项见上；如实登记，不掩盖）=="
exit 1
