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

五条判据（与 `specs/spec-governance/spec.md` 逐条对应）
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
    python3 tools/spec_bridge.py --self-test     # 为五条判据各造一个反例，反例不变红即判装饰
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


# ────────────────────────── 五条判据 ──────────────────────────
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
                bad.append("%s:%d —— 证据行里没有反引号包起来的 token" % (rel(repo, spec), i))
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


JUDGMENTS = [
    ("① 归档硬前置（归档目录必须有 review.md）", j1_archive_review),
    ("② 证据存在性（证据行的函数/脚本必须真实存在）", j2_evidence),
    ("③ 默认档守卫（config.yaml 必须为 %s）" % SCHEMA_NAME, j3_default_schema),
    ("④ 编号桥覆盖（BRIDGE.md 必须覆盖规格树下每条 Requirement）", j4_bridge_coverage),
    ("⑤ 覆盖在册（cover-* change 未归档且 tasks 有未勾项）", j5_coverage_change),
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
    "openspec/changes/archive/2026-01-01-sandbox/review.md": "# Review\n\n结论：通过\n",
    "openspec/changes/archive/2026-01-01-sandbox/tasks.md": "- [x] 1.1 沙盒\n",
    "openspec/changes/cover-gap/tasks.md": "- [ ] 1.1 未实现的能力（在册）\n",
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

        # 反例 5：撤掉覆盖 change
        shutil.rmtree(Path(tmp) / "openspec/changes/cover-gap")
        ok = not run_all(tmp)[4]["ok"]
        print("  反例⑤（覆盖 change 不在册 => 判据⑤ 应红）：%s" % ("已红 OK" if ok else "*没红"))
        if not ok:
            failures.append("反例⑤未变红")

    if failures:
        print("  => 自证不通过：%s" % "；".join(failures))
        print("  => 按本项目口径：**这条守卫是装饰，拒绝合入**。")
        return 1
    print("  => 自证通过：五条判据**逐条**在反例下变红、在正控下全绿。")
    return 0


# ────────────────────────── 主程序 ──────────────────────────
def main(argv=None):
    ap = argparse.ArgumentParser(description="规格层守卫（opsx-swe-gb 五条判据）")
    ap.add_argument("--repo", default=None, help="仓库根；默认从本脚本位置向上找含 openspec/specs 的目录")
    ap.add_argument("--json", action="store_true", help="以 JSON 输出")
    ap.add_argument("--self-test", action="store_true", help="为五条判据各造一个反例，验证它们真的会红")
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
