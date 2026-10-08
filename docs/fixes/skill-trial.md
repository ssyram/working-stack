# SKILL 实战发现修复

## 上层依据与范围

用户授权修复双模型模拟中实测的问题：正式原则上移失败；手册 pop --done 与CLI不符；同次update逐字段生成重复日志；帧reopen与计划项修改对象不清楚。只承接这些实际职责，不追加防御guard、全局序号或全批次回滚，不改旧试验文件。

## 设计

1. principles 字段在resolve中保持条目列表，principles[i]解析为该条rule；列表自身结尾保持此前聚合显示。settle已经产生该索引引用，必须由reader支持，不改业务规则或重新发明原则转存。结果栏重印保留SourcefulText格式而非展开成固定文本，避免reclass的物化引用再丢失。
2. Done是现有pop默认，手册写实际CLI；不增加只为迁就错误文档的--done别名。计划项重新推进用path progress；reopen仅更正帧的Done/Expired，mode为active/paused。
3. CLI update整次候选结束后，每个受影响帧只保留一条新增Update，包含原原因和净变更字段路径。单次命令中的非Update事件（特别Correct）原样保留；旧日志原样前缀不动，net-zero不留新增Update。库的独立操作仍各记其事件；合并职责位于CLI事务边界，不压给叶子。
4. path/apply的既有单帧Update同样追加净变更字段摘要；note变化记录栈级字段。摘要只列字段路径，不重复完整正文，不扩充Event模型。

## 叶子合同与推理

resolve：Frame→principles列表→Index边界检查→rule。聚合finish维持之前显示；由同一列表来源构造的合法下标能解析，因此release与reclass物化引用能通过既有I8，越界仍失败。

commit_log：只操作旧log长度之后、当前at的Update；循环消费每条fresh event一次，首条Update附净变化摘要，后续同提交Update省去；无净内容差则无Update。Correct等分支直接保留。前缀未动，满足历史只追加对已提交前缀的义务；省去的是尚未提交的候选重复，不是删除历史。

净变更路径由旧/新序列化字段比较产生，排除log；CLI update的合并摘要排除status/parent，因为帧reopen的状态更正由保留的Correct解释。列表长度变更记列表路径，长度相同逐项定位。由同一候选比较，没有手写变化账。

## 测试与边界

引用下标/聚合显示、release到祖先、reclass到结果/备注；真实GPT试验复制文件重跑release，不改原件。多个plan/内容更新一条Update、路径摘要、net-zero、历史前缀、Correct保留、两帧release分别记事件、独立库操作事件不合并。

不修此轮未授权的完整Closure快照或独立print/import已知问题；reopen当前Correct保留旧summary，完整其他字段仍需别的承载。旧试验的331条事件不回写，修复版日志效率单独测。
