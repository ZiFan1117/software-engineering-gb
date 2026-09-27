#!/usr/bin/env bash
# s1_sys_probe.sh —— **S1 需求验证面补建**（系统级；真实二进制端到端）
#
# ## 为什么需要它
#
# R1 的九席独立评审指出：`WC-SRS-001` §五 声明的用例里有一批**从未实存**
# （编号在 `world-core/` 全部 `.rs`/`.sh`/`.py`/`.yml` 里零命中），而若干 `REQ-F-*` 的
# 「应当失败」反例**恰恰只挂在这些不存在的用例上** ⇒ 按本项目逐字纪律
# 「**反例不红视为未校验**」，那些条目**当时不可校验**。
#
# 本脚本把其中**不需要改产品代码即可执行**的那部分**补建出来**，交给真实二进制跑。
#
# | 用例 | 需求 | 端到端断言 |
# |---|---|---|
# | `TC-042` | `REQ-F-024` 投影不新增事实 | `project language`/`project visual` 的三元组集合 P ⊆ `state --json` 的 S；**反假**：注入 S 中不存在的三元组 ⇒ 判定必须非空 |
# | `TC-046` | `REQ-F-027` 家族演进 | 换一份**只加扩展**的本体 ⇒ 旧账本折叠结果**逐字节不变**；**反例**：`world` 升版 ⇒ 必须拒启 |
# | `TC-047` | `REQ-F-028` 取值形状 | 手写类型不符账本行 ⇒ 各得**类型化**拒绝（点名 `kind` / 点名 `seq`） |
# | `TC-048` | `REQ-F-029` 未知旗标必须忽略 | 含**未知旗标**的合法事件**必须被接受**；对偶：未知 `kind` **必须被拒绝** |
# | `TC-049` | `REQ-F-030` 极小核心 + 命名空间扩展 | ① 核心面**逐项可枚举**；③ 只加扩展 ⇒ 折叠结果不变 |
# | `TC-050` | `REQ-N-006` 投影质量目标 | **跨进程**两份投影同源头（`world`/`vocab`/`last_seq`/`state`）**逐字节相同**；**反例**：换词表 ⇒ 词表 hash 必变 |
# | `TC-051` | `REQ-F-019` 排版可审计 | 调 `tools/visual_layout_audit.py`（**独立于** `visual::parse()` 的第二份解析器），三样本：普通值 / 含换行·控制字符的值 / 空状态 |
# | `TC-052` | `REQ-F-031` `trace` 字段 | 判据 (4a)(4b)：带 `trace`（指向不存在的 `id`）与不带 `trace` 两本账 ⇒ **都必须被接受**，且 `state` 与两投影结论**逐字节相同** |
#
# **不在本脚本内的用例**（如实登记，不假装）：`TC-024`／`TC-043`／`TC-045`——
# 它们的**实现面本身不存在**（度量脚本未建 / `to` 在 `src/`+`tests/` 零读写 / 通道四边界一个都没有），
# 故在 `WC-SRS-001` §五 处置表中标 **从未实存（编号作废不回收）**。
#
# ## 两类结论，**分开报告、不得互相替代**
#
# - **断言**（必须绿）——失败即 `exit 1`，是门禁结论；
# - **登记**（`⛔`，**现状为红且已如实登记**）——**不影响退出码**，但必须在每一轮评审重新复算；
#   数值或集合一变，即视为该登记**已过时**，须重新登记。
#   这一类正是本项目最怕的"把已知缺陷藏进绿灯里"，故**显式计数并逐条打印**。
#
# ## 纪律
#
# - 全程 `mktemp -d` 一次性沙箱，退出即删；**绝不触碰真实账本**（同 `check.sh` / `system_acceptance.sh`）；
# - **手写账本行**只写副本：取真实落盘行、**去掉 `chain` 摘要字段**（默认不要求摘要链，
#   `--require-chain` 才要求），再改**一个**字段——这样改的是"事件内容"，不是"摘要不符"；
# - 产品代码、出厂本体、门禁策略**一个字节都不改**；
# - 自带 `--self-test`：先证明**本脚本的判定会红**（"一个从不失败的检查不是检查，是装饰"）；
# - 不依赖网络、不依赖时钟、不依赖执行顺序。
#
# 用法：`bash tools/s1_sys_probe.sh`
#       `bash tools/s1_sys_probe.sh --self-test`
# 退出码：0 = 断言全通过；1 = 有断言失败；2 = 前置条件不满足（缺二进制且构建失败）
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2
BIN="${CARGO_TARGET_DIR:-target}"/debug/world-core

