"""门禁工具：需求追溯矩阵（RTM）完整性校验。

依据：GB/T 8567-2006（文档编制规范）、GB/T 38634.3-2020（测试文档）。

这个工具把「需求覆盖率 100%」这类门禁条件从人工核对变成 CI 自动判定。

双向追溯校验项：
  正向  需求 → 设计模块 → 测试用例
  反向  测试用例 → 需求（用例引用的需求必须存在）
  交叉  引用的模块号必须在模块登记表中存在

用法：
    python tools/trace_matrix.py                                   # 使用示例数据自检
    python tools/trace_matrix.py --matrix <RTM.csv> --srs <SRS.md>
    python tools/trace_matrix.py --matrix <RTM.csv> --srs <SRS.md> --strict

退出码：0 = 通过；1 = 未通过（门禁不通过）。
"""

from __future__ import annotations

import argparse
import csv
import os
import re
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Sequence, Set

REQUIRED_COLUMNS: Sequence[str] = (
    "需求编号",
    "需求名称",
    "优先级",
    "需求基线版本",
    "设计模块号",
    "设计文档章节",
    "接口编号",
    "实现代码位置",
    "单元测试用例",
    "集成测试用例",
    "系统测试用例",
    "验收测试用例",
    "状态",
    "备注",
)

CASE_COLUMNS: Sequence[str] = (
    "单元测试用例",
    "集成测试用例",
    "系统测试用例",
    "验收测试用例",
)

MODULE_ID_PATTERN = re.compile(r"\bM\d{2}\b")
REQ_ID_PATTERN = re.compile(r"\bREQ-[FN]-\d{3}\b")

# 允许的优先级取值（与模板一致）
VALID_PRIORITIES = {"P0", "P1", "P2"}


def split_multi(value: str) -> List[str]:
    """拆分多值单元格：支持逗号、分号、顿号分隔。"""
    if not value:
        return []
    parts = re.split(r"[,，;；、]+", value.strip())
    return [p.strip() for p in parts if p.strip()]


def split_cases(value: str) -> List[str]:
    """从测试用例单元格中提取用例编号，并过滤掉代码路径等非用例内容。

    同一个单元格里允许混写代码位置（如 `skeleton/modules.py::OrderService.submit_order`）
    与用例编号（如 `TC-001`），本函数只取后者。
    """
    cases: List[str] = []
    for part in split_multi(value):
        # 含路径分隔符或双冒号限定的，视为代码位置而非用例编号
        if "/" in part or "\\" in part or "::" in part:
            continue
        cases.append(part)
    return cases


@dataclass
class Issue:
    level: str  # "ERROR" | "WARN"
    requirement: str
    message: str


@dataclass
class CheckResult:
    requirements: int = 0
    issues: List[Issue] = field(default_factory=list)

    @property
    def errors(self) -> List[Issue]:
        return [i for i in self.issues if i.level == "ERROR"]

    @property
    def warnings(self) -> List[Issue]:
        return [i for i in self.issues if i.level == "WARN"]


def load_known_modules(repo_root: str) -> Set[str]:
    """从模块登记表文档中提取已登记的模块号。"""
    candidates = [
        os.path.join(repo_root, "templates", "03-设计类", "02-模块清单与模块号登记表.md"),
        os.path.join(repo_root, "skeleton", "modules.py"),
    ]
    known: Set[str] = set()
    for path in candidates:
        if not os.path.isfile(path):
            continue
        with open(path, "r", encoding="utf-8") as fh:
            known.update(MODULE_ID_PATTERN.findall(fh.read()))
    return known


def load_srs_requirement_ids(srs_path: str) -> Set[str]:
    """从 SRS 文档中提取需求编号（用于校验 RTM 与 SRS 一致）。"""
    if not srs_path or not os.path.isfile(srs_path):
        return set()
    with open(srs_path, "r", encoding="utf-8") as fh:
        return set(REQ_ID_PATTERN.findall(fh.read()))


