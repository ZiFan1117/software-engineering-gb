# 席T · 「agent 只能一遍遍截图」这个问题是真的吗、归谁管 —— 取证与归属裁决

> **本席的题目**（委托方逐字原文）：
> 「现在没有给 Agent 提供相应的接口。以前我们人是使用 system 的，把这个工具用来操控计算机；但是它有一个问题：**我们打进去以后，它的反馈是显示在屏幕上的**，屏幕上的人可以看到。**但是这个 Agent 呢，就一遍一遍地截图**什么的——**它是没有眼睛的，人类没有给它提供眼睛，但是它得模仿人类有眼睛然后再干事**。其实我们应该是做成**统一的一个反馈的渠道**：你想给人看、想投影到屏幕上，好，那就投影到屏幕上；但是呢，**如果它之前是有接口访问的，那就应该沿着这个接口再返、推回去——反馈走这个通道**，为什么还是要显示在屏幕上呢？所以说，整个系统在有定位之前，我觉得是有一定的问题的。」
>
> **本席只判这一段。** 不打分、不总结优点、不替作者改立场。
>
> **取证纪律**：凡引本仓库文件 → 路径 ＋ 行号 ＋ 逐字原句；凡引外部资料 → 给链接；凡推断 → 显式写「**这是我的推断**」；查不到 → 写「**查不到**」。
> **本席未修改任何被评审文件。** 本文是本次评审唯一写入的文件。
>
> **立场声明（写在最前，便于读者核对）**：这一段话里其实压着**四个彼此独立的命题**。本席的结论是——**现象命题半成立（分层成立、全称不成立）；归因命题（"没有眼睛"）修辞对、机制错；原则命题（反馈沿原通道返回）不是新东西，是 RPC 的定义，而且它在本项目自己的通道里今天就成立；架构命题（"有定位之前系统有问题"）方向对，但它指向的那个东西——出口注册表与投递面——本项目自己承认根本没有建**（`WC-BOOK-001-v0.3.md:617`、`:641`）。因此本席判：**这段抱怨是真的，但它是"应当单独修的产品缺陷"，不是"必须造一个统一语义世界"的必要性证据**；而它里面唯一够得上必要性的那一小块，作者没有说出来。

---

## 零、先把这段话拆成可判的命题

不切开，判"成立/不成立"就是空转。本席作的**定义性分离**如下（不是原文的分法）：

| 编号 | 命题（从原话里抽出） | 原话依据（逐字） | 对应问题 |
|---|---|---|---|
| **C0** | **现象命题**：今天 agent 要拿到"操作结果"，只能截图 | 「但是这个 Agent 呢，就一遍一遍地截图」 | T1 |
| **C1** | **归因命题**：agent 没有眼睛，得模仿人 | 「它是没有眼睛的，人类没有给它提供眼睛，但是它得模仿人类有眼睛然后再干事」 | T1 |
| **C2** | **原则命题**：反馈应沿请求进来的那条通道返回 | 「如果它之前是有接口访问的，那就应该沿着这个接口再返、推回去——反馈走这个通道」 | T2 |
| **C3** | **架构命题**：这是系统级缺陷，需要"定位" | 「整个系统在有定位之前，我觉得是有一定的问题的」 | T3 |
| **C4** | **必要性命题**（原话未明说，但结论必然要问）：这条抱怨能不能当作"必须造统一语义世界"的理由 | —— | T5 |

另有一处措辞要先钉死，否则 T2 会判错：原话说「**我们打进去以后，它的反馈是显示在屏幕上的**」。这句话把"反馈被显示了"当成因。本席（§二.3）会指出正确的因果方向是反的。这是本席的改述，不是原文。

---

## 一、T1 现象核实：今天 agent 要拿到"操作结果"，是不是只能截屏？

分三层核实：**A 层＝有结构化通道的**、**B 层＝只剩屏幕的**、**C 层＝主流做法与它的成因**。

### 1.1 A 层：有结构化通道，而且今天就在用

| 通道 | 机制事实（外部，带链接） | 它返回的"反馈"是什么 |
|---|---|---|
| **终端 PTY** | 进程的 `stdout`/`stderr` 与退出码沿调用者建立的伪终端返回——这是 POSIX 进程模型的基线，不需要引用 | 结构化文本流 + 一个整数终态 |
| **无障碍总线 AT-SPI2** | 「At-Spi2 is a protocol over DBus, toolkit widgets use it to provide their content to screen readers such as Orca.」；「The protocol essentially consists in dbus RPCs and **notifications**. For each application (seen as a dbus sender), its tree of widgets is represented as a tree of dbus paths.」——<https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/> | **控件树（角色/名字/状态）+ D-Bus 信号事件**；事件清单见 <https://accessibility.linuxfoundation.org/a11yspecs/atspi/adoc/atspi-events.html> |
| **AT-SPI2 的动作面** | `org.a11y.atspi.Action.DoAction()`「Allows exploring and invoking the actions of a user-actionable UI component.」——<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Action.html> | **"直接调用动作"，不需要合成指针** |
| **AT-SPI2 的事件面** | `org.a11y.atspi.Event.Object` 提供 `StateChanged` / `ChildrenChanged` / `TextChanged` / `SelectionChanged` / `Announcement` 等信号——<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Event.Object.html> | **不需要轮询、不需要截图**：变化自己推过来 |
| **Windows UI Automation（UIA）** | 微软自家 agent 项目 UFO 的功能表逐字：「**Deep OS Integration** \| Windows UIA, Win32, WinCOM native control」「**Visual + UIA Detection** \| Hybrid control detection for robustness」「**Hybrid Actions** \| GUI clicks + API calls for optimal performance」——<https://github.com/microsoft/UFO> | Windows 上同样有结构树 + 控件模式；**主流 agent 在 Windows 上是"UIA + 截图"混合，不是纯截图** |
| **Windows 的旁证** | 同上仓库的论文徽章指向 **UFO²**（arXiv:2504.14603）与 **UFO³**（arXiv:2511.11332）；本席**未打开 PDF**，故只作存在性登记 | —— |
| **D-Bus 本身** | 方法调用带 `serial`，应答带 `REPLY_SERIAL` 配对（本仓库已有逐条核验：`2-依据/13-总线与通道外部参考.md:59`，出处 <https://dbus.freedesktop.org/doc/dbus-specification.html>） | 请求-应答 + `signal` 广播 |
| **X11** | XTEST 扩展「provides **Synthetic keyboard and mouse pointer actions**」——<https://manpages.debian.org/bookworm/libx11-protocol-other-perl/X11::Protocol::Ext::XTEST.3pm> | 输入方向（不是反馈方向）；X11 同时提供窗口树/属性/`GetImage` 这类结构化或半结构化的读回 |
| **Wayland 的合成器 IPC** | Hyprland「exposes 2 UNIX Sockets, for controlling/getting info about Hyprland via code/shell utilities」；`.socket2.sock`「Used for events」，事件是 `EVENT>>DATA\n` 行，含 `workspacev2` / `activewindowv2` / `openwindow` / `closewindow` / `fullscreen` / `monitoradded` 等——<https://wiki.hypr.land/ipc/> | **窗口/工作区/焦点的结构化事件流** |
| **`/proc`、`sysfs`** | 内核提供的结构化只读面（本仓库自己在用：`agentd/README.md:10`「`brightness.set` (get/set) \| 直写 sysfs，结构化返回，超界类型化报错」） | 状态值 + 类型化错误 |
| **应用自身 API / CLI** | `2-依据/11-总线工程规格.md:20`：「动作请求/结果｜`{capability:"ui.web.fill", params:{...}}` → `{outcome:"ok"}`｜agent（经 agentd）」 | 结构化动作结果 |
| **MCP / JSON-RPC** | JSON-RPC 2.0 §5：应答对象的 `id`「**It MUST be the same as the value of the id member in the Request Object.**」——<https://www.jsonrpc.org/specification> | 应答沿请求的连接返回 |

**A 层的关键事实**：这一层不是"理论上有"，是**今天有厂商在用**。Cua 的 Linux 后端自述：「It uses **AT-SPI 2 over D-Bus for the accessibility tree, XTEST for input synthesis**, and a painted agent cursor kept separate from your physical pointer.」——<https://cua.ai/blog/inside-linux-computer-use>。

**AT-SPI2 的覆盖面**（同一份官方文档）：GTK4、Qt5、WebKit 直接调用 D-Bus 接口；GTK2/GTK3、gnome-shell、Firefox、Chromium、LibreOffice、Java Swing 经 ATK + atk-adaptor 接入；「**The atspi protocol is a set of DBus interfaces that an application must implement**」——<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/architecture.html>、<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/toolkits.html>。**即：暴露要靠工具包/应用配合**——这是 A 层的**结构性缺口**，不是细节。

