#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""module_graph.py —— 机核层的守卫：把 `WC-ATOM-001` §四 机核清单的四条做成**会真红的东西**。

为什么需要它
------------
`WC-ATOM-001`（原子化编程约定，本项目**强制**）§四 机核清单逐条给了判据，
但它的今日状态栏自己写着第 1–4 条 **「未建」/「未建闸」**：

| # | 断言 | 落点 |
|---|---|---|
| 1 | 每个原子有且只有一句 `intent` | **未建** |
| 2 | 每原子的实现／测试／契约三件齐备 | 部分（测试在 `tests/*.rs`，与 src 不同夹） |
| 3 | `deps == import` 且无环 | **未建**（`tools/module_graph.py --check`） |
| 4 | 生成物与源一致（`WC-MODREG-001`） | 未建闸 |

**一个从不失败的检查不是装饰，是假证**——本项目既有口径（`tools/spec_bridge.py` 文件头）。
本脚本就是 §四 第 1–3 条（原子侧）的**执行者**：任一条不成立即非零退出，
且 `--self-test` 为**每条判据各造一个反例**，反例不变红即判该守卫是装饰、拒绝合入。

四条判据（与 `WC-ATOM-001` §二 六条约定的 A-1/A-2/A-4 逐条对应）
-----------------------------------------------------------------
① **A-1 单意图原子性**（`WC-ATOM-001` §二 A-1）
   每个模块**有且只有一句** `intent`，且 ≤30 字；出现并列两事（`与`/`和`/`及`）⇒ 报为可疑并列出。
   **数据源**：`WC-MODREG-001` §2 模块登记表（`M01`–`M10`）。
   **字段落点（如实声明，不许假装）**：登记表**没有** `intent` 列，最接近的是
   **「职责（一句话）」**列（`WC-MODREG-001-v0.1.md:28` 表头逐字）。故本判据按
   `--intent-column` 指定的列名取值，**默认 `职责`**；占位符（`待补`/`—`/空）与超过 30 字
   同样判红。若日后登记表补出 `intent` 列，用 `--intent-column intent` 即可切换，**判据不变**。

② **A-4 依赖单向 DAG 且 `deps == import`**（`WC-ATOM-001` §二 A-4）
   **怎么抽依赖**（逐条写死口径，避免"看起来建了图"）：
     · 建**模块树**：`src/lib.rs` 的 `pub mod X;` ＋ `src/<dir>/mod.rs` 的 `pub mod Y;`
       ⇒ `X → src/X.rs`（或 `src/X/mod.rs`）、`Y → src/<dir>/Y.rs`；`crate::` 即根。
     · `crate::A::B::…` / `world_core::A::…` 的**首个路径段** `A` 经模块树解析到**其宿主 .rs 文件**，
       再看该文件登记在哪个模块号下 ⇒ 得到一条**模块间边**（`use crate::gate::…` 是 M05 内部，
       不是跨模块边，故**不产边**——这是"兄弟模块集"的字面口径）。
     · **只抽生产路径**：`#[cfg(test)]` 起始的 `mod` 块内的 `use` **不计入**（`WC-MODREG-001`
       §4.2 自己就是这么划界的：「两条看似回边、实为测试内」）。`tests/*.rs` 整文件视为测试侧，
       是 `WC-ATOM-001` §三 的「测试」落点，不是实现依赖面。
     · 无环：对**声明边 ∪ 真实 import 边**跑 Kahn 拓扑排序，排不完即有环，报出环上模块号。
     · `deps == import`：登记表「依赖模块」列 → `{模块号: 出边集}`，与真实 import 边集**逐模块相等**；
       不一致时分别列出「声明了但代码里没有」与「代码里有但没声明」。

③ **A-2 四件同夹**（`WC-ATOM-001` §二 A-2）
   登记表里每个模块，其**实现／测试／契约**三者都要有落点（本仓 `tests/` 与 `src/` 不同夹——
   登记表自己说"部分（测试在 `tests/*.rs`，与 src 不同夹）"——故按**可指认**判，不按同目录判）：
     · **实现** ＝ 登记表「源码路径」列列出的文件在磁盘上真实存在（`src/**`）；
       登记表列了「计划路径」而源码未落成 ⇒ 报「实现缺」。
     · **测试** ＝ `tests/*.rs` 里能**指到该模块**的用例：该文件 `use world_core::<seg>` 里的 `<seg>`
       经模块树归属到本模块 ⇒ 该文件的用例算它的；一个锚点都不指 ⇒ 报「测试缺」。
       （`tests/*.rs` 整文件 `use world_core;` 而无具名子模块者——即 CLI 端到端用例——
       归 `src/main.rs` 的宿主模块，即「运行时入口」；理由写在 `test_anchor()`。）
     · **契约** ＝ 文档或规格里有落点：`docs/S2-设计/WC-IC-M<NN>-*.md` 分册存在，
       **或** `openspec/specs/**/spec.md` 里有 Requirement 的证据行指到本模块的测试锚点
       （`WC-ATOM-001` §三：「规格条目（一条 Requirement）＋ `WC-IC-M*` 模块接口契约」）。
     另有**覆盖面**一项：`src/**/*.rs` 里**没有任何模块号认领**的文件（`WC-MODREG-001` §4.3 #1
       自己登记的缺口就是 `src/error.rs`）逐条列出。

④ **自己也要自证**（`--self-test`）
   照 `tools/spec_bridge.py --self-test` 的结构与输出风格：搭一个**完好沙盒**做正控（四条应全绿），
   再**为每条判据各造一个反例**，反例不变红即判该守卫是装饰 ⇒ rc=1。
   正控与反例都打印逐字结果，并带"恢复后回到绿"的第二正控。

用法
----
    python3 tools/module_graph.py [--repo <world-core 或仓库根>] [--json]
    python3 tools/module_graph.py --self-test     # 每条判据一个反例，全红才 rc=0

