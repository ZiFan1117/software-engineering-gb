//! 世界核心 —— 项目 Step 0.5（本体信封）+ 项目 Step 1（账本）+ 项目 Step 2（读模型）+ 项目 Step 5（门禁）+ 项目 Step 7/8（两个投影）的**最小可运行骨架**。
//!
//! 骨架冒烟判据（S3 准出要求"一条命令跑通"）：`world-core check` 打印 **READY**。
//!
//! 退出码（规范要求"明确退出码"）：`0` 成功 / `1` 用法错误 / `2` 法律、账本、门禁或读模型错误。

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use world_core::World;

const USAGE: &str = "\
world-core —— 世界核心（语义事件是唯一真相）
用法:
  world-core [--ontology <path>] [--ledger <path>] [--policy <path>] [--channel <path>] [--owner-uid <uid>] [--require-chain] <子命令> [参数]

子命令:
  check                      加载本体+门禁策略+账本 → 打印 READY（骨架冒烟判据；**只读**）
  kinds                      列出本体已知的家族
  policy                     打印门禁策略（能力表 + 主体白名单）
  append <kind> <json-body> [actor]
                             追加一条事件（法律校验 → 门禁裁决 → 通过才落笔）；
                             actor 缺省 world://user（全权主体，见下）
  read [from_seq]            按序打印事件（JSON Lines；**只读**）
  state [--json]             从账本**重算**状态（读模型；不缓存、不写盘）
  project language           语言投影（结构化出口，给程序读）
  project visual             视觉投影（渲染出口，给人看）
  project check              渲染两份投影并**核对同源**（词表与状态必须一致）
  checkpoint write <path>    从账本重算状态并写一份检查点（缓存，非真相）
  checkpoint verify <path>   核验检查点与账本一致（不一致即拒用）
  checkpoint resume <path>   从检查点续算，并与全量重算逐字节比对
  channel bind <socket>      按 --channel 配置建套接字（0600 + chown 到该身份）
  channel accept <socket>    接受一个连接，把请求经**唯一写入口**落笔（一次一条）
  channel serve <socket> <n> 连续接受 n 个连接（v1 长驻形态；一次往返 = 两个连接）
  carrier capabilities       列出执行清单里的能力（**只执行、不裁决**的载体侧）
  carrier check              校验执行清单（坏清单即非零退出，不静默放过）
  carrier undo               列出执行清单里「动手前要先做撤销点」的能力
  carrier orphans            从账本里找出**有意图、无结果**的请求（只报告，不重试）
  carrier serve <socket>     常驻：逐行读请求 → 先问内核 → 再执行 → 再回写结果
  carrier do <能力> <动词> <请求号> [参数JSON] <socket>
                             手工跑一次：先问内核，准了才动手，动完回写结果

默认路径: --ontology ./ontology.json  --ledger ./ledger.jsonl  --policy ./policy.json
          --channel ./channel.json（**本仓未提供出厂文件**，由部署方给出）
          --cap-dir ./cap.d（执行清单目录；本仓提供出厂样例）
缺省身份: append 不写第 4 个参数时，actor 取 `world://user`——而 `policy.json` 把该主体列为
          **唯一可执行不可逆动作**的主体，并授权它写**任意**对象（`world://*`）。
          也就是说「不写身份」的后果是**最高授权**，不是匿名。
