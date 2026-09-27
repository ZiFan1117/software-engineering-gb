#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""spec_bridge.py —— 规格层的守卫：把「应当有人拦」的判据做成会真红的东西。

为什么需要它
------------
OpenSpec 的 `validate` 只判**形态**（结构、Scenario 个数、delta 语法），它**不查**：
  · 证据行指向的测试是否真的存在（改名即失锚，且不会变红）
  · 归档目录里有没有评审记录 `review.md`
  · 默认档是不是融合档
  · 编号桥映射表有没有覆盖规格树下的每条 Requirement
  · 承载覆盖缺口的 change 还在不在（未归档）
  · 归档件签没签字、件里有没有掺修订记录／改因块
  · delta 的 ADDED 标题是否与主规格撞车（撞了就**永远归不了档**）
  · 生成物 `BRIDGE.md` 与它的生成器是否还同步
而这些恰恰是 `opsx-swe-gb` 的文字里承诺过的。**一个从不失败的检查不是装饰，是假证。**
本脚本就是它们的**执行者**：任一条不成立即非零退出。

判据（与 `specs/spec-governance/spec.md` 逐条对应）——**条数以 `JUDGMENTS` 为准，条数由 `JUDGMENTS` 长度现算——**不写死**（写死过一次：加判据时这里就烂了）**
-------------------------------------------------------
① 归档硬前置      每个 `openspec/changes/archive/*/` 必须有非空 `review.md`
② 证据存在性      `openspec/specs/**/spec.md` **与 delta** 里每条 `- **证据**：<token>` 的
                   `<path>::<fn>` 或 `<path> --self-test` 必须真实存在
③ 默认档守卫      `openspec/config.yaml` 的 `schema:` 必须为 `opsx-swe-gb`
④ 编号桥覆盖      `openspec/BRIDGE.md` 必须覆盖规格树下**每一条** Requirement（有号或显式标无号）
⑤ 覆盖在册        `openspec/changes/cover-*/` 至少有一个**未归档**、且 `tasks.md` 仍有未勾项
⑥ 评审已签        归档件的 `review.md` 结论 ∈ {批准,通过,有条件通过}，且批准人栏非空、非占位
⑦ 让路登记        件里声明了「谁让」，三要素就必须写全（让的是哪一条／为什么要让／谁批的）
⑧ 无修订记录      `world-core/docs/**`（书除外）不许有修订记录节——**标题式／加粗式／表格式都拦**
⑨ 无改因块        规格正文（**主规格 ＋ delta**）不许有 `> **改的是哪一类问题**…` 这类块
⑩ ADDED 不撞车    未归档 change 的 **ADDED** 标题不许与主规格逐字相同（撞了归档必被拒）
⑪ 生成物同步      `openspec/BRIDGE.md` 必须与 `openspec/tools/gen_bridge_md.py` 的当前输出逐字节一致

用法
----
    python3 tools/spec_bridge.py [--repo <仓库根>] [--json]
    python3 tools/spec_bridge.py --self-test     # 为**每条**判据各造一个反例（条数随 `JUDGMENTS` 增长，加一条判据必须同时加一个反例），反例不变红即判装饰
