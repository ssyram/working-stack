# Markdown 未识别内容按原归属保留

## 目的、设计与实现

用户确认未知内容可存现有备注，不需要新模型。设计合同位于 `docs/import-preservation.md`。

`src/import.rs` 将未知栏目/子标题整块缓冲到帧备注；无法解析的计划状态、模板行等逐行保留在原栏目备注；无帧归属内容进全局 note。保留 warning，不猜状态。既有已识别字段仍使用原逻辑。

`src/print.rs` 将备注/note 正文逐行缩进，并使用 SourcefulText 的格式化转义；导入去掉包装缩进，防止正文中的标题、bullet、模板被误解析成结构。备注标题含中文冒号的真实 checklist 反例已修正，往返保持完整标题。全局缓存的派生索引不再被当成正文覆盖该 note。

## 运行依据

- `cargo test`：原 39 个测试 + 新增 3 个内容保留测试全部通过。
- `cargo build --release`：成功。
- `python3 tests/json-channel.py target/release/working-stack`：37 条断言通过。
- 真实 `WS-MAIN`：715 行、31 帧，SHA256 `7f61224f866d4df525ae791156ac6a789e8d7b18ba242fd488b194a01d3d4490`。导入 → push OP-32 → validate → 重印 → 再导入成功。原文件未改，push 前后旧帧/notes 相等；21 行表格及未知状态原文在原帧备注中；全部帧备注及全局 notes 再导入相等。
- 证据：`tests/artifacts/ws-main-preserved-v3/{input.md,after-push.md,report.json}`。

warning 仍有 46 条，表示语义未解析和既有关闭帧原则处置等，不再以 warning 为这些正文的唯一载体。以上结论限定为本轮内容类型、归属和再次导入的保留测试，不声称全部历史核心字段或原文件字节相等。未知状态仅作为原文备注，并未成为可操作计划项。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约 145（含测试） |
| 修改代码行数 | 约 25 |
| 删除代码行数 | 约 23 |
| 涉及文件数 | 8，另有测试证据 |
| 新增测试用例数 | 3 个 Rust 集成测试 + WS-MAIN 归属/再导入断言 |
| 测试通过率 | Rust 42/42；JSON 37/37；真实 WS-MAIN 保留断言通过 |
| 发现 bug 数 | 3 类：正文丢失、包装缩进/归一化不稳定、标题冒号截断 |
| 修复 bug 数 | 本轮上述 3 类 |
| 迭代轮次 | 实现 1；直接功能测试驱动修正 2 |

教训已同步 `.pi/AI-learning.md`：不能识别格式不等于没有归属，必须保留原文而非仅保留 warning。
