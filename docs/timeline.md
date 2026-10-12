# 事件时间线

用途：汇总全部帧（含暂停、关闭帧）及栈级 note 等变更日志，按实际时间展示，重建本轮发生了什么。读取不修改栈，不制造新的日志。

```bash
working-stack STACK timeline
working-stack STACK timeline --json
working-stack STACK show timeline
```

timeline 默认输出 Markdown，可重定向为报告；--json 返回 `{ok,events}`，每项保留 frame（全局事件为null）、frame_id、log_index、原始 at/action/detail，以及 time_known。show timeline 保留既有文本查询入口，使用同一收集与排序实现。JSON stdin 使用 `{"stack":"NAME或路径","op":"timeline"}`。

排序解析 RFC3339 为真实时间，因此不按字符串或时区标签排序。同一时刻稳定保留来源遍历顺序与本帧日志下标，不宣称已经恢复跨帧因果顺序；一个提交产生的多帧事件可能同刻。无时间或仅日期等不能精确解析的记录放在末尾，time_known=false，原始at不变；不凭git/mtime补时间。

CLI 新事件使用纳秒表示的UTC时间，降低不同操作挤在同一秒造成的歧义；同一个操作仍共用同一时刻。不承诺墙钟永远单调，也不添加持久全局序号。

detail 保留原SourcefulText内容而非用当前对象展开成历史事实；需要了解其当时依据仍应查对应transcript。时间线证明的是已记录的操作，不证明业务结果、owner理解/验收或遗漏事件。Owner协作说明即使写入日志，也只表示操作者曾登记；采用/批准来源须回到具体对象与对话，不能把后来的理解倒写成当时已知。最新人类状态放当前所属内容，关闭时证据保留历史口径。

合同：遍历每份log恰一次，产生带来源的行；已知时刻稳定排序、未知行稳定放末尾；所有原记录的字段值保持，输出器只打印只读投影。测试覆盖混合时区、毫秒/纳秒、同刻本帧顺序、关闭帧与栈级日志、无时间，以及命令级实际事件。
