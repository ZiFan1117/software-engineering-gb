"""单元测试 —— 对应测试级别 L1（GB/T 38634.2-2020 测试过程）。

运行方式：
    python -m pytest tests -q          # 推荐
    python tests/test_module_system.py # 无 pytest 时的降级运行

覆盖要求（本仓库硬性要求）：正常、边界、异常、空值 四类路径。
"""

from __future__ import annotations

import os
import sys

_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(_ROOT, "skeleton"))

try:
    import pytest

    HAS_PYTEST = True
except ImportError:  # 允许在未安装 pytest 的环境中直接运行
    import types

    HAS_PYTEST = False
    pytest = types.ModuleType("pytest")  # type: ignore[assignment]

    class _SkipTest(Exception):
        """降级模式下的跳过信号。"""

    def _raises(exc_type, **kwargs):  # type: ignore[no-untyped-def]
        """pytest.raises 的最小替身：返回上下文管理器，退出时校验异常类型。"""
        match = kwargs.get("match")

        class _Ctx:
            def __enter__(self):
                return self

            def __exit__(self, exc_type_, exc, tb):
                if exc_type_ is None:
                    raise AssertionError(f"期望抛出 {exc_type.__name__}，但没有异常")
                if not issubclass(exc_type_, exc_type):
                    return False
                if match and not __import__("re").search(match, str(exc)):
                    raise AssertionError(
                        f"异常信息 {str(exc)!r} 未匹配 {match!r}"
                    )
                return True

        return _Ctx()

    def _mark_noop(*args, **kwargs):  # type: ignore[no-untyped-def]
        def decorator(func):  # type: ignore[no-untyped-def]
            return func

        return decorator

    pytest.raises = _raises  # type: ignore[attr-defined]
    pytest.mark = types.SimpleNamespace(  # type: ignore[attr-defined]
        parametrize=_mark_noop
    )
    pytest.fixture = _mark_noop  # type: ignore[attr-defined]
    pytest.skip = lambda reason="": (_ for _ in ()).throw(_SkipTest(reason))  # type: ignore[attr-defined]
    pytest.SkipTest = _SkipTest  # type: ignore[attr-defined]
    pytest.fail = lambda reason="": (_ for _ in ()).throw(AssertionError(reason))  # type: ignore[attr-defined]

from module_system import (  # noqa: E402
    ContractError,
    ModuleDef,
    ModuleRegistry,
    layer_of,
    parse_module_id,
)
from modules import (  # noqa: E402
    SINGLE_ORDER_LIMIT,
    IdValidator,
    OrderController,
    OrderIdFactory,
    OrderRepository,
    OrderService,
    build_registry,
)

# ---------------------------------------------------------------------------
# 模块号规则
# ---------------------------------------------------------------------------


class TestModuleId:
    def test_正常模块号可解析(self) -> None:
        assert parse_module_id("M01") == 1
        assert parse_module_id("M45") == 45
        assert parse_module_id("M99") == 99

    def test_非法模块号被拒绝(self) -> None:
        # 用循环而非 @pytest.mark.parametrize：参数化会让参数失去默认值，
        # 使得本文件无法在无 pytest 环境下降级运行（CI 已把两种模式都跑一遍）。
        for bad in ["M1", "M001", "m01", "X01", "MAA", "", "M00"]:
            with pytest.raises(ContractError):
                parse_module_id(bad)

    def test_非字符串模块号被拒绝(self) -> None:
        with pytest.raises(ContractError):
            parse_module_id(123)  # type: ignore[arg-type]

    def test_分层划分正确(self) -> None:
        for mid, want in [
            ("M01", "表现层"),
            ("M20", "业务层"),
            ("M45", "数据访问层"),
            ("M60", "基础设施层"),
            ("M80", "公共/工具层"),
        ]:
            assert layer_of(mid) == want


# ---------------------------------------------------------------------------
# 契约校验
# ---------------------------------------------------------------------------


def make_module(module_id: str = "M01", **kwargs) -> ModuleDef:
    defaults = dict(
        name="测试模块",
        responsibility="做一件事",
        not_responsible="不做另一件事",
    )
    defaults.update(kwargs)
    return ModuleDef(module_id=module_id, **defaults)  # type: ignore[arg-type]


class TestModuleContract:
    def test_合法契约可构造(self) -> None:
        module = make_module()
        assert module.layer == "表现层"

    def test_必须声明职责(self) -> None:
        with pytest.raises(ContractError, match="职责"):
            make_module(responsibility="")

    def test_模块名称不能为空(self) -> None:
        with pytest.raises(ContractError, match="名称不能为空"):
            ModuleDef(
                module_id="M01",
                name="",
                responsibility="做一件事",
                not_responsible="不做另一件事",
            )

    def test_提供接口默认空集(self) -> None:
        module = make_module()
        assert module.provides == frozenset()
        assert module.owner == "<未指派>"
        assert module.implemented is False

    def test_必须声明不负责什么(self) -> None:
        with pytest.raises(ContractError, match="不负责"):
            make_module(not_responsible="")

    def test_版本号必须语义化(self) -> None:
        with pytest.raises(ContractError, match="语义化"):
            make_module(version="1.0")

    def test_不能依赖自身(self) -> None:
        with pytest.raises(ContractError, match="自身"):
            make_module(depends_on=frozenset({"M01"}))

    def test_依赖模块号必须合法(self) -> None:
        with pytest.raises(ContractError):
            make_module(depends_on=frozenset({"BAD"}))


