# Spec Delta

## MODIFIED Requirements

### Requirement: 身份取自内核而非请求自称

> **改的是哪一类问题**：② 措辞写窄（实现有、规格无）兼 ③ 证据错位。
> **不是**"这一层能力有没有"的问题——能力在（写入侧身份确实不取自请求），是**表述与断言的强度对不上**。
>
> `audit.md` **C1**：规格写「采用内核给出的身份、自称被忽略」，证据实为「拒绝且不落笔」；
> `audit.md` **C2**：规格写「从内核提供的连接元数据中取得身份」，而实现**不取任何连接凭证**。
>
> **证据是哪条测试的哪个断言**：
> ① 拒绝侧：`world-core/tests/contract.rs:668-670` 逐字
>    `let e = serve_once(&mut w, &listener, &expect).expect_err("冒充必须被拒");`／
>    `assert!(e.contains("Impersonation"), "实得: {e}");`／
>    `assert_eq!(w.ledger().last_seq(), before, "冒充被拒后不得落笔");`
> ② 身份取自映射侧：`world-core/tests/contract.rs:653-657` 逐字
>    `assert_eq!( ev["actor"], json!("world://agent/1"), "actor 必须取自内核身份映射，而不是请求" );`
> ③ **"不取连接元数据"这一句今天没有任何断言**——实现侧只有自述：`world-core/src/channel.rs:256-257` 逐字
>    「① 身份已由**套接字文件的权限**保证：只有 expect.uid 连得上（见 bind()）。/ 因此这里不需要（也无法用）peer_cred——它在本工具链上仍是不稳定 API。」
> ④ "未被身份映射登记的套接字被默认拒绝"今天也没有断言——实现侧为 `world-core/src/main.rs:604-615` 的 `None` 分支
>    （报 `ext.world.Channel.NotConfigured`、`rc=2`）。
> ⇒ ③④ 需补断言（列进 tasks）。

通道 SHALL 使请求方身份由**套接字绑定**决定：每个监听套接字在创建时绑定到一个身份，
只有该身份能连上；系统 SHALL NOT 从连接元数据、对端凭证或请求正文取得身份。
请求正文里自称的身份与套接字绑定的身份不一致时，系统 SHALL 拒绝该请求且 SHALL NOT 落笔。
未被身份映射登记的套接字 SHALL 被默认拒绝，而不是降级为匿名。

#### Scenario: 请求自称的身份不生效

- **WHEN** 一个请求在正文里声明一个与内核给出的身份不同的主体
- **THEN** 该请求被**拒绝且不落笔**，错误串含 `Impersonation`，账本 `last_seq` 在拒绝前后不变
- **证据**：`tests/contract.rs::c14_channel_takes_identity_from_kernel_not_from_request`
      —— **⚠ 本证据证明的是"拒绝"，不是"采用内核身份并忽略自称"**；
      "`actor` 取自映射"由同函数 `world-core/tests/contract.rs:653-657` 的另一条断言承担。

## ADDED Requirements

### Requirement: 通道身份的实际保证与它的边界