**Wayland 上的 AT-SPI2 可用**：freedesktop 官方 wiki 的 Wayland 一节只有一句——「**Works just the same :D**」——<https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/>。（这是 wiki 的一句话，不是规范条文，强度有限，本席如实标注。）

### 1.2 B 层：只剩屏幕的那些

| 情形 | 证据 | 它为什么只剩屏幕 |
|---|---|---|
| **自绘 UI（canvas／游戏／SDL/OpenGL）** | AT-SPI 里**有**角色概念但没有内部结构：`ATSPI_ROLE_CANVAS`、`ATSPI_ROLE_DRAWING_AREA`（"An object used for drawing custom user interface elements"）——<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Accessible.html> | 协议里有"这是一块自绘面"这个格子，**面里的东西协议不知道** |
| **X11 本身给不出结构** | Cua：「X11 can tell you a window exists. **It cannot reliably tell you that a sidebar holds a button named "Run"**… Plenty of modern apps expose no meaningful UI structure through X11 hints at all, so you end up back at pixel-matching」——<https://cua.ai/blog/inside-linux-computer-use> | X11 是绘图协议，不是语义协议 |
| **无障碍未开启的 Chromium/Electron**（VS Code、Slack、Discord、Obsidian…） | Cua：「Chromium waits until it detects an assistive technology and **only then starts building the tree**.」；解法是把会话的 `org.a11y.Status` 标志打开，「which is exactly the signal a screen reader sets」；「If the accessibility bus is missing or the toolkit accessibility setting is off, the driver still sees some X11-level information, but it loses the element tree」——同上 | 树**默认不建**，要靠会话级信号唤起来 |
| **纯 Wayland 且绕过 XWayland 的应用** | Cua：「native-Wayland-only apps that bypass it, including some modern Firefox and GTK4 builds, **may not be visible to the backend at all**」；「Native Wayland is still in preview… The supported path today is X11 or XWayland」——同上 | 后端当时只能走 X11/XWayland |
| **"结果对不对"这一步，永远要落到像素或产物** | Cua 自己的验收流程：「test an admitted background key operation and pointer operation, then **verify the result in both a fresh snapshot and a saved/reopened SVG**」——`07-agent-native-os/omarchy-pkgs/pkgbuilds/cua-hyprland-plugin/README.md:182` | 动作被"执行了"≠"生效了"；**生效要靠观察**，而观察的最后一步往往是快照 |
| **远程桌面 / DRM 保护内容** | **查不到**：本席未取到"远程桌面窗口在 AT-SPI 里如何呈现"与"DRM 内容对截图/无障碍的影响"的原始出处。此处不写结论 | —— |

**一条必须说准的边界（来自本仓库自己打包的 Cua Hyprland 插件）**：那份验收记录把范围写死到**一个应用、一个合成器、一个键位**——「The initial app scope is native Wayland Inkscape `1.4.4-6` with the canonical US keymap」「This is concurrency inside one desktop account, not multi-user or mutually untrusted-agent isolation」「Current LibreOffice Calc `26.8`, Chromium/Electron raw background input, XWayland, Unicode/IME, non-US layouts, and modified pointer gestures are **outside this profile**」——`omarchy-pkgs/pkgbuilds/cua-hyprland-plugin/README.md:66-69`、`:99-101`。

**这是本席找到的对 B 层面积最硬的一个量**：做这件事的最强玩家之一，在**本项目的目标环境（Omarchy + Hyprland）**上，把"有把握"的范围画到了**一个应用**。

> 顺带登记一处**本席未能核实**的事实冲突（不是结论，是待核项）：Cua 的 Linux 后端文章（2026-06-18）写「the community fork that explored private Xvfb and Hyprland background capture was **closed rather than merged**」——<https://cua.ai/blog/inside-linux-computer-use>；而本仓库打包的 `cua-hyprland-plugin 0.26.1` 的 PKGBUILD 从 Cua 官方 release 取件（`omarchy-pkgs/pkgbuilds/cua-hyprland-plugin/PKGBUILD:10` `url='https://github.com/trycua/cua'`；`:23` 的 `source=` 指向 `releases/download/cua-hyprland-kit-v1.1.0-omarchy-stable-20260910/...`），且其 README 引用了 Cua 的资格记录与 Cua PR #3698。二者是"同一个东西的两个阶段"还是"两个不同的东西"，**本席查不到**。

### 1.3 C 层：主流"computer use"是不是以截屏回路为主？

**是，但要分开看"模型级"和"工具级"，因为两者结论相反。**

**(甲) 模型级 computer use：以截屏为主，这是事实。**
Anthropic 的原始公告逐字：「Claude looks at **screenshots** of what's visible to the user, then **counts how many pixels vertically or horizontally it needs to move a cursor** in order to click in the correct place. Training Claude to count pixels accurately was critical.」——<https://www.anthropic.com/news/developing-computer-use>。
**为什么这样选**，同一篇给了直接理由：「Our goal is for Claude to take **pre-existing pieces of computer software and simply use them as a person would**」——即**通用性**：不为每个软件写集成。代价同一篇也写了：「The "**flipbook**" nature of Claude's view of the screen—taking screenshots and piecing them together, rather than observing a more granular video stream—**means that it can miss short-lived actions or notifications**.」

**这条机制在参考实现里可以逐行对上**（本席另经取证子进程核到源码）：`computer.py` 里 `async def screenshot(self): """Take a screenshot of the current screen and return the base64 encoded image."""`，动作以 `coordinate: tuple[int, int]` 传坐标；`loop.py` 的 `_make_api_tool_result()` 把 `{"type": "image", …, "data": result.base64_image}` 塞进 `{"type": "tool_result", "content": …}` 再作为 `user` 消息回给模型——**即"反馈"确实是以"工具结果"这个形态沿调用通道返回的，只不过结果是图**。出处：<https://github.com/anthropics/claude-quickstarts/blob/main/computer-use-demo/computer_use_demo/tools/computer.py>、<https://github.com/anthropics/claude-quickstarts/blob/main/computer-use-demo/computer_use_demo/loop.py>（该仓库原名 `anthropics/anthropic-quickstarts`，现已更名；其 README 自述是「the essential agent loop running against a Linux desktop in Docker with **X11 + VNC**」）。

**OpenAI 一侧同样如此**（官方指南逐字）：「You provide the environment and execute the model's requests. **The model uses screenshots and other tool results to decide what to do next.**」「The API exchange has three steps: send a task, execute the returned actions, and **return a screenshot**.」；返回的 `computer_call` 带一个有序 `actions` 数组（`click`/`double_click`/`drag`/`move`/`scroll`/`keypress`/`type`/`wait`/`screenshot`），回复体是 `computer_call_output` + `{"type":"computer_screenshot","image_url":"data:image/png;base64,…"}`，**并带回原来的 `call_id`**——<https://developers.openai.com/api/docs/guides/tools-computer-use>。
（**一处必须登记的措辞更正**：委托材料与本席初稿里用的工具名 `computer_use_preview`，在当期官方指南里**查不到**；现在文档写的是 `computer` 工具 / `computer_call`。旧名按历史名对待。）

> 上面 Anthropic 那句「**flipbook** … **can miss short-lived actions or notifications**」，是本席认为**全篇最有价值的外部引文**：它说明截屏回路的真正缺陷**不是"看不清"，是"漏事件"**——两次截图之间发生的事，既不进上下文、也不留痕。这与本项目 `2-依据/14-总线词表v0.md:174`「**完工铃（`job.done`）第一优先**：长任务零轮询是 `传` 的第一硬约束」要解决的是**同一个病**。

**(乙) 工具级 agent：主流不是截屏，是结构树。**
Playwright MCP 官方文档第一句：「**Playwright MCP uses accessibility snapshots instead of screenshots.** Every tool that interacts with the page returns a structured tree of accessible elements with refs for interaction.」；并给了对照表的原因：snapshot「Low — text only / Exact — refs point to specific elements / Instant / Deterministic / Vision model not required」，screenshot「High — image tokens / Approximate — requires coordinate guessing / Slower / Variable — layout changes break coordinates / Required」——<https://playwright.dev/mcp/snapshots>。
同一项目的 README 把它们写成了主张：「This server enables LLMs to interact with web pages **through structured accessibility snapshots, bypassing the need for screenshots or visually-tuned models**.」「**Fast and lightweight.** Uses Playwright's accessibility tree, **not pixel-based input**.」「**Deterministic tool application.** Avoids ambiguity common with screenshot-based approaches.」——<https://github.com/microsoft/playwright-mcp>。

> **一处必须写下的更正（防止本席被误引）**：**Chrome DevTools MCP 不是"只用无障碍树"的服务器**——它的 README 明确宣传「take screenshots」。本席**没有**把"无障碍树优先"这条主张安到它头上；本席的数据点只有两个：**Playwright MCP（无障碍快照优先）** 与 **Cua（AT-SPI 优先、像素兜底）**。

