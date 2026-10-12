# 按路径读取与编辑

## 目的与范围

依据用户要求：直接用已有路径索引定位内容，再选择 insert 等操作；覆盖实际会话中的原位编辑、前后补项、删除/重排、拆分和整段重拟，不让 AI 用全文字符串替换维护语法。

路径复用 Reference 的解析语法：`OP-07.plan[1]`、`OP-07.plan[0].children[2].text`、`OP-07.goal[0]`、`note:checklist.body`。根帧 ID 前缀只作显示，数字为身份。`results/side_effects/remarks` 是 `content` 下同名字段的简写。get 返回实际结构，而不是引用展开后的显示文本。

写操作范围是已有 Frame 的 title/goal/origin/plan，以及 content 的 results/side_effects/remarks，和已有 note 的 body/meta。框架身份、parent、生命周期、原则处置和日志不由这些编辑器负责；仍走已有 push/pop/reopen/settle/整文件候选操作。这个范围是职责分配，不另加状态推断或防御性协议。

## 操作

- `get PATH`：返回对应 JSON 值，只读、不落盘。`--json` 直接返回 `{ok,path,value}`。
- `set PATH VALUE`：替换既有字段/列表/条目。字符串设到 plan 条目时只改正文，保留状态、children、fork；对象表示替换完整条目；数组设到 plan/children 表示重拟整个列表。
- `insert PATH VALUE`：路径末尾 `[i]` 表示在第 i 项之前插入，i==长度为末尾追加；路径直接指向列表时为追加。不改变其他条目。
- `remove PATH`：移除列表项或允许编辑的映射成员；随后由既有 codec/结构校验判断候选是否有效。
- `move PATH --from FROM`：移动到 PATH；目标路径按移除来源之后的结构解释，与 JSON Patch 的顺序含义一致。整个操作只提交一次，目标失败不留下已经删除的来源。

计划列表和新计划项支持字符串简写；对象复用原 PlanItem 字段，children 同样可用字符串数组。progress 字符串复用现有 CLI 状态简写（ready/in_progress/blocked[:原因]/done/expire[:原因]），或直接使用原 `{kind:...}` 编码。

JSON 调用不需要嵌套转义 value：

```json
{"stack":"mytask","op":"insert","args":{"path":"T-01.plan[1]","value":"插在第二项之前"},"reason":"补充步骤"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan[0].text","value":"新正文"},"reason":"细化"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan[0].progress","value":"done"},"reason":"完成"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan","value":[{"text":"已完成部分","progress":"done"},"剩余事项"]},"reason":"重拟计划"}
{"stack":"mytask","op":"move","args":{"path":"T-01.plan[2]","from":"T-01.plan[0]"},"reason":"调整顺序"}
{"stack":"mytask","op":"remove","args":{"path":"T-01.plan[1]"},"reason":"不再需要"}
```

CLI 的 `--value` 接收 JSON：`working-stack mytask insert 'T-01.plan[1]' --value '"新步骤"' -r '补项'`。读取例：`working-stack mytask get 'T-01.plan[0]' --json`。插入/删除/重排会改变位置索引含义，不新增计划 ID，不自动改写既有引用；需要稳定承接时仍用 Frame ID。

## 维护Owner的工作理解

路径编辑不判断owner是否理解或批准。复杂plan可用字符串set保留任务名称及当前协作说明，progress/children/fork不动；说明绑定具体对象/版本与对话来源，不加理解百分比或owner字段。成果最新人类状态用所属帧按标题覆盖备注/note，不用不断append代替更新。JSON value直接承载正文，格式见`working-stack doc owner-collaboration`。旧Closure是关闭时口径，不借本编辑器改成动态人类状态。

任务推进、主张依据与行动权限分别判断；人类待决定不隐藏AI欠项，局部技术自治不强制等待人审。操作者写入合法不等于取得owner权力。

## 设计及合同

stack_path 只负责语法与模型地址；path_edit 在序列化的候选值上操作，转换失败不改变原栈。候选反序列化回 WorkingStack 后，CLI 复用 apply 的净差异事件、apply_checks、validate/save/锁及托管提交。每个被改变的帧追加一个原因事件；note 修改追加栈级事件。无净变化不造事件。

容器遍历每步消费一个路径段，索引越界/字段不存在为操作错误；insert 允许恰好位于末尾，不能跳出长度。move 的移除与插入都发生在内存候选，全部成功才进入提交链。类型、引用、fate/历史等由既有合同消费，不在每一层重复校验。

## 测试与边界

测试前/中/末插入、嵌套路径、正文与状态修改、子项拆分、删除、移动、整段替换、其他文本字段/note，以及失败无写入、日志原因、无净变化与引用校验。读取路径和 SourcefulText 引用共享词法，但 get 的结构视图不替代 resolve 的显示投影。语义裁决仍归授权范围内owner，操作者据其来源登记，不由路径操作推定。

此功能不修复此前多栈测试中的 goal 多行编码/引用展开，也不把尾换行相等重新定义；那些是独立 print/import 问题。