PASS=0
FAIL=0
RED=0

ok() { echo "  ✅ $1"; PASS=$((PASS + 1)); }
bad() { echo "  ❌ $1"; FAIL=$((FAIL + 1)); }
reg() { echo "  ⛔ 登记（现状为红，如实记录，**不计入**门禁结论）: $1"; RED=$((RED + 1)); }

assert_eq() { # assert_eq <描述> <期望> <实际>
  if [ "$2" = "$3" ]; then ok "$1"; else bad "$1（期望 [$2]，实得 [$3]）"; fi
}
assert_ne() { # assert_ne <描述> <不该等于> <实际>
  if [ "$2" != "$3" ]; then ok "$1"; else bad "$1（不应等于 [$2]）"; fi
}
assert_rc() { assert_eq "$1" "$2" "$3"; }
assert_has() { # assert_has <描述> <文本> <正则>
  if printf '%s' "$2" | grep -Eq "$3"; then ok "$1"; else bad "$1（输出里找不到 /$3/：$(printf '%s' "$2" | head -3 | tr '\n' ' ')）"; fi
}

# ── 判定器自证（先证明它会红）─────────────────────────────────────────
if [ "${1:-}" = "--self-test" ]; then
  echo "== s1_sys_probe.sh 判定器自证 =="
  before=$FAIL
  assert_eq "自证：故意断言 1=2" "1" "2"
  assert_has "自证：故意在空串里找 REQUIRE_THIS" "" 'REQUIRE_THIS'
  if [ "$FAIL" -eq $((before + 2)) ]; then
    echo "  ✅ 自证通过：两个假命题都被判为失败（本判定器不是装饰）"
    exit 0
  fi
  echo "  ❌ 自证失败：假命题竟被判为通过"
  exit 1
fi

# ── 前置：二进制（缺失则构建一次）────────────────────────────────────
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

W() { "$BIN" --ontology "$SB/ontology.json" --ledger "$L" --policy "$SB/policy.json" "$@"; }
WO() { # WO <ontology-path> [args...]
  local ont="$1"
  shift
  "$BIN" --ontology "$ont" --ledger "$L" --policy "$SB/policy.json" "$@"
}
WO2() { # WO2 <ontology-path> <ledger-path> [args...]
  local ont="$1" led="$2"
  shift 2
  "$BIN" --ontology "$ont" --ledger "$led" --policy "$SB/policy.json" "$@"
}

echo "== world-core S1 验证面补建（真实二进制 $BIN）=="
echo "  目录 : $(pwd)"
echo "  沙箱 : $SB"
echo "  用例 : TC-042 / TC-046 / TC-047 / TC-048 / TC-049 / TC-050 / TC-051 / TC-052"

# ══ 种子：本账 2 条 + 一条真实落盘行副本（供手写变异用）══════════════════
W append change '{"subject":"world://sys/a","path":"p","before":null,"after":1}' >/dev/null 2>&1
W append act '{"capability":"notice.mute","verb":"do","request_id":"r-1","params":{}}' >/dev/null 2>&1
assert_eq "种子：账本 2 行" "2" "$(wc -l <"$L" | tr -d ' ')"
REAL1="$(head -1 "$L")"
assert_has "种子：真实落盘行含 chain 摘要字段（本脚本的手写变体会**去掉**它）" "$REAL1" '"chain"'

# 手写变异器：取真实落盘行 → **去掉 chain** → 按表达式改**一个**字段
# **保留第 2 行原样**（否则手写账本只有 1 行，与基线 2 行不可比 —— 那会造出假红）
mk_bad() { # mk_bad <输出文件> <python 表达式（用 ev 变量）>
  python3 - "$REAL1" "$1" "$2" "$L" <<'PY'
import json, sys
real, dst, expr, seed = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
ev = json.loads(real)
exec(expr, {"ev": ev, "json": json})
lines = open(seed, encoding="utf-8").read().rstrip("\n").split("\n")
evs = [json.loads(x) for x in lines]
evs[0] = ev
# 去掉**每一行**的 chain：默认不要求摘要链，且**不得混用**（既有 chain 又有无 chain 的账本
# 会被 `ext.world.Ledger.MixedChain` 拒绝 —— 那是产品在防"前半受保护"的误导，是对的）
for e in evs:
    e.pop("chain", None)
with open(dst, "w", encoding="utf-8") as f:
    f.write("\n".join(json.dumps(e, ensure_ascii=False, sort_keys=True) for e in evs) + "\n")
PY
  chmod 600 "$1"
}

