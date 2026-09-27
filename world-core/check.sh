#!/usr/bin/env bash
# check.sh —— **一条命令跑通**（S3 准出判据；REQ-N-004 / WC-SDP-001 §3.4）
#
# 为什么需要它：`REQ-N-004`（私有化部署形态）的判据是"**干净环境一条命令跑通**并打印 READY"，
# 而此前只有零散命令（cargo test / world-core check / project check），
# 没有一个**唯一的、能被人和 CI 同时调用**的入口。评审 T-22 指出该缺口；
# `WC-SCMP-001` §8.4 G-07 也把"入口脚本未入库"记为待办。
#
# 本脚本做什么（四步，全部可复现）：
#   ① 构建（--locked，锁依赖）
#   ② 骨架冒烟：在**一次性沙箱**里打开世界 → 必须打印 READY
#   ③ 三条专属验收测试（追加→读回 / 重启→还在 / ★删读模型→重算一致）
#   ④ 两个投影**同源**核对（同一读模型 + 同一词表身份）
#
# ⚠️ 纪律：全程只碰**一次性沙箱账本**（mktemp 目录，退出即删），
#    **绝不触碰真实账本**（依 06-swe-gb 幂等口径修订：命令幂等，数据操作天然不幂等）。
#
# 用法：bash check.sh          （在 world-core/ 内或任意位置均可）
# 退出码：0 = 全通过；非 0 = 任一步失败（与 CI 同为阻断式）
set -euo pipefail

cd "$(dirname "$0")"
BIN="${CARGO_TARGET_DIR:-target}/debug/world-core"   # 2026-09-27：感知 CARGO_TARGET_DIR（干净 target 下此前后 rc=127，见 R4S2-09）

# ── 判定助手：**显式取 rc**，不把判定交给管道语义（`W-06`）──────────────
# 为什么不用 `cmd | tail -N`：那条写法"看起来在留档、实际上把判定权交了出去"——
#   · 它只在 `set -o pipefail` 生效时才把 cmd 的 rc 传出来（换 shell、换一行写法就变恒 0）；
#   · 失败**细节**被 tail 截掉，只能看到最后几行；
#   · 真正的失败若被**工具自己**吞掉（例如命令替换里写反引号导致的
#     `command not found` 只进 stderr、不改 rc），管道这一层**看不见**。
# 现在：先跑、先取 rc、先判定，再打印尾部若干行。失败即中止（与 CI 同为阻断式）。
run_tail() { # run_tail <展示行数> <描述> <命令...>
  local n="$1"; shift
  local what="$1"; shift
  local out rc=0
  # `|| rc=$?`：`set -e` 下裸赋值会因命令替换失败而当场中止，取 rc 的机会都没有；
  # 用 `||` 抑制 `set -e` 并保住真 rc（这正是"显式取 rc"的字面意思）。
  out="$("$@" 2>&1)" || rc=$?
  printf '%s\n' "$out" | tail -n "$n" | sed 's/^/  /'
  if [ "$rc" -ne 0 ]; then
    echo "  ❌ $what 失败（rc=$rc）—— 验证留档不得吞掉失败"
    exit 1
  fi
  printf '  ✅ %s（rc=0）\n' "$what"
}

# ── 判定器自证（`W-06`：先证明"必失败"真的会失败）─────────────────────
if [ "${1:-}" = "--self-test" ]; then
  echo "== check.sh 判定器自证（W-06 验证留档管道不吞错）=="
  if ( run_tail 1 "注入的必失败命令" false ) >/dev/null 2>&1; then
    echo "  ❌ 自证失败：注入的必失败命令竟被判为通过（判定器是装饰）"
    exit 1
  fi
  if ( run_tail 1 "注入的必失败命令（藏在管道左端）" sh -c 'echo boom; exit 3' ) >/dev/null 2>&1; then
    echo "  ❌ 自证失败：管道左端的失败竟被判为通过"
    exit 1
  fi
  echo "  ✅ 自证通过：两条注入的必失败命令都被判为失败（rc 显式判定，不依赖 pipefail）"
  exit 0
fi

echo "== world-core check.sh =="
echo "  目录 : $(pwd)"
echo "  主机 : $(uname -sr)"
echo "  工具 : $(cargo --version 2>/dev/null || echo 'cargo 缺失')"

# ── ① 构建 ───────────────────────────────────────────────────────────
echo
echo "── ① 构建（--locked）────────────────────────────────────────"
cargo build --locked --quiet
echo "  ✅ 构建通过"

# ── ② 骨架冒烟（一次性沙箱）─────────────────────────────────────────
SB="$(mktemp -d)"
trap 'rm -rf "$SB"' EXIT
cp ontology.json policy.json "$SB"/
chmod 755 "$SB"; chmod 644 "$SB/ontology.json" "$SB/policy.json"

