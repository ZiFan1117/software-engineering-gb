"""门禁工具：改动范围检查（scope check）。

用途：在一次变更中，检查被修改的文件是否**超出了变更声明所覆盖的范围**。
这是"受控变更"的机器化闸门——防止静默改框架、防止顺手重构、防止越界改动。

依据：docs/04-配置与版本/配置管理与版本化.md（变更控制八步）、
      docs/01-流程与阶段/框架与模块共演化.md（R5 框架变更评审）。

用法：
    # 用 CHANGELOG / 变更声明文件限定范围
    python tools/scope_check.py --base main --scope .scope-declaration.json

    # 直接给允许的路径前缀
    python tools/scope_check.py --base HEAD~1 --allow skeleton/modules.py --allow tests/

    # 演示模式（无需 git 变更）
    python tools/scope_check.py --demo

退出码：0 = 范围合规；1 = 越界（门禁不通过）；2 = 环境不可用（不阻断，仅告警）。
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field
from typing import List, Sequence

# 无论什么变更都允许修改的路径（元数据类，不构成范围越界）
ALWAYS_ALLOWED: Sequence[str] = (
    "CHANGELOG.md",
    ".gitignore",
)

# 敏感路径：改动这些一律需要显式声明（框架契约与门禁自身）
SENSITIVE_PATHS: Sequence[str] = (
    "skeleton/module_system.py",
    "skeleton/modules.py",
    "tools/",
    ".github/workflows/",
    "docs/02-评审与门禁/",
)


@dataclass
class ScopeResult:
    changed: List[str] = field(default_factory=list)
    allowed: List[str] = field(default_factory=list)
    in_scope: List[str] = field(default_factory=list)
    out_of_scope: List[str] = field(default_factory=list)
    sensitive_touched: List[str] = field(default_factory=list)


def git_changed_files(base: str, repo_root: str) -> List[str]:
    """返回相对 base 的变更文件列表（含未跟踪文件）。"""
    def run(args: Sequence[str]) -> str:
        proc = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if proc.returncode != 0:
            raise RuntimeError(proc.stderr.strip() or f"git {' '.join(args)} 执行失败")
        return proc.stdout

    files: List[str] = []
    try:
        files.extend(run(["diff", "--name-only", f"{base}...HEAD"]).splitlines())
        files.extend(run(["diff", "--name-only"]).splitlines())
        files.extend(run(["diff", "--name-only", "--cached"]).splitlines())
    except RuntimeError:
        # 可能是首次提交、base 不存在等情况
        files.extend(run(["status", "--porcelain"]).splitlines())
        files = [line[3:].strip() for line in files]

    files.extend(run(["ls-files", "--others", "--exclude-standard"]).splitlines())

    normalized = []
    for path in files:
        path = path.strip().replace("\\", "/")
        if not path or path.startswith("..") :
            continue
        normalized.append(path)
    return sorted(set(normalized))


def evaluate(changed: Sequence[str], allowed: Sequence[str]) -> ScopeResult:
    result = ScopeResult(changed=list(changed), allowed=list(allowed))

    def matches(path: str, pattern: str) -> bool:
        pattern = pattern.replace("\\", "/").rstrip("/")
        if not pattern:
            return False
        return path == pattern or path.startswith(pattern + "/")

    for path in changed:
        if any(matches(path, a) for a in ALWAYS_ALLOWED):
            result.in_scope.append(path)
            continue
        if any(matches(path, a) for a in allowed):
            result.in_scope.append(path)
        else:
            result.out_of_scope.append(path)

        if any(matches(path, s) for s in SENSITIVE_PATHS):
            result.sensitive_touched.append(path)

    return result


def load_scope_declaration(path: str) -> List[str]:
    """读取变更范围声明文件。

    支持两种格式：
      {"allowed": ["skeleton/modules.py", "tests/"]}
      或 ["skeleton/modules.py", "tests/"]
    """
    with open(path, "r", encoding="utf-8") as fh:
        data = json.load(fh)
    if isinstance(data, dict):
        allowed = data.get("allowed", [])
    elif isinstance(data, list):
        allowed = data
    else:
        raise ValueError("范围声明文件必须是数组或含 allowed 字段的对象")
    if not isinstance(allowed, list):
        raise ValueError("allowed 必须是字符串数组")
    return [str(a) for a in allowed]


def print_report(result: ScopeResult, base: str) -> None:
    print("=" * 74)
    print("门禁校验：改动范围（scope check）")
    print("=" * 74)
    print(f"基线: {base}")
    print(f"变更文件: {len(result.changed)}    声明范围: {len(result.allowed)} 条")
    print()

    if result.allowed:
        print("声明的允许范围：")
        for item in result.allowed:
            print(f"  - {item}")
        print()

    print(f"范围内文件 {len(result.in_scope)} 个：")
    for path in result.in_scope:
        print(f"  [ OK ] {path}")
    print()

    if result.out_of_scope:
        print(f"越界文件 {len(result.out_of_scope)} 个：")
        for path in result.out_of_scope:
            print(f"  [越界] {path}")
        print()

    if result.sensitive_touched:
        print("触及敏感路径（框架契约 / 门禁自身）：")
        for path in result.sensitive_touched:
            print(f"  [!] {path}")
        print("  说明：改动这些文件等于修改框架契约或门禁规则，")
        print("        必须走 R5 框架变更评审并更新变更申请与影响分析。")
        print()


def demo() -> int:
    changed = [
        "skeleton/modules.py",
        "skeleton/order_service.py",
        "tests/test_order_service.py",
        "docs/03-需求类/软件需求规格说明.md",
        "README.md",
    ]
    allowed = ["skeleton/", "tests/"]
    result = evaluate(changed, allowed)
    print_report(result, base="--demo--")
    return 1 if result.out_of_scope else 0


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="改动范围门禁校验")
    parser.add_argument("--base", default="main", help="比较基线（分支/标签/提交）")
    parser.add_argument("--scope", default="", help="变更范围声明 JSON 文件")
    parser.add_argument(
        "--allow", action="append", default=[], help="允许的路径前缀，可重复"
    )
    parser.add_argument("--repo-root", default=".", help="仓库根目录")
    parser.add_argument("--demo", action="store_true", help="演示模式，不调用 git")
    args = parser.parse_args(argv)

    if args.demo:
        return demo()

    repo_root = os.path.abspath(args.repo_root)

    allowed: List[str] = list(args.allow)
    if args.scope:
        scope_path = os.path.join(repo_root, args.scope)
        if not os.path.isfile(scope_path):
            print(f"[WARN] 范围声明文件不存在：{scope_path}", file=sys.stderr)
        else:
            allowed.extend(load_scope_declaration(scope_path))

    if not allowed:
        print(
            "[WARN] 未声明任何允许范围（--allow / --scope），跳过范围校验。",
            file=sys.stderr,
        )
        print(
            "       建议在 PR 中填写变更范围声明，否则无法判断改动是否越界。",
            file=sys.stderr,
        )
        return 0

    try:
        changed = git_changed_files(args.base, repo_root)
    except RuntimeError as exc:
        print(f"[WARN] 无法执行 git 变更检测：{exc}", file=sys.stderr)
        print("       环境不可用，本项门禁跳过（不阻断流程）。", file=sys.stderr)
        return 2

    if not changed:
        print("没有检测到变更文件，范围检查通过。")
        return 0

    result = evaluate(changed, allowed)
    print_report(result, base=args.base)

    if result.out_of_scope:
        print("门禁结论：不通过 —— 存在越界改动")
        print("  处置：补充变更范围声明，或撤回越界改动，或走变更评审批准扩展范围。")
        return 1
    print("门禁结论：通过 —— 全部改动在声明范围内")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