# 同源头比较器：**投影种类标记**（language / visual）本来就不同，故只比四要素
hdr4() { # hdr4 <投影文本> → "world|vocab|last_seq|state"
  printf '%s' "$1" | head -1 | sed -n \
    's/.*world=\([^ ]*\) vocab=\([^ ]*\) last_seq=\([^ ]*\) state=\([^ ]*\)$/\1|\2|\3|\4/p'
}
hdr_novocab() { # hdr_novocab <投影文本> → "world|last_seq|state"（排除 vocab：本体变了，hash 本就该变）
  hdr4 "$1" | awk -F'|' '{print $1"|"$3"|"$4}'
}

# ══ TC-042 · REQ-F-024 投影不新增事实（P ⊆ S，系统级）════════════════
echo
echo "── TC-042 · REQ-F-024 投影不新增事实（真实二进制，P ⊆ S）──"

LANG_TXT="$SB/lang.txt"
VIS_TXT="$SB/vis.txt"
ST_JSON="$SB/state.json"
W project language >"$LANG_TXT" 2>/dev/null
W project visual >"$VIS_TXT" 2>/dev/null
W state --json >"$ST_JSON" 2>/dev/null

cat >"$SB/cmp_proj.py" <<'PY'
import json, re, sys

kind, proj_path, state_path = sys.argv[1], sys.argv[2], sys.argv[3]
txt = open(proj_path, encoding="utf-8").read()
st = json.load(open(state_path, encoding="utf-8"))

S = {}
for subj, fields in st.get("objects", {}).items():
    for p, v in fields.items():
        S[(subj, p)] = v

P = {}
if kind == "language":
    for line in txt.splitlines():
        if line.startswith("#world-core") or not line.strip():
            continue
        o = json.loads(line)
        for p, v in o["fields"].items():
            P[(o["subject"], p)] = v
else:
    subj = None
    for line in txt.splitlines():
        if line.startswith("#world-core") or not line.strip():
            continue
        m = re.match(r"^ {6}(\S.*?) = (.*)$", line)
        if m:
            if subj is not None:
                P[(subj, m.group(1))] = json.loads(m.group(2))
            continue
        m2 = re.match(r"^ {2}(world://\S+)$", line)
        if m2:
            subj = m2.group(1)

extra = sorted(k for k in P if k not in S)
diff = sorted(k for k in P if k in S and S[k] != P[k])
print("P=%d" % len(P))
print("S=%d" % len(S))
print("EXTRA=%d" % len(extra))
print("DIFF=%d" % len(diff))
if extra:
    print("EXTRA_KEYS=%s" % ";".join("%s|%s" % k for k in extra))
if diff:
    print("DIFF_KEYS=%s" % ";".join("%s|%s" % k for k in diff))
PY

for KIND in language visual; do
  case "$KIND" in
    language) F="$LANG_TXT" ;;
    visual) F="$VIS_TXT" ;;
  esac
  OUT="$(python3 "$SB/cmp_proj.py" "$KIND" "$F" "$ST_JSON")"
  assert_eq "① $KIND 投影的非空三元组数 = 读模型条目数（P 非空，两端都非 0）" \
    "$(printf '%s' "$OUT" | sed -n 's/^S=//p')" "$(printf '%s' "$OUT" | sed -n 's/^P=//p')"
  assert_eq "② $KIND 投影的 EXTRA（P \\ S）必须 = 0" "0" "$(printf '%s' "$OUT" | sed -n 's/^EXTRA=//p')"
  assert_eq "③ $KIND 投影同键值不一致（DIFF）必须 = 0" "0" "$(printf '%s' "$OUT" | sed -n 's/^DIFF=//p')"
done

