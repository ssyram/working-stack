# 文档入口与 Rust 跨平台伴随分发

## 用户约定及层级同步

依据用户裁决：环境变量 WS_DOCUMENTATION_PATH 优先；debug 默认读编译目录，release 本地/发布均读实际二进制旁的 documentation；ENTRY.md 是实际入口而非别名，doc 列表置顶标注、其他自动排序；优先 Rust，共用跨平台交付逻辑，不以多份 Shell 维护发布机制。

Q.I.7 已写入 principles；D.root 分配 D.docs；architecture 与 detailed-design 统一引用 doc-command 的来源/列表/分发合同；ENTRY、AI manual、README 说明实际使用。运行时路径、帮助、Rust 分发器落实叶子合同。此前调研中的内嵌候选与源码安装默认路径不作为本轮现行方案。

## 实现与直接依据

- doc_catalog：环境变量 → debug 编译期 docs / release 可执行文件相邻 documentation，动态扫描、ENTRY 首位、支持后缀与嵌套名称。
- CLI：裸命令与 --help 成功打印入口提示；doc 仍是无栈副作用的只读命令。
- examples/distribute.rs：Rust 标准库复制目录，调用 Cargo，install 生成用户级二进制 + 文档，package 生成可搬动发布目录。安装 sh/ps1 仅引导工具链并调用 Rust 实现。
- Cargo include 显式带入文档和分发器，实际 cargo package --list 已确认 ENTRY/manual/distribute/doc_catalog 在包内，栈和测试工件不在包内。未运行 cargo publish，也未选择许可证。

macOS 实测：Rust 45/45；doc 16 条断言；分发 12 条断言（含实际 Cargo 隔离根安装及搬动目录）；JSON 38 条；Meta JS 7 条。dev/examples/release 构建成功，install.sh 语法通过。Windows/Linux 未实机运行，不把标准库使用冒充跨平台实测。

裸 cargo install 只装二进制、不复制文档的边界已实际复现；补伴随目录后读取通过。Rust 安装器实际提供该复制步骤。安装均发生在项目 target 下的隔离根，本轮未改用户全局安装。

发布目录产物位于 target/working-stack-release-final（生成时包含完整 docs）。它是可交付目录，不是压缩归档；压缩及公开发布交给发布平台。

## SCCO 与边界

Sound：用户最新 profile 裁决进入 Q，D.docs 单一合同，I 与脚本/手册同源；运行时覆盖和路径基准均有直接行为测试。
Complete：入口发现、列表/读取、目录来源、本地安装及发布复制在同一责任链闭合；裸 Cargo 与安装器不互相冒充。
Concise：没有内嵌正文、自动来源回退、全局环境注入或私有安装钩子。
Optimality：公共 Rust 分发器承担跨平台核心，避免平台间重写复制/布局。

这些是本轮设计职责及实测依据，不是全项目无 bug 或所有系统兼容的证明。源码模式按 cfg(debug_assertions) 区分默认 profile，自定义 profile 改该值会影响默认来源。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约 200（Rust 分发器及直接分发测试） |
| 修改代码行数 | 约 80 |
| 删除代码行数 | 约 30（平台复制逻辑归并） |
| 涉及文件数 | 约 14，另有文档入口与测试证据 |
| 新增测试用例数 | doc 增强至16条 + 分发12条 |
| 测试通过率 | Rust45/45，doc16/16，分发12/12，JSON38/38，JS7/7 |
| 发现 bug 数 | 0（用户合同与方案更新） |
| 修复 bug 数 | 0 |
| 迭代轮次 | 随用户裁决同步设计2轮，实现1轮 |

教训记录于 .pi/AI-learning.md：公共交付机制与平台引导分开；Cargo 安装与附带文档职责分开；实际平台证据与架构可移植性分开。
