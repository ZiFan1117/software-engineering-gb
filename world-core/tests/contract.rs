//! **契约与失败路径测试**（2026-09-26 新增）。
//!
//! 本文件是为 `WC-RV-R2-001` 指出的"**判据零测试却记『已实现』**"而补的：
//! 四个独立评审角色一致发现，本项目的信心高于它的证据——
//! `trace_matrix.py` 只校验用例编号的**格式**，从不校验"这条用例是否真的
//! 验证了这条需求"，于是"已实现"在 `RTM_STRICT=false` 的窗口里
//! 是一个**门禁查不出真假**的字段。
//!
//! 这里补的全是**会失败的检查**：
//! - `c01` `change` 必须过门禁（`REQ-F-015`，修 S-01 后**必须**有回归）
//! - `c02` 信封 8 个必填字段**逐字段**被拦（`REQ-F-002`，此前零覆盖）
//! - `c03` `Policy::load` 的 5 类拒启分支（此前 `Policy::load` 在测试中从未被调用）
//! - `c04` 本体缺失 / 家族为空（`REQ-N-003`，此前只测了"坏 JSON"与"缺版本"）
//! - `c05` 事件 `id` 唯一性（`QG-01` 判定项 ④ 依赖它，此前无任何测试）
//! - `c06` 增量折叠与全量折叠一致（`REQ-F-010` 的判据原本**不可判定**）

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use world_core::{event, World};

fn tmpdir(tag: &str) -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let d = std::env::temp_dir().join(format!("wc-contract-{tag}-{n}"));
    fs::create_dir_all(&d).unwrap();
    d
}

fn ontology() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ontology.json")
}

fn policy() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("policy.json")
}

/// 写一份**合法**的临时策略（含 `writes` 段——默认拒绝要求它必须存在）。
fn write_policy(dir: &Path, name: &str, body: &Value) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, serde_json::to_string_pretty(body).unwrap()).unwrap();
    p
}

fn valid_policy_json() -> Value {
    json!({
        "policy": 1,
        "capabilities": { "notice.mute": { "reversible": true } },
        "subjects": { "allow": ["world://user"] },
        "writes": { "world://user": ["world://*"] }
    })
}

/// **c01**：`change` 必须过门禁——`act` 的效果不能靠 `change` 偷渡。
///
/// 这是 `WC-RV-R2-001` **FIND-01**（安全评审 S-01）的回归测试。
/// 修复前：`change` 只过本体形状校验，于是"被拒绝的动作"可以用一条
/// `change` 静默达成。修复后由 `Policy::authorize_write` 拦截。
#[test]
fn c01_change_is_gated_and_cannot_smuggle_an_act() {
    let d = tmpdir("c01");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    // 主人对自己世界的直接写入：允许
    w.commit(
        "change",
        "world://user",
        event::change_body("world://notice/n-1", "muted", json!(null), json!(false)),
    )
    .unwrap();

    // agent 想用 change 达成"静音"效果：必须被拒
    let err = w
        .commit(
            "change",
            "world://agent/1",
            event::change_body("world://notice/n-1", "muted", json!(false), json!(true)),
        )
        .unwrap_err();
    assert!(err.contains("门禁拒绝写入"), "实得: {err}");

    // 判据 2：状态没有被改
    let s = w.read_model().unwrap();
    assert_eq!(
        s.get("world://notice/n-1", "muted"),
        Some(&json!(false)),
        "被拒的 change 绝不允许改状态"
    );

    // 判据 3：留下可检索的流水
    let all = w.ledger().read_all().unwrap();
    assert_eq!(all.len(), 2, "1 条合法 change + 1 条拒绝流水");
    assert_eq!(all[1]["body"]["type"], json!("gate.write-rejected"));
    assert_eq!(all[1]["body"]["subject"], json!("world://agent/1"));
}

/// **c02**：信封的 8 个必填字段**逐字段**被拦，且报出字段名。
///
/// 此前 `ontology.rs` 的信封必填循环**零覆盖**（该文件单元测试只测 `vocab_hash`），
/// 而 RTM 把 `REQ-F-002` 记为"已实现"并引用了两条不相干的用例。
#[test]
fn c02_every_required_envelope_field_is_enforced() {
    let ont = world_core::ontology::Ontology::load(&ontology()).unwrap();
    let required = ["world", "kind", "id", "seq", "at", "actor", "flags", "body"];

    for field in required {
        let mut ev = event::new_event(
            1,
            "change",
            "world://user",
            event::change_body("world://s", "p", json!(null), json!(1)),
        );
        ev.as_object_mut().unwrap().remove(field);
        let e = ont.validate(&ev).unwrap_err();
        let msg = format!("{e}");
        assert!(
            msg.contains("MissingField") && msg.contains(field),
            "删除 `{field}` 应被拒且指明字段，实得: {msg}"
        );
    }

    // 对照：完整信封必须通过（否则上面的断言可能因"什么都拒"而假通过）
    let ok = event::new_event(
        1,
        "change",
        "world://user",
        event::change_body("world://s", "p", json!(null), json!(1)),
    );
    assert!(ont.validate(&ok).is_ok(), "完整信封不应被拒");

    // 顺带覆盖另两条形状分支：body 不是对象 / 未知家族
    let mut bad_body = ok.clone();
    bad_body["body"] = json!("不是对象");
    assert!(format!("{}", ont.validate(&bad_body).unwrap_err()).contains("MissingField"));

    let unknown = event::new_event(1, "guess", "world://user", json!({}));
    assert!(format!("{}", ont.validate(&unknown).unwrap_err()).contains("UnknownKind"));
}

