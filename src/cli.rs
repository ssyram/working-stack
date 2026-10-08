//! CLI：动词 = 操作面（AI 查询面含 overview/focus）；三模式目标解析；--json 供 AI 消费。

use crate::diff::diff;
use crate::model::*;
use crate::persist::{self, Target};
use crate::print;
use crate::text::{parse_st, SourcefulText};
use chrono::Utc;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// FNV-1a 64 版本指纹（无外部依赖；web 亦用）。
pub fn fnv_public(s: &str) -> String {
    version_of(s)
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
}

/// FNV-1a 64 版本指纹（无外部依赖）。
fn version_of(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

#[derive(Parser)]
#[command(name = "working-stack", about = "Working Stack：用户拥有、人/AI 共用的工作状态树",
    after_help = "使用入口：working-stack doc ENTRY\n文档列表：working-stack doc --list")]
struct Cli {
    /// 栈目标：STACK-NAME（托管 <cwd>/.working-stack/，自动 git）或文件路径（*.yaml/*.json native；*.md 解析管道）
    stack: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
    /// 机器可读 JSON 输出（AI 消费）
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum ReopenMode {
    Active,
    Paused,
}

#[derive(Subcommand)]
enum Cmd {
    /// 建栈（托管模式建 .working-stack/<name>.yaml 并 git init）
    Init {
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        prefix: Option<String>,
    },
    /// 新建子帧（parent 省略 = 建根帧）
    Push {
        parent: Option<String>,
        #[arg(long)]
        title: String,
        #[arg(long)]
        goal: Vec<String>,
        #[arg(long)]
        origin: Vec<String>,
        #[arg(short = 'r')]
        reason: String,
    },
    /// 关闭帧（默认 done；--expired 为回合结束）
    Pop {
        id: String,
        #[arg(long)]
        expired: bool,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        residual: Vec<String>,
        #[arg(long)]
        pos: Vec<String>,
        #[arg(long)]
        focus_to: Option<String>,
        #[arg(short = 'r')]
        reason: String,
    },
    Pause {
        id: String,
        #[arg(short = 'r')]
        reason: String,
    },
    Resume {
        id: String,
        #[arg(short = 'r')]
        reason: String,
    },
    /// resume 的别名（切焦点）
    Switch {
        id: String,
        #[arg(short = 'r')]
        reason: String,
    },
    Update {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        goal: Vec<String>,
        #[arg(long)]
        origin: Vec<String>,
        #[arg(long)]
        result_add: Vec<String>,
        #[arg(long)]
        side_add: Vec<String>,
        #[arg(long)]
        principle_add: Vec<String>,
        /// idx=kept_local|release:ID|invalidated:TEXT|reclass:results|reclass:remark:TITLE
        #[arg(long)]
        settle: Vec<String>,
        /// title=body
        #[arg(long)]
        remark: Vec<String>,
        #[arg(long)]
        plan_add: Vec<String>,
        /// path=state（path 如 0.1；state ∈ ready|in_progress|blocked|done|expire[:TEXT]）
        #[arg(long)]
        plan_set: Vec<String>,
        /// path=text（改计划项正文，内联编辑通道）
        #[arg(long)]
        plan_text: Vec<String>,
        /// path=text（给 path 处的项追加内嵌子项）
        #[arg(long)]
        plan_add_child: Vec<String>,
        #[arg(long, value_enum)]
        reopen: Option<ReopenMode>,
        #[arg(short = 'r')]
        reason: String,
    },
    Note {
        key: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long)]
        meta: Vec<String>,
        #[arg(short = 'r')]
        reason: Option<String>,
    },
    /// 汇总所有帧及栈级日志，按真实时间输出 Markdown / JSON
    Timeline,
    /// 按路径读取实际 JSON 值
    Get { path: String },
    /// 按路径替换内容；value 为 JSON
    Set { path: String, #[arg(long)] value: String, #[arg(short = 'r')] reason: String },
    /// 在列表下标前插入；直接指向列表为追加
    Insert { path: String, #[arg(long)] value: String, #[arg(short = 'r')] reason: String },
    /// 删除列表项或内容映射成员
    Remove { path: String, #[arg(short = 'r')] reason: String },
    /// 移动内容；目标按删除来源后的结构解释
    Move { path: String, #[arg(long)] from: String, #[arg(short = 'r')] reason: String },
    /// 查询：tree|frame|log|timeline|notes|pops|overview|focus
    Show { what: String, id: Option<String> },

    Validate,
    Export {
        format: String,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// 家族 md → 栈文件（-o 目标：托管名或路径）
    Import {
        md: PathBuf,
        #[arg(short = 'o')]
        out: String,
    },
    /// 整文件候选提交（yaml/md；净差异生成事件）
    Apply {
        candidate: PathBuf,
        #[arg(short = 'r')]
        reason: String,
    },
    /// $EDITOR 编辑 canonical 文本后提交（apply 同通道）
    Edit {
        #[arg(short = 'r')]
        reason: String,
    },
    /// 打开网页视图（同核心库 API）
    Web {
        #[arg(long, default_value_t = 7788)]
        port: u16,
    },
    /// 动态列举或读取仓库 docs（支持嵌套路径，可省略 .md）
    Doc {
        name: Option<String>,
        #[arg(long)]
        list: bool,
    },
    /// 列托管栈
    List,
}

fn st(s: &str) -> Result<SourcefulText, WsError> {
    parse_st(s).map_err(WsError::Parse)
}

fn sts(v: &[String]) -> Result<Vec<SourcefulText>, WsError> {
    v.iter().map(|s| st(s)).collect()
}

struct Out {
    ok: bool,
    diff: Vec<String>,
    version: Option<String>,
    warnings: Vec<String>,
    text: Option<String>,
}

pub fn run(args: &[String]) -> Result<i32, WsError> {
    if args.is_empty() {
        Cli::command().print_help().map_err(WsError::Io)?;
        println!();
        return Ok(0);
    }
    let cli = match Cli::try_parse_from(std::iter::once("working-stack".to_string()).chain(args.iter().cloned())) {
        Ok(cli) => cli,
        Err(e) if matches!(e.kind(), clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion) => {
            e.print().map_err(WsError::Io)?;
            return Ok(0);
        }
        Err(e) => return Err(WsError::Usage(e.to_string())),
    };
    let json = cli.json;

    if let Cmd::Doc { name, list } = &cli.cmd {
        let text = crate::doc_catalog::query(name.as_deref(), *list)?;
        if json {
            emit(&Out { ok: true, diff: vec![], version: None, warnings: vec![], text: Some(text) }, true, false);
        } else {
            use std::io::Write;
            std::io::stdout().write_all(text.as_bytes()).map_err(WsError::Io)?;
        }
        return Ok(0);
    }
    if matches!(cli.cmd, Cmd::List) {
        let names = persist::list_managed();
        if json {
            println!("{}", serde_json::json!({"ok": true, "stacks": names}));
        } else {
            for n in names {
                println!("{n}");
            }
        }
        return Ok(0);
    }
    if matches!(cli.cmd, Cmd::Import { .. }) {
        if let Cmd::Import { md, out } = &cli.cmd {
            return import_cmd(md, out, json).map(|_| 0);
        }
        unreachable!();
    }

    let target = persist::resolve_target(
        cli.stack
            .as_deref()
            .ok_or_else(|| WsError::Usage("缺少栈目标（STACK-NAME 或文件路径）".into()))?,
    )?;
    let file = match &target {
        Target::Managed { file, .. } => file.clone(),
        Target::File { file, .. } => file.clone(),
    };
    let managed = matches!(target, Target::Managed { .. });

    if let Cmd::Init { title, prefix } = &cli.cmd {
        return init_cmd(&target, file, title.clone(), prefix.clone(), json);
    }
    if let Cmd::Web { port } = &cli.cmd {
        crate::web::serve(
            cli.stack.as_deref().unwrap_or(""),
            &file,
            *port,
        );
    }

    let loaded = persist::load(&file)?;
    let mut ws = loaded.ws.clone();
    let old = loaded.ws.clone();
    let at = now();
    let reason_of = |r: &str| st(r);
    let mut out = Out { ok: true, diff: vec![], version: None, warnings: loaded.warnings.clone(), text: None };

    match &cli.cmd {
        Cmd::Push { parent, title, goal, origin, reason } => {
            let r = reason_of(reason)?;
            let id = match parent {
                Some(p) => {
                    let pid = ws.parse_id(p).ok_or_else(|| WsError::Usage(format!("父帧 id 不合法：{p}")))?;
                    ws.push(&at, pid, title.clone(), sts(goal)?, sts(origin)?, r)?
                }
                None => {
                    let id = ws.push_root(&at, title.clone(), sts(goal)?, r.clone())?;
                    ws.set_origin(&at, id, sts(origin)?, r)?;
                    id
                },
            };
            out.diff.push(format!("新建帧 {} {}", ws.print_id(id), ws.title_of(id)));
        }
        Cmd::Pop { id, expired, summary, residual, pos, focus_to, reason } => {
            let pid = ws.parse_id(id).ok_or_else(|| WsError::Usage(format!("id 不合法：{id}")))?;
            let mut cl = Closure { summary: st(summary)?, ..Default::default() };
            for p in pos {
                cl.positive.push(ClosureItem { result: st(p)?, evidence: vec![].into() });
            }
            for r in residual {
                cl.residuals.push(ClosureItem { result: st(r)?, evidence: vec![].into() });
            }
            let outcome = if *expired { Outcome::ExpiredO(cl) } else { Outcome::DoneO(cl) };
            let ft = match focus_to {
                Some(f) => Some(ws.parse_id(f).ok_or_else(|| WsError::Usage(format!("focus_to 不合法：{f}")))?),
                None => None,
            };
            ws.pop(&at, pid, outcome, ft, reason_of(reason)?)?;
        }
        Cmd::Pause { id, reason } => {
            let pid = ws.parse_id(id).ok_or_else(|| WsError::Usage("id 不合法".into()))?;
            ws.pause(&at, pid, reason_of(reason)?)?;
        }
        Cmd::Resume { id, reason } | Cmd::Switch { id, reason } => {
            let pid = ws.parse_id(id).ok_or_else(|| WsError::Usage("id 不合法".into()))?;
            ws.resume(&at, pid, reason_of(reason)?)?;
        }
        Cmd::Update { id, title, goal, origin, result_add, side_add, principle_add, settle, remark, plan_add, plan_set, plan_text, plan_add_child, reopen, reason } => {
            let pid = ws.parse_id(id).ok_or_else(|| WsError::Usage("id 不合法".into()))?;
            let r = reason_of(reason)?;
            if let Some(t) = title {
                ws.set_title(&at, pid, t.clone(), r.clone())?;
            }
            if !goal.is_empty() {
                let new_goal = if goal.len() == 1 && goal[0].trim().is_empty() {
                    vec![] // 单空串 = 清空 goal（web 编辑清空通道）
                } else {
                    sts(goal)?
                };
                ws.set_goal(&at, pid, new_goal, r.clone())?;
            }
            if !origin.is_empty() {
                let new_origin = if origin.len() == 1 && origin[0].is_empty() { vec![] } else { sts(origin)? };
                ws.set_origin(&at, pid, new_origin, r.clone())?;
            }
            for x in result_add {
                ws.result_add(&at, pid, st(x)?, r.clone())?;
            }
            for x in side_add {
                ws.side_effect_add(&at, pid, st(x)?, r.clone())?;
            }
            for x in principle_add {
                ws.principle_add(&at, pid, st(x)?, r.clone())?;
            }
            for x in remark {
                let (t, b) = x.split_once('=').ok_or_else(|| WsError::Usage(format!("remark 需 title=body：{x}")))?;
                ws.remark_set(&at, pid, t.to_string(), st(b)?, r.clone())?;
            }
            for x in plan_add {
                ws.plan_add(&at, pid, st(x)?, ItemState::Ready, r.clone())?;
            }
            for x in plan_set {
                let (p, s) = x.split_once('=').ok_or_else(|| WsError::Usage(format!("plan_set 需 path=state：{x}")))?;
                let path: Vec<usize> = p.split('.').map(|n| n.parse()).collect::<Result<_, _>>().map_err(|_| WsError::Usage("plan 路径须为点分下标".into()))?;
                let state = parse_state(s)?;
                ws.plan_set(&at, pid, &path, state, r.clone())?;
            }
            for x in plan_add_child {
                let (p, t) = x.split_once('=').ok_or_else(|| WsError::Usage(format!("plan_add_child 需 path=text：{x}")))?;
                let path: Vec<usize> = p.split('.').map(|n| n.parse()).collect::<Result<_, _>>().map_err(|_| WsError::Usage("plan 路径须为点分下标".into()))?;
                ws.plan_add_child(&at, pid, &path, st(t)?, r.clone())?;
            }
            for x in plan_text {
                let (p, t) = x.split_once('=').ok_or_else(|| WsError::Usage(format!("plan_text 需 path=text：{x}")))?;
                let path: Vec<usize> = p.split('.').map(|n| n.parse()).collect::<Result<_, _>>().map_err(|_| WsError::Usage("plan 路径须为点分下标".into()))?;
                ws.plan_text(&at, pid, &path, st(t)?, r.clone())?;
            }
            for x in settle {
                let (i, f) = x.split_once('=').ok_or_else(|| WsError::Usage(format!("settle 需 idx=fate：{x}")))?;
                let idx: usize = i.parse().map_err(|_| WsError::Usage("settle 下标须数字".into()))?;
                let fate = parse_fate(f, &ws)?;
                ws.settle(&at, pid, idx, fate, r.clone())?;
            }
            if let Some(mode) = reopen {
                let pr = match mode {
                    ReopenMode::Paused => Some(st("（reopen 后待定）")?),
                    ReopenMode::Active => None,
                };
                ws.reopen(&at, pid, pr, r)?;
            }
        }
        Cmd::Note { key, body, meta, reason } => {
            match body {
                Some(b) => {
                    ws.note_put(&at, key, meta.clone(), st(b)?, reason_of(reason.as_deref().unwrap_or("note 更新"))?)?;
                }
                None => {
                    let n = ws.notes.get(key).ok_or_else(|| WsError::Usage(format!("note 不存在：{key}")))?;
                    out.text = Some(ws.render(&n.body));
                    emit(&out, json, false);
                    return Ok(0);
                }
            }
        }
        Cmd::Timeline => {
            if json {
                println!("{}", serde_json::json!({"ok":true,"events":crate::timeline::collect(&ws)}));
            } else {
                print!("{}", crate::timeline::markdown(&ws));
            }
            return Ok(0);
        }
        Cmd::Get { path } => {
            let value = crate::path_edit::get(&ws, path)?;
            if json {
                println!("{}", serde_json::json!({"ok":true,"path":path,"value":value,"version":version_of(&loaded.raw)}));
            } else {
                println!("{}", serde_json::to_string_pretty(&value).map_err(|e| WsError::Parse(e.to_string()))?);
            }
            return Ok(0);
        }
        Cmd::Set { .. } | Cmd::Insert { .. } | Cmd::Remove { .. } | Cmd::Move { .. } => {
            use crate::path_edit::Edit;
            let parse = |text: &str| serde_json::from_str(text).map_err(|e| WsError::Parse(e.to_string()));
            let operation = match &cli.cmd {
                Cmd::Set {path,value,..} => Edit::Set {path:path.clone(),value:parse(value)?},
                Cmd::Insert {path,value,..} => Edit::Insert {path:path.clone(),value:parse(value)?},
                Cmd::Remove {path,..} => Edit::Remove {path:path.clone()},
                Cmd::Move {path,from,..} => Edit::Move {path:path.clone(),from:from.clone()},
                _ => unreachable!(),
            };
            let new = crate::path_edit::edit(&ws, operation)?;
            apply_events(&mut ws, &old, &new, &at, &reason_text(&cli.cmd))?;
            let problems = WorkingStack::apply_checks(&old, &ws);
            if !problems.is_empty() { return Err(WsError::Invalid(problems)); }
        }
        Cmd::Show { what, id } => {
            out.text = Some(show(&ws, what, id.as_deref())?);
            emit(&out, json, false);
            return Ok(0);
        }
        Cmd::Validate => {
            let problems = crate::validate::validate(&ws);
            if problems.is_empty() {
                out.text = Some("OK".into());
            } else {
                out.ok = false;
                out.diff = problems;
            }
            emit(&out, json, false);
            return Ok(if out.ok { 0 } else { 2 });
        }
        Cmd::Export { format, out: o } => {
            let content = match format.as_str() {
                "md" | "markdown" => print::export_markdown(&ws),
                "json" => serde_json::to_string_pretty(&ws).map_err(|e| WsError::Yaml(e.to_string()))?,
                "yaml" => persist::serialize(&file, &ws)?,
                other => return Err(WsError::Usage(format!("导出格式：md|json|yaml（未知 {other}）"))),
            };
            match o {
                Some(p) => std::fs::write(p, &content).map_err(WsError::Io)?,
                None => {
                    out.text = Some(content);
                }
            }
            emit(&out, json, false);
            return Ok(0);
        }
        Cmd::Apply { candidate, reason } => {
            let new = load_candidate(candidate)?;
            apply_events(&mut ws, &old, &new, &at, reason)?;
            let problems = WorkingStack::apply_checks(&old, &ws);
            if !problems.is_empty() {
                return Err(WsError::Invalid(problems));
            }
        }
        Cmd::Edit { reason } => {
            let cur = persist::serialize(&file, &ws)?;
            let ext = if persist::is_native(&file) { "yaml" } else { "md" };
            let tmp = std::env::temp_dir().join(format!("ws-edit-{}.{ext}", std::process::id()));
            std::fs::write(&tmp, &cur).map_err(WsError::Io)?;
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
            let status = std::process::Command::new(editor).arg(&tmp).status().map_err(WsError::Io)?;
            if !status.success() {
                let _ = std::fs::remove_file(&tmp);
                return Err(WsError::Usage("编辑器退出非零，放弃提交".into()));
            }
            let edited = std::fs::read_to_string(&tmp).map_err(WsError::Io)?;
            let _ = std::fs::remove_file(&tmp);
            let new = load_candidate_str(&edited, ext)?;
            apply_events(&mut ws, &old, &new, &at, reason)?;
            let problems = WorkingStack::apply_checks(&old, &ws);
            if !problems.is_empty() {
                return Err(WsError::Invalid(problems));
            }
        }
        _ => unreachable!(),
    }

    if matches!(cli.cmd, Cmd::Update { .. }) {
        crate::commit_log::coalesce_update(&mut ws, &old, &at);
    }
    // 提交：diff → save →（托管）git
    out.diff.extend(diff(&old, &ws));
    let serialized = persist::serialize(&file, &ws)?;
    out.version = Some(version_of(&serialized));
    persist::save(&file, &ws, &loaded.raw)?;
    if managed {
        if let Target::Managed { dir, ref file, ref name } = &target {
            let msg = format!("ws: {} {}", name, first_line_of(&reason_text(&cli.cmd)));
            if let Some(w) = persist::git_commit(&dir, file, &msg) {
                out.warnings.push(w);
            }
        }
    }
    let native = persist::is_native(&file);
    emit(&out, json, !native);
    Ok(0)
}

fn first_line_of(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(60).collect()
}

fn reason_text(cmd: &Cmd) -> String {
    match cmd {
        Cmd::Push { reason, .. }
        | Cmd::Pop { reason, .. }
        | Cmd::Pause { reason, .. }
        | Cmd::Resume { reason, .. }
        | Cmd::Switch { reason, .. }
        | Cmd::Update { reason, .. }
        | Cmd::Set { reason, .. }
        | Cmd::Insert { reason, .. }
        | Cmd::Remove { reason, .. }
        | Cmd::Move { reason, .. }
        | Cmd::Apply { reason, .. }
        | Cmd::Edit { reason } => reason.clone(),
        Cmd::Note { reason, .. } => reason.clone().unwrap_or_else(|| "note 更新".into()),
        _ => String::new(),
    }
}

impl WorkingStack {
    fn title_of(&self, id: Id) -> String {
        self.frames.get(&id).map(|f| f.title.clone()).unwrap_or_default()
    }
}

fn emit(out: &Out, json: bool, md_notice: bool) {
    if json {
        let mut warnings = out.warnings.clone();
        if md_notice {
            warnings.push("md 管道：本次写入重印为 canonical 导出形（原文排版不保留）".into());
        }
        println!(
            "{}",
            serde_json::json!({
                "ok": out.ok,
                "diff": out.diff,
                "version": out.version,
                "warnings": warnings,
                "text": out.text,
            })
        );
    } else {
        if let Some(t) = &out.text {
            println!("{t}");
        }
        for d in &out.diff {
            println!("· {d}");
        }
        if let Some(v) = &out.version {
            println!("已提交 {v}");
        }
        if md_notice {
            eprintln!("（md 管道：本次写入重印为 canonical 导出形）");
        }
        for w in &out.warnings {
            eprintln!("⚠ {w}");
        }
    }
}

pub(crate) fn parse_state(s: &str) -> Result<ItemState, WsError> {
    match s {
        "ready" => Ok(ItemState::Ready),
        "in_progress" => Ok(ItemState::InProgress),
        "blocked" => Ok(ItemState::Blocked { reason: SourcefulText::default() }),
        "done" => Ok(ItemState::ItemDone),
        s if s.starts_with("blocked:") => Ok(ItemState::Blocked { reason: st(&s[8..])? }),
        s if s.starts_with("expire:") => Ok(ItemState::ItemExpired { reason: st(&s[7..])? }),
        "expire" => Ok(ItemState::ItemExpired { reason: SourcefulText::default() }),
        other => Err(WsError::Usage(format!("未知计划状态：{other}"))),
    }
}

fn parse_fate(s: &str, ws: &WorkingStack) -> Result<Fate, WsError> {
    if s == "kept_local" {
        return Ok(Fate::KeptLocal);
    }
    if let Some(id) = s.strip_prefix("release:") {
        let to = ws.parse_id(id).ok_or_else(|| WsError::Usage(format!("release 目标不合法：{id}")))?;
        return Ok(Fate::ReleasedTo { id: to });
    }
    if let Some(r) = s.strip_prefix("invalidated:") {
        return Ok(Fate::Invalidated { reason: st(r)? });
    }
    if s == "reclass:results" {
        return Ok(Fate::Reclassified { target: ReclassTarget::Results });
    }
    if let Some(t) = s.strip_prefix("reclass:remark:") {
        return Ok(Fate::Reclassified { target: ReclassTarget::Remark { title: t.to_string() } });
    }
    Err(WsError::Usage(format!("未知 fate：{s}")))
}

fn load_candidate(p: &PathBuf) -> Result<WorkingStack, WsError> {
    let raw = std::fs::read_to_string(p).map_err(WsError::Io)?;
    let native = persist::is_native(p);
    load_candidate_str(&raw, if native { "yaml" } else { "md" })
}

fn load_candidate_str(raw: &str, ext: &str) -> Result<WorkingStack, WsError> {
    if ext == "yaml" || ext == "json" {
        serde_yaml::from_str(raw).map_err(|e| WsError::Yaml(e.to_string()))
    } else {
        Ok(crate::import::import_md(raw)?.ws)
    }
}

/// apply 净差异事件：新帧 Push(reason)，变更帧 Update(reason)（不虚构中间操作）。
fn apply_events(ws: &mut WorkingStack, old: &WorkingStack, new: &WorkingStack, at: &str, reason: &str) -> Result<(), WsError> {
    let r = st(reason)?;
    if r.is_empty() { return Err(WsError::Usage("reason 必填".into())); }
    let notes_changed = old.notes != new.notes;
    let mut events: BTreeMap<Id, Event> = BTreeMap::new();
    for (id, nf) in &new.frames {
        match old.frames.get(id) {
            None => {
                events.insert(*id, Event { action: Action::Push, at: at.into(), detail: r.clone() });
            }
            Some(of) => {
                let changed = of.title != nf.title
                    || of.goal != nf.goal
                    || of.origin != nf.origin
                    || of.status != nf.status
                    || of.plan != nf.plan
                    || of.content != nf.content;
                if changed {
                    let changed = crate::commit_log::fields(of, nf, false);
                    events.insert(*id, Event { action: Action::Update, at: at.into(), detail: crate::commit_log::detail(&r, &changed) });
                }
            }
        }
    }
    *ws = new.clone();
    if notes_changed {
        let changed = crate::commit_log::notes(old, new);
        ws.log.push(Event { action: Action::Update, at: at.into(), detail: crate::commit_log::detail(&r, &changed) });
    }
    for (id, e) in events {
        if let Some(f) = ws.frames.get_mut(&id) {
            f.log.push(e);
        }
    }
    Ok(())
}

fn show(ws: &WorkingStack, what: &str, id: Option<&str>) -> Result<String, WsError> {
    match what {
        "tree" => Ok(print::tree_text(ws)),
        "frame" => {
            let fid = id.ok_or_else(|| WsError::Usage("show frame 需要 id".into()))?;
            let pid = ws.parse_id(fid).ok_or_else(|| WsError::Usage("id 不合法".into()))?;
            if !ws.frames.contains_key(&pid) {
                return Err(WsError::Usage("帧不存在".into()));
            }
            Ok(print::frame_section(ws, pid))
        }
        "log" => {
            let fid = id.ok_or_else(|| WsError::Usage("show log 需要 id".into()))?;
            let pid = ws.parse_id(fid).ok_or_else(|| WsError::Usage("id 不合法".into()))?;
            let f = ws.frames.get(&pid).ok_or_else(|| WsError::Usage("帧不存在".into()))?;
            let mut s = String::new();
            for e in &f.log {
                s.push_str(&format!("- [{}] [{}] {}\n", e.action.label(), e.at, ws.render(&e.detail)));
            }
            Ok(s)
        }
        "timeline" => Ok(crate::timeline::markdown(ws)),
        "notes" => {
            let mut s = String::new();
            for (k, n) in &ws.notes {
                let head: String = ws.render(&n.body).lines().next().unwrap_or("").chars().take(60).collect();
                s.push_str(&format!("- {}（meta={}）：{}\n", k, serde_json::to_string(&n.meta).unwrap(), head));
            }
            Ok(s)
        }
        "pops" => {
            let mut s = String::new();
            for (id, f) in ws.frames.iter().filter(|(_, f)| f.status.is_closed()).rev() {
                let summary = match &f.status {
                    FrameStatus::Done { closure } | FrameStatus::Expired { closure } => ws.render(&closure.summary),
                    _ => String::new(),
                };
                s.push_str(&format!("- [{}] [{}] {} —— {}\n", ws.print_id(*id), f.status.kind_str(), f.title, summary));
            }
            Ok(s)
        }
        "overview" => Ok(overview(ws)),
        "focus" => {
            let f = ws.focus().ok_or_else(|| WsError::Usage("当前无焦点（整线暂停或多焦点，见 show tree）".into()))?;
            let mut s = print::frame_section(ws, f);
            if let Some(it) = WorkingStack::focus_item_of(&ws.frames[&f]) {
                s.push_str(&format!("\n>> FOCUS：{}\n", ws.render(&it.text)));
            }
            Ok(s)
        }
        other => Err(WsError::Usage(format!("show 子命令：tree|frame|log|timeline|notes|pops|overview|focus（未知 {other}）"))),
    }
}

/// /recalibration 视角的总览：目标、路径、焦点、暂停分支、下一步。
fn overview(ws: &WorkingStack) -> String {
    let mut s = String::new();
    if let Some(t) = &ws.title {
        s.push_str(&format!("栈：{t}\n"));
    }
    for r in ws.live_roots() {
        let f = &ws.frames[&r];
        let goal = f.goal.first().map(|g| ws.render(g)).unwrap_or_default();
        let head: String = goal.lines().next().unwrap_or("").chars().take(70).collect();
        s.push_str(&format!("[{}] {} —— {}\n", ws.print_id(r), f.title, head));
    }
    match ws.focus() {
        Some(f) => {
            let path = ws.path(f);
            let ids: Vec<String> = path.iter().map(|i| ws.print_id(*i)).collect();
            s.push_str(&format!("焦点路径：{}\n", ids.join(" → ")));
            s.push_str(&format!("TOP：[{}] {}\n", ws.print_id(f), ws.frames[&f].title));
            if let Some(it) = WorkingStack::focus_item_of(&ws.frames[&f]) {
                let state = match &it.progress {
                    ItemState::InProgress => "in_progress".to_string(),
                    ItemState::Blocked { .. } => "blocked".to_string(),
                    _ => String::new(),
                };
                s.push_str(&format!("  FOCUS：[{}] {}\n", state, ws.render(&it.text)));
            }
        }
        None => s.push_str("焦点：无（整线暂停或异常多焦点——show tree 查看）\n"),
    }
    let paused: Vec<Id> = ws.tips().into_iter().filter(|t| matches!(ws.frames[t].status, FrameStatus::Paused { .. })).collect();
    if !paused.is_empty() {
        s.push_str("Paused 分支：\n");
        for t in paused {
            let (title, reason) = match &ws.frames[&t].status {
                FrameStatus::Paused { reason } => {
                    let rr: String = ws.render(reason).lines().next().unwrap_or("").chars().take(50).collect();
                    (ws.frames[&t].title.clone(), rr)
                }
                _ => (String::new(), String::new()),
            };
            s.push_str(&format!("  - [{}] {} —— {}\n", ws.print_id(t), title, reason));
        }
    }
    let popped = ws.frames.values().filter(|f| f.status.is_closed()).count();
    s.push_str(&format!("已 pop：{popped} 帧\n"));
    s
}

fn init_cmd(target: &Target, file: PathBuf, title: Option<String>, prefix: Option<String>, json: bool) -> Result<i32, WsError> {
    if file.exists() {
        return Err(WsError::Usage(format!("栈已存在：{}", file.display())));
    }
    let mut ws = WorkingStack::default();
    ws.title = title;
    ws.prefix = prefix;
    let serialized = persist::serialize(&file, &ws)?;
    let out = Out { ok: true, diff: vec!["初始化空栈".into()], version: Some(version_of(&serialized)), warnings: vec![], text: None };
    persist::save(&file, &ws, "")?;
    if let Target::Managed { dir, name, .. } = target {
        if let Some(w) = persist::git_commit(dir, &file, &format!("ws: init {name}")) {
            let mut o = out;
            o.warnings.push(w);
            emit(&o, json, false);
            return Ok(0);
        }
    }
    emit(&out, json, false);
    Ok(0)
}

fn import_cmd(md: &PathBuf, out_target: &str, json: bool) -> Result<(), WsError> {
    let raw = std::fs::read_to_string(md).map_err(WsError::Io)?;
    let parsed = crate::import::import_md(&raw)?;
    let target = persist::resolve_target(out_target)?;
    let file = match &target {
        Target::Managed { file, .. } => file.clone(),
        Target::File { file, .. } => file.clone(),
    };
    if file.exists() {
        return Err(WsError::Usage(format!("目标已存在：{}", file.display())));
    }
    let serialized = persist::serialize(&file, &parsed.ws)?;
    let mut warnings = parsed.warnings;
    // 产物必须过 validate（C4）；未过项入 warnings 而非静默
    let problems = crate::validate::validate(&parsed.ws);
    if !problems.is_empty() {
        return Err(WsError::Invalid(problems));
    }
    persist::save(&file, &parsed.ws, "")?;
    if let Target::Managed { dir, file, name } = &target {
        if let Some(w) = persist::git_commit(dir, file, &format!("ws: import {name}")) {
            warnings.push(w);
        }
    }
    emit(
        &Out { ok: true, diff: vec!["导入完成".into()], version: Some(version_of(&serialized)), warnings, text: None },
        json,
        false,
    );
    Ok(())
}

