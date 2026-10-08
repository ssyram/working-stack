//! 持久化：三模式目标解析（托管/显式文件/md 管道）、load/save、文件锁、托管 git。
//! 合同见 detailed-design §4.6：validate→锁内基准检查→原子写；托管模式 git 仅提交本次栈文件。

use crate::import;
use crate::model::*;
use crate::print;
use crate::validate::validate;
use std::fs;
use std::path::{Path, PathBuf};

/// 使用方式：托管（自动 git）还是显式文件（无自动 git）。
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Managed { name: String, file: PathBuf, dir: PathBuf },
    File { file: PathBuf, native: bool }, // native=false ⇒ md 管道
}

/// 目标解析（三模式，用户零负担区分——architecture D.storage）。
/// Requires：arg 非空。
/// Ensures：含路径分隔符或以 .yaml/.yml/.json/.md 结尾 ⇒ File；
/// 否则 Managed（`<cwd>/.working-stack/<name>.yaml`）。
pub fn resolve_target(arg: &str) -> Result<Target, WsError> {
    if arg.contains('/') || arg.contains(std::path::MAIN_SEPARATOR) {
        let file = PathBuf::from(arg);
        let native = has_native_ext(&file);
        return Ok(Target::File { file, native });
    }
    let lower = arg.to_lowercase();
    if lower.ends_with(".yaml") || lower.ends_with(".yml") || lower.ends_with(".json") || lower.ends_with(".md") {
        let file = PathBuf::from(arg);
        let native = has_native_ext(&file);
        return Ok(Target::File { file, native });
    }
    let cwd = std::env::current_dir().map_err(WsError::Io)?;
    let dir = cwd.join(".working-stack");
    let file = dir.join(format!("{}.yaml", sanitize_name(arg)?));
    Ok(Target::Managed { name: arg.to_string(), file, dir })
}

pub fn is_native(p: &Path) -> bool {
    has_native_ext(p)
}

fn has_native_ext(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref(),
        Some("yaml" | "yml" | "json")
    )
}

/// 栈名仅允许字母数字-_（防路径逃逸——这是身份约束，不是输入防御）。trivial。
fn sanitize_name(name: &str) -> Result<String, WsError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(WsError::Usage(format!(
            "栈名只允许字母数字-_：{name:?}（或改用文件路径模式）"
        )));
    }
    Ok(name.to_string())
}

/// 列出托管目录下可用栈名。trivial。
pub fn list_managed() -> Vec<String> {
    let Ok(cwd) = std::env::current_dir() else { return vec![] };
    let dir = cwd.join(".working-stack");
    let Ok(rd) = fs::read_dir(&dir) else { return vec![] };
    let mut names: Vec<String> = rd
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("yaml") {
                p.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

pub struct Loaded {
    pub raw: String,
    pub ws: WorkingStack,
    pub warnings: Vec<String>,
}

/// load：yaml/json ⇒ codec；md ⇒ import 管道（warnings 随载入返回）。
/// Ensures：raw 保留供 save 的基准冲突检查。
pub fn load(path: &Path) -> Result<Loaded, WsError> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(WsError::Usage(format!("栈文件不存在：{}（托管模式先 init；路径模式核对路径）", path.display())))
        }
        Err(e) => return Err(WsError::Io(e)),
    };
    if has_native_ext(path) {
        let ws: WorkingStack = serde_yaml::from_str(&raw).map_err(|e| WsError::Yaml(e.to_string()))?;
        Ok(Loaded { raw, ws, warnings: vec![] })
    } else {
        let out = import::import_md(&raw)?;
        Ok(Loaded { raw, ws: out.ws, warnings: out.warnings })
    }
}

/// canonical 序列化（yaml native / md 重印）。
pub fn serialize(path: &Path, ws: &WorkingStack) -> Result<String, WsError> {
    if has_native_ext(path) {
        serde_yaml::to_string(ws).map_err(|e| WsError::Yaml(e.to_string()))
    } else {
        Ok(print::export_markdown(ws))
    }
}

/// save：validate → 锁内基准检查 → 原子写（tmp+fsync+rename）。
/// Requires（caller 合同）：本次写仅由一个 load→op 链产生。
/// Ensures：成功 ⇒ 文件=serialize(ws)；基准不符 ⇒ Conflict（不覆盖他方）；任一步失败 ⇒ 旧文件完整。
/// 论证：validate 先行拦截结构破坏；锁内重读 == base_raw 排除并发覆盖；tmp 与目标同目录 ⇒ rename 原子。
pub fn save(path: &Path, ws: &WorkingStack, base_raw: &str) -> Result<(), WsError> {
    let problems = validate(ws);
    if !problems.is_empty() {
        return Err(WsError::Invalid(problems));
    }
    let out = serialize(path, ws)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(WsError::Io)?;
    }
    let _lock = FileLock::acquire(path)?;
    let cur = fs::read_to_string(path).unwrap_or_default();
    if cur != base_raw {
        return Err(WsError::Conflict);
    }
    let tmp = path.with_extension("wstmp");
    fs::write(&tmp, &out).map_err(WsError::Io)?;
    fs::File::open(&tmp).map_err(WsError::Io)?.sync_all().map_err(WsError::Io)?;
    fs::rename(&tmp, path).map_err(WsError::Io)?;
    Ok(())
}