# 反假④⑤：注入一个 S 中不存在的三元组 ⇒ 比对器**必须**报非空（证明它有牙）
python3 - "$VIS_TXT" "$SB/vis_injected.txt" <<'PY'
import sys
src, dst = sys.argv[1], sys.argv[2]
lines = open(src, encoding="utf-8").read().rstrip("\n").split("\n")
out = []
for line in lines:
    out.append(line)
    if line.startswith("      p = "):
        out.append("  world://sys/ghost")
        out.append("      phantom = 42")
open(dst, "w", encoding="utf-8").write("\n".join(out) + "\n")
PY
INJ="$(python3 "$SB/cmp_proj.py" visual "$SB/vis_injected.txt" "$ST_JSON")"
assert_ne "④ 反假：注入 S 中不存在的三元组后 EXTRA **必须**非 0（否则比对器是装饰）" \
  "0" "$(printf '%s' "$INJ" | sed -n 's/^EXTRA=//p')"
assert_has "⑤ 反假：注入项被**点名**（不只说「有差异」）" "$INJ" 'EXTRA_KEYS=world://sys/ghost\|phantom'

# ══ TC-046 · REQ-F-027 家族演进与向前兼容 ═══════════════════════════
echo
echo "── TC-046 · REQ-F-027 家族演进（旧账本 + 只加扩展的新本体）──"

BASE_STATE="$(W state --json)"
BASE_LANG="$(W project language | head -1)"
BASE_VIS="$(W project visual | head -1)"