/// **c03**：`Policy::load` 的 5 类拒启分支（此前该方法在测试中**从未被调用**）。
///
/// 原第 ⑤ 类"可逆与需批准矛盾 ⇒ 拒载"已随 `WC-R4-DISP-001` §三 **E-5**
/// 裁定①（**删 `requires_approval` 字段**）一并删除；未知/多余键改为
/// **被忽略**，其回归见本用例末尾的「对照②」。
#[test]
fn c03_policy_load_rejects_every_malformed_shape() {
    use world_core::gate::{Decision, Policy};
    let d = tmpdir("c03");

    // ① 坏 JSON
    let p = d.join("bad-json.json");
    fs::write(&p, "{ not json").unwrap();
    assert!(Policy::load(&p).is_err(), "坏 JSON 必须被拒");

    // ② 缺 `policy` 版本
    let mut v = valid_policy_json();
    v.as_object_mut().unwrap().remove("policy");
    let p = write_policy(&d, "no-version.json", &v);
    let e = Policy::load(&p).unwrap_err();
    assert!(e.contains("policy"), "实得: {e}");

    // ③ 能力表为空
    let mut v = valid_policy_json();
    v["capabilities"] = json!({});
    let p = write_policy(&d, "empty-caps.json", &v);
    let e = Policy::load(&p).unwrap_err();
    assert!(e.contains("capabilities"), "实得: {e}");

    // ④ 白名单为空
    let mut v = valid_policy_json();
    v["subjects"] = json!({ "allow": [] });
    let p = write_policy(&d, "empty-allow.json", &v);
    let e = Policy::load(&p).unwrap_err();
    assert!(e.contains("subjects.allow"), "实得: {e}");

    // ⑤ 缺 `writes` 段（默认拒绝要求它必须存在）
    let mut v = valid_policy_json();
    v.as_object_mut().unwrap().remove("writes");
    let p = write_policy(&d, "no-writes.json", &v);
    let e = Policy::load(&p).unwrap_err();
    assert!(e.contains("writes"), "实得: {e}");

    // 对照①：合法策略必须能加载（否则"什么都拒"会假通过）
    let p = write_policy(&d, "ok.json", &valid_policy_json());
    assert!(Policy::load(&p).is_ok(), "合法策略不应被拒");

    // 对照②：能力项里的**多余/未知键被忽略**——`weird` 带的是刚被删除的
    // `requires_approval`（旧策略文件的兼容面），`odd` 带的是一个从来就不认识的键。
    //
    // 为何是"应能加载"而不是"应被拒载"（E-5 裁定①"删字段"）：
    // `Policy::load` 用 `serde_json::Value` **手工取值**——`let root: Value =
    // serde_json::from_str(..)` 之后逐键 `spec.get("reversible")`（`src/gate.rs`
    // 「`capabilities` 解析」段）；`gate.rs` 里**没有任何 `#[derive(Deserialize)]`
    // 结构**，也就无从施加 `deny_unknown_fields`（`grep -rn "Deserialize" world-core/src/` = 0 命中）
    // ⇒ 未知键既不导致拒载、也不参与裁决。本用例同时是"旧策略文件带着
    // `requires_approval` 仍能加载"的兼容回归。
    let mut v = valid_policy_json();
    v["capabilities"] = json!({
        "weird": { "reversible": true, "requires_approval": true },
        "odd": { "reversible": true, "totally_unknown_key": 1 }
    });
    let p = write_policy(&d, "extra-keys-ignored.json", &v);
    let pol = Policy::load(&p).expect("多余键应被忽略，而不是拒载");
    // 裁决只看 `reversible` + `irreversible_actors`：多余键不得改变结论
    // （`verb` 可省，`decide` 取不到时按 `-` 处理，见 `src/gate.rs`）。
    let act = json!({ "capability": "weird", "verb": "do" });
    assert_eq!(
        pol.decide("world://user", &act),
        Decision::Allow,
        "多余键不得影响裁决"
    );
}

/// **c04**：本体"文件缺失"与"家族为空"两条拒启分支。
#[test]
fn c04_ontology_load_rejects_missing_file_and_empty_families() {
    use world_core::ontology::Ontology;
    let d = tmpdir("c04");

    // ① 文件不存在
    let missing = d.join("nope.json");
    let e = Ontology::load(&missing).unwrap_err();
    assert!(e.contains("ReadFail"), "实得: {e}");

    // ② 家族为空
    let p = d.join("no-families.json");
    fs::write(
        &p,
        serde_json::to_string(&json!({
            "world": 1,
            "envelope": { "required": ["world"], "optional": [] },
            "families": {}
        }))
        .unwrap(),
    )
    .unwrap();
    let e = Ontology::load(&p).unwrap_err();
    assert!(e.contains("NoFamilies"), "实得: {e}");
}

/// **c05**：事件 `id` 唯一性（`WC-SQAP-001` 的 M-01 判定项 ④ 依赖它）。
///
/// `new_id()` = 纳秒 + 进程内计数器；计数器每次启动从 0 开始，
/// 故"跨重启唯一"依赖纳秒不重复——本测试覆盖**同进程内**的唯一性。
#[test]
fn c05_event_ids_are_unique_within_a_process() {
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..2000u64 {
        let ev = event::new_event(
            i + 1,
            "change",
            "world://user",
            event::change_body("world://s", "p", json!(null), json!(i)),
        );
        let id = ev["id"].as_str().unwrap().to_string();
        assert!(seen.insert(id.clone()), "事件 id 重复：{id}");
    }
    assert_eq!(seen.len(), 2000);
}