**(丙) 桌面驱动层：混合，且以结构优先、像素兜底。**
Cua：「It first asks what structure the app exposes. **If the element is reachable through AT-SPI it gets addressed by accessible name, role, index, and bounds instead of by pixels**, and if an action can be invoked through accessibility the driver tries that before anything else. When the app rejects that path it falls back to X11 input.」；「**Linux still needs pixels for bounds, screenshots, and dragging**, but AT-SPI gives the driver a semantic place to start instead of a screenshot and a prayer.」——<https://cua.ai/blog/inside-linux-computer-use>。

> ⚠️ **一处需回填的不一致（本席登记，不下结论）**：本仓库 `2-依据/06-重复工作审计.md:26` 把 CUA 描述成「computer-use 驱动：**AT-SPI/UIA 可访问树** + 输入注入 + 合成光标；**CLI/MCP/SDK 三接口**」（同篇 `:7` 声明此表的依据是"浅克隆亲读 README"，审计日期 2026-09-23）。本席的联网核查在 **trycua/cua 当期 README** 里**查不到**"AT-SPI/UIA/AX"这一句（README 已改写为产品页；"Connect through the CLI, MCP, or typed SDKs" 仍在）。**但 Cua 自己的 Linux 后端文章逐字写明用的是 AT-SPI 2 over D-Bus**（上文已引）。⇒ 结论（Cua 走无障碍树）不变，**但仓库那一行的引文出处需要回填**：应改引 Cua 的文档/博文，而不是 README。

### 1.4 判词（T1）

**"agent 只能截图"这句话在哪一层成立、在哪一层不成立：**

1. **在 A 层不成立，而且不成立得很彻底**：对象是浏览器时，主流工具用的是无障碍快照（Playwright MCP：「**bypassing the need for screenshots or visually-tuned models**」）；对象是 Linux 桌面应用时，最强的驱动用的是 AT-SPI 树 + `DoAction` + D-Bus 事件（Cua）；对象是 Windows 桌面应用时，微软自家的 agent 用的是「**Windows UIA, Win32, WinCOM native control**」＋「Visual + UIA Detection」（UFO）；对象是终端时，stdout/退出码本来就是这个语义。**"只能截图"在这些地方是事实错误**。
2. **在 B 层成立，但正确的说法不是"只能截图"，是"只能观察公共输出面"**：自绘面、无无障碍实现的应用、纯 Wayland 绕过 XWayland 的应用、以及**"结果是否真的生效"这一步**。这一层今天确实只剩屏幕（或产物）。
3. **"主流是截屏回路"这句要限定到模型级**：模型级（Anthropic 式 computer use、OpenAI 的 `computer` 工具）确实是截屏 + 坐标/数像素，理由是**通用性**，代价是**漏事件**；工具级不是。**并且本席要再收紧一格**：本席**查不到**任何"GUI agent 主流反馈模态"的调查统计，所以"主流"这个词在本席这里只指**产品级事实的两个方向**，不指比例（§六 第 13 项）。
4. **C1（归因命题）修辞对、机制错。** 说 agent"没有眼睛"不准确：**它有一只眼睛，只是那只眼睛不是给它配的**——人用的那块屏幕是"公共输出面"，没有寻址（§二.3）。本项目自己的判据也正是这么写的：`00-总纲.md:129`「**判断标准**：**agent 零截图、零坐标，能否读懂世界当前状态并发出动作。**」、`00-总纲.md:154`「**边界**：**N 端口注册表，全部同源**（v1 实现视觉 + 语言；听觉等留位）。」——**判据要的不是"给它一只眼睛"，是"每个角色各有一条自己的出口"。**

**本席据此判 C0：半成立。** 准确的说法是——

> **对有结构化通道的对象，agent 早就不截图了；对没有接口的程序，它只能看公共输出面；而"操作是否生效"这一步，连最强的厂商也还是回到快照。真正的问题不是"没眼睛"，是"没有给它配一条属于它的出口"。**

---

## 二、T2 「反馈应沿请求进来的那条通道返回」是新东西吗？

### 2.1 不是新东西。它是请求-响应协议的定义

| 出处 | 逐字原文 | 它说的就是 C2 |
|---|---|---|
| **JSON-RPC 2.0** §5 | 「`id` — This member is REQUIRED. **It MUST be the same as the value of the `id` member in the Request Object.**」——<https://www.jsonrpc.org/specification> | 应答**必须**回到请求者，并带请求号 |
| **JSON-RPC 2.0** §4.1 | 「A Notification is a Request object without an "id" member… **The Server MUST NOT reply to a Notification**」——同上 | 反向：**不想要反馈**的请求显式声明 |
| **D-Bus** | 「When an application handles a method call message, it is required to return a reply. **The reply is identified by a `REPLY_SERIAL` header field indicating the serial number of the `METHOD_CALL` being replied to.**」；反向规则：「If a `METHOD_CALL` message has the flag `NO_REPLY_EXPECTED`, then the application receiving the method should not send the reply message.」——<https://dbus.freedesktop.org/doc/dbus-specification.html>（本仓库已逐条核验：`2-依据/13-总线与通道外部参考.md:59`） | 应答**必须**回到请求者，并带请求号 |
| **D-Bus 的反面（同一份规范里）** | 「if the `DESTINATION` field is absent, it is considered to be a **broadcast signal**, and is sent to all applications with message matching rules that match the message. **Most signal messages are broadcasts**」——同上 | **同一条总线上就有"不返回给请求者"的合法形态**（§2.3） |
| **HTTP** | 「**HTTP is a stateless request/response protocol** for exchanging…」——<https://www.rfc-editor.org/rfc/rfc9110.html>；202 的原文更关键：「The representation sent with this response ought to describe the request's current status and **point to (or embed) a status monitor** that can provide the user with an estimate of when the request will be fulfilled.」并明写「**There is no facility in HTTP for re-sending a status code from an asynchronous operation.**」——<https://www.rfc-editor.org/rfc/rfc9110.html#section-15.3.3> | 请求-响应是默认；**而"异步的结果"HTTP 自己承认没法沿原应答回来，必须另给一个监视点**——这正是作者那句话在 HTTP 层的标准答案 |
| **DAP / LSP** | 「抄 **DAP `seq` + `request_seq`**（同一份规范里两件事都给了），与 **D-Bus `serial` + `REPLY_SERIAL`** 同构」——`2-依据/13:174`；DAP 出处 <https://debug-adapter-protocol.github.io/> | 同上 |
| **MCP** | 「**All messages between MCP clients and servers MUST follow the JSON-RPC 2.0 specification.**」「**Responses MUST include the same ID as the request they correspond to.**」「**Notifications MUST NOT include an ID.**」——<https://modelcontextprotocol.io/specification/2025-06-18/basic>；长任务的反馈：「When a party wants to receive progress updates for a request, it includes a `progressToken` in the request metadata」，接收方**MAY**回 `notifications/progress` 带同一个 token——<https://modelcontextprotocol.io/specification/2025-06-18/basic/utilities/progress>；工具结果**回到调用者而不是打到屏幕上**：「Unstructured content is returned in the `content` field of a result」「Structured content is returned as a JSON object in the `structuredContent` field」——<https://modelcontextprotocol.io/specification/2025-06-18/server/tools> | 应答与请求号严格配对；长任务的反馈**沿同一请求**返回，但形态是**通知**不是应答。**另有一条反面纪律**：MCP 明写服务器的 `stderr`「for logging purposes」——**日志通道不是反馈通道** |
| **varlink** | 接口定义原文示例：「# Monitor the drive. **The method will reply with an update whenever the drive's state changes**」——<https://uapi-group.org/specifications/specs/varlink/>；本仓库已核验并抄下「`more`/`continues`（一次调用多次回复）」`2-依据/13:114` | **"一次请求、多次反馈"的规范级先例** |
| **本项目自己** | `world-core/src/channel.rs:31-36`：协议是「**纯文本、语言无关：一行请求 → 一行应答**」，请求 `{"kind":"act","body":{...}}`，应答 `{"ok":true,"event":{…}}` / `{"ok":false,"error":"…"}` | **本项目今天就是这么做的** |

**结论**：C2 不是新原则，它是**所有点对点请求-响应系统的默认语义**，而且**本项目的通道今天已经满足它**。`WC-BOOK-001-v0.3.md:641` 自己承认这一格是"半格"，理由是别的：「能"连上"是单向（请求—应答），**"注册"这个动作不存在**」——注意它承认的正是"请求—应答已经通了"。

### 2.2 那为什么桌面上没做到？四种解释逐条验

