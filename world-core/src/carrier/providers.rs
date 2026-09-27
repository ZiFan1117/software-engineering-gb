//! 内置执行器 —— **只有这些事需要碰到载体**，所以只有这些事在这里。
//!
//! 三个执行器各自守着自己的边界：
//!
//! | 执行器 | 它碰什么 | 它特意**不**做什么 |
//! |---|---|---|
//! | `backlight` | 读/写背光设备 | 不校验"该不该调"——调之前门禁已经裁决过 |
//! | `package` | 调包管理器；高危动词前先做载体撤销点 | 不做"要不要确认"的判断——那是清单的 `confirm` 栏 |
//! | `job` | 起长任务；完工后**回写一条结果事件** | 不自己维护"待办清单"——状态由账本折叠算出 |

use crate::carrier::provider::{Outcome, Provider};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

/// 背光执行器：**真设备**（sysfs）。
///
/// 它在无背光设备的机器上**不假装成功**：明确报"找不到设备"，
/// 而不是返回一个编造的亮度值。
pub struct Backlight {
    /// 背光设备根目录（默认 `/sys/class/backlight`；测试可换成临时目录）。
    pub root: PathBuf,
}

impl Default for Backlight {
    fn default() -> Self {
        Backlight {
            root: PathBuf::from("/sys/class/backlight"),
        }
    }
}

impl Backlight {
    /// 找一个可用的背光设备目录（按名字有序，故结果确定）。
    fn device(&self) -> Result<PathBuf, String> {
        let mut names: Vec<PathBuf> = std::fs::read_dir(&self.root)
            .map_err(|e| {
                format!(
                    "ext.world.Carrier.NoDevice: 读不了背光设备目录 {}：{e}",
                    self.root.display()
                )
            })?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.join("max_brightness").is_file())
            .collect();
        names.sort();
        names.into_iter().next().ok_or_else(|| {
            format!(
                "ext.world.Carrier.NoDevice: {} 下没有可用背光设备（缺 max_brightness）",
                self.root.display()
            )
        })
    }

    fn read_u64(path: &Path) -> Result<u64, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("ext.world.Carrier.DeviceReadFail: {}：{e}", path.display()))?;
        raw.trim().parse::<u64>().map_err(|e| {
            format!(
                "ext.world.Carrier.DeviceReadFail: {} 不是整数：{e}",
                path.display()
            )
        })
    }

    /// 把"百分比"翻译成设备刻度（**单位在参数里，不靠猜**）。
    ///
    /// 口径：`scale` 只允许 `raw`（直接给设备值）或 `percent`（0–100）；
    /// 未给时按 `raw`——**不猜**，因为"3"到底是 3% 还是第 3 档是两件完全不同的事。
    fn to_raw(level: u64, max: u64, params: &Value) -> Result<u64, String> {
        let scale = params.get("scale").and_then(Value::as_str).unwrap_or("raw");
        match scale {
            "raw" => {
                if level > max {
                    return Err(format!(
                        "ext.world.Carrier.OutOfRange: 亮度 {level} 超出设备上限 {max}（scale=raw）"
                    ));
                }
                Ok(level)
            }
            "percent" => {
                if level > 100 {
                    return Err(format!(
                        "ext.world.Carrier.OutOfRange: 百分比 {level} 超出 0–100（scale=percent）"
                    ));
                }
                Ok((level * max + 50) / 100)
            }
            other => Err(format!(
                "ext.world.Carrier.BadParam: scale=`{other}` 非法（只允许 raw/percent）"
            )),
        }
    }
}