/// **c06**：**增量折叠**与**全量折叠**结果一致（`REQ-F-010` 的判据）。
///
/// 此前该判据**不可判定**：判据要求"重算与增量折叠一致"，
/// 而代码没有增量路径（`read_model()` 每次全量 fold），
/// 被比较的一方不存在 ⇒ 该判据永远无法失败（`WC-RV-R2-001` FIND-10）。
/// 本测试用 `State::apply` 逐条喂入构造出真正的增量路径，
/// 于是"增量 == 全量"成为**可失败**的断言。
#[test]
fn c06_incremental_apply_equals_full_fold() {
    use world_core::readmodel::State;

    let d = tmpdir("c06");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        for i in 0..5 {
            w.commit(
                "change",
                "world://user",
                event::change_body(
                    &format!("world://notice/n-{i}"),
                    "muted",
                    json!(null),
                    json!(i % 2 == 0),
                ),
            )
            .unwrap();
        }
        w.commit(
            "act",
            "world://user",
            event::act_body("notice.mute", "do", "r-c06", json!({})),
        )
        .unwrap();
    }

    let w = World::open(&ontology(), &lp, &policy()).unwrap();
    let all = w.ledger().read_all().unwrap();

    // 全量：一次 fold
    let full = State::fold(&all).unwrap();

    // 增量：逐条 apply（这才是"边写边算"的真实形态）
    let mut inc = State::new();
    for ev in &all {
        inc.apply(ev).unwrap();
    }

    assert_eq!(
        inc.to_json().to_string(),
        full.to_json().to_string(),
        "增量折叠与全量折叠必须逐字节一致"
    );
    assert_eq!(inc.digest(), full.digest());

    // 反假测试：只喂前缀必须得到不同结果（否则两者相同可能只是"都没算"）
    let mut prefix = State::new();
    for ev in &all[..2] {
        prefix.apply(ev).unwrap();
    }
    assert_ne!(
        prefix.to_json().to_string(),
        full.to_json().to_string(),
        "前缀折叠竟与全量相同 ⇒ 这个断言没有在真的比什么"
    );
}

/// **c07**：**同一本账本不能有两个写者**（`WC-RV-R2-001` FIND-08 / 假设 `A-01`）。
///
/// 为什么这条是 P0 级：CLI 每条命令都是新进程，而 `next_seq` 只在内存里。
/// 两个写者各自取号 ⇒ 重号或半写交错 ⇒ 下次启动 `SeqGap` 拒启、**世界锁死**。
#[test]
fn c07_second_writer_is_refused() {
    let d = tmpdir("c07");
    let lp = d.join("ledger.jsonl");

    let first = World::open(&ontology(), &lp, &policy()).expect("第一个写者应能打开");
    let err = World::open(&ontology(), &lp, &policy()).expect_err("第二个写者必须被拒绝");
    assert!(err.contains("Ledger.Locked"), "实得: {err}");
    assert!(err.contains("单写者"), "理由里要写清为什么：{err}");

    // 释放第一个写者（Drop 删锁）后，第二个必须能拿到
    drop(first);
    assert!(
        World::open(&ontology(), &lp, &policy()).is_ok(),
        "锁随 Drop 释放后应能重新打开"
    );
}

/// **c08**：**陈旧锁可自动回收**（崩溃遗留的锁不能把世界永久锁死）。
#[test]
fn c08_stale_lock_is_reclaimed() {
    let d = tmpdir("c08");
    let lp = d.join("ledger.jsonl");
    let lock = lp.with_extension("lock");

    // 造一个"持有者已不存在"的锁：pid 用一个几乎不可能存在的值
    fs::write(&lock, "999999999\n").unwrap();
    assert!(lock.exists());

    let w = World::open(&ontology(), &lp, &policy()).expect("陈旧锁应被回收，不应永久阻塞");
    assert_eq!(w.ledger().last_seq(), 0);
    // 回收后锁文件由新持有者重建（内容是本进程 pid）
    let content = fs::read_to_string(&lock).unwrap_or_default();
    assert_eq!(
        content.trim(),
        std::process::id().to_string(),
        "锁文件应记录当前持有者 pid"
    );
}

// ────────────────── 静态墙加固（round 11：FIND-04 / A-05）──────────────────

/// **c09**：法律或真相是**符号链接**时拒绝启动（`FIND-04` / `S-05`）。
///
/// 为什么：`fs::metadata` 跟随链接 ⇒"检查的"与"真正读的"可能不是同一个文件，
/// 静态墙被绕过。
#[cfg(unix)]
#[test]
fn c09_symlinked_law_is_refused() {
    use std::os::unix::fs::symlink;
    let d = tmpdir("c09");
    let lp = d.join("ledger.jsonl");

    // 真本体放在别处，policy.json 用软链指向它
    let real = d.join("real-policy.json");
    fs::copy(policy(), &real).unwrap();
    let link = d.join("policy.json");
    symlink(&real, &link).unwrap();

    let err = World::open(&ontology(), &lp, &link).expect_err("指向别处的策略软链必须被拒绝");
    assert!(err.contains("符号链接"), "实得: {err}");

    // 对照：真实文件不得被拒（否则上面的断言可能因"什么都拒"而假通过）
    let copy = d.join("policy-real.json");
    fs::copy(policy(), &copy).unwrap();
    assert!(
        World::open(&ontology(), &lp, &copy).is_ok(),
        "真实文件不应被拒"
    );
}

/// **c10**：**属主断言**（假设 `A-05`：静态墙只看 mode，属主必须另行断言）。
///
/// 它是**部署方的自证工具**，不是自动安全机制——没人传 `expected_uid` 时不会运行。
#[cfg(unix)]
#[test]
fn c10_owner_assertion_detects_wrong_owner() {
    use std::os::unix::fs::MetadataExt;
    use world_core::guard;

    let d = tmpdir("c10");
    let f = d.join("law.json");
    fs::copy(policy(), &f).unwrap();

    let my_uid = fs::metadata(&f).unwrap().uid();
    assert!(
        guard::assert_owned_by(&f, my_uid, "法律").is_ok(),
        "属主相符应通过"
    );

    let wrong = my_uid.wrapping_add(1);
    let err = guard::assert_owned_by(&f, wrong, "法律").expect_err("属主不符必须被拒");
    assert!(err.contains("属主断言未通过"), "实得: {err}");
    assert!(
        err.contains("属主永远能"),
        "理由要说清为什么属主重要：{err}"
    );
}