退出码:   0 成功 / 1 用法错误 / 2 法律、账本、门禁或读模型错误
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut ontology = PathBuf::from("ontology.json");
    let mut ledger = PathBuf::from("ledger.jsonl");
    let mut policy = PathBuf::from("policy.json");
    // 通道配置（M09 接线：生产路径可调用）。**本仓不提供出厂 channel.json**（待人裁）。
    let mut channel_cfg = PathBuf::from("channel.json");
    // 执行清单目录（M10 接线）：**只回答"怎么干"**，允不允许由门禁裁决。
    let mut cap_dir = PathBuf::from("cap.d");
    // 内核套接字（M10 提交请求的落点）。**适配器对账本零写权限**，只能走它。
    let mut kernel_sock: Option<PathBuf> = None;
    // 是否允许终端确认（高危动作的人确认入口；**默认关闭 ⇒ 需要确认的动作一律拒绝**）。
    let mut allow_confirm = false;
    // 属主断言（可选）：由部署方显式给出"法律与真相应当属于哪个 uid"。
    // 依据 WC-RV-R2-001 假设 A-05：静态墙只看 mode，属主必须另行断言。
    let mut owner_uid: Option<u32> = None;
    // `--owner-uid` 解析失败**必须拒启**（P-16）：此前 `.ok()` 把打错的 uid 静默变成
    // `None` ⇒ 属主断言**悄悄不执行**，是 fail-open（"打错反而更宽松"）。
    let mut owner_uid_bad: Option<String> = None;
    // 要求账本必须带摘要链（WC-CR-003 D3）：无链即拒启
    let mut require_chain = false;
    let mut rest: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--ontology" => {
                i += 1;
                if i < args.len() {
                    ontology = PathBuf::from(&args[i]);
                }
            }
            "--ledger" => {
                i += 1;
                if i < args.len() {
                    ledger = PathBuf::from(&args[i]);
                }
            }
            "--require-chain" => require_chain = true,
            "--owner-uid" => {
                i += 1;
                if i < args.len() {
                    match args[i].parse::<u32>() {
                        Ok(v) => owner_uid = Some(v),
                        Err(_) => owner_uid_bad = Some(args[i].clone()),
                    }
                } else {
                    owner_uid_bad = Some("<缺失>".to_string());
                }
            }
            "--channel" => {
                i += 1;
                if i < args.len() {
                    channel_cfg = PathBuf::from(&args[i]);
                }
            }
            "--cap-dir" => {
                i += 1;
                if i < args.len() {
                    cap_dir = PathBuf::from(&args[i]);
                }
            }
            "--socket" => {
                i += 1;
                if i < args.len() {
                    kernel_sock = Some(PathBuf::from(&args[i]));
                }
            }
            "--confirm" => allow_confirm = true,
            "--policy" => {
                i += 1;
                if i < args.len() {
                    policy = PathBuf::from(&args[i]);
                }
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            other => rest.push(other.to_string()),
        }
        i += 1;
    }

    match rest.first().map(String::as_str).unwrap_or("") {
        "check" => {
            if let Some(bad) = owner_uid_bad {
                eprintln!(
                    "[FAIL] ext.world.Runtime.BadOwnerUid: `--owner-uid {bad}` 不是合法的 uid 数字；\
                     拒绝启动。\n 为什么拒启而不是忽略：属主断言是「法律与真相属于谁」的唯一执行点，\
                     忽略打错的 uid 会让它**静默失效**（fail-open），比不做断言更危险"
                );
                return ExitCode::from(1);
            }
            cmd_check(&ontology, &ledger, &policy, owner_uid, require_chain)
        }
        "kinds" => cmd_kinds(&ontology),
        "policy" => cmd_policy(&policy),
        "append" => cmd_append(&ontology, &ledger, &policy, &rest),
        "read" => cmd_read(&ontology, &ledger, &policy, &rest),
        "state" => cmd_state(&ontology, &ledger, &policy, &rest),
        "project" => cmd_project(&ontology, &ledger, &policy, &rest),
        "checkpoint" => cmd_checkpoint(&ontology, &ledger, &policy, &rest),
        "channel" => cmd_channel(&ontology, &ledger, &policy, &channel_cfg, &rest),
        "carrier" => cmd_carrier(
            &cap_dir,
            kernel_sock.as_deref(),
            allow_confirm,
            &ledger,
            &rest,
        ),
        "" => {
            print!("{USAGE}");
            ExitCode::from(1)
        }
        other => {
            eprintln!("未知子命令 `{other}`\n\n{USAGE}");
            ExitCode::from(1)
        }
    }
}