| 解释 | 判决 | 依据 |
|---|---|---|
| **(a) 缺机制** | **不成立** | 机制全在：AT-SPI2 的 RPC + 事件、D-Bus、XTEST、Hyprland IPC、xdg-desktop-portal（`RemoteDesktop` 的 `ConnectToEIS()` 推荐路径 + `Notify*` 兼容路径；`InputCapture` 的「events from physical or logical devices are sent directly to the application」——<https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html>、<https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html>）。Cua 已经把其中三条拼成了产品 |
| **(b) 缺标准** | **部分成立，但要说准** | 不是"没有标准"，是**唯一那个通用标准是"应用必须自己实现"的**：AT-SPI「is a set of DBus interfaces that an application must implement」（<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/architecture.html>）。"必须实现"= 有成本、可省略、可只在被辅助技术唤醒时才建（Chromium 默认不建树，同上 Cua 文） |
| **(c) 缺商业动机** | **本席判这是主因——但这是我的推断** | 桌面软件的输出面历来只有一个客户：**人**。给应用加一条"机器可读的反馈通道"，成本由应用作者付，收益归 agent 厂商——**典型的正外部性，没人自愿付费**。Cua 那篇文章把这件事说得比本席更直白：「a lot of real Linux work still lives in GTK, Qt, Electron, Chromium, Tk, and native desktop tools **that will never expose an agent API**」（<https://cua.ai/blog/inside-linux-computer-use>）——"never" 就是动机判断，而且是厂商自己下的 |
| **(d) 根本不该那样做** | **部分成立，而且这是本席要正面反驳作者的地方** | 见 §2.3 |

### 2.3 本席对作者的一处正面反驳：屏幕不是"应答通道"，它是"公共输出面"

原话说「**它的反馈是显示在屏幕上的**」——这个因果是反的。

**先说清哪些是核实过的、哪些是本席的推断**（这一条对本席很要紧）：

- **核实过的**：X11 上任一客户端可以合成输入（XTEST 扩展存在：「provides Synthetic keyboard and mouse pointer actions」，<https://manpages.debian.org/bookworm/libx11-protocol-other-perl/X11::Protocol::Ext::XTEST.3pm>）；libei 的 README 把 X11 当反例写：「There is emulated input in X through XTEST but **it provides neither separation, distinction nor control in a useful manner**」——<https://gitlab.freedesktop.org/libinput/libei/-/raw/main/README.md>。
- **查不到的**：本席原想把这块屏幕描述成"广播架构"。取证结果是：**X11 协议规范（<https://www.x.org/releases/current/doc/xproto/x11protocol.html>）全文没有出现 "broadcast" 一词**，也没有权威文档把"共享帧缓冲、每个观察者看到同一份像素、没有按观察者寻址"写成规范表述。**因此这一层描述只能算推断——这是我的推断，不是事实。**
- **在规范里有据的"广播"反例是另一处**：D-Bus 的信号——「if the `DESTINATION` field is absent, it is considered to be a **broadcast signal**, and is sent to all applications with message matching rules that match the message」——<https://dbus.freedesktop.org/doc/dbus-specification.html>。**即：同一条总线上，既有"应答必须回到请求者"（`REPLY_SERIAL`），也有"故意广播给所有匹配者"（signal）——两种语义并存，各有各的用途。**

**这是我的推断（显式标注）**：那块屏幕之所以成为 agent 唯一的反馈来源，是因为**它是一块没有寻址的公共输出面**——同一份像素，人、屏幕阅读器、投屏/录屏/远程桌面、现在的 agent 都在看。这条推断与上一条核实事实（X11 无隔离、任一客户端可注入）方向一致，但**本席没有找到把"公共输出面"写成规范条文的外部出处**。

- Wayland **明确拒绝**把这条路做进协议，理由正是"多请求者下无法区分"：libei 开篇「We start from the baseline of: '**there is no emulated input in Wayland (the protocol)**'」，并给出三条设计目标——separation / distinction / control，其中 control 的原文是「The server is in control of emulated input - it can filter input or discard at will. For example, **if the current focus window is a password prompt, the server can simply discard any emulated input**.」——同上。
  **这条对本席的论证很重要**：Wayland 的立场不是"技术上做不到合成输入"，而是"**在多个可能的请求者之间无法区分谁是谁，所以不做**"——**这正是"反馈沿原通道返回"在多观察者拓扑下不成立的同一个理由。**

**所以正确的表述不是"反馈应该沿原通道返回、不该显示在屏幕上"，而是：**

> **那个程序只有一块公共输出面。要让它"沿原通道返回"，得先给它第二条出口；而给它第二条出口，不是世界核心能替它做的事（§三）。**

两条必须一起写下的边界：

1. **屏幕作为"被多个观察者共享的输出面"是特性，不是缺陷。** 序把"人经投影接入"写成结构（`语义世界-序.md:85`「AI agent 和语义系统是原生接入的…人经投影接入」），`WC-BOOK-001-v0.3.md:213` 的结构图把「语言投影 视觉投影 ← 出口（叶子节点）」并列——**没人主张过取消人的那块屏**。作者自己也说「你想给人看、想投影到屏幕上，好，那就投影到屏幕上」——这一半与本项目一致。（"共享输出面"这一层描述本身按 §2.3 标为推断；规范里有据的多观察者形态是 D-Bus 的 broadcast signal。）
2. **异步与多观察者让"沿原通道返回"在某些场合不成立**：长任务在响应返回之后才结束（MCP 只能用 `MAY` 级通知、varlink 得靠 `more`/`continues`）；一件事可能有多方需要知道（人、agent、审计）。**这些场合的正确机制是"订阅/位置"，不是"把应答寄回给请求者"。** 本项目在这件事上恰好有一条比作者原话更准的写法：`席B-语义总线与推送-独立评审.md:406`「**订阅的是位置，不是事件。** 订阅者的语义是"**当服务器的位置大于我的位置时，叫我**"。」

### 2.4 判词（T2）

**C2 不是新东西，也不需要"发明"。判三条：**

1. **它是 RPC／HTTP／消息系统的基本语义**，且有规范原文（§2.1）。**本项目自己的通道今天就满足它**（`channel.rs:31-36`）。
2. **桌面上没做到，主因不是缺机制、也不是缺标准，是缺动机（正外部性）**——**这是我的推断**，最硬的一条旁证是 Cua 自己的判断"这些应用 **will never expose an agent API**"。
3. **作者的原则在"有接口的对象"上是对的，在"只有公共输出面的对象"上是不可执行的**：不是"把反馈推回那条通道"就能解决，而是**必须先有第二条出口**。这一点上本项目已有的措辞比原话准：不是"沿原通道返回"，是「**若干出口**：同一份东西，各方各按自己最优的方式取」（`语义世界-序.md:69`）。

---

## 三、T3 归属判断（本席最要紧的一问）

### 3.1 结论（正面选边）

**本席判：这条抱怨必须切成两半，一半不属于世界核心，一半属于世界核心。**

| 半边 | 内容 | 归谁 | 今天有没有人在管 |
|---|---|---|---|
| **半边一** | **让"没有接口的程序"开口**：AT-SPI 适配、输入注入、CDP/PTY/应用 API 的接线 | **不属于世界核心**。属 `进`（脐带/适配器）、`载`（载体能力）、以及**独立的 GUI 操作层** | **有**。Cua（AT-SPI + XTEST + MCP/CLI）、Playwright MCP（无障碍快照）、Chrome DevTools MCP、系统自带的 `hyprctl`/D-Bus/PTY |
| **半边二** | **每个角色各有一条自己的出口，且反馈从那条出口回去；出口要登记、要能核验、要能投递** | **属于世界核心**（总纲 §2.5 `显`、序的"第三样：若干出口"） | **没有**。本仓库三处独立承认：注册表未建、投递不存在、入向没有 |

### 3.2 半边一为什么不属于世界核心（举证）

- **本仓库自己做过同型的归属裁决**：`2-依据/07-硌牙清单与单机经济账.md:72-75`——
  > 「**归属裁决（2026-09-25…）**：第 3 项**记错了归属**。"事件总线"不是内核的活——它是**用户态的世界核心**（`传`）的活；内核只需要**暴露事件源**（netlink / inotify / perf / drm 等，今天已经在做）。故第 3 项应改写为"**内核侧事件源的统一（可选、长期）**"…理由：若把总线做进内核，等于把"世界的协议"锚定在载体的接口上——与"锚定 vs 依赖"判据冲突」
  同一条推理原样适用于本问：**"给 GUI 装 agent 接口"如果做进世界核心，就等于把世界的接口锚定在具体界面技术上**。世界核心的第一条纪律恰恰是不出现这类字样（`00-总纲.md:265`「**进**｜旧世界→新世界的翻译；算力端口规范；世界核心**不出现** DOM / web 字样」）。