/// **c11**：不可逆动作有**治理出口**，且措辞不骗人（`FIND-12` / `DEBT-07`）。
///
/// 此前 `!reversible` 一律 `AwaitApproval`，而 `commit` 收到即 `Err`，
/// 仓内没有批准命令/批准事件/消费路径 ⇒ 不可逆能力在 v1 **永远无法执行**。
/// v1 的明确规则：**只允许 `irreversible_actors` 白名单里的主体执行**；
/// 其他主体收到的理由必须**明说"没有审批通道，不要等批准"**。
#[test]
fn c11_irreversible_is_owner_only_and_the_refusal_does_not_lie() {
    let d = tmpdir("c11");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    // ① 白名单主体（世界的主人）可以执行不可逆动作 —— 它不是死号
    let ok = w
        .commit(
            "act",
            "world://user",
            event::act_body("ledger.compact", "do", "r-c11a", json!({})),
        )
        .expect("主人应当能执行不可逆动作（否则该能力是死号）");
    assert_eq!(ok["kind"], json!("act"));

    // ② agent 拿到摩擦，且理由里**明说 v1 没有审批通道**
    let err = w
        .commit(
            "act",
            "world://agent/1",
            event::act_body("ledger.compact", "do", "r-c11b", json!({})),
        )
        .expect_err("agent 不得执行不可逆动作");
    assert!(err.contains("不可逆"), "实得: {err}");
    assert!(
        err.contains("没有审批通道") && err.contains("不要等批准"),
        "理由必须说清 v1 无审批通道，否则会让人以为等等就能批：{err}"
    );

    // ③ 仍然留痕（拦得住也记得下）
    let all = w.ledger().read_all().unwrap();
    assert_eq!(all.len(), 2, "1 条 act + 1 条加摩擦流水");
    assert_eq!(all[1]["body"]["type"], json!("gate.awaiting-approval"));
}

// ────────────────── M08 检查点（round 15：REQ-F-021）──────────────────

/// **c12**：快照是**缓存**——删掉它必须没有任何后果。
///
/// 四条判据一起成立，才叫"缓存"而不是"第二真相"：
/// 1. **续算 == 全量**：有快照的快路径与无快照的全量折叠**逐字节相同**；
/// 2. **删掉无后果**：把快照文件删掉再算，结果仍相同（第三条专属测试的加强形态）；
/// 3. **必须带 `base_seq`**：快照声明折叠到哪一条为止；
/// 4. **可被核验**：`verify` 拿账本重算，指纹一致才通过。
#[test]
fn c12_checkpoint_is_a_cache_and_disposable() {
    use world_core::checkpoint::{read_model_with_checkpoint, Checkpoint};

    let d = tmpdir("c12");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        for i in 0..4 {
            w.commit(
                "change",
                "world://user",
                event::change_body(
                    &format!("world://notice/n-{i}"),
                    "muted",
                    json!(null),
                    json!(i % 2 == 0),
                ),
            )
            .unwrap();
        }
        w.commit(
            "act",
            "world://user",
            event::act_body("notice.mute", "do", "r-c12", json!({})),
        )
        .unwrap();
    }

    let w = World::open(&ontology(), &lp, &policy()).unwrap();
    let all = w.ledger().read_all().unwrap();
    let full = read_model_with_checkpoint(&all, None).unwrap();

    // 在第 2 条处截一张快照
    let prefix = world_core::readmodel::State::fold(&all[..2]).unwrap();
    let cp = Checkpoint::capture(&prefix);
    assert_eq!(cp.base_seq(), 2, "快照必须声明它折叠到哪一条");

    // 判据 1：续算 == 全量（逐字节）
    let resumed = read_model_with_checkpoint(&all, Some(&cp)).unwrap();
    assert_eq!(
        resumed.to_json().to_string(),
        full.to_json().to_string(),
        "有快照的续算必须与全量折叠逐字节相同"
    );

    // 判据 4：核验通过
    cp.verify(&all).expect("快照应当能通过账本核验");

    // 判据 2：落盘 → 重新载入 → 结果不变；**删掉文件**再算仍不变
    let cp_path = d.join("checkpoint.json");
    cp.write(&cp_path).unwrap();
    let reloaded = Checkpoint::load(&cp_path).unwrap();
    assert_eq!(reloaded.base_seq(), 2);
    assert_eq!(reloaded.digest(), cp.digest());
    let from_disk = read_model_with_checkpoint(&all, Some(&reloaded)).unwrap();
    assert_eq!(from_disk.to_json().to_string(), full.to_json().to_string());

    fs::remove_file(&cp_path).unwrap();
    assert!(!cp_path.exists());
    let after_delete = read_model_with_checkpoint(&all, None).unwrap();
    assert_eq!(
        after_delete.to_json().to_string(),
        full.to_json().to_string(),
        "删掉快照后重算必须与有快照时一致——否则快照就是第二真相"
    );
}

/// **c13**：坏快照必须被**拒绝使用**，而不是被默默吞下。
///
/// 三类坏法：① 指纹被篡改（缓存与账本不符）② `base_seq` 超过账本长度（陈旧/伪造）
/// ③ 文件不是合法 JSON。
#[test]
fn c13_bad_checkpoints_are_refused() {
    use world_core::checkpoint::Checkpoint;

    let d = tmpdir("c13");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        w.commit(
            "change",
            "world://user",
            event::change_body("world://notice/n-1", "muted", json!(null), json!(true)),
        )
        .unwrap();
    }
    let w = World::open(&ontology(), &lp, &policy()).unwrap();
    let all = w.ledger().read_all().unwrap();

    // ① 篡改指纹
    let mut doc: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&json!({
            "checkpoint": 1,
            "base_seq": 1,
            "digest": "fnv1a64:0000000000000000",
            "state": world_core::readmodel::State::fold(&all).unwrap().to_json(),
        }))
        .unwrap(),
    )
    .unwrap();
    let tampered_path = d.join("tampered.json");
    fs::write(&tampered_path, serde_json::to_string(&doc).unwrap()).unwrap();
    let bad = Checkpoint::load(&tampered_path).unwrap();
    let e = bad.verify(&all).unwrap_err();
    assert!(e.contains("DigestMismatch"), "实得: {e}");
    assert!(e.contains("以账本为准"), "理由必须说清谁说了算：{e}");

    // ② base_seq 超过账本长度
    doc["base_seq"] = json!(99);
    doc["digest"] = json!("fnv1a64:1111111111111111");
    let stale_path = d.join("stale.json");
    fs::write(&stale_path, serde_json::to_string(&doc).unwrap()).unwrap();
    let stale = Checkpoint::load(&stale_path).unwrap();
    let e = stale.verify(&all).unwrap_err();
    assert!(e.contains("Stale"), "实得: {e}");

    // ③ 不是合法 JSON / 缺 base_seq
    let junk = d.join("junk.json");
    fs::write(&junk, "{ not json").unwrap();
    assert!(Checkpoint::load(&junk).is_err());
    let nobase = d.join("nobase.json");
    fs::write(
        &nobase,
        json!({"checkpoint":1,"digest":"x","state":{}}).to_string(),
    )
    .unwrap();
    let e = Checkpoint::load(&nobase).unwrap_err();
    assert!(e.contains("base_seq"), "实得: {e}");
}

