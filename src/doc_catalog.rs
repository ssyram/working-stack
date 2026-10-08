use crate::model::WsError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn scan(root: &Path, directory: &Path, docs: &mut BTreeMap<String, PathBuf>) -> Result<(), WsError> {
    for entry in std::fs::read_dir(directory).map_err(WsError::Io)? {
        let entry = entry.map_err(WsError::Io)?;
        let path = entry.path();
        let kind = entry.file_type().map_err(WsError::Io)?;
        if kind.is_dir() {
            scan(root, &path, docs)?;
        } else if kind.is_file() && path.extension().is_some_and(|e| e == "md") {
            let relative = path.strip_prefix(root).unwrap();
            let name = relative.components()
                .map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
            docs.insert(name, path);
        }
    }
    Ok(())
}

fn documentation_root() -> Result<PathBuf, WsError> {
    if let Some(root) = std::env::var_os("WS_DOCUMENTATION_PATH") {
        return Ok(PathBuf::from(root));
    }
    if cfg!(debug_assertions) {
        Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs"))
    } else {
        let executable = std::env::current_exe().and_then(std::fs::canonicalize).map_err(WsError::Io)?;
        Ok(executable.parent().unwrap().join("documentation"))
    }
}

pub fn query(name: Option<&str>, list: bool) -> Result<String, WsError> {
    let root = documentation_root()?;
    let mut docs = BTreeMap::new();
    scan(&root, &root, &mut docs)?;
    if list || name.is_none() {
        let mut names = String::new();
        if docs.contains_key("ENTRY.md") { names.push_str("ENTRY.md  (<-- the entry file)\n"); }
        for name in docs.keys().filter(|name| name.as_str() != "ENTRY.md") {
            names.push_str(&format!("{name}\n"));
        }
        return Ok(names);
    }
    let name = name.unwrap();
    let name = if name.ends_with(".md") { name.to_string() } else { format!("{name}.md") };
    let path = docs.get(&name).ok_or_else(|| WsError::Usage(format!("文档不存在：{name}")))?;
    let mut content = std::fs::read_to_string(path).map_err(WsError::Io)?;
    if name == "ENTRY.md" {
        content.push_str("\n\n## 全量文档索引\n\n");
        for other in docs.keys().filter(|name| name.as_str() != "ENTRY.md") {
            content.push_str(&format!("- [`{other}`]({other}) — `working-stack doc {other}`\n"));
        }
    }
    Ok(content)
}