# 「只加扩展」的本体：新增一个家族 + 一个概念（纯加法，不改任何既有字段含义）
python3 - "$SB/ontology.json" "$SB/ontology-ext.json" <<'PY'
import json, sys, collections
src, dst = sys.argv[1], sys.argv[2]
o = json.load(open(src, encoding="utf-8"), object_pairs_hook=collections.OrderedDict)
o["families"]["audit"] = collections.OrderedDict([
    ("_comment", "S1 补建用例用的**纯加法**扩展家族：不得影响既有三家族的语义"),
    ("required", ["scope", "result"]),
    ("optional", []),
])
o["concepts"]["audit"] = {"fields": {"result": "enum(pass,fail)"}}
json.dump(o, open(dst, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
PY
chmod 600 "$SB/ontology-ext.json"

assert_eq "① 只加扩展的新本体 ⇒ 旧账本折叠结果**逐字节不变**" "$BASE_STATE" "$(WO "$SB/ontology-ext.json" state --json)"
assert_eq "② 只加扩展 ⇒ 语言投影的 (world, last_seq, state 指纹) **逐字节不变**（排除 vocab：本体变了，hash 本就该变）" \
  "$(hdr_novocab "$BASE_LANG")" "$(hdr_novocab "$(WO "$SB/ontology-ext.json" project language)")"
assert_eq "③ 只加扩展 ⇒ 视觉投影的 (world, last_seq, state 指纹) **逐字节不变**" \
  "$(hdr_novocab "$BASE_VIS")" "$(hdr_novocab "$(WO "$SB/ontology-ext.json" project visual)")"
assert_eq "④ 只加扩展 ⇒ 折叠结果指纹**未变**（把 state= 单独再核一次）" \
  "$(hdr4 "$BASE_LANG" | awk -F'|' '{print $4}')" \
  "$(hdr4 "$(WO "$SB/ontology-ext.json" project language)" | awk -F'|' '{print $4}')"
assert_ne "⑤ 对照：本体确实变了（词表 hash **必须**变，否则上面几条是同义反复）" \
  "$(printf '%s' "$BASE_LANG" | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')" \
  "$(WO "$SB/ontology-ext.json" project language | head -1 | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')"

# 反例⑥⑦：world 升版 ⇒ 必须拒启（承接 REQ-F-002 / REQ-F-027 判据①）
python3 - "$SB/ontology.json" "$SB/ontology-v2.json" <<'PY'
import json, sys, collections
src, dst = sys.argv[1], sys.argv[2]
o = json.load(open(src, encoding="utf-8"), object_pairs_hook=collections.OrderedDict)
o["world"] = 2
json.dump(o, open(dst, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
PY
chmod 600 "$SB/ontology-v2.json"
V2OUT="$(WO "$SB/ontology-v2.json" state 2>&1)"
V2RC=$?
assert_rc "⑥ 反例：world 升版（1→2）⇒ 必须拒启（rc=2）" 2 "$V2RC"
assert_has "⑦ 反例：拒绝理由点名版本问题" "$V2OUT" '(版本|[Bb]ad[Vv]ersion|world)'

# ══ TC-047 · REQ-F-028 取值形状（类型化拒绝）══════════════════════════
echo
echo "── TC-047 · REQ-F-028 取值形状：类型不符 ⇒ **类型化** Violation ──"

mk_bad "$SB/bad_kind.jsonl" 'ev["kind"] = "bogus"'
mk_bad "$SB/bad_seq.jsonl" 'ev["seq"] = "1"'
mk_bad "$SB/bad_body.jsonl" 'ev["body"] = "not-an-object"'
mk_bad "$SB/bad_world.jsonl" 'ev["world"] = "1"'
mk_bad "$SB/bad_missing_actor.jsonl" 'del ev["actor"]'

O="$(WO2 "$SB/ontology.json" "$SB/bad_kind.jsonl" state 2>&1)"; R=$?
assert_rc "① 未知 kind ⇒ 拒绝（rc=2）" 2 "$R"
assert_has "② ①的理由**点名**那个未知家族 bogus（UnknownKind 的 kind）" "$O" 'bogus'

O="$(WO2 "$SB/ontology.json" "$SB/bad_seq.jsonl" state 2>&1)"; R=$?
assert_rc "③ seq 为字符串（应 integer）⇒ 拒绝（rc=2）" 2 "$R"
assert_has "④ ③的理由是**类型化**错误码（ext.world.*，不是一句泛泛的"解析失败"）" "$O" 'ext\.world\.[A-Za-z]+\.'

O="$(WO2 "$SB/ontology.json" "$SB/bad_body.jsonl" state 2>&1)"; R=$?
assert_rc "⑤ body 为字符串（应 object）⇒ 拒绝（rc=2）" 2 "$R"

# 反假⑥：未变异的真实账本必须通过（否则上面全部是恒红，等于没判）
GOODOUT="$(WO2 "$SB/ontology.json" "$L" state --json 2>&1)"; RG=$?
assert_rc "⑥ 反假：未变异的真实账本 ⇒ 必须通过（rc=0；证明上面不是恒红）" 0 "$RG"
assert_eq "⑦ 反假：未变异账本的状态与基线逐字节相同" "$BASE_STATE" "$GOODOUT"

# 登记⑧⑨：类型口径的**两处未落实**（现状为红，如实登记，不掩盖也不假装是断言失败）
O="$(WO2 "$SB/ontology.json" "$SB/bad_world.jsonl" state 2>&1)"; R=$?
if [ "$R" -eq 2 ]; then
  ok "⑧ world 为字符串（应 integer）⇒ 被拒（rc=2）"
else
  reg "⑧ world 为字符串（应 integer）竟**被接受**（rc=$R）：world 的类型断言在折叠层未落实 —— REQ-F-028 判据② 对本字段**不成立**"
fi
O="$(WO2 "$SB/ontology.json" "$SB/bad_missing_actor.jsonl" state 2>&1)"; R=$?
if [ "$R" -eq 2 ]; then
  ok "⑨ 缺必填信封字段 actor ⇒ 被拒（rc=2）"
else
  reg "⑨ 缺必填信封字段 actor 竟**被接受**（rc=$R）：**必填字段**校验只在写入路径（本体校验）上，折叠层不校验 —— REQ-F-028 判据② 对本字段**不成立**"
fi

# ══ TC-048 · REQ-F-029 未知旗标必须忽略（与未知家族拒绝对偶）════════
echo
echo "── TC-048 · REQ-F-029 未知旗标必须忽略 ＋ 对偶：未知 kind 必须被拒 ──"

mk_bad "$SB/flag_unknown.jsonl" 'ev["flags"] = ["future.flag", "another.flag"]'
FOUT="$(WO2 "$SB/ontology.json" "$SB/flag_unknown.jsonl" state --json 2>&1)"; FR=$?
assert_rc "① 含**未知旗标**的合法事件 ⇒ **必须被接受**（rc=0；注意方向与其它条相反）" 0 "$FR"
assert_eq "② ①的折叠结果与不带旗标时**逐字节相同**（旗标对结论无影响）" "$BASE_STATE" "$FOUT"

mk_bad "$SB/flag_empty.jsonl" 'ev["flags"] = []'
EOUT="$(WO2 "$SB/ontology.json" "$SB/flag_empty.jsonl" state --json 2>&1)"; ER=$?
assert_rc "③ 对照：空旗标数组 ⇒ 同样接受（rc=0）" 0 "$ER"
assert_eq "④ ③的结果与基线逐字节相同" "$BASE_STATE" "$EOUT"

EOUT2="$(WO2 "$SB/ontology.json" "$SB/bad_kind.jsonl" state 2>&1)"; ER2=$?
assert_rc "⑤ 对偶：未知 kind ⇒ **必须被拒绝**（rc=2）" 2 "$ER2"
assert_has "⑥ 对偶一半与 TC-047 的 ① 同源（两处必须一致，不得一处松一处紧）" "$EOUT2" 'bogus'

# 登记⑦：判据①的**可构造性**边界（出厂本体 flags 为空数组）
FLAGS_LEN="$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1],encoding="utf-8")).get("flags",[])))' "$SB/ontology.json")"
if [ "$FLAGS_LEN" = "0" ]; then
  reg "⑦ 出厂本体 flags = **空数组**（实测长度 $FLAGS_LEN）⇒ 「未知旗标」在**本体内没有任何已定义旗标可比对**；① 之所以能跑，靠的是**手写账本行**而非公开写入入口。该边界如实登记，不读作「已完备」"
else
  ok "⑦ 出厂本体已定义 $FLAGS_LEN 个旗标（判据①有本体侧对照面）"
fi

# ══ TC-049 · REQ-F-030 极小核心 + 命名空间扩展 ═══════════════════════
echo
echo "── TC-049 · REQ-F-030 极小核心 + 命名空间扩展 ──"

CORE="$(python3 - "$SB/ontology.json" <<'PY'
import json, sys
o = json.load(open(sys.argv[1], encoding="utf-8"))
env, fams = o["envelope"], o["families"]
print("REQ=%d" % len(env["required"]))
print("OPT=%d" % len(env["optional"]))
print("FAMS=%d" % len(fams))
print("EMPTY_REQ=%d" % sum(1 for f in fams.values() if not f.get("required")))
print("HAS_TO=%d" % (1 if "to" in env["optional"] else 0))
print("HAS_TRACE=%d" % (1 if "trace" in env["optional"] else 0))
print("FLAGS=%d" % len(o.get("flags", [])))
PY
)"
assert_eq "① 核心面可枚举：envelope.required = 8 项" "REQ=8" "$(printf '%s' "$CORE" | grep '^REQ=')"
assert_eq "② 核心面可枚举：envelope.optional = 2 项" "OPT=2" "$(printf '%s' "$CORE" | grep '^OPT=')"
assert_eq "③ 三个家族，各自 required **均非空**（空家族数 = 0）" "EMPTY_REQ=0" "$(printf '%s' "$CORE" | grep '^EMPTY_REQ=')"
assert_eq "④ 可选字段恰为 to" "HAS_TO=1" "$(printf '%s' "$CORE" | grep '^HAS_TO=')"
assert_eq "⑤ 可选字段恰为 trace" "HAS_TRACE=1" "$(printf '%s' "$CORE" | grep '^HAS_TRACE=')"
assert_eq "⑥ 判据③「只加扩展 ⇒ 同一账本折叠结果不变」（与 TC-046① 同断言、两处必须一致）" \
  "$BASE_STATE" "$(WO "$SB/ontology-ext.json" state --json)"

# 登记⑦：判据②「扩展项不得与核心字段重名」——现状为红（本体加载器只查形状，不查重名）
python3 - "$SB/ontology.json" "$SB/ontology-collide.json" <<'PY'
import json, sys, collections
src, dst = sys.argv[1], sys.argv[2]
o = json.load(open(src, encoding="utf-8"), object_pairs_hook=collections.OrderedDict)
o["concepts"]["body-fake"] = {"fields": {"body": "string"}}
json.dump(o, open(dst, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
PY
chmod 600 "$SB/ontology-collide.json"
COL_OUT="$(WO "$SB/ontology-collide.json" state 2>&1)"; COL_RC=$?
if [ "$COL_RC" -eq 2 ]; then
  ok "⑦ 反例②：扩展项与核心字段（body）重名 ⇒ 被拒（rc=2）"
else
  reg "⑦ 反例②**现状为红**：扩展项与核心信封字段重名（concepts 里叫 body）竟**被接受**（rc=$COL_RC）—— 本体加载器只查形状、不查重名 ⇒ REQ-F-030 判据② **不成立**"
fi

# ══ TC-050 · REQ-N-006 投影质量目标（跨进程同源）══════════════════════
echo
echo "── TC-050 · REQ-N-006 投影同源率 / 事实可回溯率（跨进程 ＋ 跨时刻）──"

P1="$(W project language | head -1)"
P2="$(W project language | head -1)"
Q1="$(W project visual | head -1)"
assert_eq "① 跨进程：两次独立运行的语言投影同源头**逐字节相同**" "$P1" "$P2"
assert_eq "② 跨进程：语言与视觉两投影的 (world, vocab, last_seq, state) **四要素逐字节相同**（投影种类标记本就不同，故只比四要素）" \
  "$(hdr4 "$P1")" "$(hdr4 "$Q1")"
assert_has "③ 同源头含**全部四要素**（world / vocab / last_seq / state）" "$P1" 'world=[0-9]+ vocab=\S+ last_seq=[0-9]+ state=\S+'
assert_has "④ project check 自报同源通过" "$(W project check)" '同源.*(一致|通过)'

python3 - "$SB/ontology.json" "$SB/ontology-vocab.json" <<'PY'
import json, sys, collections
src, dst = sys.argv[1], sys.argv[2]
o = json.load(open(src, encoding="utf-8"), object_pairs_hook=collections.OrderedDict)
o["concepts"]["job"]["fields"]["status"] = "enum(todo,doing,done,cancelled)"
json.dump(o, open(dst, "w", encoding="utf-8"), ensure_ascii=False, indent=2)
PY
chmod 600 "$SB/ontology-vocab.json"
assert_ne "⑤ 反例：改词表**语义**（枚举取值）⇒ 词表 hash **必须**变（否则换词表检不出来）" \
  "$(printf '%s' "$P1" | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')" \
  "$(WO "$SB/ontology-vocab.json" project language | head -1 | sed -n 's/.*vocab=\([^ ]*\).*/\1/p')"

RT="$(python3 "$SB/cmp_proj.py" visual "$VIS_TXT" "$ST_JSON")"
assert_eq "⑥ 事实可回溯率（M-07）：视觉投影 EXTRA = 0" "0" "$(printf '%s' "$RT" | sed -n 's/^EXTRA=//p')"
assert_ne "⑦ 样本量必须写明（否则「比例」没有口径）——本例样本 = 投影事实数" "0" "$(printf '%s' "$RT" | sed -n 's/^P=//p')"

# ══ TC-051 · REQ-F-019 排版可审计（独立解析器 ＋ 三样本）══════════════
echo
echo "── TC-051 · REQ-F-019 排版字节级样本（独立于 visual::parse() 的第二份解析器）──"

if [ ! -f tools/visual_layout_audit.py ]; then
  bad "① 缺 tools/visual_layout_audit.py（独立审计脚本）——用例不可执行"
else
  W project visual >"$SB/sample_normal.txt" 2>/dev/null
  W append change '{"subject":"world://sys/nl","path":"esc","before":null,"after":"a\nb\tc"}' >/dev/null 2>&1
  assert_rc "① 含换行/控制字符的值：追加成功（rc=0）" 0 "$?"
  W project visual >"$SB/sample_newline.txt" 2>/dev/null
  : >"$SB/empty.jsonl"
  chmod 600 "$SB/empty.jsonl"
  WO2 "$SB/ontology.json" "$SB/empty.jsonl" project visual >"$SB/sample_empty.txt" 2>/dev/null
  assert_eq "② 三样本齐备（普通值 / 含换行·控制字符的值 / 空状态）" "3" "$(ls "$SB"/sample_*.txt | wc -l | tr -d ' ')"

  for S in normal newline empty; do
    OUT="$(python3 tools/visual_layout_audit.py --file "$SB/sample_$S.txt" 2>&1)"; RC=$?
    assert_rc "③ 样本 [$S]：**独立解析器**判定通过（rc=0）" 0 "$RC"
    if [ "$RC" -ne 0 ]; then printf '%s\n' "$OUT" | head -6 | sed 's/^/      /'; fi
  done

  # 反假④：把渲染输出**改坏一处**（字段行缩进由 6 空格改 5）⇒ 独立审计器**必须**报红
  sed 's/^      /     /' "$SB/sample_normal.txt" >"$SB/sample_mutated.txt"
  python3 tools/visual_layout_audit.py --file "$SB/sample_mutated.txt" >/dev/null 2>&1
  assert_rc "④ 反假：把字段行缩进改坏 ⇒ 独立审计器**必须**报红（rc=1）" 1 "$?"

  ST="$(python3 tools/visual_layout_audit.py --self-test 2>&1)"; SRC=$?
  assert_rc "⑤ 独立审计脚本自带 --self-test：判定器会红、三份期望样本全绿" 0 "$SRC"
  assert_has "⑥ --self-test 覆盖了 >= 12 个变异样本" "$ST" '12 个变异'
fi

# ══ TC-052 · REQ-F-031 `trace` 字段判据 (4a)(4b) ══════════════════════
echo
echo "── TC-052 · REQ-F-031 trace 判据 (4a)(4b)（**不依赖**写入入口）──"

mk_bad "$SB/trace_ghost.jsonl" 'ev["trace"] = "no-such-event-id"'
mk_bad "$SB/trace_none.jsonl" 'ev.pop("trace", None)'

# ⚠ 基线必须**当下重算**：TC-051 在上面往本账追加过一条事件，早先的 BASE_STATE 已过时
BASE_STATE="$(W state --json)"
BASE_LANG="$(W project language | head -1)"

TOUT="$(WO2 "$SB/ontology.json" "$SB/trace_ghost.jsonl" state --json 2>&1)"; TR=$?
assert_rc "(4a)① trace 指向**不存在的 id** ⇒ **必须被接受**（v1 不做引用完整性校验，rc=0）" 0 "$TR"
assert_eq "(4a)② 折叠结果与基线**逐字节相同**" "$BASE_STATE" "$TOUT"

NOUT="$(WO2 "$SB/ontology.json" "$SB/trace_none.jsonl" state --json 2>&1)"; NR=$?
assert_rc "(4a)③ 不带 trace ⇒ 同样必须被接受（rc=0）" 0 "$NR"
assert_eq "(4b)④ 带 / 不带 trace 两本账的 state **逐字节相同**" "$TOUT" "$NOUT"

TL="$(WO2 "$SB/ontology.json" "$SB/trace_ghost.jsonl" project language | head -1)"
NL="$(WO2 "$SB/ontology.json" "$SB/trace_none.jsonl" project language | head -1)"
TV="$(WO2 "$SB/ontology.json" "$SB/trace_ghost.jsonl" project visual | head -1)"
assert_eq "(4b)⑤ 带 / 不带 trace ⇒ 语言投影同源头**逐字节相同**" "$TL" "$NL"
assert_eq "(4b)⑥ 带 trace ⇒ 视觉投影的四要素与语言投影**逐字节相同**（结论无关性；投影种类标记本就不同）" \
  "$(hdr4 "$TL")" "$(hdr4 "$TV")"

RD="$(WO2 "$SB/ontology.json" "$SB/trace_ghost.jsonl" read 2>&1)"; RR=$?
assert_rc "(4a)⑦ read 读回带 trace 的事件 ⇒ 必须成功（rc=0）" 0 "$RR"
assert_has "(4a)⑧ read 读回的 trace 值**逐字**保留（未被改写/丢弃）" "$RD" '"trace": ?"no-such-event-id"'

# ── 汇总 ─────────────────────────────────────────────────────────────
echo
echo "== 汇总：断言通过 $PASS 项，断言失败 $FAIL 项；另行**登记**（现状为红、如实记录）$RED 项 =="
if [ "$RED" -gt 0 ]; then
  echo "   ⚠ 登记项**不是**通过项，也**不是**本轮新增缺陷：它们是已知缺口，须在每轮评审重新复算；"
  echo "     集合或数值一变，即视为登记过时。"
fi
if [ "$FAIL" -eq 0 ]; then
  echo "== 结论：断言全通过（TC-042 / TC-046 / TC-047 / TC-048 / TC-049 / TC-050 / TC-051 / TC-052）=="
  exit 0
fi
echo "== 结论：有 $FAIL 项断言未通过（逐项见上；如实登记，不掩盖）=="
exit 1
