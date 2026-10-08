# Principle / ClosureItem 编解码修复

## 目的与实际改动

用户授权修复真实 WS-MAIN 往返的两类结构变化并重新打印。字段合同先写入 `docs/markdown-fields.md`，现有细化设计、内容保留合同与 AI manual 引用同一合同。

打印器分别输出原则/处置、结果/依据，字段值复用现有 serde/SourcefulText 编码；导入器分别填入对应字段。原则条目编号不进入 rule；证据不并入 result；正文中的自然语言括号不做猜测性拆分。既有家族输入逻辑仍保留，闭帧默认处置只补缺失者，不覆盖显式值。没有改变核心类型或追加未触发的防御机制。

模块：`src/principle_markdown.rs`、`src/closure_markdown.rs`，由 print/import 组合。新增测试覆盖所有 Fate/None、空文本、换行、字面模板、结构引用、四个结论方向、不同证据、Done/Expired、连续三轮完整模型往返。

## 运行依据

- Rust48/48通过，包含新增3个字段往返测试。
- dev/examples/release构建成功；文档命令16/16、分发13/13、JSON38/38通过。
- 重新生成 `target/working-stack-fields-fixed`，将全局 `~/.cargo/bin/working-stack` 链接到该目录的二进制；文档随包更新。
- 通过全局命令按工具内手册导入/导出真实 WS-MAIN，再次导入并比较全部模型字段：差异0。输入722行、31帧，重印969行，原文件未改。SHA256 `13593a8fa5552fd7523ee76078aa99c8e4ffb87243239bcf9861074de631add5`。
- 原先遗漏内容保留在原帧；全部备注/notes同样相等。VS Code已并排打开新的输入快照与输出。

证据：`target/ws-main-fields-fixed/` 下 report.json、comparison.diff、model-diff.json（空数组）、input.md、reprinted.md 和逐命令stdout/stderr。复跑脚本 `tests/ws-main-manual.py` 现在将完整模型相等设为断言，不再只报告差异数。

保证边界：确认的是解析后的模型经重印/再次解析保持相等；不是原 Markdown 字节一致，也不是初次导入全部历史自然语言业务语义均被正确理解。原有闭帧缺处置的默认规则本轮没有重新裁决。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约200（两个字段模块及3个测试） |
| 修改代码行数 | 约25 |
| 删除代码行数 | 约40（拼接展示改为字段编码） |
| 涉及文件数 | 约11，另有全局链接和证据 |
| 新增测试用例数 | 3个Rust集成测试，真实完整模型相等断言1 |
| 测试通过率 | Rust48/48，doc16/16，分发13/13，JSON38/38，真实WS-MAIN全字段差异0 |
| 发现 bug 数 | 2类（前一轮已定位） |
| 修复 bug 数 | 2类，另修默认处置覆盖显式值的同层缺口 |
| 迭代轮次 | 设计/实现/直接测试各1 |

经验同步 `.pi/AI-learning.md`：可读输出同时承担序列化时必须保留字段边界；复用既有codec，不猜自然语言后缀。