impl Provider for Backlight {
    fn name(&self) -> &'static str {
        "backlight"
    }

    fn capabilities(&self) -> Vec<&'static str> {
        vec!["brightness.set"]
    }

    fn call(&self, verb: &str, params: &Value) -> Result<Value, String> {
        let dev = self.device()?;
        let max = Self::read_u64(&dev.join("max_brightness"))?;
        match verb {
            "get" => {
                let cur = Self::read_u64(&dev.join("brightness"))?;
                Ok(json!({
                    "device": dev.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                    "level": cur,
                    "max": max,
                    "percent": (cur * 100 + max / 2).checked_div(max).unwrap_or(0),
                }))
            }
            "set" => {
                let level = params.get("level").and_then(Value::as_u64).ok_or_else(|| {
                    "ext.world.Carrier.BadParam: set 需要整数参数 level".to_string()
                })?;
                let raw = Self::to_raw(level, max, params)?;
                std::fs::write(dev.join("brightness"), format!("{raw}\n")).map_err(|e| {
                    format!(
                        "ext.world.Carrier.DeviceWriteFail: 写 {} 失败：{e}",
                        dev.join("brightness").display()
                    )
                })?;
                Ok(json!({
                    "device": dev.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
                    "level": raw,
                    "max": max,
                }))
            }
            other => Err(format!(
                "ext.world.Carrier.UnknownVerb: 背光执行器不认识动词 `{other}`"
            )),
        }
    }
}

/// 包管理执行器：调包管理器；**高危动词前先做载体撤销点**。
pub struct Package {
    /// 包管理器可执行文件（默认 `pacman`；测试可换成任意命令）。
    pub program: String,
    /// 载体撤销点目录（本轮载体撤销的落点）。
    pub undo_dir: PathBuf,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            program: "pacman".to_string(),
            undo_dir: PathBuf::from("/run/world-core/undo"),
        }
    }
}

impl Package {
    /// 记一份"包清单"到撤销点目录（**先用可判定的最小动作**：
    /// 记下当前已装包清单，将来可据此判断"多装了哪些"）。
    ///
    /// 为什么不做全盘快照：载体撤销是工程兜底，**不是世界回滚**；
    /// 这里刻意只做"可判定、可复现、可解释"的那一小步，不假装有全盘原子撤销。
    fn mark(&self, request_id: &str) -> Result<Value, String> {
        std::fs::create_dir_all(&self.undo_dir).map_err(|e| {
            format!(
                "ext.world.Carrier.UndoFail: 建不了撤销点目录 {}：{e}",
                self.undo_dir.display()
            )
        })?;
        let path = self.undo_dir.join(format!("pre-{request_id}.list"));
        let out = Command::new(&self.program)
            .arg("-Qq")
            .output()
            .map_err(|e| {
                format!(
                    "ext.world.Carrier.UndoFail: 撤销点无法取得包清单（{}）：{e}",
                    self.program
                )
            })?;
        std::fs::write(&path, &out.stdout).map_err(|e| {
            format!(
                "ext.world.Carrier.UndoFail: 写不了撤销点 {}：{e}",
                path.display()
            )
        })?;
        let n = String::from_utf8_lossy(&out.stdout).lines().count();
        Ok(json!({
            "kind": "carrier-undo",
            "path": path.display().to_string(),
            "packages": n,
        }))
    }
}

impl Provider for Package {
    fn name(&self) -> &'static str {
        "package"
    }

    fn capabilities(&self) -> Vec<&'static str> {
        vec!["package.install"]
    }

    /// 载体撤销点：记下当前包清单。做不到就报错（**不假装撤得回去**）。
    fn undo_mark(&self, request_id: &str) -> Result<Value, String> {
        self.mark(request_id)
    }

    fn call(&self, verb: &str, params: &Value) -> Result<Value, String> {
        match verb {
            "list" => {
                let out = Command::new(&self.program)
                    .arg("-Qq")
                    .output()
                    .map_err(|e| format!("ext.world.Carrier.ProviderFailed: {e}"))?;
                if !out.status.success() {
                    return Err(format!(
                        "ext.world.Carrier.ProviderFailed: {} -Qq 退出码 {:?}",
                        self.program,
                        out.status.code()
                    ));
                }
                let list: Vec<String> = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(str::to_string)
                    .collect();
                Ok(json!({ "count": list.len(), "packages": list }))
            }
            "install" => {
                let pkg = params
                    .get("package")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        "ext.world.Carrier.BadParam: install 需要参数 package".to_string()
                    })?;
                let out = Command::new(&self.program)
                    .arg("-S")
                    .arg("--noconfirm")
                    .arg(pkg)
                    .output()
                    .map_err(|e| format!("ext.world.Carrier.ProviderFailed: {e}"))?;
                let code = out.status.code().unwrap_or(-1) as i64;
                let detail = json!({
                    "package": pkg,
                    "stdout_tail": tail(&out.stdout, 8),
                    "stderr_tail": tail(&out.stderr, 8),
                });
                if out.status.success() {
                    Ok(detail)
                } else {
                    Err(format!(
                        "ext.world.Carrier.ProviderFailed: 装包失败（退出码 {code}）：{}",
                        tail(&out.stderr, 3).join(" | ")
                    ))
                }
            }
            other => Err(format!(
                "ext.world.Carrier.UnknownVerb: 包执行器不认识动词 `{other}`"
            )),
        }
    }
}

