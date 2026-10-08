# 全局 symlink 与按手册 WS-MAIN 重印对比

## 实际操作

用户授权将 release 二进制链接到全局并按工具内手册测试。创建用户级 PATH 入口 `~/.cargo/bin/working-stack`，指向项目 `target/working-stack-global/working-stack`；文档在该真实二进制旁的 documentation。

首次实测失败：macOS current_exe 返回链接路径，读取 ~/.cargo/bin/documentation 导致 IO 错误。doc_catalog 对可执行路径 canonicalize，设计合同同步，补 symlink 分发回归；Rust45/45、分发13/13通过。没有增加未触发的通用 guard 或来源回退。

修正后通过裸 working-stack 命令读取 ENTRY/ai-manual，与仓库原文逐字节相等。按手册第11节的 import/export 命令使用独立 stack.yaml（避免测试产生托管提交），原 WS-MAIN 不改，也没有 push 新帧。可复跑脚本：`tests/ws-main-manual.py SOURCE OUTPUT-DIR`，脚本通过 PATH 调用工作栈，而非直调库函数。

## 比较结果

输入721行、31帧，SHA256 `b39614a42348026ec88f692b51ab3d3e63a7bed10a7917c963f5d81596eb77b1`；重印864行，原文件未改。导入、validate、重印、再导入、再次validate全部成功。

此前遗漏的 P1–P14 原则、待办、checklist、未知状态正文均仍在原帧备注；全部帧备注及 notes 再导入相等。但完整模型不是相等：

- 22处：打印出的（kept-local）处置标记被再导入为原则正文的一部分。
- 1条 ClosureItem 的 evidence 文本并入 result，独立 evidence 清空，形成2个字段差异。

总计24个叶子字段差异。不是把文字全部删掉，但存在结构污染/搬移，不能报告为仅排版差异。本轮没有修改这些解析/打印规则。

证据：`target/ws-main-global-comparison/` 下 input.md、reprinted.md、comparison.diff、model-diff.json、report.json，及手册全文和所有实际命令 stdout/stderr。VS Code 已并排打开输入快照与重印结果。比较用当场冻结的输入快照，不把原文件随后可能发生的人工作业混入差异。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约100（完整模型比较脚本与 symlink 断言） |
| 修改代码行数 | 1（可执行路径规范化） |
| 删除代码行数 | 0 |
| 涉及文件数 | 6，另有全局链接及证据 |
| 新增测试用例数 | symlink 分发断言1，WS-MAIN模型比较1 |
| 测试通过率 | Rust45/45、分发13/13；命令链通过，完整模型相等判据不成立 |
| 发现 bug 数 | 3类：symlink资源定位、原则标记污染、结论证据字段搬移 |
| 修复 bug 数 | 1（symlink资源定位） |
| 迭代轮次 | 直接功能测试驱动修正1 |

经验已同步 .pi/AI-learning.md。全局 symlink 依赖项目发布目录保留，不是脱离该目录的二进制复制安装。
