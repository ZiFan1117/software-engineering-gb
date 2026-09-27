#!/usr/bin/env python3
# -*- coding: utf-8 -*-
r"""接口契约分册门禁：**九册齐否 · 每册必备节齐否 · 与模块登记表同源否**，缺一即红。

为什么有它：R4S 第 2 轮实测「删掉 `WC-IC-M03` 一册 ⇒ `check.sh` 与六个门禁**零反应**」——
即九册契约在库内**没有任何会红的判据**（"零门禁覆盖"）。
判据（三条，任一不满足 ⇒ rc=1）：
  ① **册数一致**：`docs/S2-设计/WC-IC-M<NN>-v0.1.md` 的模块号集合 == `WC-MODREG-001` §2 登记表的模块号集合；
  ② **必备节齐**：每册含 `§1 本册范围`、`§2 接口契约要点`、`§3 契约变更记录` 三节，
     且 §2 表内六项标签「提供者／输入／输出／异常与错误码／不变量／判据」**逐项非空**；
  ③ **生效即冻结声明**：每册含「生效即冻结」字样。

2026-09-27 加（`W-08`，**属门禁判据扩张 ⇒ 登记待 R5，见 `WC-RV-R4-001` §九**）：
  ④ **登记集不得为空**（原来 `if reg and set(bs) != reg` 在 `reg==空集` 时**静默跳过**
     —— 登记表被清空/格式被改到正则不命中，门禁反而**更绿**）；
  ⑤ **依赖列逐边比对**：九册 §1 声明的依赖边集 与 `WC-MODREG-001` §2 登记表「依赖模块」列
     的边集**逐模块逐边**相等（原来只比模块号**集合**，改依赖列 ⇒ rc 仍 0）。

2026-09-27 加（`W-07`，**补偿判据；默认关闭**，需 `IC_QUOTA_STRICT=1` 才生效
——"行数额度"废弃后，"宿主字节骤降"没有判据：删 500 行仍 rc=0）：
  ⑥ **宿主字节不得低于冻结件的 70%**（非行数上限的补偿判据）。
     默认关闭的理由：这是一条**新增门禁强度**，按本项目纪律**不得静默实施**；
     改法＋代价＋可回退见 `WC-RV-R4-001` §九，**是否把默认翻成开启待人裁（R5）**。

用法：`python3 tools/ic_books_check.py [--repo-root .]`；`--self-test` 自证（删一册必红、恢复必绿）。
"""
import io
import os
import re
import sys
import tempfile

LABELS = ["提供者", "输入", "输出", "异常与错误码", "不变量", "判据"]
SECTIONS = ["§1 本册范围", "§2 接口契约要点", "§3 契约变更记录"]

#: **冻结件字节数**（第五十二次冻结对象，`git cat-file -s`，提交 `4d5cb0b`）。
#: 每次冻结由执行员更新；它是 `W-07` 补偿判据的基准，不是展示数据。
FROZEN_BYTES = {
    "WC-IC-M01-v0.1.md": 1823,
    "WC-IC-M02-v0.1.md": 1772,
    "WC-IC-M03-v0.1.md": 1822,
    "WC-IC-M04-v0.1.md": 1821,
    "WC-IC-M05-v0.1.md": 1822,
    "WC-IC-M06-v0.1.md": 1796,
    "WC-IC-M07-v0.1.md": 1799,
    "WC-IC-M08-v0.1.md": 1801,
    "WC-IC-M09-v0.1.md": 1783,
}
#: 补偿判据的下限比例（宿主字节 / 冻结件字节）。低于它即判 ERROR（`W-07`）。
QUOTA_FLOOR = 0.70


def books(root):
    d = os.path.join(root, "docs", "S2-设计")
    out = {}
    if os.path.isdir(d):
        for f in os.listdir(d):
            m = re.fullmatch(r"WC-IC-(M\d{2})-v0\.1\.md", f)
            if m:
                out[m.group(1)] = os.path.join(d, f)
    return out


def registry_modules(root):
    d = os.path.join(root, "docs", "S2-设计")
    ids = set()
    p = os.path.join(d, "WC-MODREG-001-v0.1.md")
    if os.path.exists(p):
        t = io.open(p, encoding="utf-8").read()
        for m in re.finditer(r"(?m)^\|\s*\*{0,2}(M\d{2})\*{0,2}\s*\|", t):
            ids.add(m.group(1))
    return ids


