#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""spec_bridge.py —— 规格层的守卫：把 `opsx-swe-gb` 里那五条"应当有人拦"的判据做成会真红的东西。

为什么需要它
------------
OpenSpec 的 `validate` 只判**形态**（结构、Scenario 个数、delta 语法），它**不查**：
  · 证据行指向的测试是否真的存在（改名即失锚，且不会变红）
  · 归档目录里有没有评审记录 `review.md`
  · 默认档是不是融合档
  · 编号桥映射表有没有覆盖规格树下的每条 Requirement
  · 承载覆盖缺口的 change 还在不在（未归档）
而这五条恰恰是 `opsx-swe-gb` 的文字里承诺过的。**一个从不失败的检查不是装饰，是假证。**
本脚本就是那五条的**执行者**：任一条不成立即非零退出。

判据（与 `specs/spec-governance/spec.md` 逐条对应）——**条数以 `JUDGMENTS` 为准，现七条**
-------------------------------------------------------
① 归档硬前置      每个 `openspec/changes/archive/*/` 必须有非空 `review.md`
② 证据存在性      `openspec/specs/**/spec.md` 里每条 `- **证据**：<token>` 的
                   `<path>::<fn>` 或 `<path> --self-test` 必须真实存在
③ 默认档守卫      `openspec/config.yaml` 的 `schema:` 必须为 `opsx-swe-gb`
④ 编号桥覆盖      `openspec/BRIDGE.md` 必须覆盖规格树下**每一条** Requirement（有号或显式标无号）
⑤ 覆盖在册        `openspec/changes/cover-*/` 至少有一个**未归档**、且 `tasks.md` 仍有未勾项

用法
----
    python3 tools/spec_bridge.py [--repo <仓库根>] [--json]
    python3 tools/spec_bridge.py --self-test     # 为**每条**判据各造一个反例（条数随 `JUDGMENTS` 增长，加一条判据必须同时加一个反例），反例不变红即判装饰