// ────────────────── 项目 Step 6 通道（round 16：IF-006 / FIND-06）──────────────────

/// **c14**：通道的身份来自**内核**，请求自称无效。
///
/// 三条判据：
/// 1. 同 uid 连接 → 落笔成功，且事件的 `actor` 取自**套接字映射**（不是请求里的字符串）；
/// 2. 请求**自称**别的 actor → 拒绝，且**没有落笔**；
/// 3. 请求未声明 actor → 允许（身份本就由内核给出，无需自称）。
#[cfg(unix)]
#[test]
fn c14_channel_takes_identity_from_kernel_not_from_request() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use world_core::channel::{serve_once, Listener};

    let d = tmpdir("c14");
    let lp = d.join("ledger.jsonl");
    let sock = d.join("w.sock");
    let me = libc_uid();

    let expect = Listener {
        socket: sock.clone(),
        actor: "world://agent/1".to_string(),
        uid: me,
    };
    let listener = UnixListener::bind(&sock).unwrap();

    // 判据 1：同 uid（本进程）→ 成功，且 actor 取自映射
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
    let mut c = UnixStream::connect(&sock).unwrap();
    c.write_all(
        br#"{"kind":"act","body":{"capability":"notice.mute","verb":"do","request_id":"r-c14"}}"#,
    )
    .unwrap();
    c.write_all(b"\n").unwrap();
    let ev = serve_once(&mut w, &listener, &expect).unwrap();
    assert_eq!(
        ev["actor"],
        json!("world://agent/1"),
        "actor 必须取自内核身份映射，而不是请求"
    );
    // 读回应答
    let mut reply = String::new();
    BufReader::new(&c).read_line(&mut reply).unwrap();
    assert!(reply.contains("\"ok\":true"), "应答: {reply}");

    // 判据 2：自称 world://user（冒充最高权主体）→ 拒绝且不落笔
    let before = w.ledger().last_seq();
    let mut c2 = UnixStream::connect(&sock).unwrap();
    c2.write_all(br#"{"kind":"act","actor":"world://user","body":{"capability":"notice.mute","verb":"do","request_id":"r-c14b"}}"#).unwrap();
    c2.write_all(b"\n").unwrap();
    let e = serve_once(&mut w, &listener, &expect).expect_err("冒充必须被拒");
    assert!(e.contains("Impersonation"), "实得: {e}");
    assert_eq!(w.ledger().last_seq(), before, "冒充被拒后不得落笔");

    // 判据 3：未自称 → 允许
    let mut c3 = UnixStream::connect(&sock).unwrap();
    c3.write_all(
        br#"{"kind":"act","body":{"capability":"notice.mute","verb":"do","request_id":"r-c14c"}}"#,
    )
    .unwrap();
    c3.write_all(b"\n").unwrap();
    serve_once(&mut w, &listener, &expect).expect("未自称 actor 应当被允许");
}

/// 取本进程 uid（不引 libc：用 /proc/self/status）。
#[cfg(unix)]
fn libc_uid() -> u32 {
    let s = fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("Uid:") {
            if let Some(first) = rest.split_whitespace().next() {
                return first.parse().unwrap_or(0);
            }
        }
    }
    0
}

// ────────────────── 错误码契约（round 18：DEBT-01）──────────────────

/// **c15**：凡**可编程判定**的错误都必须带 `ext.world.<域>.<原因>` 前缀。
///
/// 为什么这条测试才是 `DEBT-01` 的实质：光有 `code_of()` 只是"能解析"，
/// **有人新加一个不带码的错误**照样能过。本测试逐条走真实失败路径，
/// 把"错误码契约"从**声明**变成**会失败的检查**。
#[test]
fn c15_errors_carry_machine_readable_codes() {
    use world_core::error::{code_of, has_code};

    let d = tmpdir("c15");
    let lp = d.join("ledger.jsonl");
    let mut codes: Vec<String> = Vec::new();

    // ① 本体错误
    let bad = d.join("bad.json");
    fs::write(&bad, "{ not json").unwrap();
    codes.push(World::open(&bad, &lp, &policy()).unwrap_err());

    // ② 门禁：未声明的能力
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
    codes.push(
        w.commit(
            "act",
            "world://agent/1",
            event::act_body("world.hack", "do", "r-c15a", json!({})),
        )
        .unwrap_err(),
    );

    // ③ 门禁：未授权的写入（change）
    codes.push(
        w.commit(
            "change",
            "world://agent/1",
            event::change_body("world://s", "p", json!(null), json!(1)),
        )
        .unwrap_err(),
    );

    // ④ 门禁：不可逆加摩擦
    codes.push(
        w.commit(
            "act",
            "world://agent/1",
            event::act_body("ledger.compact", "do", "r-c15b", json!({})),
        )
        .unwrap_err(),
    );

    // ⑤ 法律：违反本体的信纸
    codes.push(
        w.commit(
            "change",
            "world://user",
            json!({"subject": "world://s", "path": "p"}),
        )
        .unwrap_err(),
    );

    // ⑥ 通道：坏请求
    codes.push(world_core::channel::parse_request("不是 JSON").unwrap_err());

    // ⑦ 读模型：坏账本
    let gap = vec![event::new_event(
        5,
        "change",
        "world://user",
        event::change_body("world://s", "p", json!(null), json!(1)),
    )];
    codes.push(world_core::readmodel::State::fold(&gap).unwrap_err());

    // 全部必须带码
    for c in &codes {
        assert!(has_code(c), "错误缺少 `ext.world.<域>.<原因>` 前缀：{c}");
    }
    // 码互不相同（否则"判定种类"这件事名不副实）
    let unique: std::collections::BTreeSet<&str> =
        codes.iter().filter_map(|c| code_of(c)).collect();
    assert!(unique.len() >= 6, "错误码区分度不足，只拿到 {unique:?}");

    // 反例：散文式错误**必须**判为不符合契约（防契约被悄悄放宽）
    assert_eq!(code_of("门禁拒绝：能力未声明"), None);
}