退出码：0 = 全通过；1 = 有判据不成立（含自证失败）；2 = 用法错误。
"""

from __future__ import annotations

import argparse
import io
import json
import os
import re
import shutil
import sys
import tempfile

#: 登记表相对于 world-core 根的路径（唯一数据源，`WC-ATOM-001` §二 A-1 判据指定的数据源）。
MODREG_REL = os.path.join("docs", "S2-设计", "WC-MODREG-001-v0.1.md")
SRC_REL = "src"
TESTS_REL = "tests"
IC_DIR_REL = os.path.join("docs", "S2-设计")
SPECS_REL = os.path.join("openspec", "specs")

#: A-1 的字数上限（`WC-ATOM-001` §二 A-1：「必须有且只有一句 `intent`（≤30 字）」）。
#: 口径：**计 Unicode 字符数**，不计字节；判定前剥掉 Markdown 强调与空白（`**`／空白不算字）。
INTENT_MAX_CHARS = 30
#: 并列两事的连接词（A-1：「出现并列两事 ⇒ 拆」）。只取这三个字面，不猜别的词。
PARALLEL_MARKERS = ("与", "和", "及")
#: 取值单元格里的占位符：等于这些值即视为"没写"。
PLACEHOLDERS = ("", "—", "-", "–", "待补", "待定", "n/a", "N/A", "<待人工>", "待人工指派")

#: 模块号形态（判定面唯一编号口径，`WC-MODREG-001` §3：「`M` + 两位数字，左补零」）。
M_RE = re.compile(r"M\d{2}")


# 非 UTF-8 控制台（Windows GBK/cp936）下，中文与记号会让 print 抛 UnicodeEncodeError
# —— 那会变成"门禁自己崩了"的假失败。按本项目既有口径（`tools/spec_bridge.py:44-51`）：
# **只重配 errors，不改 encoding**（UTF-8 环境下逐字节等价）；记号一律用 ASCII。
for _s in (sys.stdout, sys.stderr):
    try:
        _s.reconfigure(errors="replace")
    except Exception:
        pass


# ────────────────────────── 工具 ──────────────────────────
def find_worldcore(start):
    """从 start 往上找 `world-core/`（判据面以它为准：`tools/` 的父目录）。"""
    p = os.path.abspath(start)
    for cand in [p] + list(_parents(p)):
        if os.path.isfile(os.path.join(cand, MODREG_REL)):
            return cand
    return None


def _parents(p):
    out = []
    cur = os.path.dirname(p)
    while True:
        out.append(cur)
        nxt = os.path.dirname(cur)
        if nxt == cur:
            return out
        cur = nxt


def rel(base, p):
    try:
        return os.path.relpath(p, base).replace("\\", "/")
    except Exception:
        return str(p)


def read_text(p):
    """按项目编码纪律读文件：UTF-8、`errors="replace"`（坏字节不使门禁崩，而由编码闸单独判）。"""
    try:
        with io.open(p, encoding="utf-8", errors="replace") as fh:
            return fh.read()
    except Exception:
        return ""


def norm_cell(s):
    """单元格归一：去 Markdown 强调、去首尾空白与全角空格。"""
    return (s or "").replace("**", "").replace("`", "").strip().strip("\u3000").strip()


def is_placeholder(s):
    return norm_cell(s) in PLACEHOLDERS


def visual_len(s):
    """判据用的"字数"：剥掉 Markdown 强调与全部空白后计 Unicode 字符数。"""
    return len(re.sub(r"\s+", "", (s or "").replace("**", "").replace("`", "")))


# ────────────────────────── 登记表解析 ──────────────────────────
def read_registry(wc):
    """返回 (path, text, {模块号: {line, name, intent, src_cell, ifs, deps, deps_cell}})。

    只在 `## §2 模块登记表` 这一节内取表（附录 A 是**旧表**、口径不同，
    混取会让判据失去意义——同 `tools/ic_books_check.py:80-91` 的口径）。
    """
    p = os.path.join(wc, MODREG_REL)
    if not os.path.isfile(p):
        return p, "", {}
    text = read_text(p)
    sec = re.search(r"(?ms)^##\s*§2\s*模块登记表(.*?)(?=^##\s|\Z)", text)
    rows = {}
    if not sec:
        return p, text, rows
    lines = sec.group(1).split("\n")
    # 该节的绝对行号偏移（报错必须给**文件里的真行号**）
    off = text[: sec.start(1)].count("\n")
    for i, line in enumerate(lines):
        m = re.match(r"^\|\s*\*{0,2}(M\d{2})\*{0,2}\s*\|(.*)$", line)
        if not m:
            continue
        cells = m.group(2).split("|")
        if len(cells) < 5:
            continue
        mid = m.group(1)
        src_cell = norm_cell(cells[2])
        # 源码路径列：取 `src/...` 形态的路径。**目录那一条保留尾斜杠**（`src/carrier/`）——
        # 去掉尾斜杠会让"目录存在"冒充"实现文件存在"（`os.path.exists('src/carrier')` 为真）。
        src_all = []
        for raw in re.findall(r"src/[A-Za-z0-9_./-]+", src_cell):
            raw = raw.rstrip("/")                         # 去掉 re 可能吃进的尾斜杠
            if re.search(re.escape(raw) + r"/", src_cell):
                raw = raw + "/"                           # 原格写的是目录 ⇒ 还原目录形态
            if raw not in src_all:
                src_all.append(raw)
        rows[mid] = {
            "line": off + i + 1,
            "name": norm_cell(re.sub(r"（[^）]*）\s*$", "", cells[0])),
            "intent": norm_cell(cells[1]),
            "intent_raw": (cells[1] or "").strip(),
            "src_cell": src_cell,
            "src_all": src_all,
            "src_files": [s for s in src_all if s.endswith(".rs")],
            "src_dirs": [s for s in src_all if s.endswith("/")],
            "ifs": sorted(set(re.findall(r"IF-\d{3}", "|".join(cells[3:-1])))),
            "deps_cell": norm_cell(cells[4]),
            "deps": set(M_RE.findall(cells[4])),
        }
        # 「计划路径」的判据：登记表在这一行里自己声明了"尚未落成 / 计划"（M10 行的字面口径）。
        rows[mid]["planned"] = ("计划" in src_cell) or ("尚未落成" in src_cell)
    return p, text, rows


#: `WC-MODREG-001` §2.1「共同模块、未登记文件与模块号边界」表的行。
SHARED_ROW_RE = re.compile(r"^\|\s*`(?P<path>[^`]+)`\s*\|(.*)$")


def read_shared_modules(text):
    """`WC-MODREG-001` §2.1 表 → {源码路径: 归属模块号}。

    为什么必须读它：`src/event.rs`、`src/project/mod.rs` 是登记表自己声明的**共同模块**
    （`WC-MODREG-001-v0.1.md:49-50`：「不占号，属 M01 的机制面」／「横跨 M06/M07」）。
    不读它，`use crate::event;` 就会被判成"宿主文件没有被任何模块号登记"——
    那是**判据自己造出来的假告警**，不是项目缺陷。归属按行内出现的第一个模块号取。
    """
    out = {}
    sec = re.search(r"(?ms)^###\s*§2\.1(.*?)(?=^###\s|^##\s|\Z)", text)
    if not sec:
        return out
    for line in sec.group(1).split("\n"):
        m = SHARED_ROW_RE.match(line)
        if not m:
            continue
        path = m.group(1).strip()
        if not path.startswith("src/"):
            continue
        ids = M_RE.findall(m.group(2))
        if ids:
            out.setdefault(path, ids[0])
    return out


# ────────────────────────── 源码面：模块树与边 ──────────────────────────
MOD_DECL_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
CFG_TEST_RE = re.compile(r"^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]")
#: `use` 的四种形态都收：
#:  ① `use crate::gate::Decision;`        ② `use world_core::readmodel::State;`
#:  ③ `use gate::{Decision, Policy};`（**根级裸名**，2018 之后合法：`src/lib.rs:23-26` 就是这么写的）
#:  ④ `use super::*;` / `use self::x;`（不是兄弟模块面，解析时丢弃）
#: `kind` 为 None ⇔ 根级裸名 ⇒ 只有首段**真在 `crate::` 作用域里**（见 `crate_mod_names`）才算边。
IMPORT_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+"
    r"(?:(?P<kind>crate|world_core|super|self)\s*::|::\s*)?"
    r"(?P<rest>[A-Za-z_][A-Za-z0-9_:{}, \t]*)"
)
#: `#[test]` 行（含 `#[tokio::test]`、带参数形态；与 `#[cfg(unix)]` 等其它属性无关）。
TEST_ATTR_RE = re.compile(r"^\s*#\s*\[\s*(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*test\s*[\]\(]")
#: 根级条目声明（`pub struct World {` / `pub enum X` / `pub trait T` / `pub fn f` / `pub type A`）。
ROOT_ITEM_RE = re.compile(
    r"^\s*pub\s+(?:struct|enum|trait|fn|type|const|static|union)\s+([A-Za-z_][A-Za-z0-9_]*)")
#: `fn name(` 行（测试函数名判据）。
FN_RE = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[(<]")


def walk_rs(root):
    """递归列出 `root` 下的 `.rs`（排序，跳过 target/.git）。"""
    out = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(
            d for d in dirnames if d not in (".git", "target", "node_modules", "__pycache__")
        )
        for f in sorted(filenames):
            if f.endswith(".rs"):
                out.append(os.path.join(dirpath, f))
    return out


def crate_mod_names(text, extra=()):
    """`crate::` 的作用域里有哪些名字：根模块的 `mod` 声明 ＋ 根文件里 `use` 的首段。

    为什么把 `use` 首段也算进来：`src/lib.rs` 里 `use gate::{Decision, Policy};`（`src/lib.rs:23`）
    与 `main.rs` 里 `use world_core::World;` 的 `World`（定义在 `src/lib.rs`）都不是
    `pub mod` 声明出来的名字，却是真实的 `crate::` 名字；只认 `mod` 声明会把这些
    "根级再导出"漏成"解析不到"。
    """
    names = dict(extra)
    for i, line in enumerate(text.split("\n"), 1):
        m = MOD_DECL_RE.match(line)
        if m:
            names.setdefault(m.group(1), i)
            continue
        # 根级**条目**声明：`use world_core::World;` 的 `World` 来自 `src/lib.rs:59`
        # `pub struct World {`——它不是 mod、也不在任何 use 里，只认前两者会漏成"解析不到"。
        m = ROOT_ITEM_RE.match(line)
        if m:
            names.setdefault(m.group(1), i)
            continue
        m = IMPORT_RE.match(line)
        if m and m.group("kind") == "crate":
            seg = re.split(r"[:{]", m.group("rest"), 1)[0].strip()
            if seg:
                names.setdefault(seg, i)
    return names


def build_module_tree(wc, files):
    """建模块树：{模块路径（`a::b`）: 承载它的 .rs 文件}；返回 (tree, test_only_mods)。

    · 根：`src/lib.rs`（库根）与 `src/main.rs`（bin 根）都挂在 `crate::` 上。
    · `src/<dir>/mod.rs` ⇒ 模块路径为 `<dir>`；其内的 `pub mod Y;` ⇒ `<dir>::Y`。
    · `#[cfg(test)]` 之后紧跟的 `mod X;` 记进 `test_only_mods`（该模块只在测试编译单元里存在）。
    """
    tree, test_only = {}, set()
    by_rel = {rel(wc, f): f for f in files}

    def visit(relpath, modpath):
        f = os.path.join(wc, relpath)
        tree[modpath] = f
        text = read_text(f)
        d = os.path.dirname(relpath).replace("\\", "/")
        lines = text.split("\n")
        for i, line in enumerate(lines, 1):
            m = MOD_DECL_RE.match(line)
            if not m:
                continue
            child = m.group(1)
            child_rel = "%s/%s.rs" % (d, child) if d else "src/%s.rs" % child
            if child_rel not in by_rel:
                if "%s/mod.rs" % child_rel[: -len(".rs")] in by_rel:
                    child_rel = "%s/mod.rs" % child_rel[: -len(".rs")]
                else:
                    continue
            prev = lines[i - 2] if i >= 2 else ""
            if CFG_TEST_RE.match(prev or ""):
                test_only.add(child_rel)
                continue
            visit(child_rel, "%s::%s" % (modpath, child) if modpath else child)

    for root in ("src/lib.rs", "src/main.rs"):
        if root in by_rel:
            visit(root, "")
    return tree, test_only


def file_use_edges(path, text, tree, test_only, crate_names):
    """单个 .rs 文件 → (生产边集, 测试边集)，元素为**首段名字**（`gate` / `World` / `carrier`…）。

    · 行级状态机：`#[cfg(test)]` 之后紧跟的 `mod NAME {` 起始的整块计入"测试边"，
      其余计入"生产边"（`WC-MODREG-001` §4.2 的划界口径：测试内的 use 不是发布产物里的边）。
    · 根级裸名（`use gate::{…};`）在 `crate_names` 里查到名字才算边；查不到即丢弃
      （不会把 `use serde_json::…` 误算成兄弟模块边）。
    """
    prod, tests = set(), set()
    in_test = 0
    lines = text.split("\n")
    for i, line in enumerate(lines, 1):
        if in_test and re.match(r"^\s*\}\s*$", line):
            in_test -= 1
            continue
        if CFG_TEST_RE.match(line):
            j = i
            while j < len(lines) and not lines[j].strip():
                j += 1
            if j < len(lines) and re.match(r"^\s*(?:pub\s+)?mod\s+\w+\s*\{", lines[j]):
                in_test += 1
            continue
        m = IMPORT_RE.match(line)
        if not m:
            continue
        kind, rest = m.group("kind"), (m.group("rest") or "")
        if kind in ("super", "self"):
            continue                                  # 模块内部，不是"兄弟模块"面
        seg = re.split(r"[:{]", rest, 1)[0].strip()
        if not seg:
            continue
        if kind is None and seg not in crate_names:
            continue                                  # 根级裸名但不在 crate 作用域里 ⇒ 外部 crate
        (tests if in_test else prod).add(seg)
    return prod, tests


def index_source(wc):
    """把 `src/**/*.rs` 过一遍，返回一份"源码面"字典。"""
    src = os.path.join(wc, SRC_REL)
    files = walk_rs(src)
    tree, test_only = build_module_tree(wc, files)
    # `crate::` 作用域的名字：根文件的 mod 声明与 use 首段 ＋ **树上每个路径的首段**。
    # 前者覆盖"根级再导出"（`World`），后者覆盖"根级裸名 use"（`use readmodel::State;`，
    # 2018 之后它与 `use crate::readmodel::State;` 同义，`src/lib.rs:23-26` 就是裸名写法）。
    crate_names = {}
    for root_rel in ("src/lib.rs", "src/main.rs"):
        p = os.path.join(wc, root_rel)
        if os.path.isfile(p):
            crate_names = crate_mod_names(read_text(p), crate_names)
    for modpath in tree:
        first = modpath.split("::")[0]
        if first:
            crate_names.setdefault(first, 0)
    # 根级再导出的**承载文件**：`src/lib.rs` 里 `use crate::gate::{…};` ⇒ `Decision` 等
    # 根级名字的宿主是 lib.rs。`use world_core::World;` 因此能落到 M04，而不是"解析不到"。
    root_exports, root_items = {}, {}
    for root_rel in ("src/lib.rs", "src/main.rs"):
        p = os.path.join(wc, root_rel)
        if not os.path.isfile(p):
            continue
        for line in read_text(p).split("\n"):
            m = ROOT_ITEM_RE.match(line)             # `pub struct World {`（`src/lib.rs:59`）
            if m:
                root_items.setdefault(m.group(1), p)
            m = IMPORT_RE.match(line)
            if m and m.group("kind") == "crate":
                seg = re.split(r"[:{]", m.group("rest") or "", 1)[0].strip()
                if seg:
                    root_exports.setdefault(seg, p)
    info = {}
    for f in files:
        text = read_text(f)
        prod, tests = file_use_edges(f, text, tree, test_only, crate_names)
        info[rel(wc, f)] = {"prod_segs": prod, "test_segs": tests, "text": text}
    return {"files": files, "tree": tree, "test_only": test_only, "info": info,
            "crate_names": crate_names, "root_exports": root_exports, "root_items": root_items}


def owning_module(wc, rows, f):
    """某 .rs 文件登记在哪个模块号下（按登记表「源码路径」列的字面前缀归属）。

    `src/carrier/` 这类**目录**路径按前缀归属（`src/carrier/run.rs` 属 `M10`）；
    `src/ontology.rs` 这类**文件**路径要求逐字相等。
    """
    r = rel(wc, f)
    hits = []
    for mid, row in rows.items():
        for sp in row["src_all"]:
            if sp.endswith("/"):
                if r.startswith(sp):
                    hits.append(mid)
                    break
            elif r == sp:
                hits.append(mid)
                break
    if not hits:
        return None, 0
    return sorted(hits)[0], len(hits)


def seg_owner(wc, rows, source, seg, shared=None):
    """`crate::<seg>` / 根级裸名 `<seg>::` 解析到哪个模块号（模块树 → 宿主文件 → 登记行）。

    三级落点（都写死，不做"看起来像"）：
      ① 模块树里的 `<seg>`（`src/<seg>.rs` / `src/<dir>/mod.rs` 的 `pub mod` 声明）；
      ② 根级再导出的承载文件（`src/lib.rs` 里 `use crate::<seg>::…;`）；
      ③ 同名文件 `src/<seg>.rs`。
    宿主文件 → 模块号：先查登记表「源码路径」列，再查 `§2.1` 声明的**共同模块**归属。

    **不做"模块内部引用"的过滤**：那属于调用方（`build_edges` 按导入文件的归属模块过滤）。
    在这里过滤会让"从测试文件里解析兄弟模块"（没有导入方模块号可比）一律返回 None——
    那正是 A-2「测试」件判据被静默判红的根因。
    """
    shared = shared or {}
    path = None
    for modpath, f in source["tree"].items():
        if modpath == seg:
            path = f
            break
    if path is None:
        path = source.get("root_items", {}).get(seg)   # ② 根级条目（`pub struct World`）
    if path is None:
        path = source.get("root_exports", {}).get(seg)
    if path is None:
        cand = os.path.join(wc, SRC_REL, seg + ".rs")
        if os.path.isfile(cand):
            path = cand
    if path is None:
        return None, "解析不到 `%s` 的宿主文件（模块树里没有它）" % seg
    mid, n = owning_module(wc, rows, path)
    if mid is None:
        # ③ `WC-MODREG-001` §2.1 明示的共同模块：按声明的归属算（不是"没登记"）
        mid = shared.get(rel(wc, path))
        if mid is None:
            return None, "`%s` 的宿主文件 %s 没有被任何模块号登记" % (seg, rel(wc, path))
        return mid, ""
    if n > 1:
        return mid, "`%s` 的宿主文件 %s 同时命中多个登记行" % (seg, rel(wc, path))
    return mid, ""


def build_edges(wc, rows, source, shared=None):
    """真实 import 边：{模块号: 出边集}（生产面），并返回测试面边与解析告警。"""
    prod_edges = {mid: set() for mid in rows}
    test_edges = {mid: set() for mid in rows}
    warn = []
    for r, d in sorted(source["info"].items()):
        owner, n = owning_module(wc, rows, os.path.join(wc, r))
        if owner is None:
            continue                                  # 未登记文件：A-2 的覆盖面单独报
        for bucket, segs in (("prod", d["prod_segs"]), ("test", d["test_segs"])):
            for seg in sorted(segs):
                tgt, why = seg_owner(wc, rows, source, seg, shared)
                if tgt is None:
                    if why:
                        warn.append("%s: `%s`：%s" % (r, seg, why))
                    continue
                if tgt == owner:
                    continue                          # 模块**内部**引用，不构成跨模块边
                (prod_edges if bucket == "prod" else test_edges)[owner].add(tgt)
    return prod_edges, test_edges, warn


# ────────────────────────── 环检测 ──────────────────────────
def find_cycles(nodes, edges):
    """Kahn 拓扑排序；排不完 ⇒ 有环（报出环上模块号）。返回 (是否有环, 环上节点列表)。"""
    nodes = sorted(nodes)
    indeg = {n: 0 for n in nodes}
    adj = {n: set() for n in nodes}
    for u in nodes:
        for v in edges.get(u, ()):
            if v in indeg and v not in adj[u]:
                adj[u].add(v)
                indeg[v] += 1
    queue = [n for n in nodes if indeg[n] == 0]
    seen = 0
    while queue:
        u = queue.pop()
        seen += 1
        for v in sorted(adj[u]):
            indeg[v] -= 1
            if indeg[v] == 0:
                queue.append(v)
    if seen == len(nodes):
        return False, []
    left = [n for n in nodes if indeg[n] > 0]
    # 从剩余节点里走出一条真环，便于人看（自环也覆盖）
    path, cur = [], left[0]
    while cur not in path:
        path.append(cur)
        nxt = [v for v in sorted(adj[cur]) if indeg[v] > 0]
        if not nxt:
            break
        cur = nxt[0]
    if cur in path:
        path = path[path.index(cur):] + [cur]
    return True, path


# ────────────────────────── 测试锚点与契约条目 ──────────────────────────
ANCHOR_RE = re.compile(r"`([^`]+?)::([A-Za-z_][A-Za-z0-9_]*)`")
REQ_RE = re.compile(r"^###\s+Requirement:\s*(.+?)\s*$")


def _test_fns(text):
    """`tests/*.rs` → [(fn名, 函数体起始行号, 函数体文本)]，只取 `#[test]` 标注的函数。

    `#[test]` 与 `fn` 之间允许夹 `#[cfg(unix)]` 等属性行；函数体按**大括号配平**截取
    （不靠缩进猜，故多行字符串里的括号会被计入——这是保守方向：宁可多截一点，
    也不要把后续用例误算进前一个用例）。
    """
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        m = FN_RE.match(line)
        if not m:
            continue
        j = i - 1
        is_test = False
        while j >= 0 and (lines[j].strip().startswith("#[") or not lines[j].strip()):
            if TEST_ATTR_RE.match(lines[j]):
                is_test = True
                break
            j -= 1
        if not is_test:
            continue
        depth, buf, started = 0, [], False
        for k in range(i, len(lines)):
            cur = lines[k]
            buf.append(cur)
            depth += cur.count("{") - cur.count("}")
            if "{" in cur:
                started = True
            if started and depth <= 0:
                break
        out.append((m.group(1), i + 1, "\n".join(buf)))
    return out


def all_test_tokens(wc):
    """`tests/**/*.rs` 里**真实存在的** `#[test]` 函数锚点集合（`tests/x.rs::fn`）。

    用途：规格证据行只算"有效证据"——它指向的用例必须真的在 `tests/` 里存在
    （这条口径与 `tools/spec_bridge.py` 判据②「证据存在性」同源）。
    """
    real = set()
    tests_dir = os.path.join(wc, TESTS_REL)
    if not os.path.isdir(tests_dir):
        return real
    for f in walk_rs(tests_dir):
        r = rel(wc, f)
        for fn, _line, _body in _test_fns(read_text(f)):
            real.add("%s::%s" % (r, fn))
    return real


def module_tokens(rows, body):
    """函数体里出现了哪些模块（按登记源码路径的**文件名词干 / 目录名**标识符指认）。

    可指认的名字：`src/readmodel.rs` ⇒ `readmodel`；`src/project/language.rs` ⇒ `language`；
    `src/carrier/` ⇒ `carrier`；并补上 `main`（程序入口）与 `lib`（运行时装配）。
    用词边界匹配，避免 `readmodel` 命中 `readmodel_x` 这类别的标识符。
    """
    hits = set()
    for mid, row in rows.items():
        names = set()
        for sp in row["src_all"]:
            if sp.endswith("/"):
                names.add(sp.rstrip("/").split("/")[-1])
            else:
                names.add(os.path.basename(sp)[: -len(".rs")])
        # 模块名里的关键段（`src/project/language.rs` ⇒ `language` 已在上列；`src/lib.rs` ⇒ `lib`）
        for name in names:
            if re.search(r"(?<![A-Za-z0-9_])" + re.escape(name) + r"(?![A-Za-z0-9_])", body):
                hits.add(mid)
                break
    return hits


def index_specs(wc, base=None):
    """`openspec/specs/**/spec.md` → 证据锚点 {`tests/x.rs::fn`: [(spec_rel, line, 需求标题)]}。

    `WC-ATOM-001` §三 把「契约文档」钉在「规格条目（一条 Requirement）」与
    「`WC-IC-M*` 模块接口契约」两处，本索引就是前者的判定面。
    `base` = 仓库根（默认 world-core 的上一级）；自证沙盒会显式传入自己的根。
    """
    idx = {}
    base = base or os.path.dirname(os.path.abspath(wc.rstrip("\\/")))
    specs_root = os.path.join(base, SPECS_REL)
    if not os.path.isdir(specs_root):
        return idx
    for dirpath, dirnames, filenames in os.walk(specs_root):
        dirnames[:] = sorted(dirnames)
        for fn in sorted(filenames):
            if fn != "spec.md":
                continue
            p = os.path.join(dirpath, fn)
            cur = ""
            for i, line in enumerate(read_text(p).split("\n"), 1):
                m = REQ_RE.match(line)
                if m:
                    cur = m.group(1)
                if "证据" not in line:
                    continue
                for tok, fnn in ANCHOR_RE.findall(line):
                    key = "%s::%s" % (tok.strip(), fnn)
                    idx.setdefault(key, []).append((rel(base, p), i, cur))
    return idx


def test_anchor(wc, rows, source, spec_idx, real_tokens=None):
    """模块号 → 测试锚点列表 [{file, fn, line, module, used_by}]。

    指认规则（写死，不做"看起来像"）：
      ① **函数体级**：`tests/*.rs` 里每个 `#[test]` 函数的**函数体**内出现 `world_core::<seg>`、
         或出现该模块登记源码路径的**文件名/目录名标识符**（`readmodel` / `language` / `visual` /
         `carrier` …）⇒ 该用例算这个模块的。
         `#[test]` 的函数体里一般没有 `use`，故**必须**按标识符指认；只按"文件级 use"指认
         会把同一文件里所有用例判给所有被 import 的模块（那种指认查不出"某模块一个用例都没有"）。
      ② **CLI 端到端**：函数体里出现 `Command` / `env!("CARGO_BIN_EXE_` ⇒ 它是打二进制的端到端用例，
         归 `src/main.rs` 的宿主模块（程序入口）。这不是"猜"：这些用例的全部证据都指向产物入口。
    """
    out = {mid: [] for mid in rows}
    tests_dir = os.path.join(wc, TESTS_REL)
    if not os.path.isdir(tests_dir):
        return out
    main_owner = None
    for mid, row in rows.items():
        if any(sf.endswith("src/main.rs") for sf in row["src_files"]):
            main_owner = mid
    for f in walk_rs(tests_dir):
        r = rel(wc, f)
        fns = _test_fns(read_text(f))
        for fn, line_no, body in fns:
            owners = module_tokens(rows, body)
            if main_owner and ("Command" in body or 'env!("CARGO_BIN_EXE_' in body):
                owners.add(main_owner)
            for mid in sorted(owners):
                out[mid].append({"file": r, "fn": fn, "line": line_no, "used_by": []})
    for mid, items in out.items():
        for it in items:
            tok = "%s::%s" % (it["file"], it["fn"])
            # 只有**真实存在**的用例才算有效证据（证据行指向不存在的函数 ⇒ 不算契约落点）
            if real_tokens is not None and tok not in real_tokens:
                continue
            it["used_by"] = [u for u in spec_idx.get(tok, [])]
    for mid in out:
        out[mid].sort(key=lambda x: (x["file"], x["line"]))
    return out


# ────────────────────────── 四条判据 ──────────────────────────
def j_a1_intent(wc, rows, reg_path, reg_text):
    """① A-1 单意图原子性：每个模块有且只有一句 `intent`（≤30 字），并列两事即报可疑。"""
    bad = []
    names = {}
    if not rows:
        return ["%s —— §2 模块登记表里一个 `| M0x |` 行都取不到（表被清空/格式被改/文件缺失）："
                "**空登记表不得被当作「一致」**" % rel(wc, reg_path)]
    for mid, row in sorted(rows.items()):
        loc = "%s:%d" % (rel(wc, reg_path), row["line"])
        raw, val = row["intent_raw"], row["intent"]
        if is_placeholder(val):
            bad.append("%s —— %s 的 `intent` **不是一句**（取值=%r，属占位符/空）"
                       % (loc, mid, val))
            continue
        n = visual_len(val)
        if n > INTENT_MAX_CHARS:
            bad.append("%s —— %s 的 `intent` **不是「一句话」**：%d 字 > 上限 %d 字；逐字=%r"
                       % (loc, mid, n, INTENT_MAX_CHARS, val))
        hits = [mk for mk in PARALLEL_MARKERS if mk in val]
        if hits:
            bad.append("%s —— %s 的 `intent` 出现**并列两事**（连接词 %s）：一句里塞了两件事 ⇒ 拆；逐字=%r"
                       % (loc, mid, "/".join("`%s`" % h for h in hits), val))
        key = re.sub(r"\s+", "", val)
        if key in names:
            bad.append("%s —— %s 与 %s 的 `intent` **逐字相同**（%r）：一句意图只能对一个模块，"
                       "复制即不成立" % (loc, mid, names[key], val))
        else:
            names[key] = mid
    return bad


def j_a4_dag_deps(wc, rows, reg_path, prod_edges, test_edges, unres):
    """② A-4 依赖单向 DAG，且 `deps == import`（逐模块逐边相等）。"""
    bad = []
    if not rows:
        return ["%s —— §2 模块登记表取不到行，「依赖模块」列**一条边都取不到**："
                "依赖判据不得因取不到而静默通过" % rel(wc, reg_path)]
    for mid, row in sorted(rows.items()):
        col = row["deps"]
        for d in sorted(col):
            if d not in rows:
                bad.append("%s:%d —— %s 的「依赖模块」列写了 %s，但登记表里没有这一行"
                           % (rel(wc, reg_path), row["line"], mid, d))
        if not row["src_files"]:
            continue                                   # 源码未落成：归 A-2「实现缺」，不在此重复报
        real = prod_edges.get(mid, set())
        missing = sorted(col - real)
        extra = sorted(real - col)
        if missing:
            bad.append("%s:%d —— %s：**声明了但代码里没有**（%s）—— 声明集 %s vs 真实 import 集 %s"
                       % (rel(wc, reg_path), row["line"], mid, ",".join(missing),
                          "{%s}" % ",".join(sorted(col)) or "{}",
                          "{%s}" % ",".join(sorted(real)) or "{}"))
        if extra:
            bad.append("%s:%d —— %s：**代码里有但没声明**（%s）—— 声明集 %s vs 真实 import 集 %s"
                       % (rel(wc, reg_path), row["line"], mid, ",".join(extra),
                          "{%s}" % ",".join(sorted(col)) or "{}",
                          "{%s}" % ",".join(sorted(real)) or "{}"))
    # 无环：判**声明边 ∪ 真实 import 边**（两面任一面成环都是"依赖不是单向 DAG"）
    nodes = sorted(rows)
    union = {mid: set(rows[mid]["deps"]) for mid in nodes}
    for mid in nodes:
        union.setdefault(mid, set())
        union[mid] |= prod_edges.get(mid, set())
    cyc, path = find_cycles(nodes, union)
    if cyc:
        bad.append("依赖图**有环**（拓扑排序失败，Kahn 剩余节点）：%s —— "
                   "`WC-ATOM-001` §二 A-4 要求单向 DAG" % " → ".join(path or ["?"]))
    for w in unres:
        bad.append("依赖抽取告警（不静默跳过）：%s" % w)
    return bad


def j_a2_four_in_one(wc, rows, reg_path, source, anchors, spec_idx, shared=None):
    """③ A-2 每模块的实现／测试／契约三件齐备（＋未登记源码文件的覆盖面）。"""
    shared = shared or {}
    bad = []
    if not rows:
        return ["%s —— §2 模块登记表取不到行，三件齐备**无从判定**" % rel(wc, reg_path)]
    for mid, row in sorted(rows.items()):
        loc = "%s:%d" % (rel(wc, reg_path), row["line"])
        # 实现
        present = [p for p in row["src_all"] if os.path.exists(os.path.join(wc, p))]
        missing = [p for p in row["src_all"] if not os.path.exists(os.path.join(wc, p))]
        if not row["src_all"]:
            bad.append("%s —— %s **实现缺**：登记表「源码路径」列一个 `src/…` 路径都没写"
                       % (loc, mid))
        elif not present:
            if row.get("planned"):
                bad.append("%s —— %s **实现缺**：登记表列的是**计划路径** %s，磁盘上不存在"
                           "（本行已自述「尚未落成」——未落成不得当作已实现）"
                           % (loc, mid, "、".join(row["src_all"])))
            else:
                bad.append("%s —— %s **实现缺**：登记表「源码路径」列的 %s 磁盘上不存在"
                           % (loc, mid, "、".join(row["src_all"])))
        elif missing:
            bad.append("%s —— %s **实现缺**：登记表列了 %s，磁盘上不存在"
                       % (loc, mid, "、".join(missing)))
        # 测试
        if not anchors.get(mid):
            bad.append("%s —— %s **测试缺**：`%s/` 下没有任何用例能指到它"
                       "（判据＝该用例函数体内出现本模块源码路径的文件名/目录名标识符，"
                       "或 `world_core::<子模块>`；CLI 端到端用例归程序入口）"
                       % (loc, mid, TESTS_REL))
        # 契约
        book = sorted(
            f for f in (os.listdir(os.path.join(wc, IC_DIR_REL))
                        if os.path.isdir(os.path.join(wc, IC_DIR_REL)) else [])
            if re.fullmatch(r"WC-IC-%s-v\d+\.\d+\.md" % mid, f)
        )
        cited = []
        for a in anchors.get(mid, []):
            cited.extend(a["used_by"])
        if not book and not cited:
            bad.append("%s —— %s **契约缺**：既没有 `%s/WC-IC-%s-*.md` 分册，"
                       "也没有 `openspec/specs/**/spec.md` 里任何 Requirement 的证据行指向它的测试锚点"
                       % (loc, mid, IC_DIR_REL.replace("\\", "/"), mid))
    # 覆盖面：src 下没有任何模块号认领、也**不在 §2.1 共同模块声明里**的 .rs
    unclaimed = []
    for f in source["files"]:
        r = rel(wc, f)
        if r in shared:
            continue                                      # §2.1 已显式声明归属
        mid, n = owning_module(wc, rows, f)
        if mid is None or n > 1:
            unclaimed.append(r)
    for r in sorted(unclaimed):
        bad.append("覆盖面 —— `src/` 下的 `%s` **没有被任何模块号认领**："
                   "它既不在登记表里，也不在「共同模块」两行内（`WC-MODREG-001` §4.3 的登记缺口口径）" % r)
    return bad


# ────────────────────────── 报告 ──────────────────────────
def check(wc, intent_column="职责", base=None):
    """跑四条判据（本文件是判据③的**执行者**，故 ①–④ 全跑）。返回 report 字典。"""
    reg_path, reg_text, rows = read_registry(wc)
    shared = read_shared_modules(reg_text)
    source = index_source(wc)
    spec_idx = index_specs(wc, base)
    real_tokens = all_test_tokens(wc)
    anchors = test_anchor(wc, rows, source, spec_idx, real_tokens)
    prod_edges, test_edges, unres = build_edges(wc, rows, source, shared)

    judgments = [
        ("① A-1 单意图原子性（每模块有且只有一句 intent，≤%d 字，无并列两事）" % INTENT_MAX_CHARS,
         "a1_intent", j_a1_intent(wc, rows, reg_path, reg_text)),
        ("② A-4 依赖单向 DAG 且 deps == import（逐模块逐边相等）",
         "a4_dag_deps", j_a4_dag_deps(wc, rows, reg_path, prod_edges, test_edges, unres)),
        ("③ A-2 四件同夹（实现／测试／契约三件齐备，＋源码文件全覆盖）",
         "a2_four_in_one", j_a2_four_in_one(wc, rows, reg_path, source, anchors, spec_idx, shared)),
    ]

    res = []
    for title, key, bad in judgments:
        res.append({"judgment": title, "key": key, "ok": not bad, "offenders": bad})

    modules = []
    for mid, row in sorted(rows.items()):
        present = [p for p in row["src_all"] if os.path.exists(os.path.join(wc, p))]
        modules.append({
            "id": mid,
            "line": row["line"],
            "name": row["name"],
            "intent": row["intent"],
            "intent_chars": visual_len(row["intent"]),
            "src_declared": row["src_all"],
            "impl_present": present,
            "ifs": row["ifs"],
            "deps_declared": sorted(row["deps"]),
            "deps_import": sorted(prod_edges.get(mid, set())),
            "deps_import_tests_only": sorted(test_edges.get(mid, set()) - prod_edges.get(mid, set())),
            "test_anchors": anchors.get(mid, []),
        })
    unclaimed = []
    for f in source["files"]:
        r = rel(wc, f)
        if r in shared:
            continue                                      # §2.1 已显式声明归属
        mid, n = owning_module(wc, rows, f)
        if mid is None or n > 1:
            unclaimed.append(r)
    return {
        "world_core": wc,
        "intent_column": intent_column,
        "registry": rel(wc, reg_path),
        "shared_modules": shared,
        "judgments": res,
        "modules": modules,
        "files_unclaimed": sorted(unclaimed),
        "test_only_modules": sorted(source["test_only"]),
        "planned_modules": sorted(m for m, r in rows.items() if not any(
            os.path.exists(os.path.join(wc, p)) for p in r["src_all"])),
        "anchors_total": sorted(
            "%s::%s" % (a["file"], a["fn"]) for m in anchors for a in anchors[m]
        ),
        "import_edges_production": {m: sorted(prod_edges.get(m, set())) for m in sorted(rows)},
        "import_edges_test_only": {m: sorted(test_edges.get(m, set())) for m in sorted(rows)},
    }


# ────────────────────────── 自证：每条判据各造一个反例 ──────────────────────────
#: 沙盒登记表（6 列，与 `WC-MODREG-001` §2 表结构一致：
#: 模块号 | 模块名 | 职责（一句话） | 源码路径 | 提供接口 | 依赖模块）。
#: ⚠️ 沙盒必须**自洽**：依赖列 == 真实 import 边集（生产面）——否则正控自己就红，
#: 反例也就无从证明"红是反例造成的"。
SANDBOX_MODREG = """# WC-MODREG-001 模块清单与模块号登记表

