# 05-Omarchy机制实证：一份外部解析稿的逐条核查（原 08-Omarchy 机制实证）

> 一句话：解析稿的三层大方向全对（Arch 底座、Quickshell 单进程桌面、Agent 启动器+技能库），
> 但有三处硬伤（Lua 插件、pacman AI 决策层、权限隔离）和一处夸大（"启动器/锁屏全在单进程里"），
> 而且漏掉了 Omarchy 真正的王牌——btrfs/snapper 快照体系。
>
> 核查日期：2026-09-23。核查对象：当日收到的《Omarchy 底层技术实现深度解析》。
> 核查依据：本地 `D:\Code\07-agent-native-os\omarchy`（omacom/omarchy，分支 quattro，版本 `4.0.0.alpha`，HEAD `588f339`）
> 与 `D:\Code\07-agent-native-os\omarchy-pkgs`（Omarchy 打包仓库，含全部 PKGBUILD）。
> 下文 omarchy 相对路径以 `07-agent-native-os\omarchy` 为根，omarchy-pkgs 相对路径以 `07-agent-native-os\omarchy-pkgs` 为根，证据可当场复验。
>
> **修订标注（2026-09-25，见 [00.9-理论修正篇](09-理论修正推导.md) 第三、八节）**：处置=**沿用事实层**——
> ① 第 108 行把 CUA 式 GUI 自动化当"互补层"，与 00.5"机器对机器直连、唯人配随行翻译"不合，
> 降为**过渡件**（其驱动实现可复用，但不作世界交互的长期形态）；
> ② 第 28 行仅据 `manual/17-ai.md` 的声称即判 #13 为 ✅，应降为 ⚠️（未亲验不作论据）；
> ③ "技能只有 2 个"（第 24、83 行）指**系统技能**，与 `agents/skills/` 下 7 篇**贡献者技能**
> （第 94 行）是两回事，须区分；逐条计数随之修正为 **7✅ + 3⚠️ + 4❌**（第 31 行）。

## 一、判定总表

| # | 解析稿论断 | 判定 | 源码证据 |
|---|---|---|---|
| 1 | 基于 Arch Linux，未修改内核源码 | ✅ 对 | `manual/01-welcome-to-omarchy.md`：明说基于 Arch + Hyprland + Quickshell；仓库内无任何内核源码 |
| 2 | 默认使用 linux-zen 或标准内核 | ❌ 都不是 | Omarchy 自有内核配方：`omarchy-pkgs/pkgbuilds/linux-omarchy`（"Adopted from the Arch Linux linux package"，构建主线 7.2.5）+ `linux-omarchy-bore`（BORE CPU 调度器 + ADIOS I/O 调度器）+ `linux-ptl`（Intel Panther Lake 修复）|
| 3 | 窗口管理用 Hyprland（wlroots/Wayland 平铺） | ✅ 对 | `config/hypr/`、`shell/README.md`（"Hyprland autostart launches the shell"）、libalpm 钩子专门处理 Hyprland reload |
| 4 | Quickshell 统一 Shell：单进程承载桌面 | ✅ 对 | `docs/omarchy-shell.md`："A single long-running Quickshell instance that hosts the Omarchy desktop. The bar, panels, overlays, menus, and services all run inside as plugins" |
| 5 | **所有组件均以 Lua 脚本插件形式存在** | ❌ 错 | 插件全是 **QML**：`shell/shell.qml`、`shell/plugins/menu/Menu.qml`、manifest 的 `entryPoints: { "barWidget": "Widget.qml" }`。Lua 只用于 **Hyprland 配置**（`default/themed/hyprland.lua.tpl`）和 neovim/gum |
| 6 | 全文本化配置（.conf/.lua/.jsonc） | ⚠️ 对但格式清单错 | 实际是五种：`.toml`（`shell.toml`/`colors.toml`/`starship.toml`）、`.json`（`shell.json`）、`.jsonc`（`omarchy-menu.jsonc`）、`.conf`（alacritty/kitty/chromium-flags）、`.lua`（hyprland/neovim） |
| 7 | Quickshell 提供标准化 IPC，外部可直接下指令 | ✅ 对 | `docs/omarchy-shell.md` IPC 方法表：`ping`/`summon`/`call`/`applyTheme`/`setBarWidget` 等；`docs/menu.md`："keybind → IPC → visible path never shells out" |
| 8 | 预置 10+ 主流 Agent 懒加载启动器 | ✅ 对 | `manual/17-ai.md`：mise 管理的 stub 共 **14 个**（claude/codex/opencode/agy/copilot/crush/grok/pi/omp/ori/hermes/muse/cursor-agent），首次运行才下载 |
| 9 | 系统技能库 | ✅ 对，但很小 | `default/agents/skills/` 只有 **2 个**：`omarchy`（系统定制）+ `diagnose-crash`（崩溃诊断）；symlink 进 Claude Code/Codex/Pi/Antigravity/Hermes/`~/.agents` 七个位置（`manual/17-ai.md`） |
| 10 | 崩溃诊断：监听 systemd-coredump 交给 Agent | ✅ 对 | `manual/17-ai.md` Crash diagnosis 节：崩溃 → 通知 → 点击后连 core dump 交给默认 agent + diagnose-crash 技能；`omarchy agent crash <pid>` 可手动触发，`omarchy toggle crash-capture` 开关 |
| 11 | 技能库直接改 WM 配置/生成主题/装插件 | ⚠️ 夸大 | 技能是"给 agent 的指南"（教它怎么改），不是自动执行器；官方原文标注 experimental，建议先 plan mode，改坏可 `omarchy reinstall configs` 回滚 |
| 12 | **pacman 基础上增加 AI 决策层、优化依赖方案** | ❌ 错 | pacman 增强是**确定性的**：3 个 libalpm 钩子（`default/libalpm/hooks/`：update-guard、hyprland-reload-pause/resume）+ snapper 快照。钩子里 grep 不到任何 agent/llm 痕迹 |
| 13 | 内置 Ollama/LM Studio 本地 LLM 支持 | ✅ 对 | `manual/17-ai.md` Local LLMs 节：两者均可从 Install > AI 安装，LM Studio 被推荐给新手 |
| 14 | 用户级权限隔离，Agent 可"安全地"修改系统 | ❌ 需纠正 | shell 插件是**无沙箱**代码、与桌面同进程：`docs/omarchy-shell.md` 原文 "The facades are API boundaries, **not same-process QML sandboxes**"、"Plugin code also has the same user-level file and process access as the shell" |