// ────────────────── 摘要链验证侧（round 23：WC-CR-003 的验证部分）──────────────────

/// 造一串**带链**的事件（手工构造，因为**写入侧尚未实现**——`WC-CR-003` 待批准）。
fn chained(events: &mut [serde_json::Value]) {
    use world_core::ledger::{event_chain, CHAIN_GENESIS};
    let mut prev = CHAIN_GENESIS.to_string();
    for ev in events.iter_mut() {
        let c = event_chain(&prev, ev).unwrap();
        ev.as_object_mut()
            .unwrap()
            .insert("chain".to_string(), json!(c));
        prev = ev["chain"].as_str().unwrap().to_string();
    }
}

fn three_events() -> Vec<serde_json::Value> {
    vec![
        event::new_event(
            1,
            "change",
            "world://user",
            event::change_body("world://s", "p", json!(null), json!(1)),
        ),
        event::new_event(
            2,
            "change",
            "world://user",
            event::change_body("world://s", "p", json!(1), json!(2)),
        ),
        event::new_event(
            3,
            "act",
            "world://user",
            event::act_body("notice.mute", "do", "r-c", json!({})),
        ),
    ]
}

/// **c16**：改动 / 重排 / 中间插入 —— 链**必须**检出（`WC-CR-003` §三 的上半）。
#[test]
fn c16_chain_detects_local_tampering() {
    use world_core::ledger::verify_chain;

    // 基线：合法链必须通过（否则下面的断言可能因"什么都拒"而假通过）
    let mut ok = three_events();
    chained(&mut ok);
    verify_chain(&ok).expect("合法链应通过");

    // ① 改中间某条的内容（chain 不动）
    let mut tampered = ok.clone();
    tampered[1]["body"]["after"] = json!(999);
    let e = verify_chain(&tampered).unwrap_err();
    assert!(e.contains("ChainMismatch"), "实得: {e}");

    // ② 重排相邻两条
    let mut reordered = ok.clone();
    reordered.swap(1, 2);
    assert!(verify_chain(&reordered)
        .unwrap_err()
        .contains("ChainMismatch"));

    // ③ 在中间插入一条（带一个"看似合理"的 chain）
    //  ⚠️ 注入的事件必须带一个"攻击者为该位置算好的" chain——否则它只是"混用"，
    //     测的是 MixedChain 而**不是**插入检测（第一版就是这么写错的，被自己抓到）。
    let mut inserted = ok.clone();
    let mut extra = event::new_event(
        99,
        "act",
        "world://user",
        event::act_body("notice.mute", "do", "r-injected", json!({})),
    );
    let prev_chain = ok[1]["chain"].as_str().unwrap().to_string();
    let injected_chain = world_core::ledger::event_chain(&prev_chain, &extra).unwrap();
    extra
        .as_object_mut()
        .unwrap()
        .insert("chain".to_string(), json!(injected_chain));
    inserted.insert(2, extra);
    assert!(verify_chain(&inserted)
        .unwrap_err()
        .contains("ChainMismatch"));
}

/// **c17**：无链 / 混用 —— 必须被**显式区分**，不得静默通过（`WC-CR-003` D2）。
#[test]
fn c17_chain_distinguishes_absent_and_mixed() {
    use world_core::ledger::verify_chain;

    let plain = three_events();
    let e = verify_chain(&plain).unwrap_err();
    assert!(e.contains("NoChain"), "无链要说清是「不可检出」，实得: {e}");
    assert!(e.contains("不可检出"), "实得: {e}");

    let mut mixed = three_events();
    chained(&mut mixed);
    mixed[2].as_object_mut().unwrap().remove("chain");
    let e = verify_chain(&mixed).unwrap_err();
    assert!(e.contains("MixedChain"), "实得: {e}");
    assert!(e.contains("比无链更危险"), "理由要说清为什么：{e}");
}

/// **c18 = `WC-CR-003` 的 `c19`**：**整文件重写（连链重算）→ 按设计"检不出"**。
///
/// 这条测试的存在本身是设计的一部分：**一个只证明自己有效的机制，
/// 不如一个同时证明自己边界在哪里的机制**。若将来有人以为链能防住全知攻击者，
/// 这条测试会告诉他不能。
#[test]
fn c18_chain_cannot_detect_a_full_rewrite() {
    use world_core::ledger::verify_chain;

    // 攻击者把第 2 条改成 999，然后**把链整条重算**（他知道算法与创世种子）
    let mut forged = three_events();
    forged[1]["body"]["after"] = json!(999);
    chained(&mut forged);

    // 结论：核验**通过** —— 这就是无密钥链的极限，不是实现的缺陷
    verify_chain(&forged)
        .expect("无密钥的链挡不住整文件重写；此断言在证明**边界**，不是在证明实现有 bug");
    // 而局部篡改（不重算链）仍会被检出——两句话一起才完整
    let mut local = three_events();
    chained(&mut local);
    local[1]["body"]["after"] = json!(999);
    assert!(verify_chain(&local).is_err(), "局部篡改必须仍被检出");
}