- **本仓库已经把这件事判给了别人**：`2-依据/15-世界核心的组成与职责.md:306` 的对齐表把 `进` 写成「脐带（`transferor` 等旧世界入口）｜**自造**（但在世界之外）」；`2-依据/06-重复工作审计.md:26`——
  > 「**CUA**（trycua/cua）｜computer-use 驱动：AT-SPI/UIA 可访问树 + 输入注入 + 合成光标；CLI/MCP/SDK 三接口；权限模式 standard/bounded/unrestricted；跨 macOS/Windows/Linux｜GUI 操作层｜**互补**：CUA 回答"agent 怎么操作 GUI"，agentd 回答"agent 能碰什么"」
  同篇 `:58`：「Omarchy 打包了 CUA，意味着"操作层已就位、门禁层仍空白"的组合格局成立」。
- **本项目自己给 GUI 操作面画的路线是"分层"而不是"统一语义"**：`1-理论与哲学/01-三者关系论.md:208-216` 的四盏灯裁决表——
  > 「| 截图+坐标（computer use 原始形态） | 四灯全灭——不是镜子，是让 agent 蒙着眼摸像素 |
  > | 语义树转译（AOM/AT-SPI → ref 寻址） | 三灯半亮，且今天就能做、覆盖一切现有界面 |
  > | 原生语义优先（对偶/A2UI） | 四灯全亮——但镜面材料要重造，只适用于新界面 |」
  > 「没有单一最好，只有分层最优：**主通道用语义树转译（全覆盖 + 三灯半），新界面用原生语义优先（四灯），坐标只兜底盲区**…computer use 的正确形态因此不是"看截图点坐标装成一个人"，而是**语义化的 computer use**：agent 使用的对象是结构，不是像素。」
  **这条路线的执行者不是世界核心，是适配器层。**
- **"Agent ↔ Computer 用什么承载"本仓库也已经定了**：`1-理论与哲学/06-协作与承载.md:112`——「| Agent ↔ Computer | **声明式语义调用**（JSON-RPC/varlink/能力接口） | 唯一双方皆机器的对：零歧义、可校验、可门禁。D-Bus 二十年已验证…」

### 3.3 半边二为什么属于世界核心（以及它"多做了什么"）

**(甲) 属于它的依据（本项目自己的合格判据就是这一条）**

- `00-总纲.md:129`：判据是「**agent 零截图、零坐标，能否读懂世界当前状态并发出动作。**」——**这是写在世界核心的第一部总纲里的判据**，不是界面层的判据。
- `00-总纲.md:154-161`：「**边界**：**N 端口注册表，全部同源**（v1 实现视觉 + 语言；听觉等留位）。…规则改为：**投影只能省略，不能添加**」
- `00-总纲.md:264`（`显` 那一行）：「投影是**人向端口、无意志**——出（渲染状态）、入（把人的操作翻成语义事件）」
- `2-依据/15:209-212`：「显 = N 端口注册表（全部同源，只许省略不许添加）├─ **语言投影 ← 给 Agent 读**（结构化，可直接进上下文）├─ **视觉投影 ← 给人看**；v1 的实现恰好是 web」
- `WC-BOOK-001-v0.3.md:237`：「| **Agent** | "请你做什么"（`act`）+ 它的结果 | **语言投影** |」

**换句话说：作者说的"统一的一个反馈的渠道"，本项目的正规名字叫"若干出口 + N 端口注册表"，而它已经在总纲里了。**

**(乙) 它比 AT-SPI／D-Bus／MCP 多做了什么（逐条，这是本席必须正面回答的）**

| # | 多出来的东西 | 依据（含"别人给不了"的对照） |
|---|---|---|
| 1 | **反馈的内容层级不同**：AT-SPI 回答"这个控件叫什么、什么角色"；世界核心要求回答"**改了哪个字段、从什么变成什么、因为哪一条、谁批准、可不可逆**" | `语义世界-序.md:65`「一条事件要能被不问你的人读懂，得自描述到七项——主体是谁…改了哪个对象的哪个字段；从什么变成什么（前值必带）；因为哪一条（因果）…谁授权、可不可逆；单位与量纲」；对照：`2-依据/13:152` 的覆盖矩阵里 **④引用（内容 hash）在 MCP/A2A/ACP/AG-UI/OpenAI/D-Bus/varlink 全部"缺"**；`:153-154`「顺序/序号与按能力治理，没有任何一件同时做好」 |
| 2 | **出口之间必须"同源"且可核验**：每个出口要能出示"我用的是哪一版词表"（hash） | `2-依据/15:240`「投影必须能出示"**我用的是哪一版词表**"（hash）——即 **词表本身也内容寻址**。拿两个投影声明的词表 hash 一比，就知道它们是不是同一套解释规则。⚠ 外部教训：Solid 与 atproto 都只保证"读同一份**数据**"，**不保证"同一套解释规则"**」——**AT-SPI/D-Bus/MCP 没有这一层** |
| 3 | **出口的准入条件是"结构"而不是"通信"**：出口之间不互相发消息，各自读同一份 | `2-依据/15:232-236`「投影是**出口（叶子节点）**，不是中继节点…'同源'不是靠**商量**达成的，是靠**结构**保证的」；`WC-BOOK-001-v0.3.md:240`「**三方之间没有直连**：它们**只跟账本对齐**，不互相聊」 |
| 4 | **同一条事实同时喂两边**：人看到的那一格与 agent 读到的那一格，是同一份状态的两个渲染 | `00-总纲.md:155`「**人看到的和 agent 读到的必须是同一份状态**——不同源比截图更危险（会出现"agent 眼里的世界"和"屏幕上的世界"两个世界）」 |

> **本席必须同时说清这条判据的反面**：`2-依据/15:251` 把「"Agent 直接读界面的内容"（零截图、零坐标的判据在此）」列进了**漏水信号清单**——「只要出现下面任何一句，**就是真相分裂了**，必须停下来查」。**即：本项目并不认为"agent 去读人的那个界面"是解法；它认为那是病。** 解法是给 agent 单独一条出口。这一点与作者原话同向，但比原话更硬。

**(丙) 这一半今天交付了没有？没有。三处独立承认：**

1. **注册表不存在**：`WC-BOOK-001-v0.3.md:617`——「五个概念里，`存` 拆成了三样、`传` 拆成了两半，**只有 `显` 一样都没拆**。于是所有具体问题都从这一格漏出来：**没有注册表、没有"自上次以来"、没有注意力预算、没有入向**。」
2. **"注册"这个动作不存在，而判据已经在引用它**：`WC-BOOK-001-v0.3.md:641`——「⚠️ **半格**：能"连上"是单向（请求—应答），**"注册"这个动作不存在**——而"投影已注册"**已是三份受控文档里的合格判据** ⇒ **判据引用了不存在的标的物**」；同判 `WC-THEORY-TRACE-001-v0.1.md:45`（T31「端口**注册表**未建」）、`:68`（「| 7 | **T31 端口注册表未建** | "新增端口必须同源"只能靠人守，不能靠机器判 |」）。
3. **投递不存在**：`WC-BOOK-001-v0.3.md:615`——「三个家族里没有"投递"的位置。**账本写了"完工铃"，没有人会被叫醒**：全仓 `subscribe/notify/push/watch` 真命中为零，消费者只能自己记 `seq` 再回来问一次——**那就是轮询**」；`:639`——「| 6 | **订阅**（`11` 篇六部件之一） | ❌ **没有任何一篇接手**：`notice` 成了**结构性死信**——它甚至不进读模型，两个投影永远看不到它 |」。
   本席独立复核：`world-core/src/` 全文 grep `subscribe|notify|watch`，命中全部是 `String::push_str` 与一处错误格式化（`src/project/visual.rs`、`src/project/language.rs`、`src/lib.rs:379`、`src/carrier/run.rs:68`）——**生产代码里没有一条推送路径**。与 `席B-语义总线与推送-独立评审.md:90`、`:95` 一致：
   > 「**实测**：没有任何送达通道。」「**"完工铃是坏的"这个说法还低估了：它不是坏了，是不存在。**」
   `席E-承载与投影-独立评审.md:18` 的总结也指向同一格：「但它目前**只有"出来的那一半"**——没有入向（人把动作写回）、没有注册表、没有"自上次以来"的时间窗口、没有注意力的预算模型」；`:224`「「**没有注册表**」成立（我全文 grep 过 `world-core/`…）」；`:229`「⇒ **这是"判据引用了不存在的标的物"**，比"功能没做"严重一档：门禁在为一个空概念发绿灯。」

### 3.4 「缺的是不是只是"一个统一的出口注册表"」？

**不是"只是"。本席判缺三样，且有先后：**

