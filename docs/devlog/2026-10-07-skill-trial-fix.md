# 修复 SKILL 实战发现

## 目的与设计

用户授权修复原则上移、错误完成flag文档、重复update日志和reopen对象发现。合同见 `docs/fixes/skill-trial.md`。旧试验保持冻结，不回写历史或给原运行补成功；不新增防御guard、全局顺序号或全批次事务。

## 实际改动

- resolve将principles保持条目列表，支持索引rule，同时维持列表本身的聚合显示。正式release/reclass物化引用可通过I8。
- CLI update提交边界按每帧聚合新增Update，原原因后附净变更字段；旧日志前缀不动，Correct等独立事件原样保留，net-zero不留Update。path/apply生成的单帧Update及note变化同样带摘要。库操作仍各自记录。
- ai-manual使用实际默认Done pop，不提供不存在的--done；明确Frame reopen的active/paused参数，与Plan progress设置区分；新增备注title=body和覆盖已有映射说明。
- 回归触发reclass结果引用重印被展开，结果栏使用SourcefulText格式保留物化引用，完整MD模型往返通过。
- 原先设计中“完整Closure已保存”的过强描述修正为当前仅类别/summary；本轮没有实现完整快照。

## 运行依据

Rust62/62通过（新增5个引用/日志测试），修复命令级17/17；JSON38/38、路径25/25、时间线9/9、文档24/24、分发15/15通过。dev/examples/release成功；原ops测试2个warning未动。

五个字段级动作合为一条新增Update，历史前缀一致，summary包含plan/results/remarks等路径；净零更新不加日志。一次release/reclass影响两个帧，各记一条Update。新日志与物化引用MD往返相等，Correct原文未被合并剥除。

GPT实际失败的两份栈复制到 `target/skill-trial-repair-replay/`，原SH-05/ON-04原则正式release均成功且validate通过；原始hash仍为ccee7c7d... / dc1ae1ccb...，文件未改。报告与修复副本MD保存于该目录。

发布包 `target/working-stack-trial-fixed`，全局symlink指向其二进制；已确认的7788 Web服务重启到新全局工具。原331条事件及transcripts不修改，不宣称已经重跑双模型。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约220（提交摘要/引用/回归测试） |
| 修改代码行数 | 约20 |
| 删除代码行数 | 约5 |
| 涉及文件数 | 约13，另有副本/全局链接 |
| 新增测试用例数 | Rust5个、CLI17条、真实失败副本回放2个 |
| 测试通过率 | Rust62/62、修复17/17、JSON38/38、路径25/25、时间线9/9、doc24/24、分发15/15 |
| 发现 bug 数 | 已定位3类；回归确认派生结果引用输出缺口1 |
| 修复 bug 数 | 引用、文档、日志粒度及关联结果引用保留 |
| 迭代轮次 | 设计/实现1，完整MD回归驱动调整1 |

经验同步.pi/AI-learning.md：事务边界聚合未提交记录，不删历史；生成引用与读取必须同源，可重读结果不展开为固定文本。