fn cmd_check(
    o: &Path,
    l: &Path,
    p: &Path,
    owner_uid: Option<u32>,
    require_chain: bool,
) -> ExitCode {
    match World::open_readonly(o, l, p) {
        Ok(w) => {
            // 属主断言（若部署方给了 --owner-uid）：法律与真相三项都要对。
            // ⚠️ 放在 open **之后**——账本可能刚被创建，此前它还不存在，
            // 提前断言会把"文件尚未创建"误报成"属主读不到"（实测抓到）。
            if let Some(uid) = owner_uid {
                for (path, role) in [
                    (o, "本体（法律·形状）"),
                    (p, "门禁策略（法律）"),
                    (l, "账本（真相）"),
                ] {
                    if let Err(e) = world_core::guard::assert_owned_by(path, uid, role) {
                        eprintln!("[FAIL] {e}");
                        return ExitCode::from(2);
                    }
                }
            }
            println!("== world-core check ==");
            println!(
                "  本体 : {}  world={}  家族={:?}",
                o.display(),
                w.ontology().world(),
                w.ontology().known_kinds()
            );
            println!(
                "  门禁 : {}  policy={}  能力={} 个  主体白名单={:?}",
                p.display(),
                w.policy().version(),
                w.policy().capabilities().count(),
                w.policy().allowed_subjects()
            );
            // 两处出厂配置的互校状态（书 §5.5）：**拒启**这一半发生在 Policy::load 里，
            // 能走到这里就说明两处一致；但"一致"与"没东西可比"必须分开说——
            // 后者是缺口（闸读不到那些能力的风险等级），不得读成"已互校"。
            match w.policy().carrier_dir() {
                Some(d) => println!(
                    "  互校 : ✅ 载体清单（{}）与世界侧可逆性一致（清单 {} 项）",
                    d.display(),
                    w.policy().carrier_manifest().names().len()
                ),
                None => println!(
                    "  互校 : ⚠️ 策略同级没有 `cap.d/` ⇒ **无从互校**（未校验要说出来）：\
                     那些能力的风险等级读不到，闸只知道可逆性布尔值"
                ),
            }
            println!(
                "  账本 : {}  条数={}  next_seq={}",
                l.display(),
                w.ledger().last_seq(),
                w.ledger().next_seq()
            );
            if w.ledger().is_chained() {
                println!("  链   : ✅ 有摘要链（局部篡改可检出）");
            } else if require_chain && w.ledger().last_seq() > 0 {
                eprintln!(
                    "[FAIL] ext.world.Ledger.NoChain: 使用了 --require-chain，但本账本没有摘要链\n\
                     \x20 无链账本的局部篡改**不可检出**（WC-CR-003 §三），故拒绝启动。\n\
                     \x20 注：**空账本不算违规**（没有东西要保护），故此处只在已有事件时拒绝"
                );
                return ExitCode::from(2);
            } else {
                println!("  链   : ⚠️ 无摘要链——**局部篡改不可检出**（未校验要说出来）");
            }
            println!("  READY");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] {e}");
            ExitCode::from(2)
        }
    }
}

/// `policy` —— 打印门禁策略（**只读**：本命令不会改任何文件）。
fn cmd_policy(p: &Path) -> ExitCode {
    match world_core::gate::Policy::load(p) {
        Ok(pol) => {
            println!("== world-core policy ==");
            println!(
                "  策略 : {}  policy={}",
                pol.path().display(),
                pol.version()
            );
            println!("  主体白名单: {:?}", pol.allowed_subjects());
            println!("  能力表（默认拒绝：未列出的能力一律不放行）:");
            for (name, cap) in pol.capabilities() {
                // v1 的不可逆口径 = `DEBT-07`：**只允许 `irreversible_actors` 白名单主体**执行。
                // v1 **没有审批通道**（无批准命令、无批准事件）⇒ 不得再打印"需批准"，
                // 那会承诺一个不存在的出口（`WC-R4-DISP-001` §三 E-5 / `WC-CR-006` B-9）。
                //
                // 2026-09-28 改（书 §5.5 第三条：**闸读不到风险等级**）：等级现在读得到，
                // 且打印出来——它是摩擦轻重的依据，不该只活在载体侧。
                let level = pol.level_name(cap);
                let grade = if cap.reversible {
                    format!("可逆  → 免检但留痕（risk={level}）")
                } else {
                    format!(
                        "不可逆 → **加摩擦**（risk={level}）：白名单主体放行并留下摩擦旗标；\
                         白名单外一律加摩擦到拒绝执行"
                    )
                };
                println!("    {name:<18} {grade}");
            }
            match pol.carrier_dir() {
                Some(d) => println!(
                    "  互校 : ✅ 载体清单（{}）与世界侧的可逆性一致（{} 项已比对）",
                    d.display(),
                    pol.carrier_manifest().names().len()
                ),
                None => println!(
                    "  互校 : ⚠️ 策略同级没有 `cap.d/`（载体侧什么都没声明）⇒ 这些能力的\
                     **风险等级读不到**（risk=未声明），不得当成低危"
                ),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] {e}");
            ExitCode::from(2)
        }
    }
}