| # | 缺什么 | 现成的设计草案在哪 | 是否只靠注册表就够 |
|---|---|---|---|
| 1 | **出口注册表**（有哪些出口、各自读什么范围、用哪版词表、能不能写回） | `席E:238-249` 已给 **10 个最小字段**（`port_id`/`direction`/`read_scope`/`coverage`/`window`/`layout_contract`/`identity`/`freshness`/`auditable`/`a11y_per_port`），并点明最承重的一行是 `coverage`：「**省略可数，才谈得上"同源"的完整性**」 | 不够 |
| 2 | **投递面**（订阅/位置/断线补齐） | `席B:400-450` 已给帧格式与状态机草案，并给了它的一条硬结论：`席B:406`「**订阅的是位置，不是事件。**」 | 不够 |
| 3 | **入向**（人/agent 的动作写回来） | `席E:241`：`direction` 字段之所以必须有，是因为「今天**两个端口都是 `out`**，而 `00-总纲.md:264` 明写投影要"出（渲染状态）、**入（把人的操作翻成语义事件）**"」 | 不够 |

**但"给遗留 GUI 装接口"确实不在这三样里**——那在 §3.2 已经判给了别人，而且已经有人在做了。

### 3.5 判词（T3）

1. **作者的病是真的，但"定位"这个词不够。** 这条抱怨指向的**不是"世界核心没做接口"**，而是**"世界核心没交付它自己承诺的那一块：出口与投递"**。前者它本来就不该做；后者它该做而没做。
2. **"该由谁管"的正面回答**：
   - **让程序开口** → GUI 操作层/适配器（Cua、Playwright MCP、CDP、PTY）+ 应用作者（AT-SPI 实现）。**有现成玩家，缺的是动机不是技术。**
   - **让反馈回到对的人手上、且各人拿的是同一份事实** → **世界核心**。它相对 AT-SPI/D-Bus/MCP 多做的，不是"再加一条通道"，而是：**内容的层级（前值/因果/授权/可逆）＋ 出口的可核验（词表 hash）＋ 出口之间靠结构同源而不是靠通信**。
3. **对"缺的只是一个统一出口注册表吗"的正面回答**：**不只是**。缺的是注册表 + 投递面 + 入向三件；但**"注册表"是三件里唯一一件今天连标的物都不存在的**（`WC-THEORY-TRACE-001-v0.1:68`），所以它确实该排第一。
4. **一处对本项目不利、必须写下的推论**：**如果世界核心今天连"注册表 + 投递"都没有，那么"给 agent 一条不靠截图的反馈通道"这件事，今天任何一方都不欠、也都还没有兑现**——包括本项目。**这是我的推断**，但它是由本项目自己的三处自认（§3.3 丙）直接推出的。

---

## 四、T4 反例与边界：已经做到"反馈沿原通道返回"的现成例子

逐条写"它做到了什么"与"它的边界在哪"。**边界一栏是本席认为比例子本身更重要的一半。**

| # | 例子 | 做到了什么（逐字/机制） | 边界在哪 |
|---|---|---|---|
| 1 | **终端 PTY** | 子进程的 `stdout`/`stderr`/退出码沿调用者建立的 PTY 返回——这是"反馈沿原通道"的原始形态 | **只有"启动者"拿得到**。第二个人要看同一程序，走的是同一块屏幕，于是回到"读屏幕"（`tmux capture-pane` 一类工具就是这个语义）。TUI 程序**界面内部的语义状态**（哪个菜单项被选中）不在任何通道里 |
| 2 | **LSP** | JSON-RPC 2.0 + `Content-Length` 分帧；请求/通知/事件三分；诊断是 `textDocument/publishDiagnostics` **推送** | 诊断是 server→client 的**推送而不是应答**；`2-依据/13:121`「**不抄**：LSP 假设"一个 server 服务一个工具"，无多路复用与文档锁」 |
| 3 | **DAP** | 「每事件一个全局单增 `seq`，请求/响应各带配对 id」——`2-依据/13:173-174`；<https://debug-adapter-protocol.github.io/> | 只覆盖"被调试进程"这一个对象；调试器之外的世界它不管 |
| 4 | **CDP / Playwright MCP** | 「**Playwright MCP uses accessibility snapshots instead of screenshots.** Every tool that interacts with the page returns a structured tree…」——<https://playwright.dev/mcp/snapshots> | 只覆盖浏览器渲染的东西；**canvas 内部仍要视觉模式**（同页：「For pages where visual context matters (canvas apps, charts, image-heavy layouts), combine snapshots with screenshots」） |
| 5 | **Kubernetes watch** | 官方原文：「**every Kubernetes object has a `resourceVersion` field**… The client can use that `resourceVersion` to initiate a **watch**.」「When you send a **watch** request, the API server responds with **a stream of events** (of type ADDED, MODIFIED, DELETED, or BOOKMARK) that occurred after the `resourceVersion` you specified.」；线形态 `GET /api/v1/namespaces/test/pods?watch=1&resourceVersion=10245`——<https://kubernetes.io/docs/reference/using-api/api-concepts/#efficient-detection-of-changes> | 只对 **API 对象**有效；对象之外的运行态（进程内部）不在其中。**它与本项目 `席B` 的"订阅位置而不是事件"是同构的**（`resourceVersion` 就是位置） |
| 5b | **HTTP 202 + 状态监视点 / SSE / WebSocket** | 202 原文：「ought to describe the request's current status and **point to (or embed) a status monitor**」，并明写「**There is no facility in HTTP for re-sending a status code from an asynchronous operation.**」——<https://www.rfc-editor.org/rfc/rfc9110.html#section-15.3.3>；SSE 规范第 9.2.8 节标题就是「**Connectionless push** and other features」——<https://html.spec.whatwg.org/multipage/server-sent-events.html>；WebSocket「enables **two-way communication**」「traffic in both directions」，握手是 HTTP `Upgrade: websocket`——<https://www.rfc-editor.org/rfc/rfc6455.html> | **"同一条连接"不等于"同一次 HTTP 应答"**：WebSocket 在 101 升级之后**替换掉**了 HTTP 的请求/响应模型；异步结果 HTTP 自己承认只能另给监视点。**这是 C2 的一个精确边界** |
| 6 | **数据库游标 / `RETURNING` / `LISTEN-NOTIFY`** | 同一条连接上返回结果集与通知 | 需要长连接与事务语义；跨系统边界仍然要轮询。**本席未取到逐字规范引文（查不到）**，故此处只作机制性陈述 |
| 7 | **systemd** | `Type=notify` + `sd_notify` 的 `READY=1`、`OnSuccess=`/`OnFailure=`、`systemctl --wait`、socket activation 的"请求排队跨重启保留"（本仓库已逐条引原文：`research/SYSTEMD_FOR_WORLD_CORE.md:175`、`:201`、`:204`、`:226`） | 边界是被本项目另一席钉死的：`席D-外部系统反面举证-独立评审.md:303`「**准确说法**："完工通知**只对 systemd 建模成 unit 的东西**存在；agent 随手 `fork/exec` 的长任务**没有自己的 job**…因此**不可订阅、只能轮询**"」 |
| 8 | **varlink `Monitor()`** | 接口定义原文：「# Monitor the drive. **The method will reply with an update whenever the drive's state changes**」——<https://uapi-group.org/specifications/specs/varlink/>；本仓库抄下「`more`/`continues`（一次调用多次回复）」`2-依据/13:114` | 仍是**点对点**：没有多观察者寻址；本仓库已划出边界 `2-依据/13:190`「**总线只判传输与名字，语义授权下沉服务**」 |
| 9 | **MCP `progress`** | 请求元数据放 `progressToken`，接收方 MAY 回 `notifications/progress` 带同一 token——<https://modelcontextprotocol.io/specification/2025-06-18/basic/utilities/progress> | 是 **MAY（可选）**，且形态是**通知**不是应答；没有"位置/断线补齐"语义 |
| 10 | **本项目自己的 `agentd`** | 完工铃已经实现且有订阅面：`agentd/README.md:12`「\| `job.start` (start/status/list) \| **完工铃**：登记长任务零轮询，结束响铃一次；登记簿落盘，断电重启标记 lost \| ✅ 真执行已验证 \|」；`agentd/internal/job/job.go:34`「// Bell 是完工事件：铃只响一次，听铃人醒来处理。」；`:178-181`「// Subscribe 领一只铃。每个通道独立响铃，缓冲 16 条防阻塞。」`func (r *Registry) Subscribe() <-chan Bell {`；`agentd/cmd/agentd/main.go:77-79`「// drainBells 把完工事件写进审计——铃响即档案。」`ch := reg.Subscribe()` | **它是进程内的 Go channel**（`bells []chan Bell`、缓冲 16、还有"已响但还没有订阅者领取的铃"这种补偿逻辑）——**跨进程那一半不存在**。世界核心（Rust）侧更是零（§3.3 丙） |

**另有一条反例要单列，因为它是"逆着原则做"的现成设计**：