> **为什么用 ADDED 而不是 MODIFIED**：本节是一条**新的 Requirement 实体**（原规格没有这一条），
> 而 `openspec validate --strict` 要求 `## MODIFIED` 的标题必须在 `openspec/specs/` 下**逐字存在**。
> 本节**不新增能力**：它挂在既有能力 `channel-identity` 之下，只声明**该既有能力的边界**——
> 这正是把"已知边界写成正式条文"所必需的那一步。
>
> **改的是哪一类问题**：④ 与项目文档冲突（规格把**已知边界**写成已成立，通篇未登记）。
> 书稿已逐字登记该边界：`world-core/docs/理论/语义世界-第五章-今天做到几分.md:69` 逐字
> 「分情况的有 1 项。经通道进来的连接，身份由入口绑定给出，非最高权限的邻居冒称会被拒；最高权限的用户可以连任何套接字，这一项对它无效。」
>
> `audit.md` **C3**：实测「套接字 actor=`world://agent/1`、uid=1001、mode=600」下，
> uid 1001 连上→落笔 `agent/1`；**root 连上→同样落笔 `agent/1`**；只有 root「自称」`world://user` 才被拒
> ⇒「身份取自内核」这条检查**只抓自称、不抓谁连上**。
>
> **证据是哪条测试的哪个断言**：`world-core/tools/system_acceptance.sh:342-344` 逐字
> `assert_rc "㉔ M09 正例：身份 uid 的连接可建立并落笔（rc=0）" 0 "$ROOT_RC"`／
> `assert_ne "㉕ M09 反例：**别的 uid 连不上**（内核在 connect 处拒绝）" 0 "$OUTB_RC"`／
> `assert_ne "㉖ 且失败理由不是「连接成功」（输出里不得出现 CONNECTED）" "CONNECTED" "$(printf '%s' "$OUTB" | tail -1)"`。
> **⚠ 这三条断言自身的边界必须一起读**：`world-core/tools/system_acceptance.sh:314` 逐字
> `if command -v setpriv >/dev/null 2>&1 && [ "$(id -u)" = "0" ]; then`，其 `else` 分支（`:346`）逐字
> `echo "  【未能校验】跨 uid 连接反例（缺 setpriv 或非 root）"`
> ⇒ 缺 `setpriv` 或非 root 时这三条整段不执行。
> 「root 连上仍落笔 `agent/1`」这一失效形态**今天没有断言** ⇒ 需补断言（列进 tasks）。

通道身份 SHALL 由套接字文件的属主与权限保证：只有套接字属主能够连接，连接即代表该身份。
该系统 SHALL NOT 区分连接方是哪一进程，SHALL NOT 约束超级用户的行为。

已知边界 SHALL 被如实声明，且此边界是**能力的正式条文**而不是注意事项：
当套接字属主为 `root` 时，**任何** root 进程连接该套接字都取得该套接字的身份。
该边界由"套接字不可被他人写"这一层承担，**不由连接受理层承担**；
故"身份取自内核"这一说法 SHALL NOT 被读成"任何越权连接都会被识别出来"。

#### Scenario: 别的 uid 连不上（权限即身份）

- **WHEN** 在一个"祖先目录也不可被他人写"的位置建套接字（0600 + chown 到配置 uid），
       再由**另一个 uid** 尝试连接
- **THEN** 连接被内核拒绝（非零退出），且输出里不出现 `CONNECTED`
- **证据**：`world-core/tools/system_acceptance.sh`（断言 ㉔–㉖；由 `world-core/check.sh` 第 ⑥ 步执行
      ——`:133` 逐字 `run_tail 3 "系统级验收（TC-037–TC-040）" bash tools/system_acceptance.sh`）

#### Scenario: 未核实机制时的如实登记

- **WHEN** 运行环境缺 `setpriv` 或当前不是 root，使跨 uid 反例无法执行
- **THEN** 该步打印「未能校验」，**不得**记为通过，也**不得**使整体退出码变绿
- **证据**：`world-core/tools/system_acceptance.sh`（`:346` 的 `【未能校验】` 分支）

#### Scenario: 超级用户连接受理层不设防（边界固定）

- **WHEN** 由套接字属主以外的 root 进程连接该套接字
- **THEN** 连接成立并取得该套接字的身份——本断言证明的是**边界**而不是实现缺陷；
      它把"身份取自内核"限定为"**只有套接字属主能连**"，SHALL NOT 被读成"能识别越权连接"
- **证据**：需补断言（列进 tasks）——实现侧为 `world-core/src/channel.rs:256-257`
      （受理层不读对端凭证）；文档出处为 `world-core/docs/理论/语义世界-第五章-今天做到几分.md:69` 逐字
      「最高权限的用户可以连任何套接字，这一项对它无效」。
