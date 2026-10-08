# 文档入口、来源与分发（D.docs）

## Upstream / P

依据 Q.I.7：人/AI 从工具本身进入使用文档；目录列表自动生成，可指定 WS_DOCUMENTATION_PATH；debug 构建保留编译期默认路径，release 构建（包括本地安装及下载发布包）把文档放在二进制旁。不内嵌正文、不设固定 doc entry 别名、不写用户 Shell 配置。

- `working-stack` 无参数与 `--help` 都提示 `working-stack doc ENTRY`。
- 文档根约定实际入口文件为 `ENTRY.md`。`doc` 等同 `doc --list`；有入口时第一行是 `ENTRY.md  (<-- the entry file)`，其他行只有相对文件名（包含 .md），按名称自动排序。
- `doc NAME[.md]` 返回 UTF-8 原文，支持嵌套路径，不额外加标题或换行；ENTRY.md 是入口特例：原文后动态附加到所有其他文档的全量索引，含链接及 doc 指引。无栈参数，不创建栈或日志。
- 每次调用扫描所选目录，动态增减即时生效。

## M：来源选择 → 扫描 → 显示

1. 运行时有 `WS_DOCUMENTATION_PATH` 时，用它作为目录。用户指定的相对路径按当前工作目录解释。
2. 否则 debug 构建使用编译期 `CARGO_MANIFEST_DIR/docs`。
3. 否则 release 构建使用实际可执行文件所在目录的 `documentation/`，不是调用时 cwd；可执行路径先 canonicalize，symlink 入口跟随到真实二进制目录。

实现按 `cfg(debug_assertions)` 区分 Cargo 默认 dev/release profile；Cargo 的 debug 构建命令是 `cargo build`，没有常用 `--debug` 参数。自定义 profile 若改变 debug-assertions，其默认文档模式也随之改变。不会修改用户环境，也不进行目录失效后的其他来源回退。读取所选目录失败就按既有 IO 错误返回。

扫描建立 `相对文件名 → 路径` 有序映射。列举时先打印实际 ENTRY.md，其余有序遍历；读取时给没有 .md 后缀的请求补后缀，再在同一集合查找。未知名称按既有用法错误返回。JSON 成功响应仍用既有 text 字段。入口索引也遍历同一扫描结果，排除入口自身，不写回文件、不维护静态索引。

## 安装及发布合同

本地安装：`cargo run --locked --example distribute -- install [--root ROOT]`，由 Rust 安装器调用 Cargo 安装 release，再复制 docs 到安装目录 bin/documentation，并将附带 skill 复制到 bin/skills。默认用户级 `~/.cargo/bin`（CARGO_HOME 或 --root 可改），该目录在 PATH 才可裸命令调用。`install.sh` / `install.ps1` 仅引导工具链并调用同一 Rust 安装器，不承担目录生成逻辑。

裸 `cargo install --path . --locked` / `cargo install working-stack --locked` 默认 release，Cargo 自身只安装二进制，不复制文档。此路径须另将 docs 复制到二进制旁的 documentation，或设置 WS_DOCUMENTATION_PATH；不能声称裸 Cargo 命令已履行整包安装合同。

下载包：`cargo run --locked --example distribute -- package DIST-DIR [--target TARGET]` 构建 release，并复制二进制与完整 docs 到：

```text
DIST-DIR/
  working-stack[.exe]
  documentation/
    ENTRY.md
    ai-manual.md
    ...
  skills/
    working-stack/
      SKILL.md
```

输出目录须尚不存在。打包核心用 Rust 标准库与 Cargo，生成目录，不要求 sh、PowerShell 或 Python；目录可交由发布平台压缩。用户解压/搬动整个目录后仍可用；只拷贝二进制而不带 documentation 不满足下载包合同。用户可把该二进制目录加入 PATH，无需 WS_DOCUMENTATION_PATH。目标平台仍需相应 Rust target/linker；脚本不安装交叉工具链。

Cargo.toml 的 include 显式包含 src/assets/docs/skills/examples/安装入口及必要源码材料，保证文档进入源码发布包，不把测试工件当发布文档。是否公开发布/许可证选择不属于本轮执行；本轮不运行 cargo publish。

## R：组合与最小性

CLI 帮助承担发现入口；运行时 env 与编译默认值共同确定唯一目录；目录扫描和名字查找承担完整列表/原文返回；发布脚本建立相对路径所依赖的文件布局。所有组件共用 ENTRY.md 约定，不另造索引账或正文副本维护职责。debug 模式依赖源码目录保留，release 模式依赖伴随目录保留；两者不互相冒充。

## 直接测试

异 cwd、ENTRY 首位及标注、其余自动排序、ENTRY 正文后全量索引且动态增减、不改写入口源文件、省略后缀/嵌套名、其他文档原文相等、JSON 输入、无栈副作用；环境变量覆盖来源；debug/release 模式；Cargo install 的隔离安装根与文档复制边界；本地安装及发布脚本产物从异 cwd 启动、整包搬动后读取及 env 覆盖、附带 SKILL 逐字节复制。测试不是机械证明；跨平台只报告实际执行的平台。