"""

import argparse
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


# ────────────────────────── 判据（现七条，见 JUDGMENTS）──────────────────────────
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
    parts = tok.split()                               # 形态 B：脚本（可带 --self-test 等参数）
    if parts:
        f = resolve_src(repo, parts[0])
        if f is None:
            return False, "脚本不存在"
        return True, ""
    return False, "无法解析的token"


def j2_evidence(repo):
    bad = []
    for spec in sorted((Path(repo) / "openspec" / "specs").rglob("spec.md")):
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
        m = re.search(r"批准人[^|\n]*\|([^|\n]*)", text)
        who = (m.group(1).strip() if m else "")
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


REVISION_HEAD_RE = re.compile(r"^#{1,4}\s*[^|]*?(修订记录|变更记录|修订历史)")
RATIONALE_HEAD_RE = re.compile(r"^> \*\*(改的是哪一类问题|为什么用 ADDED|证据是哪条测试)")


def j8_no_revision_log_in_docs(repo):
    """⑧ 流程文档不许有"修订记录"节（**修订记录＝git 提交历史**）。

    出处：作者指示「那几个文档里面也不要掺和这种什么修订的记录啥的」⇒ `.agents/skills/worldcore-sdd/SKILL.md` §三。
    为什么单列一条：正文是给读者用的（他要的是"现在是什么"），不是给作者记账用的；
    掺在一起，读者得在一堆"我曾写错什么"的括号里找那条规矩。
    **书（`docs/理论/`）除外**——那是作者的作品，其附录体例由作者定。
    """
    bad = []
    docs = Path(repo) / "world-core" / "docs"
    if not docs.is_dir():
        return []
    for f in sorted(docs.rglob("*.md")):
        if "理论" in f.parts:
            continue
        for i, ln in enumerate(f.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
            if REVISION_HEAD_RE.match(ln.strip()):
                bad.append("%s:%d —— 有修订记录节「%s」；**修订记录＝git 提交历史**，正文只写「现在是什么」"
                           % (rel(repo, f), i, ln.strip()[:48]))
    return bad


def j9_no_rationale_in_specs(repo):
    """⑨ 规格正文不许有"改因块"（改因属该 change 的 `design.md`／`audit.md`）。

    出处：同上 skill §三。实测混在正文里 76 个这样的块，形态门禁因此判 28 条
    `Requirement text is very long`——**正文只写"世界必须怎样"**。
    """
    bad = []
    specs = Path(repo) / "openspec" / "specs"
    if not specs.is_dir():
        return []
    for f in sorted(specs.rglob("spec.md")):
        for i, ln in enumerate(f.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
            if RATIONALE_HEAD_RE.match(ln.strip()):
                bad.append("%s:%d —— 规格正文里有改因块「%s…」；**改因归该 change 的 `design.md`／`audit.md`**"
                           % (rel(repo, f), i, ln.strip()[:44]))
    return bad


JUDGMENTS = [
    ("① 归档硬前置（归档目录必须有 review.md）", j1_archive_review),
    ("② 证据存在性（证据行的函数/脚本必须真实存在）", j2_evidence),
    ("③ 默认档守卫（config.yaml 必须为 %s）" % SCHEMA_NAME, j3_default_schema),
    ("④ 编号桥覆盖（BRIDGE.md 必须覆盖规格树下每条 Requirement）", j4_bridge_coverage),
    ("⑤ 覆盖在册（cover-* change 未归档且 tasks 有未勾项）", j5_coverage_change),
    ("⑥ 归档件的评审已签（结论 ∈ 批准/通过/有条件通过，且批准人非空）", j6_archived_review_signed),
    ("⑦ 让路登记（声明了「谁让」的件必须写全：让哪一条／为什么／谁批的）", j7_waiver_registered),
    ("⑧ 流程文档无修订记录（**修订记录＝git 提交历史**；书除外）", j8_no_revision_log_in_docs),
    ("⑨ 规格正文无改因块（改因归该 change 的 `design.md`／`audit.md`）", j9_no_rationale_in_specs),
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


# ────────────────────────── 自证：五条各造一个反例 ──────────────────────────
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
    "openspec/BRIDGE.md": "# 编号桥\n\n| 承诺 | 号 |\n|---|---|\n| REQ-X-001 沙盒需求 | REQ-X-001 |\n",
    "world-core/tests/t.rs": "fn the_test() {}\n",
}


def build_sandbox(root):
    for relp, content in SANDBOX.items():
        p = Path(root) / relp
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8", newline="\n")


def self_test():
    print("== spec_bridge.py --self-test ==")
    failures = []
    with tempfile.TemporaryDirectory(prefix="specbridge-") as tmp:
        build_sandbox(tmp)

        # 正控：完好沙盒必须五条全绿
        res = run_all(tmp)
        bad = [r["judgment"] for r in res if not r["ok"]]
        print("  正控（完好沙盒五条应全绿）：%s" % ("OK" if not bad else "*失败 " + str(bad)))
        if bad:
            failures.append("正控失败：%s" % bad)

        # 反例 1：删掉归档的 review.md
        rv = Path(tmp) / "openspec/changes/archive/2026-01-01-sandbox/review.md"
        backup = rv.read_text(encoding="utf-8")
        rv.unlink()
        ok = not run_all(tmp)[0]["ok"]
        print("  反例①（删 review.md => 判据① 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例①未变红")
        rv.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 2：把证据指向不存在的函数
        sp = Path(tmp) / "openspec/specs/cap-a/spec.md"
        backup = sp.read_text(encoding="utf-8")
        sp.write_text(backup.replace("::the_test", "::no_such_fn"), encoding="utf-8", newline="\n")
        ok = not run_all(tmp)[1]["ok"]
        print("  反例②（证据函数不存在 => 判据② 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例②未变红")
        sp.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 3：把默认档改回 spec-driven
        cf = Path(tmp) / "openspec/config.yaml"
        cf.write_text("schema: spec-driven\n", encoding="utf-8", newline="\n")
        ok = not run_all(tmp)[2]["ok"]
        print("  反例③（默认档改回 => 判据③ 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例③未变红")
        cf.write_text("schema: %s\n" % SCHEMA_NAME, encoding="utf-8", newline="\n")

        # 反例 4：把 BRIDGE.md 里那条 Requirement 抹掉
        br = Path(tmp) / "openspec/BRIDGE.md"
        backup = br.read_text(encoding="utf-8")
        br.write_text("# 编号桥\n\n（空）\n", encoding="utf-8", newline="\n")
        ok = not run_all(tmp)[3]["ok"]
        print("  反例④（映射表不覆盖 => 判据④ 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例④未变红")
        br.write_text(backup, encoding="utf-8", newline="\n")

        # 反例 5：撤掉覆盖 change（**改用改名、不删目录**——后面的反例还要用这个目录；
        #          新名不能以 `cover-` 开头，否则判据⑤ 仍会把它算作在册）
        cg = Path(tmp) / "openspec/changes/cover-gap"
        cg_hidden = Path(tmp) / "openspec/changes/_hidden-gap"
        cg.rename(cg_hidden)
        ok = not run_all(tmp)[4]["ok"]
        print("  反例⑤（覆盖 change 不在册 => 判据⑤ 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例⑤未变红")
        cg_hidden.rename(cg)

        # 反例 6：归档件的 review 未签（结论＝待签）——判据① 是内容盲的，必须由⑥抓住
        rv2 = Path(tmp) / "openspec/changes/archive/2026-01-01-sandbox/review.md"
        backup6 = rv2.read_text(encoding="utf-8")
        rv2.write_text("# Review\n\n| 项 | 内容 |\n|---|---|\n| **结论** | 　**待签** |\n| **批准人** | 项目负责人（本人签署时补姓名） |\n",
                      encoding="utf-8", newline="\n")
        r6 = run_all(tmp)[5]
        ok6 = not r6["ok"]
        print("  反例⑥（归档件 review 未签 => 判据⑥ 应红）：%s" % ("已红 OK" if ok6 else "*没红"))
        if not ok6:
            failures.append("反例⑥未变红")
        # 反例 6b：结论已签但批准人是占位——① 与 ⑥ 都应只看 ⑥ 抓它
        rv2.write_text("# Review\n\n| 项 | 内容 |\n|---|---|\n| **结论** | 批准 |\n| **批准人** | 　**待签** |\n",
                       encoding="utf-8", newline="\n")
        ok6b = not run_all(tmp)[5]["ok"]
        print("  反例⑥b（结论已签但批准人占位 => 判据⑥ 应红）：%s" % ("已红 OK" if ok6b else "*没红"))
        if not ok6b:
            failures.append("反例⑥b未变红")
        rv2.write_text(backup6, encoding="utf-8", newline="\n")

        # 反例 7：件里声明了「谁让」，但三要素缺一样（只写"谁让"两字、不写谁批的）
        dm = Path(tmp) / "openspec/changes/cover-gap/design.md"
        backup7 = dm.read_text(encoding="utf-8")
        dm.write_text("# Design\n\n## 与书的关系（谁让）\n\n- **让的是哪一条**：`schema.yaml:65-66`。\n- **为什么要让**：书是上位。\n",
                      encoding="utf-8", newline="\n")
        ok7 = not run_all(tmp)[6]["ok"]
        print("  反例⑦（声明了谁让却缺「谁批的」 => 判据⑦ 应红）：%s" % ("已红 OK" if ok7 else "*没红"))
        if not ok7:
            failures.append("反例⑦未变红")
        dm.write_text(backup7, encoding="utf-8", newline="\n")

        # 反例 8／9：新规矩的机器项（修订记录／改因块）——**加了违规必须变红**
        doc8 = Path(tmp) / "world-core/docs/S0-立项/WC-X-001.md"
        doc8.parent.mkdir(parents=True, exist_ok=True)
        doc8.write_text("# 沙盒文档\n\n### 修订记录\n\n| 版本 | 改了什么 |\n|---|---|\n| V0.1 | 沙盒 |\n",
                        encoding="utf-8", newline="\n")
        ok8 = not run_all(tmp)[7]["ok"]
        print("  反例⑧（流程文档里出现『修订记录』节 => 判据⑧ 应红）：%s" % ("已红 OK" if ok8 else "*没红"))
        if not ok8:
            failures.append("反例⑧未变红")
        doc8.unlink()

        sp9 = Path(tmp) / "openspec/specs/cap-a/spec.md"
        backup9 = sp9.read_text(encoding="utf-8")
        sp9.write_text(backup9 + "\n> **改的是哪一类问题**：沙盒反例。\n", encoding="utf-8", newline="\n")
        ok9 = not run_all(tmp)[8]["ok"]
        print("  反例⑨（规格正文里出现『改因块』 => 判据⑨ 应红）：%s" % ("已红 OK" if ok9 else "*没红"))
        if not ok9:
            failures.append("反例⑨未变红")
        sp9.write_text(backup9, encoding="utf-8", newline="\n")

    if failures:
        print("  => 自证不通过：%s" % "；".join(failures))
        print("  => 按本项目口径：**这条守卫是装饰，拒绝合入**。")
        return 1
    print("  => 自证通过：**每条判据**在反例下变红、在正控下全绿（条数见上方逐条清单）。")
    return 0


# ────────────────────────── 主程序 ──────────────────────────
def main(argv=None):
    ap = argparse.ArgumentParser(description="规格层守卫（opsx-swe-gb；判据见 JUDGMENTS，现七条）")
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