def check_matrix(
    matrix_path: str,
    srs_path: str = "",
    known_modules: Set[str] | None = None,
    strict: bool = False,
) -> CheckResult:
    result = CheckResult()
    known_modules = known_modules or set()

    if not os.path.isfile(matrix_path):
        result.issues.append(Issue("ERROR", "-", f"追溯矩阵文件不存在：{matrix_path}"))
        return result

    with open(matrix_path, "r", encoding="utf-8-sig", newline="") as fh:
        reader = csv.DictReader(fh)
        header = reader.fieldnames or []
        missing = [c for c in REQUIRED_COLUMNS if c not in header]
        if missing:
            result.issues.append(
                Issue("ERROR", "-", "追溯矩阵缺少必需列：" + "、".join(missing))
            )
            return result
        rows = [r for r in reader if (r.get("需求编号") or "").strip()]

    result.requirements = len(rows)
    if not rows:
        result.issues.append(Issue("ERROR", "-", "追溯矩阵为空，没有任何需求记录"))
        return result

    seen_ids: Dict[str, int] = {}
    srs_ids = load_srs_requirement_ids(srs_path)

    for line_no, row in enumerate(rows, start=2):
        req_id = (row.get("需求编号") or "").strip()
        seen_ids[req_id] = seen_ids.get(req_id, 0) + 1

        def add(level: str, message: str) -> None:
            result.issues.append(Issue(level, req_id, f"第 {line_no} 行：{message}"))

        # --- 编号格式 ---
        if not REQ_ID_PATTERN.fullmatch(req_id):
            add("ERROR", f"需求编号 {req_id!r} 不符合规则 REQ-F-xxx / REQ-N-xxx")

        # --- 名称与优先级 ---
        if not (row.get("需求名称") or "").strip():
            add("ERROR", "需求名称为空")
        priority = (row.get("优先级") or "").strip()
        if priority not in VALID_PRIORITIES:
            add("ERROR", f"优先级 {priority!r} 非法，必须为 P0/P1/P2")

        # --- 基线版本 ---
        if not (row.get("需求基线版本") or "").strip():
            add("ERROR", "需求基线版本为空——未纳入基线的需求不可追溯")

        # --- 正向：需求 → 设计模块 ---
        modules = split_multi(row.get("设计模块号") or "")
        if not modules:
            add("ERROR", "未映射到任何设计模块（正向追溯断裂：需求 → 设计）")
        for module_id in modules:
            if not MODULE_ID_PATTERN.fullmatch(module_id):
                add("ERROR", f"模块号 {module_id!r} 格式非法")
            elif known_modules and module_id not in known_modules:
                add("ERROR", f"模块号 {module_id} 在模块登记表中不存在")

        if not (row.get("设计文档章节") or "").strip():
            add("WARN", "未标注设计文档章节")

        if not (row.get("实现代码位置") or "").strip():
            add("ERROR", "未标注实现代码位置（正向追溯断裂：设计 → 代码）")

        # --- 正向：需求 → 测试用例（门禁核心）---
        case_total = 0
        for column in CASE_COLUMNS:
            cases = split_cases(row.get(column) or "")
            case_total += len(cases)
            for case_id in cases:
                if not re.fullmatch(r"TC-\d{3}", case_id):
                    add("ERROR", f"{column} 中的用例编号 {case_id!r} 格式非法（应为 TC-xxx）")
        if case_total == 0:
            add(
                "ERROR",
                "没有任何关联测试用例——写不出测试用例的需求 = 不合格需求"
                "（见 docs/03-测试与回归/测试体系与缺陷管理.md）",
            )
        if not split_multi(row.get("系统测试用例") or "") and not split_multi(
            row.get("验收测试用例") or ""
        ):
            add("WARN", "缺少系统级/验收级测试用例，仅有单元或集成测试覆盖")

        # --- 状态 ---
        status = (row.get("状态") or "").strip()
        if not status:
            add("ERROR", "状态为空")
        if strict and status not in {"已实现", "已测试", "已验收"}:
            add("ERROR", f"严格模式下状态 {status!r} 未达到可交付状态")

        # --- 与 SRS 一致性 ---
        if srs_ids and req_id not in srs_ids:
            add("ERROR", f"需求 {req_id} 在 SRS 文档中不存在（RTM 与 SRS 不一致）")

    # --- 重复编号 ---
    for req_id, count in seen_ids.items():
        if count > 1:
            result.issues.append(
                Issue("ERROR", req_id, f"需求编号重复出现 {count} 次——编号必须唯一")
            )

    # --- SRS 中未被 RTM 覆盖的需求 ---
    if srs_ids:
        uncovered = sorted(srs_ids - set(seen_ids))
        for req_id in uncovered:
            result.issues.append(
                Issue("ERROR", req_id, "SRS 中已定义但 RTM 中缺失（需求未被追溯）")
            )

    return result


def print_report(result: CheckResult, matrix_path: str, srs_path: str) -> None:
    print("=" * 74)
    print("门禁校验：需求追溯矩阵（RTM）完整性")
    print("=" * 74)
    print(f"追溯矩阵: {matrix_path}")
    if srs_path:
        print(f"SRS 文档: {srs_path}")
    print(f"需求条目: {result.requirements}")
    print()

    if result.warnings:
        print(f"警告 {len(result.warnings)} 项：")
        for issue in result.warnings:
            print(f"  [WARN ] {issue.requirement}  {issue.message}")
        print()

    if result.errors:
        print(f"错误 {len(result.errors)} 项：")
        for issue in result.errors:
            print(f"  [ERROR] {issue.requirement}  {issue.message}")
        print()
        print("门禁结论：不通过 —— 不得通过需求评审 / 测试准出评审")
    else:
        print("门禁结论：通过 —— 需求双向追溯完整，覆盖率 100%")
        if result.warnings:
            print("（存在警告项，建议在评审记录中说明处置）")


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="需求追溯矩阵完整性校验（CI 门禁工具）"
    )
    parser.add_argument(
        "--matrix",
        default=os.path.join("docs", "demo", "需求追溯矩阵.csv"),
        help="追溯矩阵 CSV 路径",
    )
    parser.add_argument(
        "--srs",
        default=os.path.join("docs", "demo", "软件需求规格说明.md"),
        help="SRS 文档路径（用于一致性校验，可省略）",
    )
    parser.add_argument("--repo-root", default=".", help="仓库根目录")
    parser.add_argument(
        "--strict", action="store_true", help="严格模式：状态必须为已实现/已测试/已验收"
    )
    args = parser.parse_args(argv)

    repo_root = os.path.abspath(args.repo_root)
    matrix_path = os.path.join(repo_root, args.matrix)
    srs_path = os.path.join(repo_root, args.srs) if args.srs else ""

    result = check_matrix(
        matrix_path=matrix_path,
        srs_path=srs_path,
        known_modules=load_known_modules(repo_root),
        strict=args.strict,
    )
    print_report(result, matrix_path, srs_path)
    return 1 if result.errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