## §1 目的与范围

沙盒用最小登记表，只为验证守卫会红。

## §2 模块登记表

| 模块号 | 模块名 | 职责（一句话） | 源码路径 | 提供接口 | 依赖模块 |
|---|---|---|---|---|---|
| **M01** | 本体 | 词表身份的唯一出处 | `src/ontology.rs` | **IF-005** | 无 |
| **M02** | 门禁 | 现在能不能做的裁决 | `src/gate.rs`、`src/guard.rs` | **IF-002** | `M01` |
| **M03** | 读模型 | 状态由账本折叠而来 | `src/readmodel.rs` | **IF-009** | `M02` |
| **M04** | 运行时 | 运行时的组装入口 | `src/lib.rs`、`src/main.rs` | **IF-008** | `M02`、`M03` |

## §3 模块编号规则

沙盒不展开。
"""

SANDBOX_SRC = {
    # ⚠️ `lib.rs` 必须写成**根级裸名 use**（`use gate::{…};`）——这是 `src/lib.rs:23-26` 的
    # 真实形态，也是抽取器最容易漏的一种；沙盒里不写它，就等于没在验这条。
    "src/lib.rs": ("pub mod gate;\npub mod guard;\npub mod ontology;\npub mod readmodel;\n"
                   "use gate::{Decision, Policy};\nuse readmodel::State;\n"),
    "src/main.rs": "fn main() {}\n",
    "src/ontology.rs": "use std::collections::BTreeMap;\n",
    "src/guard.rs": "use std::path::Path;\n",
    "src/gate.rs": "use crate::guard;\nuse crate::ontology;\n",
    "src/readmodel.rs": "use crate::gate::Decision;\n",
}

SANDBOX_TESTS = {
    # 函数体里**必须真的提到**被指认的模块（`module_tokens` 按标识符指认；
    # `tests/cli.rs` 那条用 `Command` 走"CLI 端到端 ⇒ 归程序入口"这条规则）。
    "tests/contract.rs": (
        "use world_core::gate::Policy;\n"
        "use world_core::ontology::Ontology;\n"
        "use world_core::readmodel::State;\n"
        "\n"
        "#[test]\n"
        "fn c01_gate_refuses() {\n"
        "    let _p: Option<Policy> = None;\n"
        "    let _ = world_core::gate::Decision::Allow;\n"
        "}\n"
        "\n"
        "#[test]\n"
        "fn c02_ontology_loads() {\n"
        "    let _o: Option<Ontology> = None;\n"
        "    let _ = world_core::ontology::FAMILIES;\n"
        "}\n"
        "\n"
        "#[cfg(unix)]\n"
        "#[test]\n"
        "fn c03_readmodel_folds() {\n"
        "    let _s: Option<State> = None;\n"
        "    let _ = world_core::readmodel::fold;\n"
        "}\n"
        "\n"
        "fn fixture_helper() {}\n"
    ),
    "tests/cli.rs": (
        "use std::process::Command;\n"
        "\n"
        "#[test]\n"
        "fn cli01_entry_smoke() {\n"
        "    let _c = Command::new(env!(\"CARGO_BIN_EXE_world-core\"));\n"
        "}\n"
    ),
}

SANDBOX_BOOK = """# `WC-IC-%(mid)s-v0.1` · 模块接口契约（沙盒）

