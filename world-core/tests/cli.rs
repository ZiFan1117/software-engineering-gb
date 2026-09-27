//! **CLI 层测试**（`M04`/`M09` 的入口）。
//!
//! ## 为什么必须有这一层（第 27 轮的教训）
//!
//! 第 26 轮落地 `--require-chain` 后，CLI 一度把**刚写好、带链的账本报告为"无链"**
//! （`load_chain` 里 `self.chained = true;` 漏了）。当时 **57 项测试全绿**——
//! 因为那些用例断言的是"链写出且自洽""篡改被拒"，**从没走过 CLI 一遍**。
//! 抓到那个 bug 的是**一次真实命令行调用**，不是测试套件。
//! 本文件把那一次的调用**固化成会失败的检查**。
//!
//! `WC-UT-001` §三 早就把 `M04` 标为"无模块内单元测试"，这是它欠的那部分。
//!
//! ## 做法
//!
//! 直接用 `CARGO_BIN_EXE_world-core`（Cargo 给集成测试注入的二进制路径），
//! **零新增依赖**，跑的是**真正编译出来的可执行文件**——不是内存里的函数。

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmpdir(tag: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("wc-cli-{tag}-{n}"));
    fs::create_dir_all(&d).unwrap();
    d
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 跑一次 CLI，返回 (退出码, stdout, stderr)。
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_world-core"))
        .args(args)
        .output()
        .expect("无法启动被测二进制");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// 一次 `check` 的常用参数。
fn check_args(d: &std::path::Path) -> Vec<String> {
    vec![
        "--ontology".into(),
        manifest().join("ontology.json").display().to_string(),
        "--policy".into(),
        manifest().join("policy.json").display().to_string(),
        "--ledger".into(),
        d.join("ledger.jsonl").display().to_string(),
        "check".into(),
    ]
}

fn as_refs(v: &[String]) -> Vec<&str> {
    v.iter().map(String::as_str).collect()
}

/// **cli-01**：`check` 在空账本上打印 `READY`，且**如实报告无链**（未校验要说出来）。
#[test]
fn cli01_check_reports_ready_and_chain_status() {
    let d = tmpdir("cli01");
    let args = check_args(&d);
    let (code, out, _) = run(&as_refs(&args));
    assert_eq!(code, 0, "空账本应能打开；stdout={out}");
    assert!(out.contains("READY"), "冒烟判据是打印 READY；stdout={out}");
    assert!(
        out.contains("无摘要链"),
        "空账本无链，必须**明说**不可检出；stdout={out}"
    );
}