fn tail(raw: &[u8], n: usize) -> Vec<String> {
    let text = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].iter().map(|s| s.to_string()).collect()
}

/// 长任务执行器：起任务 → **完工后回写一条结果事件**。
///
/// ## 它与旧实现的关键差别
///
/// 旧实现在载体侧**自己维护一本"完工铃登记簿"**（落盘、重启后标记丢失）。
/// 现在**不再需要那本登记簿**：登记簿要回答的问题——"哪些活还没干完"——
/// 由**账本折叠**回答：一条有意图、无结果的请求就是"还没干完"。
/// 这样"待办清单"不会成为第二份真相。
///
/// 完工本身仍然要有人去等：本执行器 spawn 一个**等待者进程**（就是它自己，
/// 用 `job wait` 形态），由它等子进程结束后把结果事件提交给内核。
pub struct Job {
    /// 运行期状态目录（记录活着的工作者）。
    pub state_dir: PathBuf,
    /// 在内存里登记的活跃任务（工作者 pid）。
    jobs: Arc<Mutex<BTreeMap<String, Value>>>,
}

impl Default for Job {
    fn default() -> Self {
        Job {
            state_dir: PathBuf::from("/run/world-core/jobs"),
            jobs: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl Job {
    /// 完工台账（工作者写下的一行一条 JSONL）。
    pub fn ledger_path(&self) -> PathBuf {
        self.state_dir.join("completed.jsonl")
    }

    /// 读完工台账。
    pub fn completed(&self) -> Vec<Value> {
        let p = self.ledger_path();
        let f = match std::fs::File::open(&p) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };
        BufReader::new(f)
            .lines()
            .map_while(Result::ok)
            .filter_map(|l| serde_json::from_str::<Value>(&l).ok())
            .collect()
    }

    /// 登记一个已启动的任务。
    pub fn remember(&self, job_id: &str, rec: Value) {
        if let Ok(mut m) = self.jobs.lock() {
            m.insert(job_id.to_string(), rec);
        }
    }

    /// 任务是否还活着（按工作者 pid 判断；pid 不在了即"丢了"）。
    pub fn alive(&self, job_id: &str) -> Option<bool> {
        let m = self.jobs.lock().ok()?;
        let rec = m.get(job_id)?;
        let pid = rec.get("waiter_pid").and_then(Value::as_i64)?;
        Some(Path::new(&format!("/proc/{pid}")).exists())
    }
}

impl Provider for Job {
    fn name(&self) -> &'static str {
        "job"
    }

    fn capabilities(&self) -> Vec<&'static str> {
        vec!["job.start"]
    }

    fn call(&self, verb: &str, params: &Value) -> Result<Value, String> {
        match verb {
            "start" => {
                let cmd = params
                    .get("command")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        "ext.world.Carrier.BadParam: start 需要参数 command".to_string()
                    })?
                    .to_string();
                let args: Vec<String> = params
                    .get("args")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                let job_id = params
                    .get("job_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        "ext.world.Carrier.BadParam: start 需要参数 job_id（由请求号派生）"
                            .to_string()
                    })?
                    .to_string();

                std::fs::create_dir_all(&self.state_dir).map_err(|e| {
                    format!(
                        "ext.world.Carrier.JobFail: 建不了任务状态目录 {}：{e}",
                        self.state_dir.display()
                    )
                })?;