判定：14 条里 8 条全对、2 条部分对（#6、#11）、4 条错（#2、#5、#12、#14）。#2 在初稿时因内核不在本仓而无法证实，拉取 `omarchy-pkgs` 后已查实。

## 二、三处硬伤，逐条纠正

### 硬伤 1：Shell 插件是 QML，不是 Lua（#5）

解析稿说"所有组件均以 Lua 脚本插件形式存在"——这是张冠李戴。Lua 在 Omarchy 里的真实身份是 **Hyprland 的配置语言**（`default/themed/hyprland.lua.tpl`，随主题模板生成），而桌面 Shell 的插件体系是纯 QML：

- 入口：`shell/shell.qml` 常驻进程，`shell/plugins/` 下每个插件带 `manifest.json`，声明 `kinds`（`bar-widget`/`bar`/`panel`/`overlay`/`menu`/`service` 六种）和 QML 入口点；
- 纯逻辑与渲染分离：菜单插件是 `Menu.qml`（渲染）+ `MenuModel.js`（逻辑，Node 也能加载跑测试）——`docs/menu.md`；
- 插件代码保存即热重载（`rescanPlugins`），`omarchy shell` 是官方开发入口。

### 硬伤 2：pacman 上没有 AI 决策层（#12）

Omarchy 对包管理的真实增强全是确定性的：

1. **三个 libalpm 钩子**（`default/libalpm/hooks/*.hook`）：更新守卫（`00-omarchy-update-guard.hook`）、更新期间暂停/恢复 Hyprland 配置重载（`10-`/`90-`）——防止"pacman -Syu 一半桌面崩掉"这类经典事故；
2. **snapper 快照**：`install/config/snapper.sh`、`docs/update-process.md`——更新前自动 btrfs 快照，更新出问题可整体回滚；
3. 包清单是静态文本：`install/omarchy-base.packages`、`install/omarchy-other.packages`。

没有任何组件会"根据系统状态优化依赖安装方案"。这个虚构很典型：把"确定性安全网"脑补成了"智能决策层"。

### 硬伤 3：不存在"用户级权限隔离"（#14）

解析稿说"通过用户级权限隔离，Agent 可安全地修改用户配置、安装软件"。源码里的事实相反，而且作者 DHH 是**明说**的：