def _deps_of_cell(cell):
    """从依赖单元格里取出模块号边集（`**无**` / `—` ⇒ 空集）。"""
    return set(re.findall(r"M\d{2}", cell))


def registry_edges(root):
    """`WC-MODREG-001` §2 登记表「依赖模块」列 → {模块号: 依赖边集}（`W-08` ④⑤）。

    只在 `## §2 模块登记表` 这一节内取表，避免误取附录 A 的旧表
    （两表口径不同，混取会让判据失去意义）。
    """
    p = os.path.join(root, "docs", "S2-设计", "WC-MODREG-001-v0.1.md")
    if not os.path.exists(p):
        return {}
    t = io.open(p, encoding="utf-8").read()
    sec = re.search(r"(?ms)^##\s*§2\s*模块登记表(.*?)(?=^##\s|\Z)", t)
    if not sec:
        return {}
    out = {}
    for line in sec.group(1).split("\n"):
        m = re.match(r"^\|\s*\*{0,2}(M\d{2})\*{0,2}\s*\|(.*)$", line)
        if not m:
            continue
        cells = m.group(2).split("|")
        if len(cells) < 5:
            continue
        # 表头为「模块号|模块名|职责|源码路径|提供接口|依赖模块」⇒ 依赖列 = 第 6 格
        # （`cells` 已去掉行首那一格，故下标 4）。
        out[m.group(1)] = _deps_of_cell(cells[4])
    return out


def book_edges(text):
    """九册 §1 的 `源码位置与依赖：…（依赖：…）` → 依赖边集（`W-08` ⑤）。"""
    m = re.search(r"源码位置与依赖[：:](.*)", text)
    if not m:
        return None
    return _deps_of_cell(m.group(1))


def check(root):
    bad = []
    bs = books(root)
    reg = registry_modules(root)
    if not bs:
        bad.append("未找到任何 `WC-IC-M<NN>-v0.1.md` 分册（契约分册缺失）")
    # ④ 登记集为空即红（`W-08`）：原来 `if reg and …` 会在 `reg` 为空时**静默跳过**
    #    整套集合判据 —— 把登记表清空或改到正则不命中，门禁反而更绿。
    if not reg:
        bad.append(
            "模块登记表**登记集为空**：`WC-MODREG-001` §2/附录 A 里一个 `| M0x |` 行都取不到"
            "（表被清空、格式被改或文件缺失）—— 空登记表不得被当作「一致」"
        )
    elif set(bs) != reg:
        miss = sorted(reg - set(bs))
        extra = sorted(set(bs) - reg)
        if miss:
            bad.append("缺少分册（模块登记表有、分册无）：%s" % ",".join(miss))
        if extra:
            bad.append("多余分册（分册有、登记表无）：%s" % ",".join(extra))

    # ⑤ 依赖列**逐边**比对（`W-08`）：原来只比模块号集合，改依赖列 ⇒ rc 仍 0。
    edges_reg = registry_edges(root)
    if not edges_reg:
        bad.append(
            "登记表「依赖模块」列**一条边都取不到**（§2 表结构被改或依赖列被删）"
            "—— 依赖判据不得因取不到而静默通过"
        )
    for mid, path in sorted(bs.items()):
        t = io.open(path, encoding="utf-8").read()
        for sec in SECTIONS:
            if sec not in t:
                bad.append("%s 缺必备节：%s" % (mid, sec))
        for lab in LABELS:
            #: 2026-09-27 **口径统一（择「判据侧归一化」）**：九册行标签写作「判据（可机械核对）」更具体，
            #: 故**不改九册**，改为**前缀归一化匹配**（行标签以 `lab` 开头即认）；缺册检测不受影响（仍按集合比较）。
            m = re.search(r"(?m)^\|\s*" + re.escape(lab) + r"[^|]*\|([^|]*)\|", t)
            if not m or not m.group(1).strip() or m.group(1).strip() in ("—", "-"):
                bad.append("%s §2 要点为空或缺：%s" % (mid, lab))
        if "生效即冻结" not in t:
            bad.append("%s 缺「生效即冻结」声明" % mid)
        if edges_reg and mid in edges_reg:
            got = book_edges(t)
            if got is None:
                bad.append("%s §1 缺「源码位置与依赖」行 —— 依赖边无法比对" % mid)
            elif got != edges_reg[mid]:
                bad.append(
                    "%s 依赖边与登记表不一致：册内={%s} 登记表={%s}"
                    % (mid, ",".join(sorted(got)) or "无", ",".join(sorted(edges_reg[mid])) or "无")
                )
        elif edges_reg:
            bad.append("%s 在登记表的「依赖模块」列里没有行 —— 依赖边无法比对" % mid)
    return len(bs), len(reg), bad


