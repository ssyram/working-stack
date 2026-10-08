# Note Meta 字符串列表与动态 doc 命令

## 设计与实际改动

用户要求 Meta 只承载展示提示，认识且适用的直接应用，其余忽略。先同步 `playground.md`、架构、细化设计与 `docs/note-meta.md`，再落实为 `Meta = Vec<String>`。CLI/JSON note 参数为 meta 列表；引用路径为 `note:key.meta[i]`。独立布局模块顺序扫描位置/order 提示，默认 Appendix；独立 Markdown 编码模块保留完整提示与 note key，并支持正文多轮导入。

工作区当前 `.working-stack/tool.yaml` 的 description meta 显式迁移为 `[before:tree]`，正文/帧/日志不变。既有已确认 Web 服务 7788 重启为新构建，GET state 实测返回列表。native 旧对象形状需要显式迁移；不提供双模型兼容。历史测试证据不修改。

用户随后要求动态文档入口。`src/doc_catalog.rs` 每次扫描编译所对应仓库的 docs；`doc` / `doc --list` 仅输出排序名称，无后缀；`doc NAME[.md]` 返回原文，支持嵌套名称。操作不依赖 cwd，不创建栈。具体合同见 `docs/doc-command.md`。

本机尚未全局安装：PATH 和安装脚本默认 `~/.cargo/bin/working-stack` 均无可执行文件。本轮没有自行安装到用户目录。项目 release 二进制可用；安装入口仍为 `./install.sh`。

## 运行依据

- Rust 45/45 测试通过：含 3 个 Meta 布局/承载/引用测试和 3 个原文保留测试。
- HTML JS 语法 + 7 条实际 Meta 引用函数断言通过；这不是完整浏览器 UI 验收。
- JSON 通道 38 条状态断言通过。
- doc 13 条断言通过：异 cwd、列表等价/名称-only、嵌套名、省略后缀、原文相等、动态增减无需重编、JSON 输入、无栈副作用。
- release 构建成功。
- WS-MAIN 717 行、31 帧，导入 → push OP-32 → 重印 → 再导入成功；21 行表格/未知状态原文仍在原帧，全部备注/notes 保持相等，原文件未改。证据：`tests/artifacts/ws-main-meta-list/report.json`。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约 240（布局/MD/doc 模块及测试） |
| 修改代码行数 | 约 70 |
| 删除代码行数 | 约 75 |
| 涉及文件数 | 约 20，另有测试证据 |
| 新增测试用例数 | 3 个 Rust 测试、7 条前端断言、13 条 doc 断言 |
| 测试通过率 | Rust 45/45，JSON 38/38，doc 13/13，Meta JS 7/7 |
| 发现 bug 数 | 0（既定合同变更） |
| 修复 bug 数 | 0 |
| 迭代轮次 | 设计/实现各 1 |

经验同步 `.pi/AI-learning.md`：开放展示提示不能升级为业务类型或未知值拒绝规则。动态 doc 的仓库依赖已在设计与手册中明示。