- 第三方插件是 git 仓库，clone 进 `~/.config/omarchy/plugins/<id>/`，**默认禁用**、启用前要求人工审阅代码、更新时先看 diff——靠的是人肉审阅，不是沙箱；
- 内置/第三方插件之间只有 **facade API 边界**：认证类服务被移出公共 service map、第三方拿不到认证能力（`docs/omarchy-shell.md` 插件安全一节）；
- 但视觉插件与宿主 bar 共享同一 QML 场景，可以沿 parent 链摸到普通宿主对象；插件代码与 shell 进程同用户、同权限。

所以准确的表述是："**能力面收窄（facade）+ 流程审阅**，而非沙箱隔离"。这恰恰是 02 篇 agentd 设计的立足点——真正的门禁在 Omarchy 里不存在。

## 三、修正后的三层架构（逐层源码证据）

### L1 底座：Arch + Hyprland + 快照体系

- Arch 滚动更新 + pacman/AUR（`install/` 下按硬件/厂商拆分的装机脚本：nvidia、surface、framework、apple 等，量产装机靠脚本不靠 GUI）；
- 显示：Hyprland Wayland 平铺 compositor（`config/hypr/`，`omarchy` 技能把 hyprland.md 列为主题指南之首）；
- 内核：Omarchy 自有配方 **`linux-omarchy`**（基于 Arch 的 linux PKGBUILD 构建主线 7.2.5），可选 **`linux-omarchy-bore`**（BORE CPU 调度器 + ADIOS I/O 调度器，桌面响应取向）与 `linux-ptl`（Intel Panther Lake 硬件修复）——"没改内核源码"基本成立（构建上游源码 + 自家 config），但内核包不是 stock linux-zen/linux，而是 Omarchy 自己发的（`omarchy-pkgs/pkgbuilds/linux-omarchy*`）；
- **被解析稿整层漏掉的部分**：btrfs 文件系统 + snapper 快照 + limine 引导器 + SDDM/plymouth 登录动画。`docs/file-layout.md` 把 "boot/snapshot story end-to-end" 列为 `omarchy-settings` 包的核心职责——**快照回滚是 Omarchy 的一等公民机制，不是后加的功能**。

### L2 桌面：Quickshell 单进程 + QML 插件 + 数据驱动的菜单

- 一个长驻 Quickshell 进程承载：顶栏、面板、浮层、菜单、通知、锁屏、OSD、服务（`docs/omarchy-shell.md` 开篇 + `shell/plugins/README.md`）；
- 配置双文件：`shell.json`（布局/插件/闲置策略，保存热重载、无深合并）+ `shell.toml`（主题 token，机器级覆盖实时生效）；
- IPC 是 CLI 与 Shell 的正式通道：`omarchy-shell shell <method>`，方法从 `ping` 到 `applyTheme` 到 `listPlugins`，stdout 返回 JSON/`ok`——Agent 和脚本走同一条路（`-q` 静默 + `OMARCHY_SHELL_IPC_TIMEOUT` 限时）；
- 菜单即数据：`default/omarchy/omarchy-menu.jsonc`（用户层叠 `~/.config/omarchy/extensions/`），启动即解析、文件变更即生效、同一插件兼任系统 dmenu（`docs/menu.md`）——**这就是解析稿想表达的"指令即操作"**；
- 一个小修正：解析稿说"启动器、锁屏全在单进程"——主菜单（`omarchy.menu`）和锁屏（`omarchy.lock`）确实是 Quickshell 插件，但**应用启动器是 walker**（独立进程，配套 elephant 系列 datasource 插件：calc/clipboard/todo/websearch 等，见 `omarchy-pkgs/pkgbuilds/walker*`、`elephant*`），主题的 `[launcher]` 节也由它消费（`docs/omarchy-shell.md` 主题节）。桌面是"一个 Quickshell + 一个 walker"，不是单一进程；

### L3 Agent 层：启动器矩阵 + 两个技能 + 一个崩溃闭环

- **14 个 mise 懒加载 stub**（`manual/17-ai.md` 表格）：`~/.local/bin/` 下的薄壳，首跑才下载，`omarchy update` 统一更新；默认 agent 可设，`Super+Shift+Ctrl+A` 直启，`omarchy agent prompt "..."` 无头执行（**明确警告以自动批准模式运行**）；
- **技能只有 2 个**：`omarchy`（Hyprland/bar/主题/终端/锁屏定制的官方指南，带 topic guides：hyprland/plugins/theming/hooks/capture/contributing）+ `diagnose-crash`（崩溃归因流程），symlink 进 7 个 harness 技能目录；
- **崩溃闭环是 Omarchy 目前唯一"Agent 主动行动"的系统级回路**：systemd-coredump 监听 → 通知 → 一键把 core dump + 诊断技能交给默认 agent → 结论可回喂（mute 名单）；
- 用量面板（agents panel）：订阅/配额/token 用量每 15 分钟刷新，多机可经同步目录合并——这是**用量计量**，不是动作审计；
- 本地 LLM：LM Studio + Ollama（Install > AI）。

