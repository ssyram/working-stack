# Note内部信息移至格式附录

## 目的与产物

用户要求正文不显示note JSON key/meta，允许末尾格式说明或引用块。note正文使用可读标题，description显示“工作线说明”、import-cache显示“补充内容”；所有key/meta集中到文末“格式信息”引用行。核心数据不改。

note_markdown负责显示/附件编码及匹配；import读取文末附件，按显示名恢复真实key/meta；print收集并在全文结束附上。body缩进解码不trim canonical正文，保留首尾空白。旧机器标题仍可读，不在新输出中保留。

## 运行依据

Rust65/65通过（新增3个附件测试，既有note-meta测试增强）；修复17/17、路径25/25、doc24/24、分发15/15通过。dev/examples/release构建成功。新标题无note:JSON，meta只出现在末尾引用行，所有位置/未知提示、特殊key、重排和正文空白往返保持。

全局链接更新为target/working-stack-clean-meta/working-stack。真实WS-MAIN输入1020行、32帧，输出1024行；再次解析完整模型差异0，原文件未改。VS Code已打开对比。

证据：target/ws-main-clean-meta/{input.md,reprinted.md,comparison.diff,report.json,model-diff.json}。仅确认解析模型往返，不声称原文件逐字一致或自动理解原业务判断。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约110（附件读取与测试） |
| 修改代码行数 | 约30 |
| 删除代码行数 | 约30（旧分节逻辑集中） |
| 涉及文件数 | 7，另有全局链接与证据 |
| 新增测试用例数 | 3个Rust测试，既有Meta测试增强 |
| 测试通过率 | Rust65/65，修复17/17，路径25/25，doc24/24，分发15/15，真实WS-MAIN模型差异0 |
| 发现 bug 数 | 0（展示合同调整） |
| 修复 bug 数 | canonical note首尾空白读取缺口随附件解码闭合 |
| 迭代轮次 | 设计/实现各1 |

经验同步.pi/AI-learning.md：可读正文与解析附件分层，不以隐藏显示丢失核心信息。