| 11 | **D-Bus `signal`（规范里的"故意不返回给请求者"）** | 「if the `DESTINATION` field is absent, it is considered to be a **broadcast signal**, and is sent to all applications with message matching rules that match the message. **Most signal messages are broadcasts**, and no other message types currently defined in this specification may be broadcast.」——<https://dbus.freedesktop.org/doc/dbus-specification.html> | 这不是缺陷，是**多观察者场景的正确设计**：一件事发生了，谁关心谁来订阅。作者原话里"想给人看就投影到屏幕上"这一半，与它同向。**屏幕/屏幕阅读器/VNC 作为多观察者公共输出面的那一半，本席未能取到规范级出处（查不到，见 §2.3），故不列为已核事实** |

### 判词（T4）

**反例足够多，"今天没做到"这个说法必须降级。** 降级后的准确说法是本席给出的这句：

> **"反馈沿请求通道返回"在点对点、单观察者、进程生命周期内的场景里，是几十年来的常态，本项目自己的通道也已经做到；没做到的只有三件——(i) 对"根本没有编程接口的程序"；(ii) 对"响应返回之后才发生的事"（长任务与他人的变更）；(iii) 对"请求者不唯一、观察者很多"的场景。**

而这三件里，(i) 已经有人在做（Cua/Playwright MCP/CDP/PTY），(ii) 有现成的规范形态（varlink `more`、Kubernetes watch、systemd notify、MCP progress），(iii) 的正确设计恰恰是**同一件事让多个观察者各取一份**（规范里有据的形态是 D-Bus 的 broadcast signal）——也就是本项目说的"若干出口"。**所以真正剩下的、既没人做也不能照抄的那一块，是"多个出口读同一份事实，且这件事可核验"。**

---

## 五、T5 这条抱怨能不能当作"必须造一个统一语义世界"的理由？

### 5.1 判词

**不能。本席判：它是一个应当单独修的产品缺陷，不是必要性的证据。** 但它**含有一小块可以通向必要性的东西**，而那一块**作者没有说出来**。

### 5.2 理由（三条，逐条可反驳）

**理由一：这条抱怨的最小满足方案与"语义世界"无关，而且已经有产品在做。**
抱怨的要求是"别让 agent 一遍遍截图"。满足它的最小方案是：给 agent 一条 AT-SPI/MCP/CDP 出口，让反馈沿那条出口回去。Cua 与 Playwright MCP 正是这么做的（§1.3 乙丙），**它们都没有"统一语义世界"**。按本项目自己的判据，这属于"只要'能协作'，没有它也能做到"——`语义世界-序.md:55`：
> 「不过这里要说准：要拿到"改了哪个字段、前值是什么、因为哪一条、谁批准、可不可逆"这几样，才必须有那个共同的东西；**如果只要求"能协作"，没有它也能做到**，上面那张表里的东西就是现成的例子。」

**理由二：这条抱怨所要求的那件东西，本项目今天自己也没有交付——所以它无法用来证明本项目的必要性。**
作者的推论是"因为系统没做这件事，所以系统需要定位（＝语义世界）"。但 §3.3 丙 已经举证：**注册表未建、投递不存在、入向没有**，而且"投影已注册"这条判据**在引用一个不存在的标的物**（`WC-BOOK-001-v0.3.md:641`）。**换句话说：用"缺反馈通道"去证明"需要语义世界"，等于用一个语义世界自己也没兑现的缺口，去证明语义世界是必要的。** 这在论证上是循环的。**这是我的推断。**

**理由三（本席认为最要紧的一条）：二者真正的连接点不是"反馈要回来"，而是"回去的必须是各方都认的那一条事实"。**
- 如果只要求"回一句话"——**不需要世界核心**（JSON-RPC 的 `id` 就够了）。
- 如果要求"回去的这条，与另一条出口（人看到的、审计读到的）指的是**同一件事**，并能被第三方独立核对"——**那就必须有那份共同事实**。这正是序立的那条根命题：`语义世界-序.md:73`「于是"共同"能不能算数，可以拿三条来量：有争执时三方认同一处说了算；第三方能独立地查出同一件事；能力强弱不同的三方各自取得到自己能用的那一份。」
- **但这条要求不是从"agent 只能截图"里推出来的**：抱怨只要求"别让我截图"，它**没有**要求"我看到的和审计记的必须是同一条"。**这是我的推断，也是本席对这段委托最重要的判定。** 把这条抱怨当必要性证据，会把一个"通道问题"误当成"语义问题"，进而把工程排期排到错误的层上。

### 5.3 可以反驳本席的地方（本席自己先写出来）

1. **反方可以说**：作者原话里其实含有"统一的一个反馈的渠道"这个措辞，"统一"两个字已经是语义要求，不是管道要求。
   **本席的回应**：如果是"统一"的意思，那它必须能回答"统一在什么上"——是统一的传输（那 D-Bus/MCP 已经够），还是统一的**事实**（那才是语义世界）。原话没有给出这个区分，而它举的例子（"沿着这个接口推回去"）是**传输层**的例子。
2. **反方可以说**：即便是产品缺陷，它也足以支撑"必要性"——因为本项目的路线本来就是"从真实痛点出发"。
   **本席的回应**：那要接受一个代价——**痛点的归层必须正确**。"agent 只能截图"这个痛点的归层是**适配器层与出口层**，不是**共同事实层**；把它记到语义世界账上，会在施工时把它做成"世界核心去驱动 GUI"，而本项目自己的纪律（`00-总纲.md:265`"世界核心不出现 DOM / web 字样"）禁止这样做。

---

## 六、未核实项（诚实边界）

| # | 本席未能核实的内容 | 状态 |
|---|---|---|
| 1 | **Windows UI Automation 的第一手文档**（树/Pattern/事件模型/覆盖面的 Microsoft Learn 原文） | **查不到**。本席只取到**二手但同源**的证据：微软自家 UFO 仓库的功能表（§1.1 表内已引）。**"UIA 有事件模型"这一条本席没有核** |
| 2 | **X11 规范中"默认不隔离"的逐字原文**；`XGetImage` 跨客户端读像素的规范依据；**把显示面描述成"广播/无寻址公共输出面"的权威出处** | **查不到**。已核到的只有：XTEST 存在、libei README 对 X11 的对比评价、以及 **X11 协议规范全文不出现 "broadcast"** 这一否定性事实 |
| 3 | **AT-SPI2 覆盖不到的具体类别**（canvas/游戏/SDL/OpenGL/DRM 内容/远程桌面窗口）的**原始出处** | **查不到**。仅有间接证据：`ATSPI_ROLE_CANVAS` / `ATSPI_ROLE_DRAWING_AREA` 这两个角色存在，且 Cua 文承认存在"只暴露 X11 级信息"的应用 |
| 4 | **Chromium/Electron 的无障碍开关在源代码层面的机制**（本席采用的是 Cua 文章的转述 + freedesktop wiki 的 `org.a11y.Status` 说明） | 部分核实 |
| 5 | **Hyprland 是否支持 `wlr-virtual-pointer` / libei(EIS)** | **查不到**（只有第三方源码树的间接痕迹，不构成支持声明） |
| 6 | **Hyprland `hyprctl -j` 的 JSON 输出**的官方页 | **查不到**（IPC socket 两条通道已核实：<https://wiki.hypr.land/ipc/>） |
| 7 | **OpenAI 的 `computer_use_preview` 这个工具名** | **查不到**（当期官方指南写的是 `computer` 工具 / `computer_call`；**机制本身已核实**，见 §1.3 甲）。**Anthropic 的 `docs.claude.com` 电脑使用文档页在本机环境被地理阻断，本席改用其参考实现源码取证** |
| 8 | **Cua Hyprland 插件与 Cua Linux 博客中"Hyprland 后台捕获 fork 被关闭"的关系** | **查不到**（两处证据并存，见 §1.2 末） |
| 9 | **libei 在 Mutter/KWin 的支持版本表** | **查不到** |
| 10 | 序正文所引的"七项自描述"与本仓库 `2-依据/14-总线词表v0.md` 的"八条判据"的并轨（本仓库自己登记为 **D-36 未修**：`WC-PREFACE-LOG-001-v0.2.md:147`） | 本席引用时**以序的"七项"为准并标注此未决项** |
| 11 | **MCP 规范的版本** | 本席全部 MCP 引文核的是 **2025-06-18** 版；<https://modelcontextprotocol.io/specification/versioning> 称当前版本为 **2026-07-28**。**同一批措辞需对 2026-07-28 复核后方可对外引用** |
| 12 | **是否存在一条跨协议的"应答必须回到请求者"总原则的规范表述** | **查不到**。已核到的形态是**逐协议的相关性规则**（JSON-RPC `id`、MCP `id`、D-Bus `REPLY_SERIAL`、HTTP 请求/响应），**没有一份规范把它写成跨行业公理** |
| 13 | **"GUI agent 主流反馈模态"的统计性证据**（委托要求的"主流做法"量化） | **查不到**。本席只取到两个方向的**产品级事实**（Anthropic/OpenAI 走截图；Playwright MCP/UFO/Cua 走结构树或混合），**没有一份可引用的调查统计**。检索命中的 GUI agent 综述（如 arXiv:2609.02309）**本席未打开 PDF**，故不引 |

