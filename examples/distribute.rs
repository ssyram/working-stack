use clap::{Parser, Subcommand};
use std::{env, error::Error, fs, path::{Path, PathBuf}, process::Command};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(about = "Working Stack 跨平台安装与发布目录生成")]
struct Options {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Cargo 用户级安装，并复制伴随 documentation
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// 构建 release，生成二进制与 documentation 目录
    Package {
        directory: PathBuf,
        #[arg(long)]
        target: Option<String>,
    },
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn cargo(arguments: &[String]) -> Result<()> {
    let status = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(arguments).status()?;
    if !status.success() { return Err(format!("Cargo 执行失败：{status}").into()); }
    Ok(())
}

fn install_root() -> Result<PathBuf> {
    if let Some(root) = env::var_os("CARGO_HOME") { return Ok(root.into()); }
    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    Ok(PathBuf::from(env::var_os(variable).ok_or("无法确定用户目录；请传 --root")?).join(".cargo"))
}

fn run(options: Options) -> Result<()> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = source.join("Cargo.toml").display().to_string();
    let target_dir = source.join("target").display().to_string();
    match options.action {
        Action::Install { root } => {
            let root = match root { Some(root) => root, None => install_root()? };
            cargo(&["install".into(), "--path".into(), source.display().to_string(), "--locked".into(),
                "--bin".into(), "working-stack".into(), "--root".into(), root.display().to_string(),
                "--target-dir".into(), target_dir])?;
            let bin = root.join("bin");
            copy_tree(&source.join("docs"), &bin.join("documentation"))?;
            copy_tree(&source.join("skills"), &bin.join("skills"))?;
            println!("已安装：{}", bin.display());
            println!("该目录需在 PATH 中。使用入口：working-stack doc ENTRY");
        }
        Action::Package { directory, target } => {
            fs::create_dir(&directory)?;
            let mut arguments = vec!["build".into(), "--release".into(), "--locked".into(),
                "--bin".into(), "working-stack".into(), "--manifest-path".into(), manifest,
                "--target-dir".into(), target_dir];
            if let Some(target) = &target { arguments.extend(["--target".into(), target.clone()]); }
            cargo(&arguments)?;
            let mut release = source.join("target");
            if let Some(target) = &target { release.push(target); }
            release.push("release");
            let windows = target.as_ref().map(|t| t.contains("windows")).unwrap_or(cfg!(windows));
            let name = if windows { "working-stack.exe" } else { "working-stack" };
            fs::copy(release.join(name), directory.join(name))?;
            copy_tree(&source.join("docs"), &directory.join("documentation"))?;
            copy_tree(&source.join("skills"), &directory.join("skills"))?;
            println!("发布目录：{}", directory.display());
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run(Options::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