                let child = Command::new(&cmd)
                    .args(&args)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|e| format!("ext.world.Carrier.JobFail: 起不了任务 `{cmd}`：{e}"))?;
                let pid = child.id();

                self.remember(
                    &job_id,
                    json!({ "job_id": job_id, "command": cmd, "args": args, "waiter_pid": pid }),
                );
                Ok(json!({
                    "job_id": job_id,
                    "command": cmd,
                    "args": args,
                    "status": "running",
                    "pid": pid,
                    "completed_ledger": self.ledger_path().display().to_string(),
                }))
            }
            "status" => {
                let job_id = params
                    .get("job_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        "ext.world.Carrier.BadParam: status 需要参数 job_id".to_string()
                    })?;
                // 先看完工台账（**完工是事实**），再看进程是否还活着。
                if let Some(rec) = self
                    .completed()
                    .into_iter()
                    .rev()
                    .find(|r| r.get("job_id").and_then(Value::as_str) == Some(job_id))
                {
                    return Ok(rec);
                }
                match self.alive(job_id) {
                    Some(true) => Ok(json!({ "job_id": job_id, "status": "running" })),
                    // 没完工、进程也不在 ⇒ **丢了**（机器断电/被杀）：如实报，不猜。
                    Some(false) => Ok(json!({ "job_id": job_id, "status": "lost" })),
                    None => Ok(json!({ "job_id": job_id, "status": "unknown" })),
                }
            }
            "list" => {
                let m = self.jobs.lock().map_err(|_| {
                    "ext.world.Carrier.JobFail: 任务表被毒化（上一次持锁线程崩了）".to_string()
                })?;
                let running: Vec<Value> = m.values().cloned().collect();
                Ok(json!({ "running": running, "completed": self.completed() }))
            }
            "wait" => {
                // 等待者形态：等一个已经起好的进程结束，然后把结果写成台账一行。
                // 由 `run` 子命令以本形态拉起，故这里只返回"该怎么等"的说明，
                // 真正的等待在 `crate::carrier::run` 里做（它需要网络与账本）。
                Err(
                    "ext.world.Carrier.Internal: wait 由 run 形态处理，不经 provider 调用"
                        .to_string(),
                )
            }
            other => Err(format!(
                "ext.world.Carrier.UnknownVerb: 任务执行器不认识动词 `{other}`"
            )),
        }
    }
}

/// 执行器注册表：能力名 → 执行器。
///
/// **注册表为空不是"什么都不允许"的安全默认，而是"没装执行器"**——
/// 与门禁策略的空能力表同一条口径：空即拒绝服务，且要说清是"没配好"。
pub struct Registry {
    providers: BTreeMap<String, Box<dyn Provider + Send + Sync>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// 空注册表。
    pub fn new() -> Self {
        Registry {
            providers: BTreeMap::new(),
        }
    }

    /// 内置三个执行器（背光 / 包 / 任务）。
    pub fn builtin() -> Self {
        let mut r = Self::new();
        r.add(Box::new(Backlight::default()));
        r.add(Box::new(Package::default()));
        r.add(Box::new(Job::default()));
        r
    }

    /// 注册一个执行器。
    pub fn add(&mut self, p: Box<dyn Provider + Send + Sync>) {
        self.providers.insert(p.name().to_string(), p);
    }

    /// 按名字取执行器。
    pub fn get(&self, name: &str) -> Option<&(dyn Provider + Send + Sync)> {
        self.providers.get(name).map(|b| b.as_ref())
    }

    /// 已注册的执行器名（有序）。
    pub fn names(&self) -> Vec<&str> {
        self.providers.keys().map(String::as_str).collect()
    }
}