## §1 本册范围
- 模块号：**`%(mid)s`**
"""


def _build_sandbox(root, wc_name="world-core"):
    """搭一个完好沙盒（正控必须四条全绿）。返回 `(仓库根, world-core 根)`。"""
    wc = os.path.join(root, wc_name)
    for d in (os.path.join(wc, "docs", "S2-设计"), os.path.join(wc, SRC_REL),
              os.path.join(wc, TESTS_REL), os.path.join(root, SPECS_REL, "cap-a")):
        os.makedirs(d, exist_ok=True)
    _write(os.path.join(wc, MODREG_REL), SANDBOX_MODREG)
    for r, t in SANDBOX_SRC.items():
        _write(os.path.join(wc, r), t)
    for r, t in SANDBOX_TESTS.items():
        _write(os.path.join(wc, r), t)
    for mid in ("M01", "M02", "M03", "M04"):
        _write(os.path.join(wc, "docs", "S2-设计", "WC-IC-%s-v0.1.md" % mid),
               SANDBOX_BOOK % {"mid": mid})
    _write(
        os.path.join(root, SPECS_REL, "cap-a", "spec.md"),
        "# cap-a Specification\n\n## Purpose\n沙盒用最小规格。\n\n## Requirements\n\n"
        "### Requirement: 门禁拒绝未声明能力\n\n沙盒正文。\n\n#### Scenario: 沙盒场景\n\n"
        "- **WHEN** 跑沙盒\n- **THEN** 通过\n"
        "- **证据**：`tests/contract.rs::c03_readmodel_folds`\n",
    )
    return root, wc


def _write(p, text):
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with io.open(p, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)


def _reg_edit(path, old, new, tag, failures):
    """按字面替换改登记表；**改不动就报错**（避免"反例其实没造出来"的假自证）。"""
    text = read_text(path)
    if old not in text:
        failures.append("%s —— 自证材料与登记表脱节：找不到 %r" % (tag, old))
        return False
    _write(path, text.replace(old, new, 1))
    return True


def self_test():
    """正控（全绿）＋ 每条判据各造反例（必红）＋ 恢复后回绿。"""
    print("== module_graph.py --self-test ==")
    failures = []
    with tempfile.TemporaryDirectory(prefix="modgraph-") as tmp:
        root, wc = _build_sandbox(tmp)
        reg = os.path.join(wc, MODREG_REL)

        # ── 正控：完好沙盒必须四条全绿 ──────────────────────────────────
        base = check(wc, base=root)
        bad = [r["judgment"] for r in base["judgments"] if not r["ok"]]
        print("  正控（完好沙盒 %d 条判据应全绿）：%s"
              % (len(base["judgments"]), "OK" if not bad else "*失败 " + str(bad)))
        if bad:
            failures.append("正控失败：%s" % bad)
            for r in base["judgments"]:
                for o in r["offenders"]:
                    print("       · %s" % o)

        # ── 正控附条：**测试面不得漏进生产面**（否则 A-4 的 deps==import 会被测试 use 污染）──
        # 沙盒：`src/**` 里除 `gate.rs`（`use crate::ontology;`）外**没有**任何文件 import M01；
        # 而 `tests/cli.rs` 里有 `use world_core::ontology::…`（M01）＋ `use world_core::{event, World};`。
        # 故 M04 的**生产** import 面必须**不含 M01**（含了就是把测试面算进来了），
        # 且它带裸 `use world_core::{…}`（非子模块名）也必须**不产边、不报错**。
        m04 = [m for m in base["modules"] if m["id"] == "M04"]
        leak = [m["id"] for m in m04 if "M01" in m["deps_import"]]
        print("  正控附条（`tests/*.rs` 与 `#[cfg(test)]` 的 use 不得算进生产面）：%s"
              % ("OK" if not leak else "*失败 生产面含 M01：%s" % leak))
        if leak:
            failures.append("测试面漏进生产面：%s" % leak)

        def run():
            return check(wc, base=root)

        def is_red(report, key):
            for r in report["judgments"]:
                if r["key"] == key:
                    return (not r["ok"]), r["offenders"]
            return False, []

        # ── 反例①：A-1 把 M03 的 intent 改成占位符 ──────────────────────
        if _reg_edit(reg, "| 状态由账本折叠而来 |", "| 待补 |", "反例①", failures):
            red, off = is_red(run(), "a1_intent")
            hit = any("M03" in x and "不是一句" in x for x in off)
            print("  反例①（M03 的 intent 改占位符 => 判据① 应红）：%s"
                  % ("已红 OK" if (red and hit) else "*没红"))
            if not (red and hit):
                failures.append("反例①未变红或未指名 M03")

        _write(reg, SANDBOX_MODREG)

        # ── 反例②：A-1 造一个**并列两事**的 intent ─────────────────────
        if _reg_edit(reg, "| 词表身份的唯一出处 |", "| 词表身份的出处与校验 |", "反例②", failures):
            red, off = is_red(run(), "a1_intent")
            hit = any("M01" in x and "并列两事" in x for x in off)
            print("  反例②（M01 的 intent 里出现「与」= 并列两事 => 判据① 应红）：%s"
                  % ("已红 OK" if (red and hit) else "*没红"))
            if not (red and hit):
                failures.append("反例②未变红或未指出并列两事")

        _write(reg, SANDBOX_MODREG)

        # ── 反例③：A-3 违反 A-2 的「实现」件——登记表列一个不存在的源码路径 ──
        if _reg_edit(reg, "`src/readmodel.rs`", "`src/nowhere.rs`", "反例③", failures):
            red, off = is_red(run(), "a2_four_in_one")
            hit = any("M03" in x and "实现缺" in x for x in off)
            print("  反例③（M03 的实现路径改成不存在的文件 => 判据③ 应红）：%s"
                  % ("已红 OK" if (red and hit) else "*没红"))
            if not (red and hit):
                failures.append("反例③未变红或未报「实现缺」")

        _write(reg, SANDBOX_MODREG)

        # ── 反例④：A-2 的「契约」件——删掉 M03 分册，并让指向它的证据行**失效** ──
        # 沙盒里 M03 的契约落点原本**两处都在**（分册 ＋ 规格证据行）：
        # 删分册而证据行仍有效 ⇒ 不该红（判据不能比事实更严）；
        # 两处都掉 ⇒ 必须红。
        book = os.path.join(wc, "docs", "S2-设计", "WC-IC-M03-v0.1.md")
        keepb = read_text(book)
        os.remove(book)
        rep4a = run()
        not_red_keeps_citation = not is_red(rep4a, "a2_four_in_one")[0]
        spec = os.path.join(root, SPECS_REL, "cap-a", "spec.md")
        keeps = read_text(spec)
        _write(spec, keeps.replace("`tests/contract.rs::c03_readmodel_folds`",
                                   "`tests/contract.rs::c99_not_a_real_test`", 1))
        red, off = is_red(run(), "a2_four_in_one")
        hit = any("M03" in x and "契约缺" in x for x in off)
        print("  反例④（删 M03 分册且证据行失锚 ⇒ 两处契约落点都没了 => 判据③ 应红；"
              "仅删分册时不应红=%s）：%s"
              % (not_red_keeps_citation, "已红 OK" if (red and hit) else "*没红"))
        if not (red and hit):
            failures.append("反例④未变红或未报「契约缺」")
        if not not_red_keeps_citation:
            failures.append("反例④附条：仅删分册就红了（判据比事实更严）")
        _write(book, keepb)
        _write(spec, keeps)

        # ── 反例④b：A-2 一整行"只有身份、没有落点"（登记表新增 M05，实现/测试/契约都缺）──
        # 这一条正对本项目的今天：`WC-MODREG-001` 登记了 `M10` 而 `src/carrier/` 曾不存在。
        _write(reg, SANDBOX_MODREG.replace(
            "\n## §3 模块编号规则",
            "| **M05** | 通道 | 一个套接字一个身份 | `src/channel.rs` | **IF-006** | 无 |\n"
            "\n## §3 模块编号规则", 1))
        red, off = is_red(run(), "a2_four_in_one")
        hit_impl = any("M05" in x and "实现缺" in x for x in off)
        hit_test = any("M05" in x and "测试缺" in x for x in off)
        hit_cont = any("M05" in x and "契约缺" in x for x in off)
        print("  反例④b（登记表新增 M05 而三件都缺 => 判据③ 应红）：%s（实现缺=%s 测试缺=%s 契约缺=%s）"
              % ("已红 OK" if (red and hit_impl and hit_test and hit_cont) else "*没红",
                 hit_impl, hit_test, hit_cont))
        if not (red and hit_impl and hit_test and hit_cont):
            failures.append("反例④b未三件齐报（实现/测试/契约）")
        _write(reg, SANDBOX_MODREG)

        # ── 反例⑤：A-4「deps == import」——加一条代码里没有的声明边 ───────
        if _reg_edit(reg, "| `src/readmodel.rs` | **IF-009** | `M02` |",
                     "| `src/readmodel.rs` | **IF-009** | `M01`、`M02` |", "反例⑤", failures):
            red, off = is_red(run(), "a4_dag_deps")
            hit = any("M03" in x and "声明了但代码里没有" in x and "M01" in x for x in off)
            print("  反例⑤（把 M01 写进 M03 的依赖列，代码里没有 => 判据② 应红）：%s"
                  % ("已红 OK" if (red and hit) else "*没红"))
            if not (red and hit):
                failures.append("反例⑤未变红或未报「声明了但代码里没有」")

        _write(reg, SANDBOX_MODREG)

        # ── 反例⑤b：A-4 另一侧——代码里新加一条没声明的 import ─────────────
        gate = os.path.join(wc, "src", "gate.rs")
        keepg = read_text(gate)
        _write(gate, "use crate::guard;\nuse crate::readmodel::State;\n")
        red, off = is_red(run(), "a4_dag_deps")
        hit = any("M02" in x and "代码里有但没声明" in x and "M03" in x for x in off)
        print("  反例⑤b（gate.rs 新 import readmodel，声明未改 => 判据② 应红）：%s"
              % ("已红 OK" if (red and hit) else "*没红"))
        if not (red and hit):
            failures.append("反例⑤b未变红或未报「代码里有但没声明」")
        _write(gate, keepg)

        # ── 反例⑤c：生产面/测试面**不可混算**——在 `#[cfg(test)] mod unit` 里加 import ──
        # 期望：A-4（deps==import）**不因测试内 use 而红**，但该边必须出现在报告的测试面里
        #（否则"只抽生产路径"就成了"把边悄悄丢掉"）。
        _write(gate, keepg + "\n#[cfg(test)]\nmod unit {\n    use super::*;\n"
                            "    use crate::readmodel::State;\n}\n")
        rep5c = run()
        red5c, off5c = is_red(rep5c, "a4_dag_deps")
        m02 = [m for m in rep5c["modules"] if m["id"] == "M02"][0]
        shown = "M03" in m02["deps_import_tests_only"]
        print("  反例⑤c（测试内 use 不进生产面 => 判据② 不应红；但须在测试面里露出来）：%s"
              % ("OK" if (not red5c and shown) else "*失败 判据②红=%s 测试面含M03=%s" % (red5c, shown)))
        if red5c or not shown:
            failures.append("反例⑤c：测试内 use 被误算进生产面，或该边未在测试面露出来")
        _write(gate, keepg)

        # ── 反例⑥：A-4 的「无环」——造一条真环（ontology 反向 import readmodel）────
        # 沙盒原本已有 M02→M01（`gate.rs: use crate::ontology;`）、M03→M02（`readmodel.rs: use crate::gate…`）；
        # 再加 `ontology.rs: use crate::readmodel;` ⇒ M01→M03→M02→M01 成环。
        onto = os.path.join(wc, "src", "ontology.rs")
        keepo = read_text(onto)
        _write(onto, "use crate::readmodel::State;\n")
        red, off = is_red(run(), "a4_dag_deps")
        hit = any("有环" in x and "M01" in x and "M02" in x and "M03" in x for x in off)
        print("  反例⑥（ontology 反向 import readmodel ⇒ M01→M03→M02→M01 成环 => 判据② 应红）：%s"
              % ("已红 OK" if (red and hit) else "*没红"))
        if not (red and hit):
            failures.append("反例⑥未变红或未报「有环」")
        _write(onto, keepo)

        # ── 反例⑥b：无环的另一面——**根级裸名**（`use <mod>::…;`）也要被抽成边并被抓 ──
        _write(onto, "use readmodel::State;\n")            # 裸名形态（`src/lib.rs:23-26` 的同款）
        _write(os.path.join(wc, "src", "readmodel.rs"), "use gate::Decision;\n")
        red, off = is_red(run(), "a4_dag_deps")
        hit = any("有环" in x and "M01" in x and "M02" in x and "M03" in x for x in off)
        print("  反例⑥b（裸名 `use readmodel::…`＋`use gate::…` ⇒ M01→M03→M02→M01 成环 => 判据② 应红）：%s"
              % ("已红 OK" if (red and hit) else "*没红"))
        if not (red and hit):
            failures.append("反例⑥b未变红或未报跨模块环（裸名形态）")
        _write(onto, keepo)
        _write(os.path.join(wc, "src", "readmodel.rs"), SANDBOX_SRC["src/readmodel.rs"])

        # ── 反例⑦：A-2 的「测试」件——删掉唯一指到 M03 的用例 ─────────────
        # 这里同时验两件事（都是"少了那个用例"的直接后果）：
        #   ① M03 的**测试缺**（函数体里再没有 `readmodel`）；
        #   ② 规格证据行 `tests/contract.rs::c03_readmodel_folds` **失锚** ⇒ 那条证据不再算契约落点。
        ct = os.path.join(wc, TESTS_REL, "contract.rs")
        keepc = read_text(ct)
        _write(ct, keepc.replace("fn c03_readmodel_folds", "fn c03_unrelated_case")
                         .replace("world_core::readmodel::fold", "world_core::ontology::FAMILIES"))
        red, off = is_red(run(), "a2_four_in_one")
        hit_test = any("M03" in x and "测试缺" in x for x in off)
        print("  反例⑦（删掉唯一指到 M03 的用例 ⇒ 测试缺，且证据行失锚 => 判据③ 应红）：%s"
              % ("已红 OK" if (red and hit_test) else "*没红"))
        if not (red and hit_test):
            failures.append("反例⑦未变红或未报「测试缺」")
        _write(ct, keepc)

        # ── 反例⑧：A-2 的覆盖面——新增一个没人认领的 .rs ─────────────────
        orphan = os.path.join(wc, "src", "orphan.rs")
        _write(orphan, "pub fn x() {}\n")
        red, off = is_red(run(), "a2_four_in_one")
        hit = any("orphan.rs" in x and "没有被任何模块号认领" in x for x in off)
        print("  反例⑧（src/orphan.rs 无模块号认领 => 判据③ 应红）：%s"
              % ("已红 OK" if (red and hit) else "*没红"))
        if not (red and hit):
            failures.append("反例⑧未变红或未报未认领文件")
        os.remove(orphan)

        # ── 第二正控：全部恢复后必须回到全绿 ─────────────────────────────
        back = run()
        bad2 = [r["judgment"] for r in back["judgments"] if not r["ok"]]
        print("  恢复后复跑（应回到全绿）：%s" % ("OK" if not bad2 else "*失败 " + str(bad2)))
        if bad2:
            failures.append("恢复后未回绿：%s" % bad2)
            for r in back["judgments"]:
                for o in r["offenders"]:
                    print("       · %s" % o)

    if failures:
        print("  => 自证不通过：%s" % "；".join(failures))
        print("  => 按本项目口径：**这条守卫是装饰，拒绝合入**。")
        return 1
    print("  => 自证通过：**三条判据**逐条在反例下变红、在正控下全绿"
          "（正控 1 ＋ 正控附条 1 ＋ 反例 10 ＋ 恢复后复跑 1）。")
    return 0


# ────────────────────────── 主程序 ──────────────────────────
def main(argv=None):
    ap = argparse.ArgumentParser(
        description="机核层守卫（WC-ATOM-001 §四 机核清单：单意图 / 四件同夹 / deps==import 且无环）")
    ap.add_argument("--repo", default=None,
                    help="world-core 目录或仓库根；默认从本脚本位置向上找含 %s 的目录" % MODREG_REL)
    ap.add_argument("--repo-base", default=None,
                    help="仓库根（含 openspec/specs 的那一级）；默认取 world-core 的上一级")
    ap.add_argument("--json", action="store_true", help="以 JSON 输出")
    ap.add_argument("--intent-column", default="职责",
                    help="A-1 取 intent 的列名（默认 `职责`——登记表暂无 intent 列，口径见文件头）")
    ap.add_argument("--self-test", action="store_true",
                    help="每条判据各造一个反例，验证它们真的会红")
    args = ap.parse_args(argv)

    if args.self_test:
        return self_test()

    wc = args.repo or find_worldcore(os.path.dirname(os.path.abspath(__file__)))
    if not wc:
        print("* 找不到 world-core 根（向上找不到含 %s 的目录）；用 --repo 指定。" % MODREG_REL)
        return 2
    wc = os.path.abspath(wc)
    if not os.path.isfile(os.path.join(wc, MODREG_REL)) and \
            os.path.isfile(os.path.join(wc, "world-core", MODREG_REL)):
        wc = os.path.join(wc, "world-core")            # 传进来的是仓库根

    rep = check(wc, args.intent_column, args.repo_base)
    failed = [r for r in rep["judgments"] if not r["ok"]]

    if args.json:
        out = dict(rep)
        out["passed"] = len(rep["judgments"]) - len(failed)
        out["failed"] = len(failed)
        print(json.dumps(out, ensure_ascii=False, indent=1))
    else:
        print("== module_graph.py —— 机核层守卫（WC-ATOM-001 §四 机核清单）==")
        print("   world-core：%s" % wc)
        print("   登记表    ：%s" % rep["registry"])
        print("   A-1 取数列：%s（登记表暂无 intent 列，口径见本脚本文件头）" % rep["intent_column"])
        print("   模块      ：%d 个（%s）；其中源码未落成：%s"
              % (len(rep["modules"]),
                 ",".join(m["id"] for m in rep["modules"]),
                 ",".join(rep["planned_modules"]) or "无"))
        print("   真实 import 边（生产面）：")
        for mid, deps in rep["import_edges_production"].items():
            print("     %s → %s" % (mid, ",".join(deps) or "—"))
        print("   真实 import 边（仅测试面，不计入 deps==import）：")
        for mid, deps in rep["import_edges_test_only"].items():
            if deps:
                print("     %s → %s" % (mid, ",".join(deps)))
        print()
        for r in rep["judgments"]:
            print("  %s %s" % ("[OK]" if r["ok"] else "[FAIL]", r["judgment"]))
            for o in r["offenders"][:40]:
                print("       · %s" % o)
            if len(r["offenders"]) > 40:
                print("       · ……（还有 %d 条同类）" % (len(r["offenders"]) - 40))
        print("  —— 通过 %d / 失败 %d ——" % (len(rep["judgments"]) - len(failed), len(failed)))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