// ────────────────── 摘要链写入侧（round 25：WC-CR-003 实施）──────────────────

/// **c19**：写出的账本**每条都带链**，且链自洽。
#[test]
fn c19_written_ledger_carries_a_verifiable_chain() {
    let d = tmpdir("c19");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        for i in 0..3 {
            w.commit(
                "change",
                "world://user",
                event::change_body(&format!("world://s{i}"), "p", json!(null), json!(i)),
            )
            .unwrap();
        }
    }
    let w = World::open(&ontology(), &lp, &policy()).unwrap();
    let all = w.ledger().read_all().unwrap();
    assert_eq!(all.len(), 3);
    assert!(
        all.iter().all(|e| e.get("chain").is_some()),
        "写入侧应给每条事件带上 chain"
    );
    world_core::ledger::verify_chain(&all).expect("写出的链必须自洽");
    // 链不影响状态：读模型不认 chain，指纹与"无链时代"一致
    assert_eq!(w.read_model().unwrap().last_seq(), 3);
}

/// **c20**：链的核验**接在启动路径上**——篡改一行即**拒绝启动**。
///
/// 这条才是"链有用"的证据：`c16`–`c18` 只证明核验函数会判，
/// 本用例证明**世界真的会因此拒绝打开**。
#[test]
fn c20_startup_refuses_a_tampered_ledger() {
    let d = tmpdir("c20");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        for i in 0..3 {
            w.commit(
                "change",
                "world://user",
                event::change_body(&format!("world://s{i}"), "p", json!(null), json!(i)),
            )
            .unwrap();
        }
    }
    // 篡改中间一行（保留原 chain）：改一个值，其余一切不动
    let text = fs::read_to_string(&lp).unwrap();
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut v: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
    v["body"]["after"] = json!(999);
    lines[1] = v.to_string();
    fs::write(&lp, format!("{}\n", lines.join("\n"))).unwrap();

    let err = World::open(&ontology(), &lp, &policy()).expect_err("被篡改的账本必须拒绝启动");
    assert!(err.contains("ChainMismatch"), "实得: {err}");
    assert!(err.contains("唯一真相"), "理由要说清为什么拒绝：{err}");
}

// ────────────────── 链状态判定（round 27：把 round 26 的 bug 变成会失败的检查）──────────────────

/// **c21**：`is_chained()` 必须**如实反映账本现实**。
///
/// 为什么补这条：第 26 轮 CLI 实测发现"刚写好、带链的账本被报告为无链"——
/// 根因是 `load_chain` 里 `self.chained = true;` 漏了。
/// 当时 **57 项测试全绿**，因为 `c19`/`c20` 断言的是"链写出且自洽""篡改被拒"，
/// **从没断言过状态位本身**。本用例把这个洞堵上。
#[test]
fn c21_is_chained_reflects_reality() {
    // ① 新账本：写入即带链 ⇒ 启动判定必须为 true
    let d = tmpdir("c21");
    let lp = d.join("ledger.jsonl");
    {
        let mut w = World::open(&ontology(), &lp, &policy()).unwrap();
        w.commit(
            "change",
            "world://user",
            event::change_body("world://s", "p", json!(null), json!(1)),
        )
        .unwrap();
    }
    let w = World::open(&ontology(), &lp, &policy()).unwrap();
    assert!(
        w.ledger().is_chained(),
        "账本已带链，is_chained() 却为 false —— 状态位没被置上（第 26 轮真实踩过）"
    );

    // ② 空账本（没有任何事件）⇒ 无链可言，应为 false
    let empty = d.join("empty.jsonl");
    {
        let _w = World::open(&ontology(), &empty, &policy()).unwrap();
    }
    let w2 = World::open(&ontology(), &empty, &policy()).unwrap();
    assert!(
        !w2.ledger().is_chained(),
        "空账本不应声称有链（那会让人以为它受保护）"
    );

    // ③ 手工造的无链账本 ⇒ false，且**能打开**（v1 兼容：无链须放行但可被警示）
    let legacy = d.join("legacy.jsonl");
    let ev = event::new_event(
        1,
        "notice",
        "world://user",
        event::notice_body("t", "world://s", json!({})),
    );
    fs::write(
        &legacy,
        format!("{}\n", serde_json::to_string(&ev).unwrap()),
    )
    .unwrap();
    let w3 = World::open(&ontology(), &legacy, &policy()).unwrap();
    assert!(
        !w3.ledger().is_chained(),
        "无链账本必须被判为 false（未校验要说出来）"
    );
    assert_eq!(w3.ledger().last_seq(), 1, "无链账本仍应能打开（v1 兼容）");
}

// ────────────────── 行边界不变量（round 31：为 FIND-03 的前提上锁）──────────────────

/// **c22**：**每次成功追加后，文件必须以 `\n` 结尾**，且每行都是完整 JSON。
///
/// 为什么单独立一条：`Ledger::append` 的整套加固（写入前记长度、失败回滚、
/// 回滚再失败则标记污染）**全部建立在同一个前提上**——
/// "**文件末尾永远是完整行**"。前提若不成立：
/// ① 半行会与下一条成功写入**粘连**成坏行；② 重启读到坏 JSON 会**拒绝启动**（整世界锁死）；
/// ③ 若粘连处含此前已 ack 的事件，启动"截到最后一个 `\n`"会**静默删掉它们**。
///
/// 纯粹的 I/O 失败注入（ENOSPC/EINTR）本项目**尚无工具**（`WC-TP-001` §五 缺口 2，
/// 照实登记）。本用例守的是那条路径**可被检测**的前提——前提没了，加固就无从谈起。
#[test]
fn c22_file_always_ends_on_a_line_boundary() {
    let d = tmpdir("c22");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    for i in 0..5u64 {
        w.commit(
            "change",
            "world://user",
            event::change_body(&format!("world://s{i}"), "p", json!(null), json!(i)),
        )
        .unwrap();

        // 判据 ①：末尾必须是换行
        let raw = fs::read(&lp).unwrap();
        assert_eq!(
            raw.last().copied(),
            Some(b'\n'),
            "第 {} 次追加后文件未以 \\n 结尾——末尾可能残留半行",
            i + 1
        );

        // 判据 ②：每一行都必须是完整 JSON（粘连会立刻表现为某行解析失败）
        let text = String::from_utf8(raw).expect("账本必须是合法 UTF-8");
        for (n, line) in text.lines().enumerate() {
            serde_json::from_str::<serde_json::Value>(line)
                .unwrap_or_else(|e| panic!("第 {} 行不是完整 JSON（行粘连？）：{e}", n + 1));
        }
    }

    // 判据 ③：条数与追加次数一致（没有静默丢失）
    // ⚠️ 必须先释放第一个写者——否则会被**单写者锁**挡下（第一版就是这样写错的，
    //    而它恰好证明了那把锁是真的在工作）。
    drop(w);
    let w2 = World::open(&ontology(), &lp, &policy()).unwrap();
    assert_eq!(w2.ledger().read_all().unwrap().len(), 5);
    assert_eq!(w2.ledger().last_seq(), 5);
}