## 四、解析稿漏掉的五个机制（都值得进 04 篇的素材库）

1. **双包架构 + 三层 home 播种**：一个仓库产出两个 Arch 包（`omarchy` 运行时 + `omarchy-settings` 预装机文件），新用户 home 经 `/etc/skel` 播种 → `omarchy finalize user` 收尾 → `omarchy-reinstall-configs` 显式回出厂（`docs/file-layout.md`）。"重装回出厂"是 Omarchy 给 agent 乱改兜底的机制——技能文档直接引用它。
2. **btrfs/snapper/limine 快照故事**：见 L1。更新前快照、引导器集成快照回滚入口。**agentd 05 篇自造的裸 btrfs 快照钩子在这里可以直接换成对接 snapper**——见第五节。
3. **插件安全模型**：默认禁用 + 审阅 + 更新看 diff + facade 能力收窄 + 认证服务隔离 + `--yes` 全自动通道（官方注明"the path for scripts and agents"）。这套"审阅式"模型与 agentd 的"门禁式"模型互补，不是替代。
4. **主题模板跨应用同步**：`default/themed/*.tpl` 约 20 个模板——`claude.json.tpl`、`pi.json.tpl`、`hermes.yaml.tpl`、`vscode-theme.json.tpl`、`neovim.lua.tpl`、`hyprland.lua.tpl`……切一次主题，桌面 + 终端 + 编辑器 + **三家 agent 的皮肤**同步换色。agent 皮肤跟随主题（`manual/17-ai.md`）不是修辞，是模板生成系统。
5. **贡献者 agent 指南**：`agents/skills/` 下 7 篇（shell-dev/acceptance-tests/migrations/install-scripts/command-metadata/icon-font/visual-verification）——**Omarchy 用 agent 开发自己**：shell-dev.md 教 agent 怎么改桌面，acceptance-tests.md 教它怎么自测。这是"系统用 agent 生产"的现存先例。

## 五、对 agentd 的直接启示（本篇落点）

对照 02/05 篇的组件清单，Omarchy 源码把每块的"该造/该接"都改了答案：

| agentd 组件 | Omarchy 现状 | 结论 |
|---|---|---|
| 快照钩子 | snapper + btrfs + limine 已是一等公民（更新前自动快照） | **改对接**：05 篇"裸 btrfs 快照"应降级为 `snapper create` 编排，避免双快照体系 |
| 能力路由 cap.d | 只有 2 个技能（指南），无运行时门禁；插件无沙箱 | **agentd 的核心增量**不变，而且是 Omarchy 官方文档自己承认的缺口 |
| 结构化审计 | agents panel 只有用量计量；journald 无动作审计 | **照建**：Omarchy 无对应物 |
| 完工铃 job | 无长任务零轮询机制 | **照建**：Omarchy 无对应物 |
| 崩溃诊断 | systemd-coredump → agent 闭环已完整 | **不重复建设**：agentd 只做审计钩子，不做诊断 |
| IPC | omarchy-shell IPC 面完整（CLI → JSON） | **复用为 provider**：agentd 的 shell 类能力直接转发 `omarchy-shell shell call ...` |
| GUI 自动化 | Omarchy 已打包 **CUA 驱动**（trycua/cua：accessibility-tree 快照 + 输入注入，含 Hyprland 插件，`omarchy-pkgs/pkgbuilds/cua-driver-bin`、`cua-hyprland-plugin`） | **不重做**：CUA 回答"agent 怎么操作 GUI"，agentd 回答"agent 能碰什么"——两者是操作层与门禁层的关系，互补不重叠 |
| 集成形态 | systemd + libalpm 钩子 + `omarchy` 命令族 + 技能 symlink 是 Omarchy 的四大接入面 | agentd 落地形态：`agent.service` + 一个 `omarchy agentd` 命令族 + 可选 libalpm 钩子，完全一致 |

一句话收束：**Omarchy 已经把"桌面"做到了 agent 可用（IPC + 技能 + 崩溃闭环），把"系统"留成了空白（无门禁、无审计、快照没接给 agent 当 API）。agentd 补的是后者，而且不用重造前者里的任何东西。**