"""

import argparse
import contextlib
import hashlib
import io
import json
import os
import re
import shutil
import sys
import tempfile
from pathlib import Path

SCHEMA_NAME = "opsx-swe-gb"
EVIDENCE_RE = re.compile(r"-\s*\*\*证据\*\*：(.+)$")
REQ_RE = re.compile(r"^###\s+Requirement:\s*(.+?)\s*$")

# 非 UTF-8 控制台（Windows GBK/cp936）下，中文与记号会让 print 抛 UnicodeEncodeError
# —— 那会变成"门禁自己崩了"的假失败。按本项目既有口径：**只重配 errors，不改 encoding**
# （UTF-8 环境下逐字节等价）；记号一律用 ASCII，避免依赖控制台字体。
for _s in (sys.stdout, sys.stderr):
    try:
        _s.reconfigure(errors="replace")
    except Exception:
        pass


# ────────────────────────── 工具 ──────────────────────────
def find_repo(start):
    """从 start 往上找含 openspec/specs 的目录。"""
    p = Path(start).resolve()
    for cand in [p] + list(p.parents):
        if (cand / "openspec" / "specs").is_dir():
            return cand
    return None


def rel(repo, p):
    try:
        return str(Path(p).resolve().relative_to(Path(repo).resolve())).replace("\\", "/")
    except Exception:
        return str(p)


def read_text(p):
    try:
        return Path(p).read_text(encoding="utf-8", errors="replace")
    except Exception:
        return ""


def resolve_src(repo, p):
    """证据行里的路径按仓库根或 world-core/ 解析（两种基准都试，写死口径）。"""
    for cand in (Path(repo) / p, Path(repo) / "world-core" / p):
        if cand.is_file():
            return cand
    return None


# ────────────────────── 判据（条数以 `JUDGMENTS` 为准，条数由 `JUDGMENTS` 长度现算——**不写死**（写死过一次：加判据时这里就烂了））──────────────────────
def j1_archive_review(repo):
    bad = []
    arch = Path(repo) / "openspec" / "changes" / "archive"
    if arch.is_dir():
        for d in sorted(arch.iterdir()):
            if not d.is_dir() or d.name.startswith("."):
                continue
            rv = d / "review.md"
            if not rv.is_file():
                bad.append("%s —— 缺 review.md（归档硬前置）" % rel(repo, d))
            elif rv.stat().st_size == 0:
                bad.append("%s —— review.md 是空文件" % rel(repo, rv))
    return bad


def looks_like_location(tok):
    """这个反引号 token 是不是**一个位置**（＝在声称"某处存在某物"）？

    为什么需要这一层（2026-09-27 扩面到 delta 后一次照出的）：
    判据② 的扫描面一扩到 delta，立刻报了 21 条"脚本不存在"，但逐条看全是**位置引用**与**行文强调**——
    `world-core/src/gate.rs:44-48`（源码位置）、`risk`／`K-3`／`REQ-F-015`（标识符）、`:32`／`【未能校验】`（行文）。
    这些**不是"声称某脚本存在"**，拿"脚本是否存在"去判它们是**判据用错了尺子**。
    ⇒ 只对"长得像位置"的 token 追存在性：
      ① 带路径分隔符：`world-core/...`、`openspec/...`
      ② 带代码/文档后缀：`.rs`／`.py`／`.sh`／`.md`／`.json`／`.csv`／`.yaml`／`.yml`／`.toml`
      ③ 带 `::`（`文件::函数` 形态）
    其余一律按**行文**处置（不追存在性）。**delta 因此不必为迁就判据而改写**
    （合并时立下的不变式是"delta 逐字节未动"）。
    """
    if "::" in tok:
        return True
    if "/" in tok or "\\" in tok:
        return True
    low = tok.lower()
    return low.endswith((".rs", ".py", ".sh", ".md", ".json", ".csv", ".yaml", ".yml", ".toml"))


def strip_line_suffix(tok):
    """`world-core/src/gate.rs:44-48` ⇒ `world-core/src/gate.rs`（**行号是引用的一部分，不是路径**）。

    只剥**末尾**的 `:数字` 或 `:数字-数字`；`path::fn` 形态不在此处理（它有 `::`，走 check_token 的形态 A）。
    """
    return re.sub(r":\d+(?:-\d+)?$", "", tok.strip())


def check_token(repo, tok):
    tok = tok.strip()
    if "::" in tok:                                   # 形态 A：文件::函数名
        p, fn = tok.split("::", 1)
        f = resolve_src(repo, p.strip())
        if f is None:
            return False, "文件不存在（按仓库根与 world-core/ 两种基准都找不到）"
        src = read_text(f)
        if re.search(r"\bfn\s+" + re.escape(fn.strip()) + r"\s*[(<]", src):
            return True, ""
        return False, "文件在，但函数不存在"
    p = strip_line_suffix(tok)                        # 形态 B：路径（可带 :行号）／脚本（可带参数）
    parts = p.split()
    if parts:
        f = resolve_src(repo, parts[0])
        if f is None:
            return False, "脚本不存在"
        return True, ""
    return False, "无法解析的token"


def j2_evidence(repo):
    """② 证据存在性：证据行的 token 必须指向真实存在的函数/脚本。

    **扫描面含 delta**（2026-09-27 扩）：`openspec/changes/**/specs/**/spec.md` 里的证据行
    在**归档合并**时才会进主规格——只扫主规格等于**漏检一整片**，而且是在归档那一刻才红（太晚）。
    实测：扩面前 delta 侧有 1 条无 token 的证据行（`fc-2026-002/specs/ledger-integrity/spec.md:189`），
    当天不变红、合并后必红；已按同一口径改为 `- **证据（待补）**：`。
    """
    bad = []
    roots = [(Path(repo) / "openspec" / "specs", "spec.md")]
    ch = Path(repo) / "openspec" / "changes"
    if ch.is_dir():
        roots.append((ch, "spec.md"))                   # delta：归档时会并入主规格 ⇒ 同一把尺子
    for root, pat in roots:
        for spec in sorted(root.rglob(pat)):
            if "archive" in spec.parts:                 # 归档件自有历史口径，不追改
                continue
            for i, line in enumerate(read_text(spec).splitlines(), 1):
                m = EVIDENCE_RE.search(line)
                if not m:
                    continue
                toks = re.findall(r"`([^`]+)`", m.group(1))
                if not toks:
                    bad.append("%s:%d —— 证据行里没有反引号包起来的 token。"
                               "**若本条尚无断言，请改用 `- **证据（待补）**：` 并写明落点**——"
                               "用「证据」这个标记而不给 token，形态上等于声称存在" % (rel(repo, spec), i))
                    continue
                for tok in toks:
                    if not looks_like_location(tok):
                        continue                        # 行文强调／标识符：不追存在性（见 looks_like_location）
                    ok, why = check_token(repo, tok)
                    if not ok:
                        bad.append("%s:%d —— `%s`：%s" % (rel(repo, spec), i, tok, why))
    return bad


def j3_default_schema(repo):
    cfg = Path(repo) / "openspec" / "config.yaml"
    if not cfg.is_file():
        return ["%s —— 文件不存在" % rel(repo, cfg)]
    for i, line in enumerate(read_text(cfg).splitlines(), 1):
        m = re.match(r"^schema:\s*(\S+)\s*$", line)
        if m:
            if m.group(1) == SCHEMA_NAME:
                return []
            return ["%s:%d —— 默认档是 `%s`，应为 `%s`（被改回即失败；忘了加 --schema 会静默走回）"
                    % (rel(repo, cfg), i, m.group(1), SCHEMA_NAME)]
    return ["%s —— 找不到 `schema:` 行" % rel(repo, cfg)]


def iter_requirement_titles(repo):
    out = []
    for spec in sorted((Path(repo) / "openspec" / "specs").rglob("spec.md")):
        for i, line in enumerate(read_text(spec).splitlines(), 1):
            m = REQ_RE.match(line)
            if m:
                out.append((rel(repo, spec), i, m.group(1)))
    return out


def j4_bridge_coverage(repo):
    bridge = Path(repo) / "openspec" / "BRIDGE.md"
    if not bridge.is_file():
        return ["%s —— 编号桥映射表不存在（规格树下每条 Requirement 都必须在此在册）" % rel(repo, bridge)]
    text = read_text(bridge)
    bad = []
    for f, i, title in iter_requirement_titles(repo):
        if title not in text:
            bad.append("%s:%d —— `%s` 不在编号桥映射表里（既没给号，也没标「无号」）" % (f, i, title))
    return bad


def j5_coverage_change(repo):
    ch = Path(repo) / "openspec" / "changes"
    if not ch.is_dir():
        return ["%s —— changes 目录不存在" % rel(repo, ch)]
    found = []
    for d in sorted(ch.iterdir()):
        if not d.is_dir() or d.name.startswith(".") or d.name == "archive":
            continue
        if not d.name.startswith("cover-"):
            continue
        t = d / "tasks.md"
        if not t.is_file():
            continue
        if re.search(r"^\s*-\s*\[ \]", read_text(t), re.M):
            found.append(d.name)
    if found:
        return []
    return ["openspec/changes/ —— 找不到「未归档且 tasks 仍有未勾项」的覆盖 change（cover-*）；"
            "未实现的能力失去落点，等于把「未定」当「已定」"]


# 结论栏里"已签"的取值；其余（待签／退回／驳回／空缺）一律判未签
SIGNED = ("批准", "通过", "有条件通过")


def _verdict_of(review_text):
    """从 review.md 里取「结论」栏的取值（支持表格式与 `**结论**：x` 两种写法）。"""
    for line in review_text.splitlines():
        if "结论" not in line:
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        for idx, c in enumerate(cells):
            if "结论" in c and idx + 1 < len(cells):
                v = cells[idx + 1].strip("* 　")
                if v:
                    return v
        m = re.search(r"结论\D{0,4}[:：]\s*(.+)$", line.strip())
        if m:
            return m.group(1).strip("* 　")
    return ""


def j6_archived_review_signed(repo):
    """⑥ 归档件的评审必须**已签**：结论 ∈ {批准,通过,有条件通过}，且批准人栏非空、非占位。

    **为什么单列一条**：判据① 只判"在场"，它是**内容盲**的——一份"结论：待签"的 review 照样通过它。
    评审没签字却在账上记成"已归档"，正是本项目最忌的形态（把未定当已定）。
    **只对归档件判红**：在办件还没到归档，不该被这条挡住。
    """
    bad = []
    arch = Path(repo) / "openspec" / "changes" / "archive"
    if not arch.is_dir():
        return []
    for d in sorted(arch.iterdir()):
        if not d.is_dir() or d.name.startswith("."):
            continue
        rv = d / "review.md"
        if not rv.is_file():
            continue                                     # 在场与否归判据①
        text = read_text(rv)
        verdict = _verdict_of(text)
        if not verdict:
            bad.append("%s —— review.md 里读不出「结论」栏（签字留人不等于可以没有结论栏）" % rel(repo, rv))
            continue
        if not any(verdict.startswith(k) for k in SIGNED):
            bad.append("%s —— 结论 =「%s」：**未签**（已签应为 %s 之一）；"
                       "归档前必须签，缺签一律回退补签" % (rel(repo, rv), verdict, "/".join(SIGNED)))
            continue
        # ★ 2026-09-28 修：原来取**文件里第一处**「批准人」字样（不管它是不是**栏位**），
        #   于是正文里提一句"批准人"就会把它带到错误位置、捕到空 ⇒ **误判**。
        #   现在只认**栏位形态** `| **批准人** | 值 |`，且**列出全部栏位逐一检查**。
        cells = re.findall(r"\|\s*\*\*批准人\*\*\s*\|([^|\n]*)", text)
        if not cells:
            bad.append("%s —— 结论已签，但**找不到「批准人」栏位**（应为 `| **批准人** | 姓名 |`）"
                       % rel(repo, rv))
            continue
        for cell in cells:
            who = cell.strip().strip("*\u3000 ")
            if (not who) or ("待" in who) or ("补姓名" in who) or who in ("—", "-", "无"):
                bad.append("%s —— 结论已签，但**批准人栏是空的或占位**（实得：%s）" % (rel(repo, rv), who or "空"))
    return bad


# 「让路三要素」：任何一次"不按书来"的处置，三样都必须写全（书自己的纪律：冲突时要说明谁让）
WAIVER_KEYS = (("让的是哪一条", r"让的是哪一条"),
               ("为什么要让", r"为什么要让|为什么让"),
               ("谁批的", r"谁批的|谁批"))


WAIVER_LABEL_RE = re.compile(r"(^#{1,6}[^\n]*谁让)|(\*\*谁让\*\*)|(让的是哪一条)", re.M)


def _strip_code_blocks(text):
    """去掉围栏代码块——**引用的原始输出不是声明**。

    2026-09-27 实测误报：`fc-2026-002-spec-revisions/tasks.md` 把门禁的原始输出粘进件里，
    那段输出含「⑦ 让路登记（声明了「谁让」的件必须写全…）」⇒ 判据⑦ 把它当成让路声明而误报。
    ⇒ 本判据只在**正文**里找声明与三要素，围栏代码块一律不参与。
    """
    return re.sub(r"```.*?```", "", text, flags=re.S)


def j7_waiver_registered(repo):
    """⑦ 让路登记：件里只要声明了"谁让"，三要素就必须写全。

    为什么单列一条：书自己的纪律说「规格与流程文档**不受**十条写作纪律约束，但**冲突时要说明谁让**」；
    而 `schemas/README.md` §〇 又写「不写＝违规」。半写的让路（只写"谁让"两字、不写让哪一条／为什么／谁批的）
    与不写等价——这是"承诺与实现不符"的又一处入口。
    **只在件里出现了"谁让"时才检**：不声明让路的 change 不会被这条误伤。
    """
    bad = []
    ch = Path(repo) / "openspec" / "changes"
    if not ch.is_dir():
        return []
    for d in sorted(ch.iterdir()):
        if not d.is_dir() or d.name.startswith(".") or d.name == "archive":
            continue
        files = sorted(d.rglob("*.md"))
        # 按**件整体**判：三要素只要在该 change 的任一产物里写全即可（不必挤在同一份文件里）
        # **围栏代码块不参与**：引用的原始输出不是声明。
        # （2026-09-27 实测误报：fc-2026-002/tasks.md 粘了门禁原始输出，输出里含判据⑦ 的名字 ⇒ 被当成声明。）
        union = _strip_code_blocks("\n".join(read_text(f) for f in files))
        # **只在"真的在登记让路"时才检**：判据要的是**结构化的声明**，不是顺口提到的两个字。
        # 2026-09-27 实测误报：`fc-2026-003/design.md:5` 写「…一套是『我曾经写错什么、谁让我这么改的』」
        # ——那是行文里的顺口话，不是让路声明，却被裸子串匹配抓成"声明了让路却缺三要素"。
        # ⇒ 触发条件改为：**标题里带「谁让」**、或 **`**谁让**` 加粗标签**、或 **出现三要素的第一个标签「让的是哪一条」**。
        if not WAIVER_LABEL_RE.search(union):
            continue                                     # 没有结构化声明 ⇒ 不受本条约束
        missing = [label for label, pat in WAIVER_KEYS if not re.search(pat, union)]
        if missing:
            where = "、".join(rel(repo, f) for f in files)
            bad.append("%s —— 声明了让路，却缺 %s（书纪律：冲突时要说明谁让；半写＝不写）"
                       % (rel(repo, d), "、".join("「%s」" % m for m in missing)))
            bad.append("      （本条按件整体判、且**只查正文**（围栏代码块不参与）：查的是 `%s` 的全部 `.md`）" % where)
    return bad


REV_KEY = r"(修订记录|变更记录|修订历史)"
# ⑧ 的**三种形态**——旧口径只认标题式（`^#{1,4}`），而**实盘的真实违规没有一处带 `#`**
# ⇒ 旧守卫在真违规上照样报 [OK]（假绿）。2026-09-27 独立评审席注入实验逐字：
#     向 `world-core/docs/S0-立项/WC-ATOM-001-v0.1.md` 追加一行 `**修订记录**` ⇒ rc=1、⑧ 报 [OK]、offender = []
#     再追加一行 `### 修订记录` ⇒ ⑧ 变 [FAIL]、offender 1 条
# 形态一 · 标题式：`### 修订记录`
REVISION_HEAD_RE = re.compile(r"^#{1,4}\s*[^|]*?" + REV_KEY)
# 形态二 · 加粗式：行的**首个可见内容**就是加粗的那三个词（可带 `>`／`#` 前缀）
#   实盘逐字：`world-core/docs/S0-立项/WC-ATOM-001-v0.1.md:67` = `**修订记录**`
#   实盘逐字：`world-core/docs/评审/WC-RV-R0-001-v0.1.md:2958` = `> **修订记录**：**V0.2（2026-09-27）**…`
#   ⚠ 不误伤「不设修订记录」的正当声明（实盘 14 处）：`> **本文件不设修订记录**：…` 的加粗 span
#     是「本文件不设修订记录」，**不是**这三个词本身 ⇒ 不命中（自证里有 `对照⑧n` 钉着）。
REVISION_BOLD_RE = re.compile(r"^\s*(?:[>#]+\s*)*(?:[-*+]\s+)?\*\*\s*" + REV_KEY + r"\s*\*\*")
# 形态三 · 表格式：表格行里以那三个词为**首列**词（首列可加粗）
#   实盘逐字：`world-core/docs/S0-立项/WC-SCMP-001-v0.1.md:1845` = `| 修订记录 | **V0.2（2026-09-27）**… |`
REVISION_TABLE_RE = re.compile(r"^\s*\|+\s*\**\s*" + REV_KEY + r"\s*\**\s*\|")
# 表格式的**必要条件**（2026-09-27 精确化：**减少误报，不是放宽**）：
#   实盘 `world-core/docs/S1-需求/WC-IRS-001-v0.1.md:364` 逐字 `| 变更记录 | 本文档 §十二 登记；`WC-IC-001` 同步 |`
#   —— 它在一张**属性表**里说的是"变更登记在哪一节"：**没有版本号、没有日期、没有"改了什么"** ⇒ 那是**指路**，不是记录。
#   ⇒ 表格行必须**同时**出现"像记录"的东西：版本号（`V0.2`／`v0.1` 一类）或日期（`2026-09-27` 一类）。
REV_TABLE_RECORD_RE = re.compile(r"([Vv]\d+(?:\.\d+)+|\d{4}-\d{2}-\d{2})")
# ⑧ 的**豁免目录**（按 `world-core/docs/` 下的目录名；豁免范围与理由一并写在这里，别让下一个人以为漏了）：
#   · `理论`：那是**作者的书**，其附录体例由作者定（既有口径，未改）
#   · `评审`：评审记录里**逐字留存了别人提交的文档**（"并入件"）⇒ 改它＝**篡改记录**（与"书除外"同一个道理）
#   **豁免范围只到 `docs/评审/`**；`docs/S0-立项/`…`docs/S4-实现/`、`docs/阶段外-待启用/` **一律照判**。
REV_EXEMPT_TOPDIRS = ("评审",)
RATIONALE_HEAD_RE = re.compile(r"^> \*\*(改的是哪一类问题|为什么用 ADDED|证据是哪条测试)")
# ⑩ 用：delta 的 ADDED 节标题、以及 Requirement 标题（`REQ_RE` 见文件头）
ADDED_HEAD_RE = re.compile(r"^##\s+ADDED\s+Requirements\s*$")


def revision_hit(line):
    """这一行是不是「修订记录」的三种形态之一？返回形态名，否则 None。

    表格式带一条**必要条件**：要像记录（得**有版本号或日期**），否则按"指路"看待——
    理由与实盘反例见 `REV_TABLE_RECORD_RE` 上方（`--self-test` 的 `对照8p` 钉着它）。
    """
    s = line.strip()
    if REVISION_HEAD_RE.match(s):
        return "标题式"
    if REVISION_BOLD_RE.match(s):
        return "加粗式"
    if REVISION_TABLE_RE.match(s) and REV_TABLE_RECORD_RE.search(s):
        return "表格式"
    return None


def j8_no_revision_log_in_docs(repo):
    """⑧ 流程文档不许有"修订记录"节（**修订记录＝git 提交历史**）。

    出处：作者指示「那几个文档里面也不要掺和这种什么修订的记录啥的」⇒ `.agents/skills/worldcore-sdd/SKILL.md` §三。
    为什么单列一条：正文是给读者用的（他要的是"现在是什么"），不是给作者记账用的；
    掺在一起，读者得在一堆"我曾写错什么"的括号里找那条规矩。

    **判三种形态**（2026-09-27 修：旧口径只认 `^#{1,4}` 的标题式 ⇒ **真违规抓不到**）：
      · 标题式 `### 修订记录`
      · 加粗式 `**修订记录**`（下面常接一张表）——实盘 `WC-ATOM-001-v0.1.md:67` 逐字就是这一种
      · 表格式 `| 修订记录 | … |`（那三个词作**首列**词）——实盘 `WC-SCMP-001-v0.1.md:1845` 逐字。
        带**必要条件**：行里得有版本号或日期（否则是"指路"不是记录，见 `REV_TABLE_RECORD_RE`）。
    实盘真实违规**没有一处带 `#`**；三种形态由 `revision_hit()` 一处判定，`--self-test` 每种形态各一个反例。

    **豁免两处**（`REV_EXEMPT_TOPDIRS` ＋ 书）：`docs/理论/`（作者的书）与 `docs/评审/`（评审记录里逐字留存了
    别人的文档，改它＝篡改记录）。其余目录（`S0-立项`…`S4-实现`、`阶段外-待启用`）**一律照判**。
    """
    bad = []
    docs = Path(repo) / "world-core" / "docs"
    if not docs.is_dir():
        return []
    for f in sorted(docs.rglob("*.md")):
        if "理论" in f.parts:                           # 书：作者的作品
            continue
        top = f.relative_to(docs).parts[0]              # 豁免只认 `docs/` 下**顶层**目录名
        if top in REV_EXEMPT_TOPDIRS:                   # 评审记录：含别人文档的逐字留存
            continue
        for i, ln in enumerate(f.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
            form = revision_hit(ln)
            if form:
                bad.append("%s:%d —— 有修订记录节（%s）「%s」；**修订记录＝git 提交历史**，正文只写「现在是什么」"
                           % (rel(repo, f), i, form, ln.strip()[:48]))
    return bad


def j9_no_rationale_in_specs(repo):
    """⑨ 规格正文不许有"改因块"（改因属该 change 的 `design.md`／`audit.md`）。

    出处：同上 skill §三。实测混在正文里 76 个这样的块，形态门禁因此判 28 条
    `Requirement text is very long`——**正文只写"世界必须怎样"**。

    **扫描面含 delta**（2026-09-27 扩，与判据② 同一口径）：delta 在**归档合并**时才会进主规格，
    只扫主规格等于**漏检一整片**，而且是到归档那一刻才红（太晚）。
    实测（独立评审席注入实验）：
      向 `openspec/changes/fc-2026-002-spec-revisions/specs/read-model/spec.md` 追加
      `> **改的是哪一类问题**：…` ⇒ 旧口径下 rc=1、⑨ 报 [OK]、offender = []
      ；同一句注入**主规格** `openspec/specs/read-model/spec.md` ⇒ ⑨ 变 [FAIL]。
    """
    bad = []
    roots = [Path(repo) / "openspec" / "specs"]
    ch = Path(repo) / "openspec" / "changes"
    if ch.is_dir():
        roots.append(ch)                                # delta：归档时会并入主规格 ⇒ 同一把尺子
    for root in roots:
        if not root.is_dir():
            continue
        for f in sorted(root.rglob("spec.md")):
            if "archive" in f.parts:                    # 归档件自有历史口径，不追改
                continue
            for i, ln in enumerate(f.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
                if RATIONALE_HEAD_RE.match(ln.strip()):
                    bad.append("%s:%d —— 规格正文里有改因块「%s…」；**改因归该 change 的 `design.md`／`audit.md`**"
                               % (rel(repo, f), i, ln.strip()[:44]))
    return bad


def j10_delta_added_not_colliding(repo):
    """⑩ 未归档 change 的 **ADDED** 标题不许与主规格撞车——撞了它**永远归不了档**。

    为什么单列一条（2026-09-27 独立评审席沙箱实跑）：
      `openspec archive fc-2026-002-spec-revisions --yes`
      ⇒ **rc=1**，逐字
      `channel-identity ADDED failed for header "### Requirement: 通道身份的实际保证与它的边界" - already exists`、
      `Aborted. No files were changed.`
    根因：`4a4ab4e` 那次合并已把该 delta 的 **10 条 ADDED** 并进主规格，而 delta 里的 ADDED 标题**逐字还在**
    ⇒ 这个 change **永远归不了档**。`openspec validate` 只把它报成 **6 条 INFO**
    （`Archive would refuse this delta: … already exists`）——**INFO 不判红、也没人看**。
    ⇒ 这正是"一个 change 永远归不了档而没人报"，故必须有执行者。

    **只判 ADDED**：`MODIFIED`／`REMOVED`／`RENAMED` 的标题**本来就该**在主规格里存在，
    拿本条去判它们是**用错了尺子**（`--self-test` 的 `对照⑩n` 钉着这一条）。
    """
    bad = []
    ch = Path(repo) / "openspec" / "changes"
    main = Path(repo) / "openspec" / "specs"
    if not ch.is_dir():
        return []
    for d in sorted(ch.iterdir()):
        if not d.is_dir() or d.name.startswith(".") or d.name == "archive":
            continue
        sd = d / "specs"
        if not sd.is_dir():
            continue
        for sp in sorted(sd.rglob("spec.md")):
            cap = sp.parent.name
            msp = main / cap / "spec.md"
            existing = set()
            if msp.is_file():
                for ln in read_text(msp).split("\n"):
                    m = REQ_RE.match(ln)
                    if m:
                        existing.add(m.group(1))
            if not existing:
                continue                                # 主规格里没有这个能力 ⇒ 无从撞车
            in_added = False
            for i, ln in enumerate(read_text(sp).split("\n"), 1):
                s = ln.strip()
                if ADDED_HEAD_RE.match(s):
                    in_added = True
                    continue
                if s.startswith("## "):
                    in_added = False                    # 出了 ADDED 节（`#### Scenario` 不在此列）
                    continue
                if not in_added:
                    continue
                m = REQ_RE.match(ln)
                if m and m.group(1) in existing:
                    bad.append("%s:%d —— ADDED 标题「%s」**已存在于主规格**（`openspec/specs/%s/spec.md`）"
                               " ⇒ 归档会被拒（`already exists`）；要么该 delta 作废、"
                               "要么用 `openspec archive --skip-specs` 并在此处登记原因"
                               % (rel(repo, sp), i, m.group(1), cap))
    return bad


# ⑪ 用：生成物与它的生成器（都在仓内受控；**闸必须在仓内**，见 skill §九）
BRIDGE_REL = "openspec/BRIDGE.md"
BRIDGE_GEN_REL = "openspec/tools/gen_bridge_md.py"
MIRROR_SKIP = (".git", "target", "node_modules", "__pycache__")


def _bridge_from_generator(repo, tmp_root):
    """在**临时镜像**里跑一遍 BRIDGE 生成器，返回它当场产出的字节。**真仓一个字节都不动。**

    为什么要镜像（而不是直接跑）：`gen_bridge_md.py` 的 `SRC`／`OUT` **按 `__file__` 推导**，
    且它在**模块级**就写文件 ⇒ 在真仓跑它＝**改真仓的生成物**（门禁不许改被检对象）。
    ⇒ 把 `openspec/` 与 `world-core/` 复制进临时目录，再 `exec` **镜像里那份**生成器
    （`__file__` 指向镜像 ⇒ `SRC`／`OUT` 一并落在镜像内），比对镜像产出与仓内文件的**字节**。
    依赖仓内**任何**被生成器读到的输入（今天：`openspec/specmap.json`、`openspec/specs/**`、
    `world-core/docs/**`、`openspec/changes/fc-2026-001-*/audit.md`）都随整树复制 ⇒ 生成器日后
    加了新输入也不必改这里。失败一律返回 `(None, 原因)`，由调用方**判红**（读不到＝失败，不是"没有该项"）。
    """
    gen = Path(repo) / "openspec" / "tools" / "gen_bridge_md.py"
    if not gen.is_file():
        return None, "生成器 `%s` 不存在（生成物没有生成器＝不可复算）" % BRIDGE_GEN_REL
    mirror = Path(tmp_root)
    for sub in ("openspec", "world-core"):
        s = Path(repo) / sub
        if not s.is_dir():
            continue
        try:
            shutil.copytree(s, mirror / sub, ignore=shutil.ignore_patterns(*MIRROR_SKIP))
        except Exception as e:
            return None, "镜像 `%s/` 失败：%r" % (sub, e)
    gdst = mirror / "openspec" / "tools" / "gen_bridge_md.py"
    if not gdst.is_file():
        return None, "镜像里没有生成器"
    try:
        with contextlib.redirect_stdout(io.StringIO()):   # 生成器会 print；别污染门禁输出
            exec(compile(gdst.read_text(encoding="utf-8"), str(gdst), "exec"),
                 {"__file__": str(gdst), "__name__": "_spec_bridge_gen_probe"})
    except Exception as e:
        return None, "在镜像里跑生成器抛异常：%r" % (e,)
    out = mirror / "openspec" / "BRIDGE.md"
    if not out.is_file():
        return None, "生成器跑完却没有产出 `openspec/BRIDGE.md`"
    return out.read_bytes(), ""


def _bridge_diff_hint(want, got):
    """给一条**可核**的定位：首处不同的行号与两边逐字（截断），外加两边的 sha256／字节数。"""
    a = want.decode("utf-8", "replace").split("\n")
    b = got.decode("utf-8", "replace").split("\n")
    n = min(len(a), len(b))
    i = 0
    while i < n and a[i] == b[i]:
        i += 1
    if i >= n:
        where = "行数不同（仓内 %d 行／生成器 %d 行）" % (len(a), len(b))
    else:
        where = "首处不同在第 %d 行 —— 仓内「%s」／生成器「%s」" % (i + 1, a[i].strip()[:56], b[i].strip()[:56])
    return "（%s；仓内 %d 字节 sha256=%s，生成器 %d 字节 sha256=%s）" % (
        where, len(want), hashlib.sha256(want).hexdigest()[:12],
        len(got), hashlib.sha256(got).hexdigest()[:12])


def j11_bridge_in_sync_with_generator(repo):
    """⑪ `openspec/BRIDGE.md` 必须与它的生成器**当前输出**逐字节一致。

    为什么单列一条：同一个病**复发过两次**——建表那次提交上就错了两个数（写 `133` 实测 `56`、
    写 `0 命中` 实测 `1`）；2026-09-27 评审席重跑生成器又报"提交里是旧值"。
    而 `BRIDGE.md` 被 `review.md`／`proposal.md`／`design.md` 四处援引为**数值唯一权威**。
    skill §九：**生成物不许手编；生成链必须在仓内；"闸在版本控制之外"等于没有闸**——
    但今天没有执行者 ⇒ 本条就是那个执行者（判据④ 只判"覆不覆盖"，**判不出数对不对**）。
    """
    out = Path(repo) / "openspec" / "BRIDGE.md"
    if not out.is_file():
        return ["%s —— 文件不存在（生成物缺件；覆盖与否另有判据④）" % BRIDGE_REL]
    try:
        want = out.read_bytes()
    except Exception as e:
        return ["%s —— 读不到（%r）；**读不到＝失败**，不许当成「没有该项」" % (BRIDGE_REL, e)]
    with tempfile.TemporaryDirectory(prefix="specbridge-bridge-") as tmp:
        got, why = _bridge_from_generator(repo, tmp)
    if got is None:
        return ["%s —— **无法判定**（%s）；按本仓口径：读不到就判红，不许静默跳过" % (BRIDGE_REL, why)]
    if got == want:
        return []
    return ["%s —— 与 `%s` 的**当前输出不一致**%s ⇒ 它引用的数可能已过期；"
            "跑 `python openspec/tools/gen_bridge_md.py` 重生成（**它是生成物，不许手改**）"
            % (BRIDGE_REL, BRIDGE_GEN_REL, _bridge_diff_hint(want, got))]


def j12_specmap_generator_hash(repo):
    """⑫ `openspec/specmap.json` 必须记录**它自己生成器的当前内容哈希**。

    为什么单列一条：skill §九「生成物不许手编」——`BRIDGE.md` 有判据⑪ 盯着（重跑生成器逐字节比），
    而 `specmap.json`（**149 KB、被 8 处引用**）**此前没有任何判据**；实测它的生成链原本在**仓外**、
    产物还写在仓外，搬进仓内后一度与生成器脱节（`chapters` 7→1、`judges` 17→0 的**静默退化**）。
    ⇒ 本条是它今天的执行者。

    **★ 本判据的射程（如实写，不假装管得更多）**：它抓的是「**生成器改了、产物没重生成**」。
    **它抓不到**「产物被手工改过数据」——那需要**重跑生成器逐字节比对**（判据⑪ 的强形态），
    而生成器读四类输入（`openspec/specs/**`、`WC-SRS-001`、书《合订本》、`world-core/tests/**`），
    在 `--self-test` 的沙盒里供不齐、且解析器在最小输入上不保证不崩。
    ⇒ 强形态**登记为后续可加强项**（生成器已支持 `SPECMAP_OUT`，把产物写临时目录即可比对），**今天不做，且不冒充做了**。
    """
    art = Path(repo) / "openspec" / "specmap.json"
    gen = Path(repo) / "openspec" / "tools" / "gen_specmap.py"
    if not art.is_file():
        return ["openspec/specmap.json —— 文件不存在（生成物缺件）"]
    if not gen.is_file():
        return ["openspec/tools/gen_specmap.py —— 生成器不在仓内（skill §九：**「闸在版本控制之外」等于没有闸**）"]
    try:
        doc = json.loads(art.read_text(encoding="utf-8"))
    except Exception as e:
        return ["openspec/specmap.json —— 读不出 JSON（%r）；**读不到＝失败**" % e]
    want = hashlib.sha256(gen.read_bytes()).hexdigest()
    got = doc.get("_generator_sha256")
    if not got:
        return ["openspec/specmap.json —— 没记录 `_generator_sha256` ⇒ **无法判定它是不是当前生成器的输出**；"
                "跑 `python openspec/tools/gen_specmap.py` 重生成（**它是生成物，不许手改**）"]
    if got != want:
        return ["openspec/specmap.json —— 记录的生成器哈希 `%s…` 与当前 `openspec/tools/gen_specmap.py` 的 `%s…` **不一致** ⇒ "
                "**生成器改过而产物没重生成**；跑 `python openspec/tools/gen_specmap.py` 重生成" % (got[:12], want[:12])]
    return []


JUDGMENTS = [
    ("① 归档硬前置（归档目录必须有 review.md）", j1_archive_review),
    ("② 证据存在性（证据行的函数/脚本必须真实存在）", j2_evidence),
    ("③ 默认档守卫（config.yaml 必须为 %s）" % SCHEMA_NAME, j3_default_schema),
    ("④ 编号桥覆盖（BRIDGE.md 必须覆盖规格树下每条 Requirement）", j4_bridge_coverage),
    ("⑤ 覆盖在册（cover-* change 未归档且 tasks 有未勾项）", j5_coverage_change),
    ("⑥ 归档件的评审已签（结论 ∈ 批准/通过/有条件通过，且批准人非空）", j6_archived_review_signed),
    ("⑦ 让路登记（声明了「谁让」的件必须写全：让哪一条／为什么／谁批的）", j7_waiver_registered),
    ("⑧ 流程文档无修订记录（**修订记录＝git 提交历史**；标题式/加粗式/表格式都拦；"
     "豁免目录：`docs/评审/`、书 `docs/理论/`）", j8_no_revision_log_in_docs),
    ("⑨ 规格正文无改因块（**主规格 ＋ delta**；改因归该 change 的 `design.md`／`audit.md`）", j9_no_rationale_in_specs),
    ("⑩ ADDED 标题不与主规格撞车（撞了该 change 永远归不了档）", j10_delta_added_not_colliding),
    ("⑪ `BRIDGE.md` 与生成器的当前输出逐字节一致（生成物不许手编）", j11_bridge_in_sync_with_generator),
    ("⑫ `specmap.json` 记录了当前生成器的内容哈希（生成物不许手编）", j12_specmap_generator_hash),
]


def run_all(repo):
    res = []
    for name, fn in JUDGMENTS:
        try:
            bad = fn(repo)
        except Exception as e:                        # 守卫自己崩了，按不通过处理（fail-closed）
            bad = ["判据自身异常：%r" % (e,)]
        res.append({"judgment": name, "ok": not bad, "offenders": bad})
    return res


# ────── 自证：**每条判据**至少一个反例（条数随 `JUDGMENTS` 增长；另配"不该红"的对照） ──────
# 沙盒规格（主规格）与 delta 规格共用同一个 Requirement 标题体系：
#   `REQ-X-001 沙盒需求` 只在主规格 ⇒ delta 里把它放进 `## MODIFIED` 是**正常**的（判据⑩ 不许判它红），
#   而 delta 的 `## ADDED` 放 `REQ-X-002 沙盒 delta 需求`（主规格里没有）⇒ 正控绿。
SANDBOX_BRIDGE = "# 编号桥\n\n| 承诺 | 号 |\n|---|---|\n| REQ-X-001 沙盒需求 | REQ-X-001 |\n"
SANDBOX_DELTA = (
    "# Spec Delta\n\n## ADDED Requirements\n\n"
    "### Requirement: REQ-X-002 沙盒 delta 需求\n\n"
    "沙盒 delta 正文（判据②／⑨ 的扫描面含 delta）。\n\n"
    "#### Scenario: 沙盒 delta 场景\n\n"
    "- **WHEN** 跑沙盒 delta\n- **THEN** 通过\n"
    "- **证据**：`tests/t.rs::the_test`\n\n"
    "## MODIFIED Requirements\n\n"
    "### Requirement: REQ-X-001 沙盒需求\n\n"
    "沙盒：MODIFIED 的标题**本来就该**在主规格里存在 ⇒ 判据⑩ **不许**判它红（对照⑩n）。\n"
)
SANDBOX = {
    "openspec/config.yaml": "schema: %s\n" % SCHEMA_NAME,
    "openspec/specs/cap-a/spec.md": (
        "# cap-a Specification\n\n## Purpose\n沙盒用最小规格，只为验证守卫会红。\n\n"
        "## Requirements\n\n### Requirement: REQ-X-001 沙盒需求\n\n"
        "沙盒正文。\n\n#### Scenario: 沙盒场景\n\n"
        "- **WHEN** 跑沙盒\n- **THEN** 通过\n"
        "- **证据**：`tests/t.rs::the_test`\n"
    ),
    "openspec/changes/archive/2026-01-01-sandbox/review.md": (
        "# Review\n\n| 项 | 内容 |\n|---|---|\n"
        "| **结论** | 通过 |\n| **批准人** | 沙盒批准人（非占位）|\n"
    ),
    "openspec/changes/archive/2026-01-01-sandbox/tasks.md": "- [x] 1.1 沙盒\n",
    "openspec/changes/cover-gap/tasks.md": "- [ ] 1.1 未实现的能力（在册）\n",
    # 正控用的"让路登记"：三要素齐全（判据⑦ 只在件里出现「谁让」时才检）
    "openspec/changes/cover-gap/design.md": (
        "# Design\n\n## 与书的关系（谁让）\n\n"
        "- **让的是哪一条**：`schema.yaml:65-66`「只装已成立且可复现的行为」。\n"
        "- **为什么要让**：书是上位，Purpose 不是行为承诺。\n"
        "- **谁批的**：作者（2026-01-01 指示）。\n"
    ),
    # 未归档 change 的 delta 规格：判据②（证据存在性）与⑨（无改因块）**都扫 delta**、
    # 判据⑩ 也读它 ⇒ 正控里它必须干净。
    "openspec/changes/cover-gap/specs/cap-a/spec.md": SANDBOX_DELTA,
    "openspec/BRIDGE.md": SANDBOX_BRIDGE,
    # 沙盒用的**最小生成器**：判据⑪ 拿"它的当前输出"比对 `openspec/BRIDGE.md`。
    # 它刻意照抄真生成器的取径方式（`__file__` → 上一级目录），因为⑪ 的实现正是靠这一点
    # 把镜像里的 `OUT` 关在临时目录内。产出必须与 `SANDBOX_BRIDGE` 逐字节相同（正控绿）。
    "openspec/tools/gen_bridge_md.py": (
        "# -*- coding: utf-8 -*-\n"
        "# 沙盒最小生成器：只证明判据⑪「生成物与生成器不同步即红」真的会红。\n"
        "from pathlib import Path\n"
        "OUT = str(Path(__file__).resolve().parent.parent / 'BRIDGE.md')\n"
        "open(OUT, 'w', encoding='utf-8', newline='\\n').write(" + repr(SANDBOX_BRIDGE) + ")\n"
    ),
    "world-core/tests/t.rs": "fn the_test() {}\n",
}


def build_sandbox(root):
    for relp, content in SANDBOX.items():
        p = Path(root) / relp
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8", newline="\n")
    # ★ 判据⑫ 的沙盒件：把**仓内真生成器**字节复制进去，并造一份「已同步」的 `specmap.json`
    #   （哈希＝沙盒里那份生成器的真实哈希 ⇒ 正控应当全绿）
    #   路径：本文件在 `world-core/tools/` ⇒ 仓根＝上两级；生成器在 `<仓根>/openspec/tools/gen_specmap.py`
    _repo = Path(__file__).resolve().parent.parent.parent
    _gen_src = _repo / "openspec" / "tools" / "gen_specmap.py"
    _gen_dst = Path(root) / "openspec" / "tools" / "gen_specmap.py"
    _gen_dst.parent.mkdir(parents=True, exist_ok=True)
    _gen_dst.write_bytes(_gen_src.read_bytes())            # 字节复制（哈希才对得上）
    _sha = hashlib.sha256(_gen_dst.read_bytes()).hexdigest()
    _art = Path(root) / "openspec" / "specmap.json"
    _art.write_text(json.dumps({"caps": [], "_generator_sha256": _sha},
                               ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")


def self_test():
    print("== spec_bridge.py --self-test ==")
    failures = []
    covered = set()
    with tempfile.TemporaryDirectory(prefix="specbridge-") as tmp:

        def jname(idx):
            """判据的圈号（`JUDGMENTS` 的标签一律以圈号＋空格开头）。"""
            return JUDGMENTS[idx][0].split()[0]

        def _red(idx, tag, why):
            """反例：跑第 idx 条判据，**必须变红**；不变红 ⇒ 这条守卫是装饰。"""
            is_red = not run_all(tmp)[idx]["ok"]
            print("  反例%s（%s => 判据%s 应红）：%s" % (tag, why, jname(idx), "已红 OK" if is_red else "*没红"))
            if is_red:
                covered.add(idx)
            else:
                failures.append("反例%s未变红" % tag)
            return is_red

        def _green(idx, tag, why):
            """对照：**不该红的**必须不红（否则下一个人就不敢写那句话了）。"""
            is_red = not run_all(tmp)[idx]["ok"]
            print("  对照%s（%s => 判据%s **不应红**）：%s" % (tag, why, jname(idx), "*误伤" if is_red else "未红 OK"))
            if is_red:
                failures.append("对照%s被误伤" % tag)

        build_sandbox(tmp)

        # 正控：完好沙盒必须**每条判据**全绿（条数现算，不写死——写死过一次就再也不会对）
        res = run_all(tmp)
        bad = [r["judgment"] for r in res if not r["ok"]]
        print("  正控（完好沙盒 %d 条应全绿）：%s" % (len(JUDGMENTS), "OK" if not bad else "*失败 " + str(bad)))
        if bad:
            failures.append("正控失败：%s" % bad)

        # 反例 1：删掉归档的 review.md
        rv = Path(tmp) / "openspec/changes/archive/2026-01-01-sandbox/review.md"
        backup = rv.read_text(encoding="utf-8")
        rv.unlink()
        _red(0, "①", "删 review.md")
        rv.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 2a：把**主规格**的证据指向不存在的函数
        sp = Path(tmp) / "openspec/specs/cap-a/spec.md"
        backup = sp.read_text(encoding="utf-8")
        sp.write_text(backup.replace("::the_test", "::no_such_fn"), encoding="utf-8", newline="\n")
        _red(1, "②a", "**主规格**证据函数不存在")
        sp.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 2b：把 **delta** 的证据指向不存在的函数。
        #   为什么必须有这一条：判据② 上一轮扩面到 delta（delta 归档合并时才进主规格），
        #   而反例原来**只造主规格** ⇒ delta 那一支**没有自证**（＝那一支可能是假绿）。
        dsp = Path(tmp) / "openspec/changes/cover-gap/specs/cap-a/spec.md"
        backup2b = dsp.read_text(encoding="utf-8")
        dsp.write_text(backup2b.replace("::the_test", "::no_such_fn"), encoding="utf-8", newline="\n")
        _red(1, "②b", "**delta** 证据行指向不存在的函数（② 的 delta 支必须自证）")
        dsp.write_text(backup2b, encoding="utf-8", newline="\n")

        # 反例 3：把默认档改回 spec-driven
        cf = Path(tmp) / "openspec/config.yaml"
        cf.write_text("schema: spec-driven\n", encoding="utf-8", newline="\n")
        _red(2, "③", "默认档改回 spec-driven")
        cf.write_text("schema: %s\n" % SCHEMA_NAME, encoding="utf-8", newline="\n")

        # 反例 4：把 BRIDGE.md 里那条 Requirement 抹掉
        br = Path(tmp) / "openspec/BRIDGE.md"
        backup = br.read_text(encoding="utf-8")
        br.write_text("# 编号桥\n\n（空）\n", encoding="utf-8", newline="\n")
        _red(3, "④", "映射表不覆盖")
        br.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 5：撤掉覆盖 change（**改用改名、不删目录**——后面的反例还要用这个目录；
        #          新名不能以 `cover-` 开头，否则判据⑤ 仍会把它算作在册）
        cg = Path(tmp) / "openspec/changes/cover-gap"
        cg_hidden = Path(tmp) / "openspec/changes/_hidden-gap"
        cg.rename(cg_hidden)
        _red(4, "⑤", "覆盖 change 不在册")
        cg_hidden.rename(cg)

        # 反例 6：归档件的 review 未签（结论＝待签）——判据① 是内容盲的，必须由⑥抓住
        rv2 = Path(tmp) / "openspec/changes/archive/2026-01-01-sandbox/review.md"
        backup6 = rv2.read_text(encoding="utf-8")
        rv2.write_text("# Review\n\n| 项 | 内容 |\n|---|---|\n| **结论** | 　**待签** |\n| **批准人** | 项目负责人（本人签署时补姓名） |\n",
                       encoding="utf-8", newline="\n")
        _red(5, "⑥", "归档件 review 未签（结论＝待签）")
        # 反例 6b：结论已签但批准人是占位——① 与 ⑥ 都应只看 ⑥ 抓它
        rv2.write_text("# Review\n\n| 项 | 内容 |\n|---|---|\n| **结论** | 批准 |\n| **批准人** | 　**待签** |\n",
                       encoding="utf-8", newline="\n")
        _red(5, "⑥b", "结论已签但批准人占位")
        rv2.write_text(backup6, encoding="utf-8", newline="\n")

        # 反例 7：件里声明了「谁让」，但三要素缺一样（只写"谁让"两字、不写谁批的）
        dm = Path(tmp) / "openspec/changes/cover-gap/design.md"
        backup7 = dm.read_text(encoding="utf-8")
        dm.write_text("# Design\n\n## 与书的关系（谁让）\n\n- **让的是哪一条**：`schema.yaml:65-66`。\n- **为什么要让**：书是上位。\n",
                      encoding="utf-8", newline="\n")
        _red(6, "⑦", "声明了谁让却缺「谁批的」")
        dm.write_text(backup7, encoding="utf-8", newline="\n")

        # ── 反例 8a／8b／8c：修订记录的**三种形态都必须红** ──
        #   8b／8c 是**本轮补的**：旧例只造了带 `#` 的标题式，而**实盘的真实违规没有一处带 `#`**
        #   （逐字：`world-core/docs/S0-立项/WC-ATOM-001-v0.1.md:67` = `**修订记录**`；
        #          `world-core/docs/S0-立项/WC-SCMP-001-v0.1.md:1845` = `| 修订记录 | … |`）
        #   ⇒ 旧守卫在真违规上照样报 [OK]（假绿：评审席注入实验照出的正是这一条）。
        #   ⚠ 三条都写在 **`docs/S0-立项/`** 下——这正是"豁免**没**扩大到别处"的证明（豁免只到 `docs/评审/`）。
        print("  ⑧ 的豁免目录：`docs/评审/`（评审记录含别人文档的逐字留存，改它＝篡改记录）、"
              "书 `docs/理论/`；其余目录（S0-立项…S4-实现、阶段外-待启用）**一律照判**")
        doc8 = Path(tmp) / "world-core/docs/S0-立项/WC-X-001.md"
        doc8.parent.mkdir(parents=True, exist_ok=True)
        for tag, body, why in (
            ("8a", "# 沙盒文档\n\n### 修订记录\n\n| 版本 | 改了什么 |\n|---|---|\n| V0.1 | 沙盒 |\n",
                   "标题式 `### 修订记录`（写在 `docs/S0-立项/`）"),
            ("8b", "# 沙盒文档\n\n**修订记录**\n\n| 版本 | 改了什么 |\n|---|---|\n| V0.1 | 沙盒 |\n",
                   "加粗式 `**修订记录**`（与 `WC-ATOM-001-v0.1.md:67` **同形态**；写在 `docs/S0-立项/`）"),
            ("8c", "# 沙盒文档\n\n| 项 | 内容 |\n|---|---|\n| 修订记录 | **V0.2（2026-09-27）**：沙盒 |\n",
                   "表格式**带版本号/日期** `| 修订记录 | V0.2（2026-09-27）… |`"
                   "（与 `WC-SCMP-001-v0.1.md:1845` **同形态**；写在 `docs/S0-立项/`）"),
        ):
            doc8.write_text(body, encoding="utf-8", newline="\n")
            _red(7, tag, why)
        doc8.unlink()

        # 对照 8p：**指路不是记录**——`| 变更记录 | 见 §十二 |` 既无版本号也无日期 ⇒ **不该红**
        #   （实盘同形态：`world-core/docs/S1-需求/WC-IRS-001-v0.1.md:364` 逐字
        #     `| 变更记录 | 本文档 §十二 登记；`WC-IC-001` 同步 |`——它在属性表里说的是"登记在哪一节"。）
        doc8.write_text("# 沙盒文档\n\n| 属性 | 值 |\n|---|---|\n| 变更记录 | 见 §十二 |\n",
                        encoding="utf-8", newline="\n")
        _green(7, "8p", "表格式**指路**行（无版本号、无日期）")
        doc8.unlink()

        # 对照 8n：**「本文件不设修订记录」的正当声明不许被误伤**。
        #   实盘有 14 处这样的声明，它们是**正相反**的写法（声明"本件不设修订记录"）。
        #   判据若把它们也判红，作者下次就不敢写这条红线了 ⇒ 必须钉住"不该红的"。
        doc8.write_text("# 沙盒文档\n\n> **本文件不设修订记录**：历次改动写在 **git 提交信息**里；正文只写**现在是什么**。\n",
                        encoding="utf-8", newline="\n")
        _green(7, "8n", "「本文件不设修订记录」的正当声明")
        doc8.unlink()

        # 反例 8d：**同一形态改放到 `docs/S0-立项/`** ⇒ 仍应红
        #   （证明豁免**只到 `docs/评审/`**，没有扩大；8a–8c 亦同在此目录下）
        doc8d = Path(tmp) / "world-core/docs/S0-立项/WC-Y-001.md"
        doc8d.write_text("# 沙盒文档\n\n**修订记录**\n", encoding="utf-8", newline="\n")
        _red(7, "8d", "同一条加粗式放在 `docs/S0-立项/` 下（豁免**没**扩到别处）")
        doc8d.unlink()

        # 对照 8e：**同一条**放到 `docs/评审/` 下 ⇒ **不应红**（评审记录是别人文档的逐字留存，改它＝篡改记录）
        doc8e = Path(tmp) / "world-core/docs/评审/WC-RV-X-001.md"
        doc8e.parent.mkdir(parents=True, exist_ok=True)
        doc8e.write_text("# 沙盒评审记录\n\n**修订记录**\n", encoding="utf-8", newline="\n")
        _green(7, "8e", "`docs/评审/` 下的同一条（豁免目录：评审记录含别人文档的逐字留存）")
        doc8e.unlink()

        # ── 反例 9a／9b：改因块的**两个扫描面都必须红**（主规格 ＋ delta） ──
        sp9 = Path(tmp) / "openspec/specs/cap-a/spec.md"
        backup9 = sp9.read_text(encoding="utf-8")
        sp9.write_text(backup9 + "\n> **改的是哪一类问题**：沙盒反例。\n", encoding="utf-8", newline="\n")
        _red(8, "9a", "**主规格**正文里出现改因块")
        sp9.write_text(backup9, encoding="utf-8", newline="\n")

        d9 = Path(tmp) / "openspec/changes/cover-gap/specs/cap-a/spec.md"
        backup9d = d9.read_text(encoding="utf-8")
        d9.write_text(backup9d + "\n> **改的是哪一类问题**：沙盒 delta 反例。\n", encoding="utf-8", newline="\n")
        _red(8, "9b", "**delta** 正文里出现改因块（评审席注入实验照出的漏检）")
        d9.write_text(backup9d, encoding="utf-8", newline="\n")

        # ── 反例 10：未归档 change 的 **ADDED** 标题与主规格逐字撞车 ──
        #   （与实盘同形态：`fc-2026-002-spec-revisions` 的 ADDED 标题逐字已在主规格里 ⇒ 归档被拒）
        d10 = Path(tmp) / "openspec/changes/cover-gap/specs/cap-a/spec.md"
        backup10 = d10.read_text(encoding="utf-8")
        d10.write_text(backup10.replace("REQ-X-002 沙盒 delta 需求", "REQ-X-001 沙盒需求"),
                       encoding="utf-8", newline="\n")
        _red(9, "⑩", "delta 的 ADDED 标题与主规格逐字撞车")
        # 对照 10n：`MODIFIED` 的标题**本来就该**在主规格里存在 ⇒ 判据⑩ 不许判它红（别误伤）
        d10.write_text("# Spec Delta\n\n## MODIFIED Requirements\n\n"
                       "### Requirement: REQ-X-001 沙盒需求\n\n沙盒：MODIFIED 撞车是正常的。\n",
                       encoding="utf-8", newline="\n")
        _green(9, "10n", "`MODIFIED` 标题与主规格相同（本来就该相同）")
        d10.write_text(backup10, encoding="utf-8", newline="\n")

        # ── 反例 11：手编生成物——往 `openspec/BRIDGE.md` 里加一行（生成器不会产出它） ──
        br11 = Path(tmp) / "openspec/BRIDGE.md"
        backup11 = br11.read_text(encoding="utf-8")
        br11.write_text(backup11 + "\n（手编：这一行不是生成器产出的）\n", encoding="utf-8", newline="\n")
        _red(10, "⑪", "`BRIDGE.md` 被手编（与生成器当前输出不同）")
        br11.write_text(backup11, encoding="utf-8", newline="\n")

        # ── 反例 12：生成器改了、产物没重生成 —— 把产物里记录的生成器哈希改掉 ──
        sp12 = Path(tmp) / "openspec/specmap.json"
        backup12 = sp12.read_text(encoding="utf-8")
        _d12 = json.loads(backup12)
        _d12["_generator_sha256"] = "0" * 64
        sp12.write_text(json.dumps(_d12, ensure_ascii=False, indent=1), encoding="utf-8", newline="\n")
        _red(11, "⑫", "产物记录的生成器哈希与当前生成器不一致（＝改了生成器没重生成）")
        sp12.write_text(backup12, encoding="utf-8", newline="\n")

        # 对照 12：**不该红的** —— 沙盒里那份产物与生成器是同步的（哈希一致）
        _green(11, "12n", "产物与生成器同步（哈希一致）")

        # 反面自检：**每条判据都必须有反例**（没有反例的那条＝装饰）
        missing = [jname(i) for i in range(len(JUDGMENTS)) if i not in covered]
        print("  反例覆盖：%d/%d 条判据各有 >=1 个反例%s"
              % (len(covered), len(JUDGMENTS), "" if not missing else "；**缺**：" + "、".join(missing)))
        if missing:
            failures.append("这些判据没有反例（＝装饰）：%s" % "、".join(missing))

    if failures:
        print("  => 自证不通过：%s" % "；".join(failures))
        print("  => 按本项目口径：**这条守卫是装饰，拒绝合入**。")
        return 1
    print("  => 自证通过：**每条判据**（%d 条）在反例下变红、在正控下全绿；" % len(JUDGMENTS))
    print("     （四处「不应红」的对照：⑧ 的「不设修订记录」声明／⑧ 的表格**指路**行／⑧ 的 `docs/评审/` 豁免目录／"
          "⑩ 的 MODIFIED 标题。）")
    return 0


# ────────────────────────── 主程序 ──────────────────────────
def main(argv=None):
    ap = argparse.ArgumentParser(description="规格层守卫（opsx-swe-gb；判据见 JUDGMENTS，现 %d 条）" % len(JUDGMENTS))
    ap.add_argument("--repo", default=None, help="仓库根；默认从本脚本位置向上找含 openspec/specs 的目录")
    ap.add_argument("--json", action="store_true", help="以 JSON 输出")
    ap.add_argument("--self-test", action="store_true", help="为**每条**判据各造一个反例（条数随 `JUDGMENTS` 增长，加一条判据必须同时加一个反例），验证它们真的会红")
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    repo = args.repo or find_repo(Path(__file__).parent)
    if not repo:
        print("* 找不到仓库根（向上找不到含 openspec/specs 的目录）；用 --repo 指定。")
        return 1

    res = run_all(repo)
    failed = [r for r in res if not r["ok"]]

    if args.json:
        print(json.dumps({"repo": str(repo), "judgments": res,
                          "passed": len(res) - len(failed), "failed": len(failed)},
                         ensure_ascii=False, indent=1))
    else:
        print("== spec_bridge.py —— 规格层守卫 ==")
        print("   仓库：%s" % repo)
        for r in res:
            print("  %s %s" % ("[OK]" if r["ok"] else "[FAIL]", r["judgment"]))
            for o in r["offenders"]:
                print("       · %s" % o)
        print("  —— 通过 %d / 失败 %d ——" % (len(res) - len(failed), len(failed)))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