// ═══════════════════════════════════════════════════════════════════════════
// c23 —— **通告也必须过闸**（`D-13` / `D-14`；专家席 C 发现，实测复现后修）
//
// 修前的病：`adjudicate` 的 `_ => Ok(())` 让 `notice` 族完全不过闸 ⇒
// 任何主体可往账本写任意通告，**包括与内核真流水逐字同形的伪造 `gate.rejected`**
// ⇒ 整套审计可被一行伪造。实测复现：不在白名单的 `world://stranger` 提交
// `type=gate.rejected` 得到 rc=0 且进账本，而同一主体写 `change` 得 rc=2。
// ═══════════════════════════════════════════════════════════════════════════

/// 反例 ①：**保留前缀**不许外部主体写。
///
/// 为什么这条最要紧：`gate.*` 是"世界对某次请求的裁决"的流水。若外部可写，
/// 则任何人都能**替世界说"我拒绝过这件事"**，而伪造行与真行逐字同形、事后不可区分。
#[test]
fn c23_notice_with_reserved_prefix_is_refused_for_outsiders() {
    let d = tmpdir("c23a");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    let e = w.commit(
        "notice",
        "world://stranger",
        event::notice_body("gate.rejected", "world://user", json!({ "reason": "伪造" })),
    );
    let msg = e.expect_err("保留前缀 `gate.` 必须拒绝外部主体");
    assert!(
        msg.contains("ext.world.Gate.NoticeNotAllowed"),
        "拒绝理由必须点名错误码，实得：{msg}"
    );
    assert!(msg.contains("内核保留前缀"), "理由必须说清为什么：{msg}");

    // 判据②：**被拒的伪造通告不得落笔**（拒绝也要留痕，但留的是内核对这次拒绝的流水）
    let evs = w.ledger().read_all().unwrap();
    let forged = evs
        .iter()
        .filter(|ev| ev["body"]["type"] == json!("gate.rejected"))
        .count();
    assert_eq!(forged, 0, "伪造的 gate.rejected 绝不允许进账本");
}

/// 反例 ②：**不在册的主体**不许写普通通告。
#[test]
fn c23_notice_from_unlisted_actor_is_refused() {
    let d = tmpdir("c23b");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    let e = w.commit(
        "notice",
        "world://stranger",
        event::notice_body("my.own.notice", "world://user", json!({})),
    );
    let msg = e.expect_err("不在白名单的主体不得写通告");
    assert!(msg.contains("ext.world.Gate.NoticeRejected"), "{msg}");

    // 正例对照：在册主体写同样形状的通告必须通过（否则修法把正常路径打死了）
    w.commit(
        "notice",
        "world://agent/1",
        event::notice_body("my.own.notice", "world://job/1", json!({})),
    )
    .expect("在册主体写普通通告应当通过");
}

/// `D-14`：门禁流水必须点名"它在拒绝什么"。
///
/// 只保留前缀是不够的——那解决了"谁能写"，没解决"写的是什么"：
/// 外部提交被拒时的流水与内核自己发起的裁决流水**类型相同**，
/// 读账本的人分不清"内核在裁别人"还是"别人的尝试被裁了"。
/// 修法：流水 payload 带 `refused`（被拒对象的规范形式指纹）。
#[test]
fn c23_gate_notice_says_what_it_refused() {
    let d = tmpdir("c23c");
    let lp = d.join("ledger.jsonl");
    let mut w = World::open(&ontology(), &lp, &policy()).unwrap();

    // 触发一条真实的内核裁决流水：agent 请求不可逆动作 ⇒ 加摩擦
    let _ = w.commit(
        "act",
        "world://agent/1",
        event::act_body("ledger.compact", "do", "r-9", json!({})),
    );

    let evs = w.ledger().read_all().unwrap();
    let gate = evs
        .iter()
        .find(|ev| ev["body"]["type"] == json!("gate.awaiting-approval"))
        .expect("应当有一条门禁流水");
    let refused = gate["body"]["payload"]["refused"]
        .as_str()
        .expect("流水必须带 `refused` 字段（D-14）");
    assert!(
        refused.starts_with("fnv1a64:"),
        "`refused` 必须是可复算的规范形式指纹，实得：{refused}"
    );

    // 可复算：同一 actor + 同一信纸 ⇒ 同一指纹（跨进程、跨时间都一样）
    let again = w.commit(
        "act",
        "world://agent/1",
        event::act_body("ledger.compact", "do", "r-9", json!({})),
    );
    assert!(again.is_err(), "第二次同样会被加摩擦");
    let evs2 = w.ledger().read_all().unwrap();
    let refused2 = evs2
        .iter()
        .filter(|ev| ev["body"]["type"] == json!("gate.awaiting-approval"))
        .nth(1)
        .expect("第二条门禁流水")["body"]["payload"]["refused"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        refused, refused2,
        "同一次尝试的指纹必须可复算（同输入同输出）"
    );
}
