# AI方向上下文与完整实战案例

## 目的与变更

用户要求把SKILL案例补齐真实MD/YAML/diff、计划变化、多para、改挂与更正，完成后commit/push；随后提供实战refinement，要求增加AI易用性、快速取回焦点，明确不改核心功能/数据结构。

交互合同见context-view.md：新增show context [ID]只读投影，使用既有父链、全部目标、未完成计划路径、最后3条结果、备注/原则及暂停末端。不输出完整日志，不自动判断当前裁决/可信结论/用户验收。help给push后落实首计划和update组合例，remark格式错误附可复制示例。SKILL/ENTRY/manual接入一次view及get/set替换事实recipe；不改核心操作、模型、持久化、diff事件用途或返回schema。apply仍无提交前预览，文档如实说明。

skills/working-stack/references/practice.md完整模拟14阶段、6帧，每种模式72条命令；测试从文档提取执行。practice-output下保存两种模式每阶段完整MD/YAML/JSON、MD/YAML diff、context及回执、最终输出和时间线。README直接展示重要输出并链接全部文件。

## 实际运行

- Rust68/68通过，context CLI/JSON只读、显式分支、无焦点、help和错误示例通过。
- JSON38、修复17、路径25、timeline9、doc25通过。
- 案例managed/single各72条命令通过；根目标保留、四个para暂停快照、改挂parent而原PUSH不变、Correct与原则去向、验收前保持open、最终4done/2expired均由实际工件断言。
- managed目录Git仅跟踪practice.yaml且clean，single无自动Git。业务事实是模拟，非真实执行/用户验收。

## 实战边界

案例调试遇到JSON候选帧键解码失败，明确使用既有YAML候选，不修改核心codec。改挂时Paused父变内部帧不合法，按既有合同先resume父。results列表插值需下标，focus_to只接受Paused tip；案例修正实际用法，不加别名或guard。

结果文本持续append可能仍有旧记录，本视图不按关键词过滤它们；调用方替换当前事实、对照本次裁决。完整旧Closure不是Correct已实现的能力，本案例外置逐阶段快照。以上不是新机制的开发授权。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约350（只读视图、测试、案例重跑） |
| 修改代码行数 | 约35（help/入口） |
| 删除代码行数 | 约8 |
| 涉及文件数 | 约12源码/文档，另有210个模拟证据文件 |
| 新增测试用例数 | Rust3，CLI脚本1，双模式完整场景2 |
| 测试通过率 | Rust68/68，JSON38/38，修复17/17，路径25/25，timeline9/9，doc25/25，场景2/2 |
| 发现 bug 数 | JSON候选codec局限1（未修核心）；案例误用若干 |
| 修复 bug 数 | 交互发现与查询缺口；未声称修core |
| 迭代轮次 | 设计/交互实现1，案例命令调试若干 |

经验落.pi/AI-learning.md：减少读取往返、不以快照替代语义校准，模拟输出与业务真相分开。
