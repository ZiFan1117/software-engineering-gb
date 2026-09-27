//! 通道（`M09`/`IF-006`）—— **跨进程入口，身份由内核给出，不由请求自称**。
//!
//! ## 它解决的那个洞
//!
//! 此前唯一入口是 CLI，而 CLI 的 `actor` 是**命令行参数**——被管者只要自称
//! `world://user` 就能冒充最高权主体（`WC-RV-R2-001` **FIND-06** / 假设 `A-06`）。
//! 门禁的白名单因此只在"没人说谎"的前提下成立。
//!
//! 本模块把身份来源换成**内核**：每个监听套接字绑定一个 `uid → actor` 映射，
//! 连接建立后用 `peer_cred()` 取对端 uid，**与本套接字允许的 uid 比对**；
//! 请求体里若写了别的 `actor`，**直接拒绝**——身份不可自称。
//!
//! ## 身份是怎么被强制的（以及为什么不需要 libc）
//!
//! **实测事实**：`UnixStream::peer_cred()` 在 `rustc 1.98.1` 上仍是
//! **不稳定 API**（`error[E0658]: use of unstable library feature
//! peer_credentials_unix_socket`）——原设计打算用它，编译直接把该假设否掉了。
//!
//! 改用**更强也更简单**的机制：**一个套接字对应一个身份**。
//! `bind()` 在创建套接字后就地 `chown` 给该身份的目标 uid、`chmod 0600`，
//! 并拒绝在"对 group/other 可写"的目录里创建。于是：
//!
//! - **只有那个 uid 连得上**——内核在 `connect()` 时就挡住别人，
//!   **比"连上来再问你是谁"更早、更硬**；
//! - 身份来源不再依赖任何"取对端凭证"的 API，故**零新增依赖**
//!   （只用 `std::os::unix::fs::chown`，它是 std 稳定 API），符合 `REQ-N-002`。
//!
//! ⚠️ 代价：套接字文件的权限与目录权限成为**安全前提**（与法律/账本同一类保证），
//! 故 `bind()` 会像静态墙一样检查目录，并把 mode 收紧到 `0600`。
//!
//! ## 协议（纯文本、语言无关：一行请求 → 一行应答）
//!
//! ```text
//! 请求: {"kind":"act","body":{...}}        （**不含** actor；含则必须与内核身份一致）
//! 应答: {"ok":true,"event":{…}}            / {"ok":false,"error":"…"}
//! ```
//!
//! ## 配置（纯文本 JSON，`channel.json`）
//!
//! ```json
//! { "channel": 1,
//!   "listeners": [ { "socket": "/run/world/agent-1.sock", "actor": "world://agent/1", "uid": 1001 } ] }
//! ```
//!
//! ## v1 的局限（不假装满足）
//!
//! - 只在 **Unix** 上可用（`cfg(unix)`）；
//! - `serve_once` 每次只处理**一个连接**就返回——足够验证"身份绑定"这件事，
//!   长驻服务与并发留待后续（且需与 `A-01` 单写者锁一并设计）；
//! - 不做鉴权之外的传输保护（本机 Unix 套接字 + 文件权限即其边界）。

use crate::World;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// 一条监听项：套接字路径 ↔ 身份。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listener {
    pub socket: PathBuf,
    pub actor: String,
    pub uid: u32,
}

/// 通道配置。
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    listeners: Vec<Listener>,
}

impl ChannelConfig {
    /// 从纯文本 JSON 加载。
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("ext.world.Channel.ReadFail: {}: {e}", path.display()))?;
        let v: Value = serde_json::from_str(&text)
            .map_err(|e| format!("ext.world.Channel.BadJson: {}: {e}", path.display()))?;
        let ver = v
            .get("channel")
            .and_then(Value::as_u64)
            .ok_or_else(|| "ext.world.Channel.NoVersion: 缺 `channel` 版本号".to_string())?;
        if ver != 1 {
            return Err(format!("ext.world.Channel.BadVersion: 期望 1，实得 {ver}"));
        }
        let arr = v
            .get("listeners")
            .and_then(Value::as_array)
            .ok_or_else(|| "ext.world.Channel.NoListeners: 缺 `listeners`".to_string())?;
        if arr.is_empty() {
            return Err(
                "ext.world.Channel.NoListeners: listeners 为空——没有身份映射的通道等于无门之门"
                    .to_string(),
            );
        }
        let mut listeners = Vec::new();
        for item in arr {
            let socket = item
                .get("socket")
                .and_then(Value::as_str)
                .ok_or_else(|| "ext.world.Channel.BadListener: 缺 socket".to_string())?;
            let actor = item
                .get("actor")
                .and_then(Value::as_str)
                .ok_or_else(|| "ext.world.Channel.BadListener: 缺 actor".to_string())?;
            let uid = item
                .get("uid")
                .and_then(Value::as_u64)
                .ok_or_else(|| "ext.world.Channel.BadListener: 缺 uid".to_string())?;
            listeners.push(Listener {
                socket: PathBuf::from(socket),
                actor: actor.to_string(),
                uid: uid as u32,
            });
        }
        Ok(ChannelConfig { listeners })
    }

    pub fn listeners(&self) -> &[Listener] {
        &self.listeners
    }

    /// 按套接字路径找监听项（服务端据它决定"这条连接代表谁"）。
    pub fn listener_for(&self, socket: &Path) -> Option<&Listener> {
        self.listeners.iter().find(|l| l.socket == socket)
    }
}

