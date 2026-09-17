"""模块清单 —— 本项目全部模块的契约声明与实现。

本文件同时充当**模块登记表**的代码化形式。改这里 = 改框架契约，
按 docs/01-流程与阶段/框架与模块共演化.md 必须走 R5 框架变更评审。

模块号分段（见 templates/03-设计类/02-模块清单与模块号登记表.md）：
  M01–M19 表现层 | M20–M39 业务层 | M40–M59 数据访问层
  M60–M79 基础设施层 | M80–M99 公共/工具层

骨架阶段（S3）：全部模块以**可运行的最小实现**装配，先证明框架能跑通。
模块实现阶段（S4）：逐模块替换为完整实现，每替换一个就回判框架适配性。
"""

from __future__ import annotations

from typing import Any, Dict, List, Sequence, Tuple

from module_system import ModuleDef, ModuleRegistry

# ---------------------------------------------------------------------------
# M80 公共/工具层
# ---------------------------------------------------------------------------


class IdValidator:
    """M80 提供的稳定工具能力：订单号格式校验。

    抽到工具层的理由（FC-4 重复实现触发条件）：校验规则被业务层与
    表现层同时需要，若各自实现会出现第 3 次复制。
    """

    PATTERN = r"^ORD-\d{8}-\d{4}$"

    @staticmethod
    def is_valid(order_id: str) -> bool:
        import re

        return bool(re.match(IdValidator.PATTERN, order_id or ""))


class OrderIdFactory:
    """M80：订单号生成器（确定性，便于测试与追溯）。"""

    def __init__(self, start: int = 0) -> None:
        self._seq = start

    def next_id(self, date: str) -> str:
        self._seq += 1
        return f"ORD-{date}-{self._seq:04d}"


# ---------------------------------------------------------------------------
# M40 数据访问层
# ---------------------------------------------------------------------------


class OrderRepository:
    """M40：订单持久化（骨架阶段为内存实现）。

    S4 实现时替换为真实存储实现，**契约不变**——这样上层模块无需改动。
    这正是"契约先行"的价值：换实现不动调用方。
    """

    def __init__(self) -> None:
        self._store: Dict[str, Dict[str, Any]] = {}

    def save(self, order: Dict[str, Any]) -> None:
        self._store[order["order_id"]] = dict(order)

    def find(self, order_id: str) -> Dict[str, Any] | None:
        found = self._store.get(order_id)
        return dict(found) if found else None

    def count(self) -> int:
        return len(self._store)

    def all_orders(self) -> List[Dict[str, Any]]:
        return [dict(v) for v in self._store.values()]


# ---------------------------------------------------------------------------
# M20 业务层
# ---------------------------------------------------------------------------

# 业务规则：单笔订单金额上限（与 REQ-N-004 对应，不得硬编码在表现层）
SINGLE_ORDER_LIMIT = 100000.00


class OrderService:
    """M20：订单业务逻辑。

    职责：校验订单、生成订单号、落库。
    不负责：任何输入输出格式处理（那是表现层的事）、任何存储细节（数据访问层的事）。
    依赖：M40（存储）、M80（订单号生成与校验）。
    """

    def __init__(self, repository: OrderRepository, id_factory: OrderIdFactory) -> None:
        self._repo = repository
        self._ids = id_factory

    def submit_order(self, amount: float, date: str) -> Tuple[bool, str, str]:
        """提交订单。

        Returns:
            (是否成功, 订单号或空串, 结果说明)
        """
        if amount is None:
            return False, "", "金额不能为空"
        if amount <= 0:
            return False, "", "金额必须大于 0"
        if amount > SINGLE_ORDER_LIMIT:
            return False, "", f"单笔金额不得超过 {SINGLE_ORDER_LIMIT:.2f}"

        order_id = self._ids.next_id(date)
        if not IdValidator.is_valid(order_id):
            # 契约自检：生成器与校验器必须一致，否则是框架层缺陷
            raise RuntimeError(f"生成的订单号不符合契约: {order_id}")

        self._repo.save(
            {"order_id": order_id, "amount": round(amount, 2), "status": "CREATED"}
        )
        return True, order_id, f"下单成功，订单号 {order_id}"

    def get_order(self, order_id: str) -> Dict[str, Any] | None:
        return self._repo.find(order_id)


# ---------------------------------------------------------------------------
# M01 表现层
# ---------------------------------------------------------------------------


class OrderController:
    """M01：订单主流程编排（表现层）。

    职责：接收外部请求、调用业务层、组织输出。
    不负责：业务规则判断、存储细节。
    依赖：M20（业务）、M80（校验）。
    """

    def __init__(self, service: OrderService) -> None:
        self._service = service

    def submit(self, amount: float, date: str) -> Dict[str, Any]:
        ok, order_id, message = self._service.submit_order(amount, date)
        return {
            "success": ok,
            "order_id": order_id,
            "message": message,
            "exit_code": 0 if ok else 2,
        }


# ---------------------------------------------------------------------------
# 模块清单（登记表的代码化形式）
# ---------------------------------------------------------------------------


def module_definitions() -> Sequence[ModuleDef]:
    """全部模块的契约声明。"""
    return (
        ModuleDef(
            module_id="M01",
            name="订单主流程编排",
            version="0.1.0",
            responsibility="接收外部下单请求，编排业务层调用并组织输出",
            not_responsible="业务规则判断、数据存储、金额合法性校验",
            depends_on=frozenset({"M20", "M80"}),
            provides=frozenset({"OrderController.submit"}),
            owner="<姓名>",
            implemented=True,
        ),
        ModuleDef(
            module_id="M20",
            name="订单业务逻辑",
            version="0.1.0",
            responsibility="执行订单业务规则校验、生成订单号、驱动落库",
            not_responsible="输入输出格式处理、存储实现、日志持久化",
            depends_on=frozenset({"M40", "M80"}),
            provides=frozenset({"OrderService.submit_order", "OrderService.get_order"}),
            owner="<姓名>",
            implemented=True,
        ),
        ModuleDef(
            module_id="M40",
            name="订单数据访问",
            version="0.1.0",
            responsibility="提供订单的持久化与查询能力",
            not_responsible="业务规则校验、订单号生成",
            depends_on=frozenset(),
            provides=frozenset({"OrderRepository.save", "OrderRepository.find"}),
            owner="<姓名>",
            implemented=True,
        ),
        ModuleDef(
            module_id="M80",
            name="公共工具",
            version="0.1.0",
            responsibility="提供跨层复用的无状态工具能力（订单号生成与格式校验）",
            not_responsible="任何业务规则、任何外部 IO",
            depends_on=frozenset(),
            provides=frozenset({"OrderIdFactory.next_id", "IdValidator.is_valid"}),
            owner="<姓名>",
            implemented=True,
        ),
    )


def build_registry() -> ModuleRegistry:
    """构造并校验模块注册表。"""
    registry = ModuleRegistry()
    for module in module_definitions():
        registry.register(module)
    return registry