echo
echo "── ② 骨架冒烟（沙箱账本：$SB）──────────────────────────────"
OUT="$("$BIN" --ontology "$SB/ontology.json" --ledger "$SB/ledger.jsonl" --policy "$SB/policy.json" check)"
echo "$OUT" | sed 's/^/  /'
echo "$OUT" | grep -q READY || { echo "  ❌ 未打印 READY"; exit 1; }
echo "  ✅ READY（本体/门禁/账本三项都开得起来）"
# 新账本必须带摘要链（WC-SCMP-001《软件配置管理计划》变更请求台账 · 记录 WC-CR-003）：
#   写一条再断言"有链"且 --require-chain 通过。
# 若写入侧哪天不再产链，这一步会当场红——把"默认受保护"变成入口断言。
W0() { "$BIN" --ontology "$SB/ontology.json" --ledger "$SB/ledger.jsonl" --policy "$SB/policy.json" "$@"; }
W0 append change '{"subject":"world://check/probe","path":"p","before":null,"after":1}' >/dev/null
OUT2="$(W0 check)"
echo "$OUT2" | grep -q '有摘要链' || { echo "  ❌ 新账本应带摘要链（WC-SCMP-001 变更请求台账 · WC-CR-003），实得：$(echo "$OUT2" | grep '链')"; exit 1; }
W0 --require-chain check >/dev/null || { echo "  ❌ --require-chain 未通过"; exit 1; }
echo "  ✅ 摘要链：新账本带链，--require-chain 通过"

# ── ③ 三条专属验收测试 ──────────────────────────────────────────────
echo
echo "── ③ 三条专属验收测试（WC-SQAP-001 §2.4）─────────────────"
cargo test --locked --test acceptance -- t1_ t2_ t7_ 2>&1 | grep -E 'running [0-9]+ tests|test result:' | sed 's/^/  /'
echo "  ✅ 追加→读回 / 重启→还在 / ★删读模型→重算一致"

echo
echo "── ③b 契约测试（覆盖 M05 门禁 / M08 检查点 / M09 通道）──────"
cargo test --locked --test contract 2>&1 | grep -E 'running [0-9]+ tests|test result:' | sed 's/^/  /'
echo "  ✅ 门禁失败路径 / 静态墙 / 单写者 / 检查点 / 通道身份 / 错误码契约"

# ── ④ 两个投影同源 ──────────────────────────────────────────────────
echo
echo "── ④ 投影与同源核对（REQ-F-018/019/020）──────────────────"
W() { "$BIN" --ontology "$SB/ontology.json" --ledger "$SB/ledger.jsonl" --policy "$SB/policy.json" "$@"; }
W append change '{"subject":"world://notice/n-1","path":"muted","before":null,"after":true}' >/dev/null
W append act '{"capability":"notice.mute","verb":"do","request_id":"r-check","params":{}}' >/dev/null
# ⚠️ 这两行**不是展示行**：其退出码已参与判定——`set -euo pipefail` 下，赋值语句里的管道失败
#    （含 head 的 SIGPIPE）会使整个 check.sh 非零退出。原先写成 echo "... $(W ... | head -1)"，
#    失败被吞在 echo 的实参里、不参与判定。2026-09-27 按三评委判词改为"先赋值再 echo"（只增不减）。
L1="$(W project language | head -1)"
L2="$(W project visual | head -1)"
echo "  语言投影首行: $L1"
echo "  视觉投影首行: $L2"
W project check | sed 's/^/  /'

echo
echo
echo "── ⑤ 纯文本审计（REQ-N-001 / AC-07）────────────────────"
run_tail 2 "纯文本审计" python3 tools/plain_text_audit.py ontology.json policy.json "$SB/ledger.jsonl"
echo "  ✅ 账本/词表/策略均为纯文本（UTF-8、无 NUL、无可疑控制字符、逐行可解析）"

echo
echo "── ⑥ 系统级验收（TC-037–TC-040，真实二进制端到端）──────────"
# 为什么放在这里：`cargo test` 验模块与接口（L1/L2），本步验**产物本身**（L3）——
# 只看退出码、真实文件字节与命令输出。它同时是 S5 起 `RTM_STRICT=true` 的
# 系统级/验收级证据（见 WC-SCMP-001 §8.4 G-16）。
run_tail 1 "系统级验收判定器自证" bash tools/system_acceptance.sh --self-test
run_tail 3 "系统级验收（TC-037–TC-040）" bash tools/system_acceptance.sh