/// 一条请求的解析结果。
#[derive(Debug, PartialEq)]
pub struct Request {
    pub kind: String,
    pub body: Value,
    /// 请求**自称**的 actor（可选）。存在时必须与内核身份一致，否则拒绝。
    pub claimed_actor: Option<String>,
    /// 因果（可选）：引发本条的那条事件的 `id`。
    ///
    /// 跨进程的"请求—结果"配对靠它闭环：结果的 `trace` 指向意图的 `id`。
    /// **不做引用完整性校验**（`REQ-F-031` 的 v1 口径）。
    pub trace: Option<String>,
}

/// 解析一行请求。
pub fn parse_request(line: &str) -> Result<Request, String> {
    let v: Value = serde_json::from_str(line)
        .map_err(|e| format!("ext.world.Channel.BadRequest: 不是合法 JSON：{e}"))?;
    let kind = v
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| "ext.world.Channel.BadRequest: 缺 kind".to_string())?
        .to_string();
    let body = v.get("body").cloned().unwrap_or(Value::Null);
    let claimed_actor = v.get("actor").and_then(Value::as_str).map(str::to_string);
    // 空串视为未给：否则会写出一条指不到任何事件的 trace。
    let trace = v
        .get("trace")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Ok(Request {
        kind,
        body,
        claimed_actor,
        trace,
    })
}

/// 处理**一个**连接（v1：一次一条）。
///
/// 身份规则（本模块的全部意义所在）：连接能建立这件事本身**已经**证明了对端 uid
/// （套接字权限由 `bind()` 收紧，见其文档）；请求若自称 actor，必须与
/// `listener.actor` 一致，否则拒绝；落笔时的 `actor` 一律取自**身份映射**，
/// **绝不取请求里的字符串**。
/// 按监听项创建套接字：**权限即身份**。
///
/// 步骤：删掉可能存在的陈旧套接字文件 → 拒绝在"对 group/other 可写"的目录里建
/// （否则套接字可被替换，与法律/账本同理）→ `bind` 后立刻 `chmod 0600`
/// 并 `chown` 给该身份的目标 uid ⇒ **只有那个 uid 能 connect**
/// （内核在连接时就挡住别人）。
#[cfg(unix)]
pub fn bind(expect: &Listener) -> Result<std::os::unix::net::UnixListener, String> {
    use std::os::unix::fs::PermissionsExt;

    if let Some(dir) = expect.socket.parent() {
        crate::guard::assert_not_other_writable(dir, "通道目录")?;
    }
    let _ = std::fs::remove_file(&expect.socket);
    let listener = std::os::unix::net::UnixListener::bind(&expect.socket).map_err(|e| {
        format!(
            "ext.world.Channel.BindFail: {}: {e}",
            expect.socket.display()
        )
    })?;
    std::fs::set_permissions(&expect.socket, std::fs::Permissions::from_mode(0o600))
        .map_err(|e| format!("ext.world.Channel.ChmodFail: {e}"))?;
    std::os::unix::fs::chown(&expect.socket, Some(expect.uid), None).map_err(|e| {
        format!(
            "ext.world.Channel.ChownFail: {}: {e}",
            expect.socket.display()
        )
    })?;
    // **把套接字留在文件系统里**（刻意）：
    //
    // `UnixListener` 的 `Drop` 会把 socket 文件删掉。若照默认行为走，
    // "起一个监听者 → 收一个连接 → 退出"就会**把套接字一起带走**，
    // 下一个连接只会得到 `Connection refused`——而这是**监听者的生命周期**
    // 问题，不该表现成"服务没了"。
    //
    // 这一步把它转成原始描述符：进程退出后**描述符关闭、文件留下**，
    // 于是"再起一个监听者接着收"是可行的（世界核心 v1 一次一连接的分帧方式）。
    // 文件残留由下一次 `bind` 开头的 `remove_file` 负责清理（与原先同口径）。
    let raw = std::os::unix::io::IntoRawFd::into_raw_fd(listener);
    // SAFETY：`raw` 来自刚 `bind` 成功的监听套接字，所有权随此次转换移交给我们，
    // 之后不再有第二个所有者会关闭它（`FromRawFd` 只在同一处使用一次）。
    let listener = unsafe {
        <std::os::unix::net::UnixListener as std::os::unix::io::FromRawFd>::from_raw_fd(raw)
    };
    Ok(listener)
}