/// 执行一次调用：**清单 → 动词 → 执行器 → 结果**。
///
/// 顺序固定，且**每一步失败都不进入下一步**：
///
/// 1. 清单里没有这项能力 ⇒ **拒绝**（根本不动手）；
/// 2. 清单里没有这个动词 ⇒ **拒绝**；
/// 3. 没有对应执行器 ⇒ **拒绝**（并点名"注册表里没有"）；
/// 4. 需要人确认 ⇒ 问确认入口；确认未给或入口不可用 ⇒ **拒绝**；
/// 5. 需要载体撤销点 ⇒ 先做撤销点；做不成 ⇒ **拒绝**（不带着"撤不回去"的风险动手）；
/// 6. 执行；
/// 7. 组装结果（成功/失败）。
///
/// ⚠️ **本函数不做"允不允许"的裁决**：它能做的只有"拒绝"，永远不能"放行"——
/// 放行由调用方在**问过门禁之后**才走到这里。
pub fn execute(
    manifest: &crate::carrier::capd::Manifest,
    registry: &Registry,
    inv: &crate::carrier::Invocation,
    confirm: &dyn Fn(&str, &Value) -> bool,
    undo_marker: &dyn Fn(&str) -> Result<Value, String>,
) -> Outcome {
    let cap = match manifest.lookup(&inv.capability) {
        Some(c) => c,
        None => {
            return Outcome::refused(json!({
                "reason": "能力不在执行清单里",
                "capability": inv.capability,
                "manifest": manifest.names(),
            }))
        }
    };
    if !cap.allows(&inv.verb) {
        return Outcome::refused(json!({
            "reason": "动词未被该能力授权",
            "capability": inv.capability,
            "verb": inv.verb,
            "allowed": cap.verbs,
        }));
    }
    let provider = match registry.get(&cap.provider) {
        Some(p) => p,
        None => {
            return Outcome::refused(json!({
                "reason": "没有对应的执行器",
                "provider": cap.provider,
                "registered": registry.names(),
            }))
        }
    };
    if cap.needs_confirm() && !confirm(&inv.capability, &inv.params) {
        return Outcome::refused(json!({
            "reason": "需要人确认而未获确认（默认拒绝）",
            "capability": inv.capability,
        }));
    }
    let mut undo_ref = None;
    if cap.needs_undo() {
        match undo_marker(&inv.request_id) {
            Ok(u) => undo_ref = Some(u),
            Err(e) => {
                return Outcome::refused(json!({
                    "reason": "撤销点做不成，故不动手",
                    "error": e,
                }))
            }
        }
    }
    match provider.call(&inv.verb, &inv.params) {
        Ok(data) => {
            let o = Outcome::ok(data);
            match undo_ref {
                Some(u) => o.with_undo(u),
                None => o,
            }
        }
        Err(e) => {
            let code = crate::error::code_of(&e).unwrap_or("ext.world.Carrier.ProviderFailed");
            if code.ends_with("UnknownVerb") || code.ends_with("BadParam") {
                // 参数/动词层面的错属于"根本没动手"。
                let mut o = Outcome::refused(json!({ "reason": e }));
                if let Some(u) = undo_ref {
                    o = o.with_undo(u);
                }
                o
            } else {
                let mut o = Outcome::failed(-1, json!({ "reason": e }));
                if let Some(u) = undo_ref {
                    o = o.with_undo(u);
                }
                o
            }
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use crate::carrier::capd::Manifest;
    use crate::carrier::Invocation;

    fn manifest(raw: &str) -> Manifest {
        let c = Manifest::parse(raw, Path::new("t.json")).unwrap();
        Manifest::from_caps(vec![c])
    }

    fn inv(cap: &str, verb: &str, params: Value) -> Invocation {
        Invocation {
            capability: cap.to_string(),
            verb: verb.to_string(),
            request_id: "r-1".to_string(),
            params,
        }
    }

    fn no_confirm(_: &str, _: &Value) -> bool {
        false
    }
    fn yes_confirm(_: &str, _: &Value) -> bool {
        true
    }
    fn no_undo(_: &str) -> Result<Value, String> {
        Err("测试：未配置撤销点".to_string())
    }
    fn ok_undo(_: &str) -> Result<Value, String> {
        Ok(json!({"kind":"carrier-undo","path":"/tmp/x"}))
    }

    #[test]
    fn refuses_a_capability_not_in_the_manifest() {
        let m = manifest(r#"{"capability":"a.b","provider":"p","verbs":["x"]}"#);
        let mut reg = Registry::new();
        reg.add(Box::new(Job::default()));
        let o = execute(
            &m,
            &reg,
            &inv("not.declared", "x", json!({})),
            &no_confirm,
            &no_undo,
        );
        assert_eq!(o.result, crate::carrier::outcome::REFUSED);
        assert!(o.detail.to_string().contains("不在执行清单里"));
    }

    #[test]
    fn refuses_a_verb_not_allowed_by_the_manifest() {
        let m = manifest(r#"{"capability":"a.b","provider":"job","verbs":["start"]}"#);
        let mut reg = Registry::new();
        reg.add(Box::new(Job::default()));
        let o = execute(
            &m,
            &reg,
            &inv("a.b", "list", json!({})),
            &no_confirm,
            &no_undo,
        );
        assert_eq!(o.result, crate::carrier::outcome::REFUSED);
        assert!(o.detail.to_string().contains("动词未被该能力授权"));
    }

    #[test]
    fn refuses_when_no_provider_is_registered() {
        let m = manifest(r#"{"capability":"a.b","provider":"ghost","verbs":["x"]}"#);
        let reg = Registry::new();
        let o = execute(&m, &reg, &inv("a.b", "x", json!({})), &no_confirm, &no_undo);
        assert_eq!(o.result, crate::carrier::outcome::REFUSED);
        assert!(o.detail.to_string().contains("没有对应的执行器"));
    }

    #[test]
    fn refuses_when_confirmation_is_required_but_missing() {
        let m = manifest(
            r#"{"capability":"a.b","provider":"job","verbs":["list"],"confirm":"required"}"#,
        );
        let mut reg = Registry::new();
        reg.add(Box::new(Job::default()));
        let o = execute(
            &m,
            &reg,
            &inv("a.b", "list", json!({})),
            &no_confirm,
            &no_undo,
        );
        assert_eq!(o.result, crate::carrier::outcome::REFUSED);
        assert!(o.detail.to_string().contains("需要人确认"));
        // 确认给了 ⇒ 继续走到执行
        let o2 = execute(
            &m,
            &reg,
            &inv("a.b", "list", json!({})),
            &yes_confirm,
            &no_undo,
        );
        assert_eq!(o2.result, crate::carrier::outcome::OK);
    }

    #[test]
    fn refuses_to_act_when_the_undo_point_cannot_be_made() {
        let m = manifest(
            r#"{"capability":"a.b","provider":"job","verbs":["list"],"risk":"high","undo":"before-each"}"#,
        );
        let mut reg = Registry::new();
        reg.add(Box::new(Job::default()));
        let o = execute(
            &m,
            &reg,
            &inv("a.b", "list", json!({})),
            &yes_confirm,
            &no_undo,
        );
        assert_eq!(o.result, crate::carrier::outcome::REFUSED);
        assert!(o.detail.to_string().contains("撤销点做不成"));
        // 撤销点做成了 ⇒ 结果里必须带上内容引用
        let o2 = execute(
            &m,
            &reg,
            &inv("a.b", "list", json!({})),
            &yes_confirm,
            &ok_undo,
        );
        assert_eq!(o2.result, crate::carrier::outcome::OK);
        assert!(o2.undo_ref.is_some());
    }

    #[test]
    fn backlight_refuses_rather_than_pretending() {
        // 指向一个不存在的设备根：必须报 NoDevice，**不得**返回编造的亮度
        let b = Backlight {
            root: PathBuf::from("/definitely/not/here"),
        };
        let e = b.call("get", &json!({})).unwrap_err();
        assert!(e.contains("NoDevice"), "{e}");
    }

    #[test]
    fn backlight_percent_scale_is_explicit_not_guessed() {
        // scale 缺失 ⇒ 按 raw；给了 percent 才换算
        assert_eq!(Backlight::to_raw(3, 100, &json!({})).unwrap(), 3);
        assert_eq!(
            Backlight::to_raw(3, 100, &json!({"scale":"percent"})).unwrap(),
            3
        );
        assert_eq!(
            Backlight::to_raw(50, 255, &json!({"scale":"percent"})).unwrap(),
            128
        );
        assert!(Backlight::to_raw(3, 100, &json!({"scale":"percent"})).is_ok());
        assert!(Backlight::to_raw(101, 255, &json!({"scale":"percent"})).is_err());
        assert!(Backlight::to_raw(300, 255, &json!({})).is_err());
        assert!(Backlight::to_raw(1, 255, &json!({"scale":"nonsense"})).is_err());
    }
}