def quota_check(root):
    """⑥ 宿主字节 / 冻结件 < `QUOTA_FLOOR` ⇒ ERROR（`W-07` 补偿判据）。

    **默认不启用**：只有 `IC_QUOTA_STRICT=1` 时才返回问题。
    理由：这是新增门禁强度，按纪律不得静默实施（改法＋代价＋可回退见 R4 记录 §九）。
    """
    if os.environ.get("IC_QUOTA_STRICT") != "1":
        return None
    d = os.path.join(root, "docs", "S2-设计")
    bad = []
    for name, frozen in sorted(FROZEN_BYTES.items()):
        p = os.path.join(d, name)
        if not os.path.exists(p):
            bad.append("%s 不存在 —— 缺行补偿判据无法评估（按 ERROR 处理）" % name)
            continue
        host = os.path.getsize(p)
        if host < frozen * QUOTA_FLOOR:
            bad.append(
                "%s 宿主字节 %d < 冻结件 %d 的 %.0f%%（下限 %d）—— 内容被大幅删除"
                % (name, host, frozen, QUOTA_FLOOR * 100, int(frozen * QUOTA_FLOOR))
            )
    return bad


def main(argv):
    if "--self-test" in argv:
        src = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        with tempfile.TemporaryDirectory() as tmp:
            d = os.path.join(tmp, "docs", "S2-设计")
            os.makedirs(d)
            # ⚠️ **分册清单必须从源目录扫出来，不能写死 M01–M09**（2026-09-27 修）。
            # 原先这里写死 `range(1, 10)`：新增第十册（`M10`）之后，自证用的临时目录里
            # 只有 9 册、而登记表有 10 行 ⇒ 自证自己被判红（"缺少分册(M10)"），
            # 于是 `check.sh` 报"契约分册门禁判定器自证 失败"——
            # **那是自证材料过期，不是产品缺陷**。写死册数会让每次加模块都要改这里。
            src_dir = os.path.join(src, "docs", "S2-设计")
            book_names = sorted(
                f
                for f in os.listdir(src_dir)
                if f.startswith("WC-IC-M") and f.endswith("-v0.1.md")
            )
            for f in ["WC-MODREG-001-v0.1.md"] + book_names:
                s = os.path.join(src_dir, f)
                if os.path.exists(s):
                    io.open(os.path.join(d, f), "w", encoding="utf-8").write(
                        io.open(s, encoding="utf-8").read()
                    )
            n0, r0, b0 = check(tmp)
            victim = os.path.join(d, "WC-IC-M03-v0.1.md")
            keep = io.open(victim, encoding="utf-8").read()
            os.remove(victim)
            n1, r1, b1 = check(tmp)
            io.open(victim, "w", encoding="utf-8").write(keep)
            n2, r2, b2 = check(tmp)
            red = any("缺少分册" in x and "M03" in x for x in b1)

            # ── 2026-09-27 加（`W-08`）：④⑤ 两条新判据**各自**都要有"会红"的证据 ──
            # ④ 登记集为空即红：把 §2 表头/行全部打散 ⇒ 取不到 M0x 行
            #
            # ⚠️ **必须打散「所有」登记行，不能只打散模块号带两位数字的那种写法**
            # （2026-09-27 修）：原先替换的是 `| **M0`——它只命中 §2 的短格式行
            # （`| **M01** | 本体…`），而**附录 A 的全格式行**（`| **M01** | **本体…`）
            # 与 **`M10`**（`| **M10** |`，第三位不是 `0`）都躲过了替换。
            # 后果有两层：① 该条的"会红"证据在此之前名不副实；
            # ② 新增第十册后，登记集仍有残留行 ⇒ 这一格**反转成假绿**。
            # 现在改成按行首正则打散，凡是登记行一律命中，与册数、数字位数无关。
            import re as _re

            regp = os.path.join(d, "WC-MODREG-001-v0.1.md")
            keep_reg = io.open(regp, encoding="utf-8").read()

            def _scatter(text):
                return _re.sub(r"(?m)^\|\s*\*\*(M\d{2})\*\*", r"| x\1", text)

            scattered = _scatter(keep_reg)
            if scattered == keep_reg:
                # 一条都没打散 ⇒ 判据根本没被触发，必须报出来而不是"看起来通过"
                bad.append("自证材料问题：打散登记表时一条登记行都没命中（正则与登记表格式脱节）")
            io.open(regp, "w", encoding="utf-8").write(scattered)
            _, _, b_empty = check(tmp)
            red_empty = any("登记集为空" in x for x in b_empty)
            io.open(regp, "w", encoding="utf-8").write(keep_reg)

            # ⑤ 依赖列逐边比对：改 M06 的依赖列（M03 → M09）⇒ 必须报"依赖边与登记表不一致"
            io.open(regp, "w", encoding="utf-8").write(
                keep_reg.replace(
                    "| `M03`　**（2026-09-27 订正：与源码面一致）** |\n| **M07**",
                    "| `M09`　**（2026-09-27 订正：与源码面一致）** |\n| **M07**",
                    1,
                )
            )
            _, _, b_edge = check(tmp)
            red_edge = any("依赖边与登记表不一致" in x and "M06" in x for x in b_edge)
            io.open(regp, "w", encoding="utf-8").write(keep_reg)

            # ⑥ 补偿判据（默认关闭）：宿主字节砍半 ⇒ 只有开启时才红
            cpath = os.path.join(d, "WC-IC-M05-v0.1.md")
            keepc = io.open(cpath, encoding="utf-8").read()
            io.open(cpath, "w", encoding="utf-8").write(keepc[: len(keepc) // 2])
            os.environ.pop("IC_QUOTA_STRICT", None)
            off = quota_check(tmp)
            os.environ["IC_QUOTA_STRICT"] = "1"
            on = quota_check(tmp)
            os.environ.pop("IC_QUOTA_STRICT", None)
            red_quota = (off is None) and bool(on) and any("宿主字节" in x for x in on)
            io.open(cpath, "w", encoding="utf-8").write(keepc)

            print("[self-test] 初始：册 %d / 登记 %d / 问题 %d" % (n0, r0, len(b0)))
            print("[self-test] 删一册（M03）⇒ 问题 %d，含「缺少分册(M03)」= %s" % (len(b1), red))
            print("[self-test] 恢复 ⇒ 问题 %d（应回到 %d）" % (len(b2), len(b0)))
            print("[self-test/W-08④] 打散登记表 ⇒ 含「登记集为空」= %s" % red_empty)
            print("[self-test/W-08⑤] 改 M06 依赖列 ⇒ 含「依赖边与登记表不一致(M06)」= %s" % red_edge)
            print("[self-test/W-07⑥] 宿主字节砍半 ⇒ 默认关闭时无问题=%s；IC_QUOTA_STRICT=1 时报「宿主字节」=%s"
                  % (off is None, red_quota))
            return (
                0
                if (
                    red
                    and len(b2) == len(b0)
                    and len(b0) == 0
                    and red_empty
                    and red_edge
                    and red_quota
                )
                else 1
            )
    root = "."
    for i, a in enumerate(argv):
        if a == "--repo-root" and i + 1 < len(argv):
            root = argv[i + 1]
    n, r, bad = check(root)
    qbad = quota_check(root)
    if qbad:
        bad = bad + ["[补偿判据 IC_QUOTA_STRICT=1] " + x for x in qbad]
    if bad:
        print("[FAIL] 契约分册门禁：册 %d / 登记 %d / 问题 %d" % (n, r, len(bad)))
        for x in bad:
            print("        " + x)
        return 1
    quota_note = "补偿判据已启用（IC_QUOTA_STRICT=1）" if qbad is not None else "补偿判据未启用（IC_QUOTA_STRICT≠1，默认关闭）"
    print(
        "[ OK ] 契约分册门禁：册 %d == 登记 %d，必备节与要点齐备，逐册含「生效即冻结」，"
        "依赖列逐边一致；%s" % (n, r, quota_note)
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