echo
echo "── ⑦ S1 需求验证面补建（第一轮 TC-042/046–052；第二轮 TC-053–TC-075）──"
# 为什么放在这里：R1 的九席独立评审实测指出，SRS §五 声明的一批用例**从未实存**，
# 而若干 `REQ-F-*` 的「应当失败」反例**只挂在这些不存在的用例上**——
# 按本项目逐字纪律「反例不红视为未校验」，那些条目当时**不可校验**。
# 本步把其中**不需要改产品代码即可执行**的部分补建成真实断言，并**显式计数**「现状为红」的登记项
# （登记项不影响退出码，但必须每轮复算 —— 防"把已知缺陷藏进绿灯里"）。
run_tail 1 "S1 验证面补建①判定器自证" bash tools/s1_sys_probe.sh --self-test
run_tail 4 "S1 验证面补建①（TC-042/046–052）" bash tools/s1_sys_probe.sh
run_tail 1 "S1 验证面补建②判定器自证" bash tools/s1_sys_probe2.sh --self-test
run_tail 4 "S1 验证面补建②（TC-053–TC-076）" bash tools/s1_sys_probe2.sh
run_tail 1 "排版审计判定器自证" python3 tools/visual_layout_audit.py --self-test
run_tail 1 "契约分册门禁判定器自证" python3 tools/ic_books_check.py --self-test
run_tail 8 "契约分册门禁（九册齐·要点齐·依赖列逐边一致）" python3 tools/ic_books_check.py
# 表块行宽审计（**转义感知**）：`\|` 是单元格内的**字面竖线**，朴素的 `s.count("|")` 会把
# **本来就正确**的行报成「体行行窄」（R1 三轮席 S-07 的 `G-20` 即此类误报；本轮 8 处复算 = 0）。
# 该行**不过滤退出码**：真有不符 ⇒ 本步直接失败（既防误报、也防漏报）。
run_tail 1 "表块行宽审计判定器自证" python3 tools/table_width_audit.py --self-test
run_tail 20 "表块行宽审计（转义感知）" python3 tools/table_width_audit.py "docs/S1-需求/WC-IRS-001-v0.1.md" "docs/S1-需求/WC-SRS-001-v0.1.md" "docs/评审/WC-RV-R1-001-v0.1.md"

# 2026-09-27 修（W-06）：本步原先登记的三类噪声里，(c) `WC-SQAP-001: command not found`
#   已**在源头消除**（`s1_sys_probe2.sh` 描述串内的反引号改成字面词，不再做命令替换）；
#   (a)(b) 两类仍属无害噪声、**不改判据强度**，故保留。

echo
echo "── ⑧ 规格层守卫（OpenSpec 层：五条判据）─────────────────────"
# 为什么放在这里：`openspec validate` 只判**形态**（结构、每个 Scenario 恰好 4 个 `#`、delta 语法），
# 它**不查**：证据行指向的测试是否真的存在（改名即失锚，且不会变红）、归档目录有没有 `review.md`、
# 默认档是不是融合档、编号桥有没有覆盖规格树下每条 Requirement、承载覆盖缺口的 change 还在不在。
# 这五条此前**只写在 schema 的文字里，没有任何执行者**——实测：一个**没有** `review.md` 的 change
# `openspec archive --yes` 照样 rc=0 归档。`tools/spec_bridge.py` 就是这五条的执行者。
# 它自己也要能自证会红（`--self-test`：五条各造一个反例，反例不变红即判该守卫是装饰）。
# 仓库根由脚本自身位置向上定位（world-core/tools/ → 仓库根）；VM 上已同步 `openspec/` 层，故两边都能跑。
run_tail 2 "规格层守卫自证（五条判据各造反例，反例必红）" python3 tools/spec_bridge.py --self-test
run_tail 8 "规格层守卫（归档硬前置／证据存在性／默认档／编号桥／覆盖在册）" python3 tools/spec_bridge.py

echo
echo "== 结论：全通过（构建 / 冒烟 / 三条专属测试 / 契约测试 / 投影同源 / 纯文本审计 / 系统级验收 / S1 验证面补建 / 规格层守卫）=="

echo
echo "── ⑨ 机核层守卫（WC-ATOM-001 §四 机核清单：单意图／四件同夹／deps==import 且无环）──"
# 为什么放在这里：`WC-ATOM-001`（原子化编程约定，本项目**强制**）§四 机核清单第 1–3 条
# 此前**没有执行者**（该表自己写着「未建」/「未建闸」）。三条判据：
#   ① A-1 每个模块有且只有一句 `intent`（≤30 字）；出现并列两事（与／和／及）即报可疑；
#   ② A-4 从 `src/**/*.rs` 抽 `mod`／`use crate::`／根级裸名 `use <mod>::` 建模块图，
#      声明依赖集必须**逐模块逐边等于**真实 import 集，且图必须无环（拓扑排序失败即红）；
#   ③ A-2 每模块的实现／测试／契约三件都要有落点（本仓 `tests/` 与 `src/` 不同夹，
#      故按"可指认"判：实现＝登记表源码路径真实存在；测试＝`tests/` 里有用例能指到它；
#      契约＝`WC-IC-M*` 分册或规格里一条 Requirement 的证据行）。
# 今天这三条**应该是红的**（本项目尚未按原子化组织）——红就如实报红；
# 交付的是"判据立起来且会红"，不是把红刷成绿。
run_tail 12 "机核层守卫（WC-ATOM-001 §四：单意图／四件同夹／deps==import 且无环）" python3 tools/module_graph.py