/// **连续收 `n` 个连接**（v1 的"长驻"形态）。
///
/// 为什么需要它：一次跨进程往返天然是**两个连接**（先交意图、后交结果）。
/// 每次调用方都自己 `bind` 一遍，就等于把套接字反复删建——那既不是"总线"该有的样子，
/// 也会把正在排队的连接一起弄丢。
///
/// 口径：**收满 `n` 个就把监听者交还给调用方**（不自己退出），
/// 由调用方决定还要不要继续收。任一连接处理失败**不中止后续**——
/// 一次坏请求不该让总线停摆；但错误会如实打印。
#[cfg(unix)]
pub fn serve_n(
    world: &mut World,
    listener: &std::os::unix::net::UnixListener,
    expect: &Listener,
    n: usize,
) -> Result<usize, String> {
    let mut ok = 0usize;
    for _ in 0..n {
        match serve_once(world, listener, expect) {
            Ok(_) => ok += 1,
            Err(e) => eprintln!("[FAIL] {e}"),
        }
    }
    Ok(ok)
}

#[cfg(unix)]
pub fn serve_once(
    world: &mut World,
    listener: &std::os::unix::net::UnixListener,
    expect: &Listener,
) -> Result<Value, String> {
    let (stream, _) = listener
        .accept()
        .map_err(|e| format!("ext.world.Channel.AcceptFail: {e}"))?;

    // ① 身份已由**套接字文件的权限**保证：只有 expect.uid 连得上（见 bind()）。
    //    因此这里不需要（也无法用）peer_cred——它在本工具链上仍是不稳定 API。
    let mut out = stream;

    // ② 读一行请求
    let mut line = String::new();
    {
        let mut r = BufReader::new(&out);
        r.read_line(&mut line)
            .map_err(|e| format!("ext.world.Channel.ReadFail: {e}"))?;
    }
    if line.trim().is_empty() {
        return Err("ext.world.Channel.EmptyRequest: 空请求".to_string());
    }
    let req = parse_request(line.trim())?;

    // ③ 自称必须与内核身份一致
    if let Some(claimed) = &req.claimed_actor {
        if claimed != &expect.actor {
            let msg = format!(
                "请求自称 actor=`{claimed}`，而本套接字的内核身份是 `{}`——**身份不可自称**，拒绝",
                expect.actor
            );
            let _ = writeln!(out, "{}", json!({"ok": false, "error": msg}));
            return Err(format!("ext.world.Channel.Impersonation: {msg}"));
        }
    }

    // ④ 落笔：actor 取自映射，不取自请求。
    //
    //    `trace`（因果）**透传**：请求里给了就带上信封。跨进程的"请求—结果"配对
    //    靠它闭环——结果事件的 `trace` 指向意图事件的 `id`（`M10` 接线，2026-09-27）。
    //    它**不做引用完整性校验**（`REQ-F-031` 的 v1 口径）：指向不存在的 id 不拒绝。
    match world.commit_requested(
        &req.kind,
        &expect.actor,
        req.body,
        req.trace.as_deref(),
        None,
    ) {
        Ok(ev) => {
            let _ = writeln!(out, "{}", json!({"ok": true, "event": ev}));
            Ok(ev)
        }
        Err(e) => {
            let _ = writeln!(out, "{}", json!({"ok": false, "error": e}));
            Err(e)
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn parses_request_and_keeps_claimed_actor() {
        let r = parse_request(
            r#"{"kind":"act","body":{"capability":"notice.mute"},"actor":"world://agent/1"}"#,
        )
        .unwrap();
        assert_eq!(r.kind, "act");
        assert_eq!(r.claimed_actor.as_deref(), Some("world://agent/1"));
        assert!(parse_request("不是 JSON").is_err());
        assert!(parse_request(r#"{"body":{}}"#).is_err(), "缺 kind 应被拒");
    }

    #[test]
    fn empty_listeners_is_refused() {
        let d = std::env::temp_dir().join(format!("wc-ch-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("ch.json");
        std::fs::write(&p, r#"{"channel":1,"listeners":[]}"#).unwrap();
        let e = ChannelConfig::load(&p).unwrap_err();
        assert!(e.contains("listeners 为空"), "实得: {e}");
        let _ = std::fs::remove_dir_all(&d);
    }
}
