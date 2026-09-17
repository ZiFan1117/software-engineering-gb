"""模块接口契约与注册表 —— 框架骨架的核心。

本模块实现「框架-模块共演化」中最关键的两件事：
  1. 模块定义与依赖的**声明式**描述（契约）；
  2. 依赖解析：拓扑排序 + 环检测（框架结构性错误的最早暴露点）。

设计原则（对应 docs/01-流程与阶段/框架与模块共演化.md）：
  - 契约是框架与模块之间的唯一法定接口；
  - 改契约 = 框架变更，必须走 R5 变更评审；
  - 框架 v0.x 期间允许破坏性调整，但每次调整必须留下变更记录。

零第三方依赖，仅用标准库，保证骨架在任何环境都能跑起来。
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from typing import Dict, Iterable, List, Sequence, Set

# ---------------------------------------------------------------------------
# 模块号规则（与 templates/03-设计类/02-模块清单与模块号登记表.md 一致）
# ---------------------------------------------------------------------------
#
# 格式：M + 2 位数字，按分层分段：
#   M01–M19 表现层    M20–M39 业务层    M40–M59 数据访问层
#   M60–M79 基础设施层 M80–M99 公共/工具层
#
# 规则：编号一经分配永不复用；模块废弃后编号作废不回收。

MODULE_ID_PATTERN = re.compile(r"^M(\d{2})$")

LAYERS: Sequence[tuple[str, int, int]] = (
    ("表现层", 1, 19),
    ("业务层", 20, 39),
    ("数据访问层", 40, 59),
    ("基础设施层", 60, 79),
    ("公共/工具层", 80, 99),
)


class ContractError(Exception):
    """契约违规。抛出即意味着框架层的结构性错误，应走 R5 框架变更评审。"""


def parse_module_id(module_id: str) -> int:
    """校验并解析模块号，返回其数字部分。"""
    if not isinstance(module_id, str):
        raise ContractError(f"模块号必须是字符串，收到 {type(module_id).__name__}")
    match = MODULE_ID_PATTERN.match(module_id)
    if not match:
        raise ContractError(
            f"模块号 {module_id!r} 不符合规则：必须为 'M' + 两位数字，例如 M01、M45"
        )
    number = int(match.group(1))
    if number == 0:
        raise ContractError(f"模块号 {module_id!r} 非法：序号从 01 开始，不存在 M00")
    return number


def layer_of(module_id: str) -> str:
    """返回模块号所属分层。"""
    number = parse_module_id(module_id)
    for name, low, high in LAYERS:
        if low <= number <= high:
            return name
    raise ContractError(f"模块号 {module_id!r} 未落入任何已定义分层")  # pragma: no cover


@dataclass(frozen=True)
class ModuleDef:
    """模块契约声明。

    Attributes:
        module_id:   模块号，如 M01。
        name:        模块名称。
        version:     模块版本（语义化 MAJOR.MINOR.PATCH）。
        responsibility: 职责（一句话，动词开头）。
        not_responsible: 明确不负责什么——防止职责蔓延，比职责更重要。
        depends_on:  依赖的模块号集合。
        provides:    对外提供的接口名集合。
        owner:       负责人。
        implemented: 是否已实现（False 表示当前以桩/Mock 装配）。
    """

    module_id: str
    name: str
    version: str = "0.1.0"
    responsibility: str = ""
    not_responsible: str = ""
    depends_on: frozenset[str] = field(default_factory=frozenset)
    provides: frozenset[str] = field(default_factory=frozenset)
    owner: str = "<未指派>"
    implemented: bool = False

    def __post_init__(self) -> None:
        parse_module_id(self.module_id)
        if not self.name:
            raise ContractError(f"{self.module_id}: 模块名称不能为空")
        if not re.match(r"^\d+\.\d+\.\d+$", self.version):
            raise ContractError(
                f"{self.module_id}: 版本号 {self.version!r} 不符合语义化版本 MAJOR.MINOR.PATCH"
            )
        if not self.responsibility:
            raise ContractError(f"{self.module_id}: 必须声明职责（responsibility）")
        if not self.not_responsible:
            # 这条规则看起来苛刻，但它拦住了最常见的模块边界模糊问题。
            raise ContractError(
                f"{self.module_id}: 必须声明「不负责什么」（not_responsible）——"
                "职责边界不清是模块划分失败的头号原因"
            )
        if self.module_id in self.depends_on:
            raise ContractError(f"{self.module_id}: 模块不能依赖自身")
        for dep in self.depends_on:
            parse_module_id(dep)

    @property
    def layer(self) -> str:
        return layer_of(self.module_id)


class ModuleRegistry:
    """模块注册表 + 依赖解析器。"""

    def __init__(self) -> None:
        self._modules: Dict[str, ModuleDef] = {}
        # 编号复用检测：作废的编号不得回收
        self._retired: Set[str] = set()

    # -- 注册 ---------------------------------------------------------------

    def register(self, module: ModuleDef) -> None:
        if module.module_id in self._modules:
            raise ContractError(
                f"{module.module_id}: 模块号重复登记（已存在 {self._modules[module.module_id].name}）"
            )
        if module.module_id in self._retired:
            raise ContractError(
                f"{module.module_id}: 该编号已作废，不得回收复用——请分配新编号"
            )
        self._modules[module.module_id] = module

    def retire(self, module_id: str) -> None:
        """作废模块编号（永不复用）。"""
        if module_id not in self._modules:
            raise ContractError(f"{module_id}: 未登记的模块无法作废")
        del self._modules[module_id]
        self._retired.add(module_id)

    # -- 查询 ---------------------------------------------------------------

    @property
    def modules(self) -> Dict[str, ModuleDef]:
        return dict(self._modules)

    def get(self, module_id: str) -> ModuleDef:
        try:
            return self._modules[module_id]
        except KeyError:
            raise ContractError(f"{module_id}: 未登记。请先在模块清单中登记后再引用") from None

    def __len__(self) -> int:
        return len(self._modules)

    # -- 校验 ---------------------------------------------------------------

    def validate(self) -> None:
        """依赖完整性 + 分层方向 + 环检测，一次性给出全部问题。

        框架评审（R2）与骨架跑通评审（R3）的门禁依据。
        """
        problems: List[str] = []

        # 1) 依赖必须已登记
        for module in self._modules.values():
            for dep in sorted(module.depends_on):
                if dep not in self._modules:
                    problems.append(
                        f"依赖缺失：{module.module_id} 依赖 {dep}，但 {dep} 未登记"
                    )

        # 2) 分层方向：低层不得依赖高层（分层架构的基本约束）
        #
        # 公共/工具层（M80–M99）是**横切关注点**，不参与层级排序：
        #   - 任何层都可以依赖它（它只提供无状态工具能力）；
        #   - 它自身只能依赖同层，不得依赖任何业务/基础设施层，否则就不再是"公共工具"。
        utility_layer = "公共/工具层"
        layer_index = {name: i for i, (name, _, _) in enumerate(LAYERS)}
        for module in self._modules.values():
            for dep in sorted(module.depends_on):
                if dep not in self._modules:
                    continue
                dep_module = self._modules[dep]

                if module.layer == utility_layer and dep_module.layer != utility_layer:
                    problems.append(
                        f"工具层越界：{module.module_id}（公共/工具层）依赖 "
                        f"{dep}（{dep_module.layer}）——工具层必须无状态、可被复用，"
                        "不得依赖业务或基础设施层"
                    )
                    continue

                if dep_module.layer == utility_layer or module.layer == utility_layer:
                    continue

                # 层号越小 = 层级越高（M01 表现层 → M79 基础设施层）。
                # 上层依赖下层是正常的；下层依赖上层才是倒置。
                if layer_index[module.layer] > layer_index[dep_module.layer]:
                    problems.append(
                        f"分层倒置：{module.module_id}（{module.layer}）依赖 "
                        f"{dep}（{dep_module.layer}）——低层不得依赖高层"
                    )
                    continue

                # 跨层跳跃：除相邻层外不得直连（表现层不得直连数据访问层等）。
                # 依据 templates/03-设计类/02-模块清单与模块号登记表.md「依赖方向铁律」。
                if layer_index[dep_module.layer] - layer_index[module.layer] > 1:
                    problems.append(
                        f"跨层跳跃：{module.module_id}（{module.layer}）直连 "
                        f"{dep}（{dep_module.layer}）——必须经由中间层转发，"
                        "禁止跨层跳跃"
                    )

        # 3) 环检测
        cycles = self.find_cycles()
        for cycle in cycles:
            problems.append("循环依赖：" + " → ".join(cycle))

        if problems:
            raise ContractError(
                "框架契约校验未通过（共 %d 项）：\n  - %s"
                % (len(problems), "\n  - ".join(problems))
            )

    def find_cycles(self) -> List[List[str]]:
        """返回所有检测到的循环依赖路径（DFS 三色标记）。"""
        WHITE, GRAY, BLACK = 0, 1, 2
        color: Dict[str, int] = {mid: WHITE for mid in self._modules}
        cycles: List[List[str]] = []
        stack: List[str] = []

        def visit(node: str) -> None:
            color[node] = GRAY
            stack.append(node)
            for dep in sorted(self._modules[node].depends_on):
                if dep not in self._modules:
                    continue
                if color[dep] == GRAY:
                    cycles.append(stack[stack.index(dep):] + [dep])
                elif color[dep] == WHITE:
                    visit(dep)
            stack.pop()
            color[node] = BLACK

        for mid in sorted(self._modules):
            if color[mid] == WHITE:
                visit(mid)
        return cycles

    # -- 装配顺序 -----------------------------------------------------------

    def assembly_order(self) -> List[str]:
        """返回拓扑装配顺序：被依赖者在前。

        用法：这个顺序就是骨架启动时初始化模块的顺序。
        有环则抛 ContractError —— 环依赖意味着框架设计有结构性缺陷（FC-2）。
        """
        cycles = self.find_cycles()
        if cycles:
            raise ContractError(
                "无法确定装配顺序，存在循环依赖：" + "; ".join(" → ".join(c) for c in cycles)
            )

        visited: Set[str] = set()
        order: List[str] = []

        def visit(mid: str) -> None:
            if mid in visited:
                return
            visited.add(mid)
            for dep in sorted(self._modules[mid].depends_on):
                if dep in self._modules:
                    visit(dep)
            order.append(mid)

        for mid in sorted(self._modules):
            visit(mid)
        return order

    def dependents_of(self, module_id: str) -> List[str]:
        """反向依赖查询：谁依赖了我。

        变更影响分析（R5 / S7 回归范围推导）的核心查询——改一个模块，
        必须知道会波及谁，回归范围由此推导，而不是靠"感觉没影响"。
        """
        self.get(module_id)
        return sorted(
            m.module_id for m in self._modules.values() if module_id in m.depends_on
        )

    def impact_of(self, module_ids: Iterable[str]) -> List[str]:
        """传递闭包影响分析：给定变更模块，返回全部受影响的模块（含自身）。

        回归范围推导的机器依据。
        """
        affected: Set[str] = set()
        frontier = list(module_ids)
        for mid in frontier:
            self.get(mid)
        while frontier:
            current = frontier.pop()
            if current in affected:
                continue
            affected.add(current)
            frontier.extend(self.dependents_of(current))
        return sorted(affected)

    # -- 概览 ---------------------------------------------------------------

    def summary(self) -> str:
        lines = [
            f"模块总数: {len(self._modules)}",
            f"已实现:   {sum(1 for m in self._modules.values() if m.implemented)}",
            f"仍为桩:   {sum(1 for m in self._modules.values() if not m.implemented)}",
            "",
            f"{'模块号':<7}{'名称':<16}{'分层':<14}{'版本':<9}{'依赖':<16}{'状态'}",
            "-" * 78,
        ]
        for mid in self.assembly_order():
            m = self._modules[mid]
            deps = ",".join(sorted(m.depends_on)) or "-"
            state = "实现" if m.implemented else "桩"
            lines.append(
                f"{m.module_id:<8}{m.name:<17}{m.layer:<15}{m.version:<10}{deps:<17}{state}"
            )
        return "\n".join(lines)