---

## 附录 A：本席真读过的本仓库文件（路径 ＋ 用到的行）

| 文件 | 用到的行 |
|---|---|
| `00-总纲.md` | 129、154、155-161、264、265 |
| `1-理论与哲学/01-三者关系论.md` | 208-216 |
| `1-理论与哲学/06-协作与承载.md` | 112、120-121 |
| `2-依据/05-Omarchy机制实证.md` | 114 |
| `2-依据/06-重复工作审计.md` | 25、26、50、58 |
| `2-依据/07-硌牙清单与单机经济账.md` | 27、72-75 |
| `2-依据/11-总线工程规格.md` | 17-22、27-29、65、99-101 |
| `2-依据/13-总线与通道外部参考.md` | 59、102-105、110-115、117-121、147-155、173-174、190 |
| `2-依据/14-总线词表v0.md` | 162-163、174、176 |
| `2-依据/15-世界核心的组成与职责.md` | 188、209-212、232-236、240、251、305-306 |
| `agentd/README.md` | 10、12、32 |
| `agentd/internal/job/job.go` | 34-35、178-192 |
| `agentd/cmd/agentd/main.go` | 77-83 |
| `omarchy-pkgs/pkgbuilds/cua-hyprland-plugin/PKGBUILD` | 5、6、10、12、21、23 |
| `omarchy-pkgs/pkgbuilds/cua-hyprland-plugin/README.md` | 3、9、13、66-69、99-103、182 |
| `world-core/src/channel.rs` | 31-36、45-50 |
| `world-core/src/project/*.rs`、`src/lib.rs`、`src/carrier/run.rs` | `subscribe\|notify\|watch` 全仓 grep 结果（生产代码零推送路径） |
| `world-core/docs/理论/语义世界-序.md` | 55、65、69、73、85、105、121 |
| `world-core/docs/理论/WC-BOOK-001-v0.3.md` | 213、237、240、615、617、639、641 |
| `world-core/docs/理论/WC-PREFACE-LOG-001-v0.2.md` | 16、19、51、53、147 |
| `world-core/docs/理论/WC-THEORY-TRACE-001-v0.1.md` | 40、45、64、68 |
| `world-core/docs/理论/专家评审/席B-语义总线与推送-独立评审.md` | 85-97、233、247、400-450 |
| `world-core/docs/理论/专家评审/席D-外部系统反面举证-独立评审.md` | 303 |
| `world-core/docs/理论/专家评审/席E-承载与投影-独立评审.md` | 18、26-28、36-40、208-249 |
| `research/SYSTEMD_FOR_WORLD_CORE.md` | 169-243、664-793 |

> 本席未读：`D:\Code\world-core-topology-research\world-core-进程拓扑对标调研.md` 全文（只核了目录层级，与本题无直接关系）。

## 附录 B：本席真取到的外部页面（链接）

1. <https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/>（AT-SPI2 over D-Bus、accessibility bus、`org.a11y.Status`、Wayland 一句）
2. <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Action.html>（`DoAction`）
3. <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Event.Object.html>（事件信号）
4. <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/doc-org.a11y.atspi.Accessible.html>（角色与状态枚举，含 `CANVAS`/`DRAWING_AREA`）
5. <https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/architecture.html>、<https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/toolkits.html>（"应用必须自己实现"）
6. <https://accessibility.linuxfoundation.org/a11yspecs/atspi/adoc/atspi-events.html>（事件表；本席由 wiki 转引，未逐字取全文）
7. <https://gitlab.freedesktop.org/libinput/libei/-/raw/main/README.md>（"there is no emulated input in Wayland"、三条设计目标、对 XTEST 的评价、uinput）
8. <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html>
9. <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html>
10. <https://gitlab.freedesktop.org/wlroots/wlr-protocols/-/raw/master/unstable/wlr-virtual-pointer-unstable-v1.xml>
11. <https://gitlab.freedesktop.org/wlroots/wlr-protocols/-/raw/master/unstable/wlr-screencopy-unstable-v1.xml>（已标废弃）
12. <https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/staging/ext-image-copy-capture/ext-image-copy-capture-v1.xml>（staging，测试期）
13. <https://wiki.hypr.land/ipc/>（两条 IPC socket、事件清单、同步求值的冻结风险）
14. <https://www.jsonrpc.org/specification>（`id` 必须相同；Notification 不得应答）
15. <https://modelcontextprotocol.io/specification/2025-06-18/basic/utilities/progress>（`progressToken` + `notifications/progress`）
16. <https://playwright.dev/mcp/snapshots>（"uses accessibility snapshots instead of screenshots" + 对照表）
17. <https://www.anthropic.com/news/developing-computer-use>（截屏 + 数像素；"flipbook"会漏事件）
18. <https://cua.ai/blog/inside-linux-computer-use>（AT-SPI + XTEST + 合成光标；X11 给不出结构；Chromium 默认不建树；纯 Wayland 不可见）
19. <https://manpages.debian.org/bookworm/libx11-protocol-other-perl/X11::Protocol::Ext::XTEST.3pm>（XTEST 合成输入）
20. <https://uapi-group.org/specifications/specs/varlink/>（`Monitor()` 一次请求多次回复；由本仓库 `2-依据/13` 转引并核验）
21. <https://dbus.freedesktop.org/doc/dbus-specification.html>（`REPLY_SERIAL` 逐字原文、`NO_REPLY_EXPECTED`、**broadcast signal**）
22. <https://kubernetes.io/docs/reference/using-api/api-concepts/#efficient-detection-of-changes>（list/get + **watch**、`resourceVersion`、ADDED/MODIFIED/DELETED/BOOKMARK 事件流）
23. <https://www.rfc-editor.org/rfc/rfc9110.html> 与 <https://www.rfc-editor.org/rfc/rfc9110.html#section-15.3.3>（"HTTP is a stateless request/response protocol"；202 + 状态监视点；"no facility … for re-sending a status code from an asynchronous operation"）
24. <https://html.spec.whatwg.org/multipage/server-sent-events.html>（§9.2.8 "Connectionless push"）、<https://www.rfc-editor.org/rfc/rfc6455.html>（WebSocket 双向；101 升级后替换 HTTP 请求/响应）
25. <https://modelcontextprotocol.io/specification/2025-06-18/basic>、<https://modelcontextprotocol.io/specification/2025-06-18/basic/transports>、<https://modelcontextprotocol.io/specification/2025-06-18/server/tools>、<https://modelcontextprotocol.io/specification/versioning>（"Responses MUST include the same ID"；stdio 与 Streamable HTTP；工具结果回到调用者；**当前版本 2026-07-28**）
26. <https://github.com/microsoft/UFO>（"Windows UIA, Win32, WinCOM native control"；"Visual + UIA Detection"；"Hybrid Actions"）
27. <https://github.com/microsoft/playwright-mcp>（"through structured accessibility snapshots, bypassing the need for screenshots"；"accessibility tree, not pixel-based input"）
28. <https://developers.openai.com/api/docs/guides/tools-computer-use>（"The model uses screenshots and other tool results"；`computer_call` / `computer_call_output` / `computer_screenshot` / `call_id`）
29. <https://github.com/anthropics/claude-quickstarts/blob/main/computer-use-demo/computer_use_demo/tools/computer.py> 与 <https://github.com/anthropics/claude-quickstarts/blob/main/computer-use-demo/computer_use_demo/loop.py>（`screenshot()` 返回 base64 图；结果以 `tool_result` 的 image 块回给模型；README 自述 Docker + **X11 + VNC**）
30. <https://www.x.org/releases/current/doc/xproto/x11protocol.html>（**否定性证据**：全文不含 "broadcast"；本席据此拒绝把"显示面是广播架构"写成已核事实）

> **取证副产物披露**：本席派出的两个取证子进程在工作区**之外**留有取件副本（`D:\Code\_factcheck\`）。本席**未向被评审仓库写入除本文之外的任何文件**。

---

## 结论栏（**留空**）

| 项 | 结论 |
|---|---|
| C0 现象命题（"只能截图"）是否成立 | `<待人工>` |
| C2 原则命题（反馈沿原通道返回）是否为本项目应接管之原则 | `<待人工>` |
| T3 归属：出口注册表与投递面是否确属世界核心 | `<待人工>` |
| T5 本条抱怨能否作为必要性证据 | `<待人工>` |
| 是否据此调整施工顺序（注册表 / 投递面 / 入向 的先后） | `<待人工>` |

## 签字栏（**留空**）

| 角色 | 姓名 | 日期 | 意见 |
|---|---|---|---|
| 项目负责人 | | | |
| 技术负责人 | | | |
| 质量负责人 | | | |
| 配置管理员 | | | |

> 依项目纪律：AI 不代签、不代指派。
