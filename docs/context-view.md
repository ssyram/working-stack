# 一次查询恢复工作方向

## 目的与边界

`working-stack STACK show context [ID]` 从现有栈生成只读方向上下文，减少AI反复读取overview和多个frame的成本。默认使用唯一焦点；没有焦点则显示各暂停末端及原因，不自动resume。显式ID可查看指定分支，包括关闭快照。

这不是自动recalibration结论：工具不知道对话中未入栈的新裁决，不会判断某条自由文本是否过时、报告是否可信或用户是否接受。输出先说明这份边界。调用方须把当前用户消息与快照对照，再决定该改什么。

## 查询合同

- 根到所选帧：每帧全部目标、来源，当前未完成计划和具体可get/set的数字路径。
- 当前焦点：既有FOCUS投影（首个in_progress或blocked项），没有则明确未指定；ready项只作为待做计划，不自动认定为下一步。
- 各父帧/所选帧：最后3条结果及其路径、全部备注和原则。它们只是已记录内容，不通过关键词推断“当前裁决”或“已验证”。
- 最近事件的原时间用于说明“最近记录”，不是各条事实的观察时间；出处/审批/返回条件若操作者写在备注中，直接呈现，不创造新字段。
- 暂停旁支只列状态与恢复原因。完整历史仍通过show log/timeline读取，完整正文通过show frame或get读取。
- 单项展示至240字符，超长明确截断并提供字段路径；结果省略的数量明确显示。上下文不是完整导出，也不隐藏旧记录的存在。

函数只读取WorkingStack，所有路径来自实际结构；不改持久化、事件、核心状态机或JSON请求schema。`show --json`仍返回原有Out，context正文在text中；CLI与Web已有show接口均复用它。

## AI使用流程

1. `show context`恢复根目的与当前焦点；无焦点时看暂停原因，必要时显式选择ID查询，不擅自切换。
2. 对照这次用户裁决：哪些父计划/备注/结果不再适用？产物返回是否仍缺确认？只读具体涉及项，不默认重读全部历史。
3. 用get取得完整结构；set替换当前项，update --remark按标题覆盖，历史交给自动log。追加结果仅适用于确实新增而非替代的内容。
4. 修改后再show context；validate只保证结构，不表示全栈语义已校准。

```sh
working-stack mytask show context
working-stack mytask show context T-02
working-stack mytask get 'T-02.results[0]' --json
working-stack mytask set 'T-02.results[0]' --value '"报告已返回，结论待确认"' -r '收到报告，替换旧运行状态，不代签结论'
working-stack mytask update T-02 --remark '当前边界=报告已收到，尚无用户接受' -r '同步当前事实及审批边界'
```

多帧同步复用export yaml → 编辑候选 → apply。当前apply直接校验并保存，返回的是提交后diff，**没有内置提交前dry-run/preview**。需要预看时比较原YAML与候选；不要对仍引用旧ID/索引的候选盲目apply，也不要把多个顺序update称为整批事务。本轮不增加提交协议。

## 必要交互改进

push/update help提供可复制组合；push后用update填写首计划并标in_progress，goal并不等于FOCUS。已有替换/路径/批候选能力通过SKILL、ENTRY和手册可达。暂不改变旧查询默认输出、提交回执schema、Web视觉布局或核心diff（diff还用于生成apply事件）；细粒度旧→新回执与候选预览是独立后续设计，不搭车加入。

测试覆盖多层目标、嵌套计划路径、blocked原因、暂停无焦点/显式分支、关闭快照、截断标记、CLI/JSON只读不落盘。实际模拟案例中相同查询也可用于每个阶段恢复方向。