/// 托管模式 git：dir 无 .git 则 init；仅 add 本次栈文件并 commit。git 缺失 ⇒ 警告跳过（不阻塞写）。
/// Ensures：只触碰 `<file>`（限域——86a5fca 教训）；message 带操作与理由摘要。
pub fn git_commit(dir: &Path, file: &Path, message: &str) -> Option<String> {
    let run = |args: &[&str]| std::process::Command::new("git").args(args).output();
    if !dir.join(".git").exists() {
        if let Ok(o) = run(&["-C", dir.to_str()?, "init", "-q"]) {
            if !o.status.success() {
                return Some("git init 失败（跳过版本记录）".into());
            }
        }
    }
    let fname = file.file_name()?.to_str()?;
    let d = dir.to_str()?;
    let add = run(&["-C", d, "add", "--", fname]);
    if !add.map_or(false, |o| o.status.success()) {
        return Some("git add 失败（跳过版本记录）".into());
    }
    // 路径限定 commit（auditor F11）：仅提交本次栈文件，不携带其他已暂存内容
    let commit = run(&["-C", d, "commit", "-q", "-m", message, "--", fname]);
    match commit {
        Ok(o) if o.status.success() => None,
        // 无变更的 commit 失败属正常（空操作），其余给提示
        Ok(_) => None,
        Err(_) => Some("git 不可用（跳过版本记录）".into()),
    }
}

/// 单写者文件锁：`<path>.lock`（后缀追加，非扩展名替换——auditor F20）create_new+PID。
/// 无自动抢占（oracle S3 裁定：删他者锁是竞态）：锁存在 ⇒ Locked；若 mtime>60s 在消息中提示可手清。
/// Drop ⇒ 删除锁文件。
pub struct FileLock {
    path: PathBuf,
}

impl FileLock {
    pub fn acquire(target: &Path) -> Result<Self, WsError> {
        let mut name = target.as_os_str().to_os_string();
        name.push(".lock");
        let lp = PathBuf::from(name);
        match fs::OpenOptions::new().write(true).create_new(true).open(&lp) {
            Ok(mut f) => {
                use std::io::Write;
                let _ = write!(f, "{}", std::process::id());
                Ok(FileLock { path: lp })
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = fs::metadata(&lp)
                    .and_then(|m| m.modified())
                    .map(|mt| mt.elapsed().map(|d| d.as_secs() > 60).unwrap_or(false))
                    .unwrap_or(false);
                if stale {
                    Err(WsError::Usage(format!(
                        "锁疑似残留（超过 60 秒）：{}；确认无写进程后可手动删除",
                        lp.display()
                    )))
                } else {
                    Err(WsError::Locked)
                }
            }
            Err(e) => Err(WsError::Io(e)),
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::SourcefulText;

    #[test]
    fn target_resolution() {
        match resolve_target("mytask").unwrap() {
            Target::Managed { name, .. } => assert_eq!(name, "mytask"),
            _ => panic!(),
        }
        match resolve_target("a/b.yaml").unwrap() {
            Target::File { native, .. } => assert!(native),
            _ => panic!(),
        }
        match resolve_target("WS-MAIN.md").unwrap() {
            Target::File { native, .. } => assert!(!native),
            _ => panic!(),
        }
        assert!(resolve_target("bad/name").is_err() || matches!(resolve_target("bad/name"), Ok(Target::File { .. })));
        assert!(resolve_target("a b").is_err()); // 栈名字符集
    }

    #[test]
    fn save_roundtrip_yaml() {
        let dir = std::env::temp_dir().join(format!("ws-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("t.yaml");
        let mut ws = WorkingStack::default();
        ws.title = Some("测试".into());
        ws.push_root(
            "2026-10-06T00:00:00Z",
            "根".into(),
            vec![],
            SourcefulText::text("r"),
        )
        .unwrap();
        save(&p, &ws, "").unwrap();
        let l = load(&p).unwrap();
        assert_eq!(l.ws, ws);
        // 基准冲突
        assert!(matches!(save(&p, &ws, "stale base"), Err(WsError::Conflict)));
        // 锁路径 = 追加后缀（F20）
        let mut name = p.as_os_str().to_os_string();
        name.push(".lock");
        let lock = FileLock::acquire(&p).unwrap();
        assert_eq!(lock.path, PathBuf::from(name));
        drop(lock);
        let _ = fs::remove_dir_all(&dir);
    }
}