/// `state` —— 从账本**重算**状态。
///
/// 刻意**不提供 `--cache` / 不写盘**：v1 的读模型没有持久形态，
/// 也就不存在"读模型与账本不一致"这种故障。这是第三条专属验收测试的接口面。
fn cmd_state(o: &Path, l: &Path, p: &Path, rest: &[String]) -> ExitCode {
    let as_json = rest.iter().any(|a| a == "--json");
    let w = match World::open_readonly(o, l, p) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    match w.read_model() {
        Ok(s) => {
            if as_json {
                println!("{}", s.to_json());
            } else {
                println!("== world-core state ==");
                println!("  账本 : {}", l.display());
                println!(
                    "  已折叠: {} 条（last_seq={}）  act={}  notice={}",
                    s.seen(),
                    s.last_seq(),
                    s.acts(),
                    s.notices()
                );
                println!("  指纹 : {}", s.digest());
                for (subject, path, value) in s.entries() {
                    println!("  {subject}#{path} = {value}");
                }
                println!("  （状态由账本重算得来，未缓存、未写盘）");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] 读模型拒绝折叠：{e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_kinds(o: &Path) -> ExitCode {
    match world_core::ontology::Ontology::load(o) {
        Ok(ont) => {
            println!(
                "world={}  家族: {}",
                ont.world(),
                ont.known_kinds().join(", ")
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] {e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_append(o: &Path, l: &Path, p: &Path, rest: &[String]) -> ExitCode {
    if rest.len() < 3 {
        eprintln!("用法: world-core append <kind> <json-body> [actor]");
        eprintln!(
            "      actor 不写 ⇒ 取 world://user：policy.json 里**唯一**可执行不可逆动作的主体，"
        );
        eprintln!("      且被授权写任意对象（world://*）。**不写身份 = 最高授权，不是匿名。**");
        return ExitCode::from(1);
    }
    let kind = &rest[1];
    let body: serde_json::Value = match serde_json::from_str(&rest[2]) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[FAIL] body 不是合法 JSON: {e}");
            return ExitCode::from(1);
        }
    };
    // 缺省身份 = `world://user`：`policy.json` 第 32 行把它列为**唯一**可执行不可逆动作的主体，
    // 同文件第 28 行授权它写**任意**对象（`world://*`）。⇒ 这里的 `unwrap_or_else` 不是"匿名兜底"，
    // 而是**全权兜底**。用法串（`USAGE` 与上面那段 `eprintln!`）必须把这件事写出来，
    // 否则"没写身份"这一默认行为的后果在帮助里是隐形的（缺陷台账 **D-43**）。
    let actor = rest
        .get(3)
        .cloned()
        .unwrap_or_else(|| "world://user".to_string());
    let mut w = match World::open(o, l, p) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    match w.commit(kind, &actor, body) {
        Ok(ev) => {
            println!("{}", serde_json::to_string(&ev).unwrap_or_default());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] {e}");
            ExitCode::from(2)
        }
    }
}

/// `project` —— 两个投影，以及"同源"核对。
///
/// 两个投影都**只**从读模型 + 词表渲染（`07/2-依据/15` §三：投影之间不交互）。
/// `project check` 是 `REQ-F-020` 的机器判定面：比对两份输出的同源头。
fn cmd_project(o: &Path, l: &Path, p: &Path, rest: &[String]) -> ExitCode {
    use world_core::project::{self, language, visual};

    let which = rest.get(1).map(String::as_str).unwrap_or("");
    let w = match World::open_readonly(o, l, p) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    let state = match w.read_model() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[FAIL] 读模型拒绝折叠：{e}");
            return ExitCode::from(2);
        }
    };
    let (world, vocab) = (w.ontology().world(), w.ontology().vocab_hash());

    match which {
        "language" => {
            print!("{}", language::render(&state, world, vocab));
            ExitCode::SUCCESS
        }
        "visual" => {
            print!("{}", visual::render(&state, world, vocab));
            ExitCode::SUCCESS
        }
        "check" => {
            let a = language::render(&state, world, vocab);
            let b = visual::render(&state, world, vocab);
            match project::assert_same_source(&a, &b) {
                Ok(()) => {
                    println!("== world-core project check ==");
                    println!("  词表 : {vocab}（世界版本 {world}）");
                    println!(
                        "  状态 : last_seq={} 指纹={}",
                        state.last_seq(),
                        state.digest()
                    );
                    println!("  同源 : ✅ 语言投影与视觉投影一致（同一读模型 + 同一词表）");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    ExitCode::from(2)
                }
            }
        }
        other => {
            eprintln!("未知投影 `{other}`（可用：language / visual / check）");
            ExitCode::from(1)
        }
    }
}

fn cmd_read(o: &Path, l: &Path, p: &Path, rest: &[String]) -> ExitCode {
    let from: u64 = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let w = match World::open_readonly(o, l, p) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    match w.ledger().read_from(from) {
        Ok(evs) => {
            for e in evs {
                println!("{}", serde_json::to_string(&e).unwrap_or_default());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[FAIL] {e}");
            ExitCode::from(2)
        }
    }
}

/// `checkpoint` —— **M08 接线的生产调用点**（`W-03` / `P-09`）。
///
/// 为什么要有这三个子命令：`src/checkpoint.rs` 此前**整册生产零调用点**
/// ——`capture` / `write` / `load` / `verify` / `resume_unverified` /
/// `read_model_with_checkpoint` 在生产代码里各 0 命中，即"已实现"只是**文件里存在**，
/// 用户永远用不到。接线的判据不是"文件里有"，而是"**生产路径可调用且契约要点逐条成立**"：
///
/// | 契约要点（`WC-IC-M08`） | 本命令的落点 |
/// |---|---|
/// | 提供者：`M08` | `world_core::checkpoint` |
/// | 输入：账本 + 快照路径 | `--ledger` / `<path>` |
/// | 输出：快照文件 / 核验结论 / 续算指纹 | 三个子命令各自的 stdout |
/// | 异常与错误码：`ext.world.Checkpoint.*` | 见 `src/checkpoint.rs` |
/// | 不变量：**缓存不得成为第二真相** | `verify` 拿账本重算前 `base_seq` 条比对指纹；`resume` 与全量重算**逐字节**比对 |
/// | 判据：删掉快照再算，结果不变 | `resume` 的输出必须等于 `state --json` |
fn cmd_checkpoint(o: &Path, l: &Path, p: &Path, rest: &[String]) -> ExitCode {
    use world_core::checkpoint::{read_model_with_checkpoint, Checkpoint};
    let sub = rest.get(1).map(String::as_str).unwrap_or("");
    let Some(cp_arg) = rest.get(2) else {
        eprintln!("用法: world-core checkpoint <write|verify|resume> <path>");
        return ExitCode::from(1);
    };
    let cp_path = Path::new(cp_arg);
    let w = match World::open_readonly(o, l, p) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    let events = match w.ledger().read_all() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    match sub {
        "write" => {
            let st = match w.read_model() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[FAIL] 读模型拒绝折叠：{e}");
                    return ExitCode::from(2);
                }
            };
            let cp = Checkpoint::capture(&st);
            if let Err(e) = cp.write(cp_path) {
                eprintln!("[FAIL] {e}");
                return ExitCode::from(2);
            }
            println!("== world-core checkpoint write ==");
            println!("  快照 : {}", cp_path.display());
            println!("  base_seq={} 指纹={}", cp.base_seq(), cp.digest());
            println!("  （快照是**缓存、不是真相**：删掉它不影响任何结论）");
            ExitCode::SUCCESS
        }
        "verify" => {
            let cp = match Checkpoint::load(cp_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            if let Err(e) = cp.verify(&events) {
                eprintln!("[FAIL] {e}");
                return ExitCode::from(2);
            }
            println!(
                "  核验 : ✅ 快照 base_seq={} 与账本前 {} 条一致（缓存未成为第二真相）",
                cp.base_seq(),
                cp.base_seq()
            );
            ExitCode::SUCCESS
        }
        "resume" => {
            let cp = match Checkpoint::load(cp_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            let fast = match read_model_with_checkpoint(&events, Some(&cp)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            let full = match w.read_model() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[FAIL] 读模型拒绝折叠：{e}");
                    return ExitCode::from(2);
                }
            };
            if fast.to_json() != full.to_json() {
                eprintln!(
                    "ext.world.Checkpoint.ResumeMismatch: 从快照续算与全量重算结果不一致——\
                     缓存成了第二真相，拒绝使用"
                );
                return ExitCode::from(2);
            }
            println!("== world-core checkpoint resume ==");
            println!("  快路径指纹 : {}", fast.digest());
            println!("  全量指纹   : {}", full.digest());
            println!("  一致 : ✅ 续算 == 全量重算（逐字节）");
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("未知 checkpoint 子命令 `{other}`（可用：write / verify / resume）");
            ExitCode::from(1)
        }
    }
}

/// `channel` —— **M09 接线的生产调用点**（`W-03` / `P-10`）。
///
/// 为什么要有它：`src/channel.rs` 此前**整册生产零调用点**，`channel::bind` 连测试都零调用
/// ⇒「权限即身份」（`0600` + `chown` + 通道目录静态墙）**从未被执行过**；
/// 原有测试用 `UnixListener::bind` 自己建套接字，**绕过了** `bind()`，
/// 于是只验了"自称被拒"，**没验"别人连不上"**。
///
/// 用法：`world-core --channel <channel.json> channel bind <socket>`
///       `world-core --channel <channel.json> channel accept <socket>`
///
/// ⚠️ 本仓**不提供出厂 `channel.json`**（`WC-SCMP-001` §8.4 `G-28` 记"是否补出厂文件**待人裁定**"），
/// 故 `--channel` 指向的文件由部署方给出；缺文件即 `ext.world.Channel.ReadFail` 拒启。
#[cfg(unix)]
fn cmd_channel(o: &Path, l: &Path, p: &Path, cfg: &Path, rest: &[String]) -> ExitCode {
    use world_core::channel::{self, ChannelConfig};
    let sub = rest.get(1).map(String::as_str).unwrap_or("");
    let Some(sock_arg) = rest.get(2) else {
        eprintln!("用法: world-core --channel <path> channel <bind|accept> <socket>");
        return ExitCode::from(1);
    };
    let sock = PathBuf::from(sock_arg);
    let conf = match ChannelConfig::load(cfg) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };
    let expect = match conf.listener_for(&sock) {
        Some(x) => x.clone(),
        None => {
            eprintln!(
                "ext.world.Channel.NotConfigured: 套接字 {} 不在 {} 的身份映射里——\
                 没有身份映射的连接不得建立（默认拒绝）",
                sock.display(),
                cfg.display()
            );
            return ExitCode::from(2);
        }
    };
    match sub {
        "bind" => match channel::bind(&expect) {
            Ok(_lst) => {
                println!("== world-core channel bind ==");
                println!(
                    "  套接字 : {}  actor={}  uid={}",
                    expect.socket.display(),
                    expect.actor,
                    expect.uid
                );
                println!("  （0600 + chown 到该 uid ⇒ **只有那个 uid 连得上**）");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("[FAIL] {e}");
                ExitCode::from(2)
            }
        },
        "accept" | "serve" => {
            let n = if sub == "serve" {
                match rest.get(3).and_then(|s| s.parse::<usize>().ok()) {
                    Some(n) if n > 0 => n,
                    _ => {
                        eprintln!(
                            "用法: world-core --channel <path> channel serve <socket> <个数>"
                        );
                        return ExitCode::from(1);
                    }
                }
            } else {
                1
            };
            let lst = match channel::bind(&expect) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            let mut w = match World::open(o, l, p) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            if sub == "accept" {
                return match channel::serve_once(&mut w, &lst, &expect) {
                    Ok(ev) => {
                        println!("{}", serde_json::to_string(&ev).unwrap_or_default());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("[FAIL] {e}");
                        ExitCode::from(2)
                    }
                };
            }
            match channel::serve_n(&mut w, &lst, &expect, n) {
                Ok(ok) => {
                    println!("== world-core channel serve ==");
                    println!(
                        "  套接字 : {}  actor={}",
                        expect.socket.display(),
                        expect.actor
                    );
                    println!("  已处理 : {ok} / {n} 个连接");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    ExitCode::from(2)
                }
            }
        }
        other => {
            eprintln!("未知 channel 子命令 `{other}`（可用：bind / accept / serve）");
            ExitCode::from(1)
        }
    }
}

#[cfg(not(unix))]
fn cmd_channel(_o: &Path, _l: &Path, _p: &Path, _cfg: &Path, _rest: &[String]) -> ExitCode {
    eprintln!("ext.world.Channel.Unsupported: 通道只在 Unix 上可用（v1 局限）");
    ExitCode::from(1)
}

/// `carrier` —— **载体侧的手**（`M10`）：只执行、不裁决。
///
/// 三个只读子命令（`capabilities` / `check` / `undo`）**不碰载体、不连内核**，
/// 只读执行清单；`orphans` 只读账本；两个动作子命令（`do` / `serve`）
/// 一律走「**先问内核 → 准了才动 → 动完回写**」，且**连不上内核即拒绝执行**。
///
/// ⚠️ 它**没有**"允不允许"的话语权：清单里有的能力，门禁说不行照样不行。
/// 这条不对称（可以拒绝、永远不能放行）是"门禁不可绕过"在跨进程形态下的落点。
#[cfg(unix)]
fn cmd_carrier(
    cap_dir: &Path,
    kernel_sock: Option<&Path>,
    allow_confirm: bool,
    ledger: &Path,
    rest: &[String],
) -> ExitCode {
    use world_core::carrier::capd::Manifest;
    use world_core::carrier::providers::Registry;
    use world_core::carrier::run::{self, NoConfirm, TerminalConfirm};

    let sub = rest.get(1).map(String::as_str).unwrap_or("");
    let manifest = match Manifest::load_dir(cap_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FAIL] {e}");
            return ExitCode::from(2);
        }
    };

    match sub {
        "capabilities" => {
            println!("== world-core carrier capabilities ==");
            println!("  执行清单 : {}", cap_dir.display());
            println!("  已注册执行器 : {:?}", Registry::builtin().names());
            if manifest.is_empty() {
                println!(
                    "  能力 : （空）——**空清单不是『什么都不允许』的安全默认，是『没装清单』**"
                );
            } else {
                println!("  能力（默认拒绝：清单外的能力一律不执行）：");
                for c in manifest.iter() {
                    let mut flags = Vec::new();
                    if c.needs_undo() {
                        flags.push("动手前先做撤销点");
                    }
                    if c.needs_confirm() {
                        flags.push("需要人确认");
                    }
                    let tail = if flags.is_empty() {
                        String::new()
                    } else {
                        format!("  ← {}", flags.join(" / "))
                    };
                    println!(
                        "    {:<18} 动词={:?} 风险={:?} 执行器={}{}",
                        c.name, c.verbs, c.risk, c.provider, tail
                    );
                }
            }
            println!("  ⚠️ 本清单只回答「**怎么干**」；「允不允许」由内核的门禁裁决。");
            ExitCode::SUCCESS
        }
        "check" => {
            // 复核：清单里的执行器必须都已注册，否则那项能力永远执行不了（死号）。
            let reg = Registry::builtin();
            let mut dead: Vec<String> = Vec::new();
            for c in manifest.iter() {
                if reg.get(&c.provider).is_none() {
                    dead.push(format!("{}（执行器 {} 未注册）", c.name, c.provider));
                }
            }
            println!("== world-core carrier check ==");
            println!("  清单文件目录 : {}", cap_dir.display());
            println!("  能力数 : {}", manifest.names().len());
            if dead.is_empty() {
                println!("  结论 : ✅ 全部能力的执行器都已注册");
                ExitCode::SUCCESS
            } else {
                println!("  结论 : ❌ 有 {} 项能力是死号：", dead.len());
                for d in &dead {
                    println!("    - {d}");
                }
                ExitCode::from(2)
            }
        }
        "undo" => {
            println!("== world-core carrier undo（载体撤销点）==");
            let mut n = 0;
            for c in manifest.iter() {
                if c.needs_undo() {
                    println!("  {} → 每次动手前先做撤销点（风险={:?}）", c.name, c.risk);
                    n += 1;
                }
            }
            if n == 0 {
                println!("  （无）");
            }
            println!(
                "  ⚠️ 载体撤销撤的是**文件系统的字节**，不是世界状态；\
                 「坏了能回滚」靠的是**追加补偿事件**，不是它。"
            );
            ExitCode::SUCCESS
        }
        "orphans" => {
            // 只读账本：**有意图、无结果**的请求。
            let events = match world_core::carrier::recover::read_ledger_readonly(ledger) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("[FAIL] {e}");
                    return ExitCode::from(2);
                }
            };
            let orphans = world_core::carrier::recover::orphans(&events);
            println!("== world-core carrier orphans ==");
            println!("  账本 : {}（{} 条事件）", ledger.display(), events.len());
            if orphans.is_empty() {
                println!("  结论 : ✅ 没有『有意图、无结果』的请求");
                ExitCode::SUCCESS
            } else {
                println!("  结论 : ⚠️ 有 {} 条请求有意图、无结果：", orphans.len());
                for o in &orphans {
                    println!(
                        "    seq={} {} {}  请求号={}  距今={}s",
                        o.intent_seq, o.capability, o.verb, o.request_id, o.age_secs
                    );
                    println!("      {}", o.hint());
                }
                // 有孤儿**不是**"跑失败了"：它是账本如实显示的状态。
                // 故退出码 0，但结论里点明要人看。
                ExitCode::SUCCESS
            }
        }
        "do" | "serve" => {
            let Some(sock) = kernel_sock else {
                eprintln!(
                    "用法: world-core carrier {sub} … --socket <内核套接字>\n\
                     ⚠️ 必须显式给出内核套接字：适配器**对账本零写权限**，\
                     它写世界的唯一通道就是这条连接。"
                );
                return ExitCode::from(1);
            };
            let client = world_core::carrier::kernel::KernelClient::new(sock);
            let reg = Registry::builtin();
            let confirmer: &dyn run::Confirmer = if allow_confirm {
                &TerminalConfirm
            } else {
                &NoConfirm
            };
            let adapter = run::Adapter {
                client: &client,
                manifest: &manifest,
                registry: &reg,
                confirmer,
            };
            if sub == "serve" {
                return match run::serve(&adapter) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("[FAIL] {e}");
                        ExitCode::from(2)
                    }
                };
            }
            // do：手工跑一次
            let (cap, verb, rid) = (
                rest.get(2).cloned().unwrap_or_default(),
                rest.get(3).cloned().unwrap_or_default(),
                rest.get(4).cloned().unwrap_or_default(),
            );
            if cap.is_empty() || verb.is_empty() || rid.is_empty() {
                eprintln!(
                    "用法: world-core carrier do <能力> <动词> <请求号> [参数JSON] --socket <内核套接字>"
                );
                return ExitCode::from(1);
            }
            let params: serde_json::Value = match rest.get(5) {
                None => serde_json::Value::Null,
                Some(s) if s.starts_with("--") => serde_json::Value::Null,
                Some(s) => match serde_json::from_str(s) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("[FAIL] ext.world.Carrier.BadParam: 参数不是合法 JSON：{e}");
                        return ExitCode::from(1);
                    }
                },
            };
            let attempt = adapter.attempt(&world_core::carrier::Invocation {
                capability: cap.clone(),
                verb: verb.clone(),
                request_id: rid.clone(),
                params,
            });
            println!("== world-core carrier do ==");
            println!(
                "  能力={cap} 动词={verb} 请求号={rid} 内核={}",
                sock.display()
            );
            println!("  {}", attempt.summary());
            if attempt.admitted {
                ExitCode::SUCCESS
            } else {
                // 未获准或连不上内核：**一次都没动**，退非零让调用方知道。
                ExitCode::from(2)
            }
        }
        other => {
            eprintln!(
                "未知 carrier 子命令 `{other}`\
                 （可用：capabilities / check / undo / orphans / do / serve）"
            );
            ExitCode::from(1)
        }
    }
}

#[cfg(not(unix))]
fn cmd_carrier(
    _cap_dir: &Path,
    _kernel_sock: Option<&Path>,
    _allow_confirm: bool,
    _ledger: &Path,
    _rest: &[String],
) -> ExitCode {
    eprintln!("ext.world.Carrier.Unsupported: 载体适配器的动作面只在 Unix 上可用（v1 局限）");
    ExitCode::from(1)
}
