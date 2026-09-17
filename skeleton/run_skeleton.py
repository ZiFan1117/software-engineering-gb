"""骨架跑通脚本 —— 对应流程阶段 S3 / 测试级别 L0（骨架冒烟测试）。

一条命令验证框架是否真的跑得起来：
    python skeleton/run_skeleton.py

验证内容：
  1. 模块登记表可构造，模块号合规；
  2. 依赖解析无环，分层方向正确（低层不依赖高层）；
  3. 按拓扑顺序装配全部模块（骨架阶段全部为可运行的最小实现）；
  4. 冒烟用例：正常流程 + 边界 + 异常 + 空值 四类路径；
  5. 反向依赖与影响分析可查询（R5 变更评审 / S7 回归范围推导的依据）。

退出码：0 = 骨架跑通；1 = 未跑通（门禁不通过）。
"""

from __future__ import annotations

import os
import sys

# 允许 `python skeleton/run_skeleton.py` 直接执行（无需安装为包）
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from module_system import ContractError  # noqa: E402
from modules import (  # noqa: E402
    SINGLE_ORDER_LIMIT,
    IdValidator,
    OrderController,
    OrderIdFactory,
    OrderRepository,
    OrderService,
    build_registry,
)

RESULTS: list[tuple[str, bool, str]] = []


def check(name: str, passed: bool, detail: str = "") -> None:
    RESULTS.append((name, passed, detail))
    mark = "PASS" if passed else "FAIL"
    line = f"  [{mark}] {name}"
    if detail:
        line += f" — {detail}"
    print(line)


def section(title: str) -> None:
    print()
    print(f"── {title} " + "─" * max(0, 66 - len(title)))


def main() -> int:
    print("=" * 74)
    print("骨架跑通验证（S3 / L0 骨架冒烟测试）")
    print("=" * 74)

    # ---------------------------------------------------------------- 1. 契约
    section("1. 模块登记与契约校验")
    try:
        registry = build_registry()
        check("模块登记表构造成功", True, f"共 {len(registry)} 个模块")
    except ContractError as exc:
        check("模块登记表构造成功", False, str(exc))
        return report()

    try:
        registry.validate()
        check("契约校验通过（依赖完整 / 分层方向 / 无循环依赖）", True)
    except ContractError as exc:
        check("契约校验通过", False, str(exc))
        return report()

    order = registry.assembly_order()
    check("装配顺序可确定（拓扑排序）", True, " → ".join(order))

    # ---------------------------------------------------------------- 2. 装配
    section("2. 按拓扑顺序装配模块")
    repo = OrderRepository()
    ids = OrderIdFactory()
    service = OrderService(repository=repo, id_factory=ids)
    controller = OrderController(service=service)
    instances: dict[str, object] = {
        "M01": controller,
        "M20": service,
        "M40": repo,
        "M80": ids,
    }
    for mid in order:
        check(f"装配 {mid}（{registry.get(mid).name}）", mid in instances)

    # ------------------------------------------------------- 3. 冒烟用例四类路径
    section("3. 冒烟用例：正常 / 边界 / 异常 / 空值")

    # 正常路径
    resp = controller.submit(amount=199.00, date="20260214")
    check(
        "正常路径：合法金额下单成功",
        resp["success"] and IdValidator.is_valid(resp["order_id"]),
        resp["message"],
    )

    # 边界路径（正好等于上限，应通过）
    repo2 = OrderRepository()
    svc2 = OrderService(repository=repo2, id_factory=OrderIdFactory())
    resp = OrderController(svc2).submit(amount=SINGLE_ORDER_LIMIT, date="20260214")
    check(
        f"边界路径：金额 = 上限 {SINGLE_ORDER_LIMIT:.2f} 应通过",
        resp["success"],
        resp["message"],
    )

    # 异常路径（超过上限，应拒绝）
    # 注意：必须使用**干净的仓库实例**——上一个边界用例已经成功落库一条订单，
    # 复用同一实例会让 count()==0 的断言失败。这是测试隔离的经典坑。
    repo3 = OrderRepository()
    svc3 = OrderService(repository=repo3, id_factory=OrderIdFactory())
    resp = OrderController(svc3).submit(amount=SINGLE_ORDER_LIMIT + 0.01, date="20260214")
    check(
        "异常路径：金额超上限应被拒绝且不落库",
        (not resp["success"]) and repo3.count() == 0,
        resp["message"],
    )

    # 异常路径（负数）
    resp = OrderController(svc3).submit(amount=-1.0, date="20260214")
    check("异常路径：负数金额应被拒绝", not resp["success"], resp["message"])

    # 空值路径
    resp = OrderController(svc3).submit(amount=None, date="20260214")  # type: ignore[arg-type]
    check("空值路径：金额为 None 应被拒绝", not resp["success"], resp["message"])

    # 数据一致性
    found = service.get_order("ORD-20260214-0001")
    check(
        "落库一致性：订单可查回且金额一致",
        found is not None and found["amount"] == 199.00,
        str(found),
    )

    # ------------------------------------------------- 4. 反向依赖与影响分析
    section("4. 反向依赖与影响分析（R5 / 回归范围推导依据）")
    dependents = registry.dependents_of("M40")
    check("M40 的直接依赖方查询", dependents == ["M20"], " → ".join(dependents) or "-")
    impact = registry.impact_of(["M40"])
    expected_closure = ["M01", "M20", "M40"]
    check(
        "M40 变更的传递影响闭包",
        impact == expected_closure,
        " → ".join(impact),
    )
    print("  说明：改动 M40 时，回归范围至少覆盖 " + "、".join(impact))

    # ------------------------------------------------------------- 5. 模块概览
    section("5. 模块概览")
    print(registry.summary())

    return report()


def report() -> int:
    total = len(RESULTS)
    passed = sum(1 for _, ok, _ in RESULTS if ok)
    failed = [name for name, ok, _ in RESULTS if not ok]
    print()
    print("=" * 74)
    if not failed:
        print(f"骨架跑通 [OK]   检查项 {passed}/{total} 全部通过")
        print("门禁结论：S3 骨架跑通评审（R3）的自动化部分 —— 通过")
        return 0
    print(f"骨架未跑通 [NG]  通过 {passed}/{total}，失败 {len(failed)} 项：")
    for name in failed:
        print(f"  - {name}")
    print("门禁结论：不通过 —— 不得进入 S4 模块实现阶段")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