/// **cli-02**：写入后 `check` 必须报告**有链**（第 26 轮就是这里错了）。
#[test]
fn cli02_after_append_check_reports_chained() {
    let d = tmpdir("cli02");
    let lp = d.join("ledger.jsonl");
    let base = || -> Vec<String> {
        vec![
            "--ontology".into(),
            manifest().join("ontology.json").display().to_string(),
            "--policy".into(),
            manifest().join("policy.json").display().to_string(),
            "--ledger".into(),
            lp.display().to_string(),
        ]
    };

    let mut a = base();
    a.push("append".into());
    a.push("change".into());
    a.push(r#"{"subject":"world://s","path":"p","before":null,"after":1}"#.into());
    let (code, _, err) = run(&as_refs(&a));
    assert_eq!(code, 0, "append 应成功；stderr={err}");

    let mut c = base();
    c.push("check".into());
    let (code, out, _) = run(&as_refs(&c));
    assert_eq!(code, 0);
    assert!(
        out.contains("有摘要链"),
        "写入侧带链，CLI 必须报告有链（第 26 轮真实踩过恒报无链）；stdout={out}"
    );
}

/// **cli-03**：`--require-chain` 对无链账本**拒绝启动**（rc=2），有链则放行。
#[test]
fn cli03_require_chain_refuses_chainless_ledger() {
    let d = tmpdir("cli03");
    let lp = d.join("legacy.jsonl");
    // 手工造一个合法但**无链**的事件行（模拟 v1 账本）
    fs::write(
        &lp,
        concat!(
            r#"{"world":1,"kind":"notice","id":"x","seq":1,"at":1,"actor":"world://user","#,
            r#""flags":[],"body":{"type":"t","subject":"world://s"}}"#,
            "\n"
        ),
    )
    .unwrap();

    let common = || -> Vec<String> {
        vec![
            "--ontology".into(),
            manifest().join("ontology.json").display().to_string(),
            "--policy".into(),
            manifest().join("policy.json").display().to_string(),
            "--ledger".into(),
            lp.display().to_string(),
        ]
    };

    // 默认：放行（v1 兼容），但必须警示
    let mut a = common();
    a.push("check".into());
    let (code, out, _) = run(&as_refs(&a));
    assert_eq!(code, 0, "无链账本默认应能打开");
    assert!(out.contains("无摘要链"), "必须警示；stdout={out}");

    // --require-chain：拒绝启动，且理由可编程判定（错误码）
    let mut b = common();
    b.push("--require-chain".into());
    b.push("check".into());
    let (code, _, err) = run(&as_refs(&b));
    assert_eq!(code, 2, "应拒绝启动；stderr={err}");
    assert!(
        err.contains("ext.world.Ledger.NoChain"),
        "拒绝理由须带错误码；stderr={err}"
    );
}

/// **cli-04**：用法错误的退出码是 **1**（与"世界有问题"的 2 区分开）。
#[test]
fn cli04_usage_error_exits_one() {
    let d = tmpdir("cli04");
    let args = check_args(&d);
    let mut bad = args[..args.len() - 1].to_vec();
    bad.push("no-such-subcommand".into());
    let (code, _, err) = run(&as_refs(&bad));
    assert_eq!(code, 1, "未知子命令应 rc=1；stderr={err}");

    // `--help` 也应 rc=0
    let (code, out, _) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(out.contains("world-core"), "帮助应打印用法；stdout={out}");
}

/// **cli-07**：`append` 的**缺省身份**必须在用法串里如实地看得见（缺陷台账 **D-43**）。
///
/// ## 这条为什么要单独有（它是一处"从不失败的检查"的反面）
///
/// `cmd_append` 从第 4 个参数取 `actor`、缺省 `world://user`；而 `policy.json` 把该主体列为
/// **唯一**可执行不可逆动作、且授权它写**任意**对象。也就是说：**不写身份 = 最高授权**。
/// 在本次修改之前，用法串里**一个字都没提这个参数**，`--help` 里 `actor` 出现 **0 次**——
/// 于是"最省事的用法"恰好是"后果最大的用法"，而这件事在帮助里是隐形的。
///
/// 本测试把"帮助里必须看得见缺省身份"钉成会失败的断言：
/// 谁再把 `[actor]` 或 `world://user` 从用法串里删掉，**这里就会红**。
#[test]
fn cli07_usage_string_discloses_default_actor() {
    let (code, _out, err) = run(&["append"]);
    assert_eq!(code, 1, "参数不足应 rc=1；err={err}");
    assert!(
        err.contains("[actor]"),
        "用法串必须显示身份这个可选参数；stderr={err}"
    );
    assert!(
        err.contains("world://user"),
        "用法串必须显示缺省身份；stderr={err}"
    );

    let (code, out, _) = run(&["--help"]);
    assert_eq!(code, 0);
    assert!(
        out.contains("append <kind> <json-body> [actor]"),
        "帮助里的 append 行必须带上 [actor]；stdout={out}"
    );
    assert!(
        out.contains("world://user"),
        "帮助必须写明缺省身份是 world://user；stdout={out}"
    );
}

/// **cli-05**：两个投影的**同源核对**在 CLI 上真的跑通（`project check`）。
#[test]
fn cli05_project_check_reports_same_source() {
    let d = tmpdir("cli05");
    let lp = d.join("ledger.jsonl");
    let base = || -> Vec<String> {
        vec![
            "--ontology".into(),
            manifest().join("ontology.json").display().to_string(),
            "--policy".into(),
            manifest().join("policy.json").display().to_string(),
            "--ledger".into(),
            lp.display().to_string(),
        ]
    };
    let mut a = base();
    a.push("append".into());
    a.push("change".into());
    a.push(r#"{"subject":"world://s","path":"p","before":null,"after":true}"#.into());
    assert_eq!(run(&as_refs(&a)).0, 0);

    let mut c = base();
    c.push("project".into());
    c.push("check".into());
    let (code, out, _) = run(&as_refs(&c));
    assert_eq!(code, 0, "同源核对应通过");
    assert!(
        out.contains("同源") && out.contains("✅"),
        "应报告同源一致；stdout={out}"
    );
}

/// **cli-06**：`--require-chain` 与**空账本**的关系（本轮的修正）。
///
/// 修正前：空账本被判"无链" ⇒ `--require-chain` **第一天就不可用**
/// （新部署一启动就被拒）。修正后：**空账本不算违规**（没有东西要保护），
/// 一旦有了**无链**事件才拒绝。
#[test]
fn cli06_require_chain_allows_empty_but_refuses_chainless_data() {
    let d = tmpdir("cli06");
    let lp = d.join("l.jsonl");
    let base = || -> Vec<String> {
        vec![
            "--ontology".into(),
            manifest().join("ontology.json").display().to_string(),
            "--policy".into(),
            manifest().join("policy.json").display().to_string(),
            "--ledger".into(),
            lp.display().to_string(),
            "--require-chain".into(),
            "check".into(),
        ]
    };

    // ① 空账本（文件尚不存在 ⇒ 新建）→ 应放行
    let (code, out, err) = run(&as_refs(&base()));
    assert_eq!(code, 0, "空账本不应被 --require-chain 拒绝；stderr={err}");
    assert!(out.contains("READY"), "stdout={out}");

    // ② 放一条**无链**事件进去 → 应拒绝（理由带错误码）
    let legacy = d.join("legacy.jsonl");
    fs::write(
        &legacy,
        concat!(
            r#"{"world":1,"kind":"notice","id":"x","seq":1,"at":1,"actor":"world://user","#,
            r#""flags":[],"body":{"type":"t","subject":"world://s"}}"#,
            "\n"
        ),
    )
    .unwrap();
    let mut args: Vec<String> = base();
    let pos = args.iter().position(|a| a == "--ledger").unwrap();
    args[pos + 1] = legacy.display().to_string();
    let (code, _, err) = run(&as_refs(&args));
    assert_eq!(code, 2, "有事件但无链 ⇒ 拒绝；stderr={err}");
    assert!(err.contains("NoChain"), "stderr={err}");
}
