# Principle / ClosureItem 的 Markdown 字段边界

## 上层义务与已复现错误

import/print 必须保留已解析核心字段，不把展示标记读成正文。真实 WS-MAIN 往返发现22处 fate 标记混入 rule，1条证据从 evidence 搬进 result。只修这两类编码/解码，不改核心类型，不猜原文括号的意义。

## 对称格式

```markdown
- 确立原则：
  - P1
    - 原则："原则正文"
    - 处置：{"kind":"kept_local"}
- 达成情况：
  - 结果：[done] 整体结论
  - 对应：
    - 正向：
      - 结果："结果正文"
      - 依据："证据正文"
```

原则字段值使用现有 SourcefulText JSON 字符串编码；处置为现有 Option<Fate> JSON（None 为 null）。ClosureItem 的两字段同样分别编码为 SourcefulText JSON 字符串。JSON 转义保留换行、括号、字面模板和结构引用，不需要自然语言后缀猜测，也不会把用户正文中真实的“（依据：…）”剥走。

原有家族的 `P1 正文`、独立“对应/依据”行继续按现有输入规则解析；生成器只输出上述对称格式。历史文件中缺失 fate 的闭帧继续使用既有默认处置，但只补未设置者，不能覆盖已经读出的处置字段。不回写原 WS-MAIN，不试图从原文 `[released → …]` 自行推出新的用户裁决。

## Owner与复杂达成条目的状态边界

ClosureItem.result/evidence记录本层关闭时成立的结论及证据/裁决范围，不是可随时重写的人类理解栏。技术子目标的owner可为明确受托主线程AI；它的Done不替代父层用户验收。最新人类对报告的审阅、具体对齐/疑问及采用范围放仍负责该判断的帧备注或单份note，按对象/版本/来源更新；不要让动态note反向改变历史验收口径。

数据类型、对称格式和校验不变。不新增owner或human_status键、不把“完全理解/理解大部分”固化为枚举。具体表达推荐见owner-collaboration.md。

## 叶子合同与组合推理

原则条目头建立一个独立记录；两个子字段分别写 rule/fate，逐条编号仅作为打印位置，不进入正文。输出值经 serde 编码，输入经同一类型解码；合法规范形的 fields 值相等。

ClosureItem 的方向头建立当前方向中的新条目；仅其两个子字段填入 result/evidence。不借用全帧 evidence 缓存，避免多个不同证据被混合。旧家族解析与新字段解析组合时，新方向条目完整追加到对应列表，summary 及旧输入行为保持。

Direct tests：所有 Fate 构造器与 None、空文本、含换行/括号/引用的正文；四个 Closure 方向、不同条目的不同证据、Done/Expired；多轮往返不增长处置标记。真实 WS-MAIN 通过全局命令按手册导入/重印后，完整模型字段比较必须为零，不再只检查备注/notes。