# ---------------------------------------------------------------------------
# 注册表：登记、复用、作废
# ---------------------------------------------------------------------------


class TestRegistry:
    def test_重复登记被拒绝(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M01"))
        with pytest.raises(ContractError, match="重复登记"):
            registry.register(make_module("M01"))

    def test_作废编号不得回收(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M01"))
        registry.retire("M01")
        with pytest.raises(ContractError, match="不得回收"):
            registry.register(make_module("M01"))

    def test_未登记模块查询报错(self) -> None:
        registry = ModuleRegistry()
        with pytest.raises(ContractError, match="未登记"):
            registry.get("M09")

    def test_依赖缺失被检出(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M01", depends_on=frozenset({"M40"})))
        with pytest.raises(ContractError, match="依赖缺失"):
            registry.validate()

    def test_分层倒置被检出(self) -> None:
        registry = ModuleRegistry()
        # M20 业务层 依赖 M01 表现层 —— 低层依赖高层，必须报错
        registry.register(make_module("M01"))
        registry.register(make_module("M20", depends_on=frozenset({"M01"})))
        with pytest.raises(ContractError, match="分层倒置"):
            registry.validate()

    def test_跨层跳跃被检出(self) -> None:
        registry = ModuleRegistry()
        # M01 表现层 直连 M40 数据访问层 —— 中间隔着业务层，属跨层跳跃
        registry.register(make_module("M40"))
        registry.register(make_module("M01", depends_on=frozenset({"M40"})))
        with pytest.raises(ContractError, match="跨层跳跃"):
            registry.validate()

    def test_相邻层依赖不算跨层跳跃(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M40"))
        registry.register(make_module("M20", depends_on=frozenset({"M40"})))
        registry.validate()  # 业务层 → 数据访问层为相邻层，不应抛错

    def test_循环依赖被检出(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M20", depends_on=frozenset({"M21"})))
        registry.register(make_module("M21", depends_on=frozenset({"M20"})))
        cycles = registry.find_cycles()
        assert cycles, "应检测到循环依赖"
        with pytest.raises(ContractError, match="循环依赖"):
            registry.validate()

    def test_工具层可被任何层依赖(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M80"))
        registry.register(make_module("M01", depends_on=frozenset({"M80"})))
        registry.validate()  # 不应抛错

    def test_工具层不得依赖业务层(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M60"))
        registry.register(make_module("M80", depends_on=frozenset({"M60"})))
        with pytest.raises(ContractError, match="工具层越界"):
            registry.validate()

    def test_工具层内部互相依赖允许(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M81"))
        registry.register(make_module("M80", depends_on=frozenset({"M81"})))
        registry.validate()  # 同层依赖允许

    def test_作废未登记模块报错(self) -> None:
        registry = ModuleRegistry()
        with pytest.raises(ContractError, match="未登记的模块无法作废"):
            registry.retire("M09")

    def test_模块集合查询与计数(self) -> None:
        registry = build_registry()
        assert len(registry) == 4
        assert set(registry.modules) == {"M01", "M20", "M40", "M80"}
        # 返回的是副本，外部修改不影响注册表
        registry.modules.clear()
        assert len(registry) == 4

    def test_概览输出包含全部模块(self) -> None:
        text = build_registry().summary()
        for mid in ("M01", "M20", "M40", "M80"):
            assert mid in text
        assert "模块总数: 4" in text
        assert "已实现:   4" in text

    def test_概览输出区分桩与实现(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M01"))
        registry.register(make_module("M02", implemented=True, depends_on=frozenset({"M01"})))
        text = registry.summary()
        assert "仍为桩:   1" in text
        assert "已实现:   1" in text

    def test_装配顺序满足被依赖者在前(self) -> None:
        registry = ModuleRegistry()
        registry.register(make_module("M01", depends_on=frozenset({"M20"})))
        registry.register(make_module("M80"))
        registry.register(make_module("M20", depends_on=frozenset({"M40", "M80"})))
        registry.register(make_module("M40"))
        order = registry.assembly_order()
        assert order.index("M40") < order.index("M20")
        assert order.index("M80") < order.index("M20")
        assert order.index("M20") < order.index("M01")

    def test_影响分析闭包正确(self) -> None:
        registry = build_registry()
        assert registry.dependents_of("M40") == ["M20"]
        assert registry.impact_of(["M40"]) == ["M01", "M20", "M40"]
        # 变更工具层会波及所有直接使用方
        assert registry.impact_of(["M80"]) == ["M01", "M20", "M80"]


# ---------------------------------------------------------------------------
# 骨架模块行为：正常 / 边界 / 异常 / 空值
# ---------------------------------------------------------------------------


@pytest.fixture()
def controller() -> OrderController:
    return OrderController(
        OrderService(repository=OrderRepository(), id_factory=OrderIdFactory())
    )


def make_controller() -> OrderController:
    """显式工厂：降级执行器（无 pytest）下也能构造被测对象。"""
    return OrderController(
        OrderService(repository=OrderRepository(), id_factory=OrderIdFactory())
    )


class TestOrderFlow:
    def test_正常路径_合法金额下单成功(self, controller=None) -> None:
        ctl = controller or make_controller()
        resp = ctl.submit(199.00, "20260214")
        assert resp["success"] is True
        assert IdValidator.is_valid(resp["order_id"])
        assert resp["exit_code"] == 0

    def test_边界路径_金额等于上限应通过(self, controller=None) -> None:
        ctl = controller or make_controller()
        resp = ctl.submit(SINGLE_ORDER_LIMIT, "20260214")
        assert resp["success"] is True

    def test_边界路径_超过上限一个最小单位应拒绝(self, controller=None) -> None:
        ctl = controller or make_controller()
        resp = ctl.submit(SINGLE_ORDER_LIMIT + 0.01, "20260214")
        assert resp["success"] is False
        assert resp["exit_code"] == 2

    def test_异常路径_金额为零应拒绝(self, controller=None) -> None:
        ctl = controller or make_controller()
        assert ctl.submit(0, "20260214")["success"] is False

    def test_异常路径_负数金额应拒绝(self, controller=None) -> None:
        ctl = controller or make_controller()
        assert ctl.submit(-0.01, "20260214")["success"] is False

    def test_空值路径_金额为None应拒绝(self, controller=None) -> None:
        ctl = controller or make_controller()
        assert ctl.submit(None, "20260214")["success"] is False  # type: ignore[arg-type]

    def test_被拒绝的订单不落库(self) -> None:
        repo = OrderRepository()
        ctl = OrderController(OrderService(repository=repo, id_factory=OrderIdFactory()))
        ctl.submit(SINGLE_ORDER_LIMIT + 1, "20260214")
        assert repo.count() == 0

    def test_落库订单可查回(self) -> None:
        repo = OrderRepository()
        svc = OrderService(repository=repo, id_factory=OrderIdFactory())
        resp = OrderController(svc).submit(88.88, "20260214")
        found = svc.get_order(resp["order_id"])
        assert found is not None
        assert found["amount"] == 88.88
        assert found["status"] == "CREATED"

    def test_查询不存在的订单返回空(self) -> None:
        repo = OrderRepository()
        svc = OrderService(repository=repo, id_factory=OrderIdFactory())
        assert svc.get_order("ORD-20260214-9999") is None


class TestIdValidator:
    def test_合法订单号(self) -> None:
        assert IdValidator.is_valid("ORD-20260214-0001") is True

    def test_非法订单号(self) -> None:
        assert IdValidator.is_valid("ORD-2026-1") is False

    def test_空值(self) -> None:
        assert IdValidator.is_valid("") is False
        assert IdValidator.is_valid(None) is False  # type: ignore[arg-type]


class TestIdFactory:
    def test_序列递增且格式合规(self) -> None:
        factory = OrderIdFactory()
        first = factory.next_id("20260214")
        second = factory.next_id("20260214")
        assert first == "ORD-20260214-0001"
        assert second == "ORD-20260214-0002"
        assert IdValidator.is_valid(first) and IdValidator.is_valid(second)


# ---------------------------------------------------------------------------
# 无 pytest 时的降级运行入口
# ---------------------------------------------------------------------------


def _run_without_pytest() -> int:
    """极简测试执行器：仅用于没有 pytest 的环境。"""
    import inspect
    import traceback

    total = passed = 0
    failures: list[str] = []

    for name, obj in sorted(globals().items()):
        if not inspect.isclass(obj) or not name.startswith("Test"):
            continue
        for method_name, method in sorted(vars(obj).items()):
            if not method_name.startswith("test_"):
                continue
            total += 1
            instance = obj()
            try:
                # 除 self 外的可选参数（如 pytest 注入的 controller）在降级模式下补 None，
                # 但若方法已给出有意义的默认值（如参数化用例的示例参数），则保留默认值。
                params = inspect.signature(method).parameters
                extras = [name for name in params if name != "self"]
                if extras:
                    args = [
                        params[name].default
                        if params[name].default is not inspect.Parameter.empty
                        else None
                        for name in extras
                    ]
                    method(instance, *args)
                else:
                    method(instance)
                passed += 1
                print(f"  [PASS] {name}.{method_name}")
            except Exception:
                failures.append(f"{name}.{method_name}")
                print(f"  [FAIL] {name}.{method_name}")
                traceback.print_exc()

    print()
    print(f"共 {total} 项，通过 {passed}，失败 {len(failures)}")
    if failures:
        print("失败用例：" + "、".join(failures))
        return 1
    print("全部通过")
    return 0


if __name__ == "__main__":
    try:
        import pytest as _pytest  # noqa: F401

        raise SystemExit(_pytest.main([os.path.abspath(__file__), "-q"]))
    except ImportError:
        print("未安装 pytest，使用降级执行器")
        raise SystemExit(_run_without_pytest())
