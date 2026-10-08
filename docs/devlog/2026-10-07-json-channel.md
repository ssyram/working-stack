# JSON 通道修正与 WS-MAIN 当场往返

## 已完成

依据 `docs/json-channel.md` 的合同，将 stdin 读取移至 main，将 JSON 字段转换归并至 `src/json_input.rs`；Web 复用转换层。修复字符串列表丢失、空值忽略、多行 JSON 回退、note 原因丢失及读取落盘、根帧 origin 丢失、import/apply/edit/web 参数未转发。新增规则仅对应已复现失败：未知字段和错误类型报错、非空坏 JSON 不再回退执行另一命令。没有增加重试、自动纠错、抢锁或后备执行。

直接依据：`cargo test` 39/39；release 构建成功；`python3 tests/json-channel.py target/release/working-stack` 37 条状态、日志及无写入断言通过。原有 ops 测试两个编译 warning（unused_parens/unused_mut）未涉及本次修改。

CLI 与 Web 请求转换一致，响应不同；查询 JSON 的 text 仍为字符串，结构化栈由 export json 获取。手册同步说明。

## WS-MAIN 结果与未闭合保证

运行 `tests/ws-main-roundtrip.py`，当前输入 707 行、31 帧，SHA256 `e1c3e82b06aca632b281e3d3cc13acc84e640bf1fd585ff78265ed94c2c36ec8`。导入、push、validate、重印均成功；以实际焦点 OP-31 为 parent 新建 OP-32，旧 31 帧模型及 notes 完全未变，原文件未改。

证据：`tests/artifacts/ws-main-final/input.md`、`before-push.md`、`after-push.md`、`report.json` 和各请求响应。

**原 Markdown 无损保证尚未成立**。导入仍有 60 条 warning（含 22 条既有 closed-principle 默认处置 warning）；实际输出缺 P1–P14 原则正文、OP-16 待办、OP-19 checklist 表格、非标准计划状态等。push 没有丢失已导入模型；丢失发生在 import。FOCUS 不作为比较判据。此前把这些差异称为全部排版差异的结论撤回。

本次只修 JSON 通道已复现问题并复跑 Markdown 链；未将未识别 Markdown 自动归入其他语义栏目，也未臆造非标准状态的含义。原有 import/print 保真职责仍待修复，不把已知缺陷包装成用户接受的能力限制。SCCO 的 JSON owner 合同已有直接测试依据，但项目整体保真 Complete 不能宣称通过。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约 310（含两个直接功能脚本） |
| 修改代码行数 | 约 75 |
| 删除代码行数 | 约 240（重复映射迁入共享模块） |
| 涉及文件数 | 约 11，另有本轮证据文件 |
| 新增测试用例数 | 37 条 JSON 断言 + WS-MAIN 往返检查 |
| 测试通过率 | Rust 39/39，JSON 37/37；Markdown 无损判据不通过 |
| 发现 bug 数 | 10 类 JSON/dispatch 问题，另确认 Markdown 丢内容 |
| 修复 bug 数 | 本轮 JSON/dispatch 已复现项修复；Markdown 未修 |
| 迭代轮次 | 设计/实现各 1；根帧 origin 反例驱动修正 1 |
