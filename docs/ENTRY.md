# Working Stack 使用入口

Working Stack以本层owner为主体保存工作与认识进程：目的、已有依据/选择、未完成责任和下一步。用户主导时AI协助用户理解与裁决；明确委派的局部自治中主线程AI可作本层判断，不代签上层保留权。人/AI操作同一栈文件，CLI与Web共用操作链，日志自动生成。

## 怎么使用本入口

先用下面的命令进入具体文档；需要某个操作的准确格式、查询方式或边界时按需阅读，不根据旧文本维护习惯猜命令。`doc NAME` 的 NAME 可直接取列表名称，.md 后缀可省略，目录嵌套保持原样。

通过 `working-stack doc ENTRY` 读取本入口时，末尾自动附加所选文档目录的**全量文档索引**：每项都有文档链接及可执行的 doc 指引，包含设计、手册、格式合同和日志。索引与 doc --list 使用同一次目录扫描，不另维护静态列表；正文先介绍如何使用工具。

## 从这里开始

```bash
working-stack doc                 # ENTRY.md 置顶，其他文档自动列举
working-stack doc owner-collaboration  # Owner协作及复杂项人类状态填写
working-stack doc ai-manual       # AI 全面使用手册
working-stack --help              # 命令语法
```

列表中的相对名称可直接传给 doc，后缀 .md 可有可无，支持嵌套路径。入口文件实际叫 ENTRY.md，不存在 doc entry 别名。

## 安装与文档来源

已安装 Rust/Cargo 时，Windows/Linux/macOS 使用同一个安装命令：

```text
cargo run --locked --example distribute -- install
```

Rust 安装器调用 Cargo 安装 release，将 docs 复制到二进制旁的 documentation，并附带 skills/working-stack/SKILL.md。默认安装根为 CARGO_HOME 或用户的 .cargo；可传 `--root ROOT`，二进制位于 ROOT/bin。该 bin 目录在 PATH 后可从任意目录调用。缺工具链时可用现有 install.sh（macOS/Linux）或 install.ps1（Windows）引导后调用同一 Rust 安装器。

生成下载发布目录的跨平台命令（输出目录须尚不存在，父目录已存在）：

```text
cargo run --locked --example distribute -- package target/working-stack-release
```

可传 `--target TARGET`，相应 Rust target/linker 需已可用。生成目录可直接压缩用于下载；解压后保留以下目录结构，可一起移动：

```text
working-stack[.exe]
documentation/
  ENTRY.md
  ai-manual.md
  ...
skills/
  working-stack/
    SKILL.md
```

附带SKILL介绍owner中心的协作、帧/计划、注意力与结束依据，不把AI交付当人类理解/接受，也不把自治任务全部绑到人审。完整教程含owner要求、提问与裁决、AI欠项、技术自治及真实输出；操作格式仍由手册承担。需要 Agent 使用时，把 skills/working-stack 目录放到所用 Agent 宿主的 skills 目录；工具不替你修改宿主配置或全局提示词。

默认 `cargo build` 是 debug：读取编译时仓库的 docs；`cargo build --release`：读取可执行文件旁的 documentation。手动直接运行 target/release 二进制时，先复制 docs 到 target/release/documentation，或指定环境变量。

`WS_DOCUMENTATION_PATH` 始终优先覆盖文档目录，例如：

```bash
WS_DOCUMENTATION_PATH=/path/to/docs working-stack doc
```

裸 `cargo install --path . --locked` 默认 release，但 Cargo 只安装二进制；须另外复制 docs 到安装目录的 documentation，或设置 WS_DOCUMENTATION_PATH。Cargo 不会自动修改用户 Shell 的环境变量。

## 最小工作流

```bash
working-stack mytask init --title '工作线' --prefix T
working-stack mytask push --title '当前工作' --goal '完成判据' -r '进入原因'
working-stack mytask update T-01 --plan-add '第一步' --plan-set 0=in_progress -r '落实当前焦点'
working-stack mytask show context
working-stack mytask web
```

mytask 是托管名称；single/独立文件模式直接传 stack.yaml、path/stack.yaml 或 path/WS-MAIN.md，不需要另一个开关，接口相同但不自动 Git 提交。用文件路径 init 即创建独立栈，查询/修改始终传该文件路径。Markdown 写操作会解析后整体重印，未识别内容按原归属保留为备注。

AI 推荐 JSON stdin：

```bash
printf '%s\n' '{"stack":"mytask","op":"show","args":{"what":"context"}}' | working-stack
```

复工先理解description中本层owner的目的/授权及材料定位，再从`show context [ID]`取得责任链、计划路径、已记录结果/实际执行事项与暂停条件。人类对哪些命题已对齐、AI尚欠什么及谁有权决定，由调用方依据对话写入；工具不读心/认定批准。description就是Markdown标题下段落；可介绍全局原则等材料位置，正文显式引用会展开，普通note指路仍按需读取。字段齐全和最近写入不代表方向正确或用户最新意图。填写/替换当前内容及多帧候选用法见`working-stack doc context-view`；完整历史按需show log或timeline。

事件时间线可汇总所有帧/全局日志按实际时间输出，含来源与原日志下标；使用方法见 `working-stack doc timeline`。

计划可直接按路径 get/set/insert/remove/move，不必改整份 Markdown；例子与写域见 `working-stack doc path-edit/README`。现有索引是位置，重排不会自动改写引用。

完整操作、边界及原因记录规则见 `working-stack doc ai-manual`；文档选择/分发合同见 `working-stack doc doc-command`；Note Meta 提示见 `working-stack doc note-meta`。
