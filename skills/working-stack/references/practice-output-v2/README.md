# 导览与路线承接案例：实际输出

2026-10-12实际执行产生，业务事实与用户裁决均为模拟。managed/single各77条案例命令、15阶段、6帧。原时间戳保留；重跑时间和版本指纹自然变化。

[全部操作与解释](../practice.md)。重跑：`cargo build --locked`后执行`python3 tests/practice-case.py target/practice-replay --binary target/debug/working-stack`，输出目录须未存在。

## managed

| 阶段 | 完整MD | 完整YAML | MD变更 | YAML变更 | context | 回执 |
|---|---|---|---|---|---|---|
| 1. 初始委托：产物可用，并由用户按约定验收 | [MD](managed/01.md) | [YAML](managed/01.yaml) | [diff](managed/01.md.diff) | [diff](managed/01.yaml.diff) | [context](managed/01.context.md) | [回执](managed/01.commands.json) |
| 2. 新信息：验收条件尚有一个独立裁决问题 | [MD](managed/02.md) | [YAML](managed/02.yaml) | [diff](managed/02.md.diff) | [diff](managed/02.yaml.diff) | [context](managed/02.context.md) | [回执](managed/02.commands.json) |
| 3. 不依赖该裁决的工作继续：建立para | [MD](managed/03.md) | [YAML](managed/03.yaml) | [diff](managed/03.md.diff) | [diff](managed/03.yaml.diff) | [context](managed/03.context.md) | [回执](managed/03.commands.json) |
| 4. 实施中发现缺口：插项、拆分、重拟，不滥建帧 | [MD](managed/04.md) | [YAML](managed/04.yaml) | [diff](managed/04.md.diff) | [diff](managed/04.yaml.diff) | [context](managed/04.context.md) | [回执](managed/04.commands.json) |
| 4A. 用户裁决改变路线：更新父计划与继续理由 | [MD](managed/05.md) | [YAML](managed/05.yaml) | [diff](managed/05.md.diff) | [diff](managed/05.yaml.diff) | [context](managed/05.context.md) | [回执](managed/05.commands.json) |
| 5. 出现可选线索：独立支线与取消 | [MD](managed/06.md) | [YAML](managed/06.yaml) | [diff](managed/06.md.diff) | [diff](managed/06.yaml.diff) | [context](managed/06.context.md) | [回执](managed/06.commands.json) |
| 5A. 多个para：先保留上下文，再修正归属 | [MD](managed/07.md) | [YAML](managed/07.yaml) | [diff](managed/07.md.diff) | [diff](managed/07.yaml.diff) | [context](managed/07.context.md) | [回执](managed/07.commands.json) |
| 5A1. 改挂责任归属：不重写最初PUSH | [MD](managed/08.md) | [YAML](managed/08.yaml) | [diff](managed/08.md.diff) | [diff](managed/08.yaml.diff) | [context](managed/08.context.md) | [回执](managed/08.commands.json) |
| 5A2. 子帧收尾与原则归属 | [MD](managed/09.md) | [YAML](managed/09.yaml) | [diff](managed/09.md.diff) | [diff](managed/09.yaml.diff) | [context](managed/09.context.md) | [回执](managed/09.commands.json) |
| 5B. 计划重排与移除：先读路径，再按新位置操作 | [MD](managed/10.md) | [YAML](managed/10.yaml) | [diff](managed/10.md.diff) | [diff](managed/10.yaml.diff) | [context](managed/10.context.md) | [回执](managed/10.commands.json) |
| 6. 候选产物完成：只关闭对应子目标 | [MD](managed/11.md) | [YAML](managed/11.yaml) | [diff](managed/11.md.diff) | [diff](managed/11.yaml.diff) | [context](managed/11.context.md) | [回执](managed/11.commands.json) |
| 6A. 误关闭更正：保留错误历史，不倒写成功 | [MD](managed/12.md) | [YAML](managed/12.yaml) | [diff](managed/12.md.diff) | [diff](managed/12.yaml.diff) | [context](managed/12.context.md) | [回执](managed/12.commands.json) |
| 7. 等待结束：恢复旧帧，不重建问题 | [MD](managed/13.md) | [YAML](managed/13.yaml) | [diff](managed/13.md.diff) | [diff](managed/13.yaml.diff) | [context](managed/13.context.md) | [回执](managed/13.commands.json) |
| 8. 只有收到验收事实，才关闭根帧 | [MD](managed/14.md) | [YAML](managed/14.yaml) | [diff](managed/14.md.diff) | [diff](managed/14.yaml.diff) | [context](managed/14.context.md) | [回执](managed/14.commands.json) |
| 9. 同一操作的JSON输入与只读浏览 | [MD](managed/15.md) | [YAML](managed/15.yaml) | [diff](managed/15.md.diff) | [diff](managed/15.yaml.diff) | [context](managed/15.context.md) | [回执](managed/15.commands.json) |

最终：[MD](managed/final.md)、[YAML](managed/final.yaml)。完整时间线：[MD](managed/timeline.md)、[JSON](managed/timeline.json)。[全部回执](managed/commands.json)、[断言](managed/report.json)。

## single

| 阶段 | 完整MD | 完整YAML | MD变更 | YAML变更 | context | 回执 |
|---|---|---|---|---|---|---|
| 1. 初始委托：产物可用，并由用户按约定验收 | [MD](single/01.md) | [YAML](single/01.yaml) | [diff](single/01.md.diff) | [diff](single/01.yaml.diff) | [context](single/01.context.md) | [回执](single/01.commands.json) |
| 2. 新信息：验收条件尚有一个独立裁决问题 | [MD](single/02.md) | [YAML](single/02.yaml) | [diff](single/02.md.diff) | [diff](single/02.yaml.diff) | [context](single/02.context.md) | [回执](single/02.commands.json) |
| 3. 不依赖该裁决的工作继续：建立para | [MD](single/03.md) | [YAML](single/03.yaml) | [diff](single/03.md.diff) | [diff](single/03.yaml.diff) | [context](single/03.context.md) | [回执](single/03.commands.json) |
| 4. 实施中发现缺口：插项、拆分、重拟，不滥建帧 | [MD](single/04.md) | [YAML](single/04.yaml) | [diff](single/04.md.diff) | [diff](single/04.yaml.diff) | [context](single/04.context.md) | [回执](single/04.commands.json) |
| 4A. 用户裁决改变路线：更新父计划与继续理由 | [MD](single/05.md) | [YAML](single/05.yaml) | [diff](single/05.md.diff) | [diff](single/05.yaml.diff) | [context](single/05.context.md) | [回执](single/05.commands.json) |
| 5. 出现可选线索：独立支线与取消 | [MD](single/06.md) | [YAML](single/06.yaml) | [diff](single/06.md.diff) | [diff](single/06.yaml.diff) | [context](single/06.context.md) | [回执](single/06.commands.json) |
| 5A. 多个para：先保留上下文，再修正归属 | [MD](single/07.md) | [YAML](single/07.yaml) | [diff](single/07.md.diff) | [diff](single/07.yaml.diff) | [context](single/07.context.md) | [回执](single/07.commands.json) |
| 5A1. 改挂责任归属：不重写最初PUSH | [MD](single/08.md) | [YAML](single/08.yaml) | [diff](single/08.md.diff) | [diff](single/08.yaml.diff) | [context](single/08.context.md) | [回执](single/08.commands.json) |
| 5A2. 子帧收尾与原则归属 | [MD](single/09.md) | [YAML](single/09.yaml) | [diff](single/09.md.diff) | [diff](single/09.yaml.diff) | [context](single/09.context.md) | [回执](single/09.commands.json) |
| 5B. 计划重排与移除：先读路径，再按新位置操作 | [MD](single/10.md) | [YAML](single/10.yaml) | [diff](single/10.md.diff) | [diff](single/10.yaml.diff) | [context](single/10.context.md) | [回执](single/10.commands.json) |
| 6. 候选产物完成：只关闭对应子目标 | [MD](single/11.md) | [YAML](single/11.yaml) | [diff](single/11.md.diff) | [diff](single/11.yaml.diff) | [context](single/11.context.md) | [回执](single/11.commands.json) |
| 6A. 误关闭更正：保留错误历史，不倒写成功 | [MD](single/12.md) | [YAML](single/12.yaml) | [diff](single/12.md.diff) | [diff](single/12.yaml.diff) | [context](single/12.context.md) | [回执](single/12.commands.json) |
| 7. 等待结束：恢复旧帧，不重建问题 | [MD](single/13.md) | [YAML](single/13.yaml) | [diff](single/13.md.diff) | [diff](single/13.yaml.diff) | [context](single/13.context.md) | [回执](single/13.commands.json) |
| 8. 只有收到验收事实，才关闭根帧 | [MD](single/14.md) | [YAML](single/14.yaml) | [diff](single/14.md.diff) | [diff](single/14.yaml.diff) | [context](single/14.context.md) | [回执](single/14.commands.json) |
| 9. 同一操作的JSON输入与只读浏览 | [MD](single/15.md) | [YAML](single/15.yaml) | [diff](single/15.md.diff) | [diff](single/15.yaml.diff) | [context](single/15.context.md) | [回执](single/15.commands.json) |

最终：[MD](single/final.md)、[YAML](single/final.yaml)。完整时间线：[MD](single/timeline.md)、[JSON](single/timeline.json)。[全部回执](single/commands.json)、[断言](single/report.json)。

## 初始完整输出

```markdown
# 交付一个可用版本

  本栈承接可用版本的交付与验收，不记录全部内部执行流水。全局范围约束：{{note:累计原则.body}}。共享检查提示在note checklist中，需要时读取；具体裁决与继续理由保留在所属帧。

## 栈状态

```text
[W-01] [TOP] 完成交付与验收

```

## [W-01] [TOP] 完成交付与验收

- 上帧：无（根帧）
- 弹帧目标：候选版本通过直接测试
  用户确认约定的验收条件满足
- 计划：
  - **[FOCUS/in_progress]** 澄清验收条件
  - [ready] 准备并测试候选版本
  - [ready] 请用户验收
- 外部执行事项：无。

### 帧变化记录

- [PUSH] [2026-10-12T07:46:05.670682000Z] 登记模拟委托的完整目标
- [UPDATE] [2026-10-12T07:46:05.698579000Z] 把根目标分成当前可执行计划并落实当前步骤；变更字段：plan

## 栈变化记录

- [UPDATE] [2026-10-12T07:46:05.612477000Z] 登记已有模拟裁决，不由重要性自行造原则
- [UPDATE] [2026-10-12T07:46:05.643108000Z] 建立全局导览并引用单份已确认约束

## Appendix

### 累计原则

  模拟用户已确认：产物直接测试与用户验收分别建立；共通部分可先推进，最终验收仍由用户决定。

## 格式信息

> 以下仅用于再次解析，非业务内容。
> notes: [{"key":"description","meta":[],"at_title":true},{"key":"累计原则","meta":[]}]


```

```yaml
title: 交付一个可用版本
prefix: W
notes:
  description:
    meta: []
    body: 本栈承接可用版本的交付与验收，不记录全部内部执行流水。全局范围约束：{{note:累计原则.body}}。共享检查提示在note checklist中，需要时读取；具体裁决与继续理由保留在所属帧。
  累计原则:
    meta: []
    body: 模拟用户已确认：产物直接测试与用户验收分别建立；共通部分可先推进，最终验收仍由用户决定。
frames:
  1:
    parent: null
    title: 完成交付与验收
    goal:
    - 候选版本通过直接测试
    - 用户确认约定的验收条件满足
    origin: []
    status:
      kind: active
    plan:
    - text: 澄清验收条件
      children: []
      progress:
        kind: in_progress
      fork: []
    - text: 准备并测试候选版本
      children: []
      progress:
        kind: ready
      fork: []
    - text: 请用户验收
      children: []
      progress:
        kind: ready
      fork: []
    content:
      results: []
      side_effects: []
      principles: []
      remarks: {}
    log:
    - action: push
      at: 2026-10-12T07:46:05.670682000Z
      detail: 登记模拟委托的完整目标
    - action: update
      at: 2026-10-12T07:46:05.698579000Z
      detail: 把根目标分成当前可执行计划并落实当前步骤；变更字段：plan
log:
- action: update
  at: 2026-10-12T07:46:05.612477000Z
  detail: 登记已有模拟裁决，不由重要性自行造原则
- action: update
  at: 2026-10-12T07:46:05.643108000Z
  detail: 建立全局导览并引用单份已确认约束


```

## 新裁决同步父计划与继续理由

```diff
--- previous.yaml
+++ current.yaml
@@ -27,7 +27,7 @@
       progress:
         kind: in_progress
       fork: []
-    - text: 准备并测试候选版本
+    - text: 准备候选：先核对材料，再最终重测
       children: []
       progress:
         kind: ready
@@ -41,7 +41,8 @@
       results: []
       side_effects: []
       principles: []
-      remarks: {}
+      remarks:
+        顺序裁决: 模拟用户新增顺序：材料核对先于最终重测；验收仍留给用户
     log:
     - action: push
       at: 2026-10-12T07:46:05.670682000Z
@@ -49,6 +50,9 @@
     - action: update
       at: 2026-10-12T07:46:05.698579000Z
       detail: 把根目标分成当前可执行计划并落实当前步骤；变更字段：plan
+    - action: update
+      at: 2026-10-12T07:46:06.138660000Z
+      detail: 按新的模拟裁决更新父计划，不缩根目标；变更字段：content.remarks.顺序裁决、plan[1].text
   2:
     parent: 1
     title: 澄清验收条件
@@ -109,7 +113,7 @@
       progress:
         kind: ready
       fork: []
-    - text: 重新运行直接测试
+    - text: 核对材料后重新运行直接测试
       children: []
       progress:
         kind: ready
@@ -120,6 +124,7 @@
       principles: []
       remarks:
         测试发现: 模拟：一个边界输入尚不符合约定，需要修正后重测
+        继续理由: 落实父帧候选准备责任；先修正问题并核对材料，再完成最终重测，不能代签验收
     log:
     - action: push
       at: 2026-10-12T07:46:05.870085000Z
@@ -139,6 +144,9 @@
     - action: update
       at: 2026-10-12T07:46:06.079534000Z
       detail: 登记具体失败，不把测试已运行写成通过；变更字段：content.remarks.测试发现
+    - action: update
+      at: 2026-10-12T07:46:06.168579000Z
+      detail: 同步实施路线及当前继续理由，保留原进入历史；变更字段：content.remarks.继续理由、plan[3].text
 log:
 - action: update
   at: 2026-10-12T07:46:05.612477000Z

```

## 多para快照

```markdown
# 工作上下文：交付一个可用版本

仅为已录入快照，可能不含最新对话裁决；记录不等于已验证/已接受，事件时间不等于事实观察时间。

- `note:description.body`：本栈承接可用版本的交付与验收，不记录全部内部执行流水。全局范围约束：模拟用户已确认：产物直接测试与用户验收分别建立；共通部分可先推进，最终验收仍由用户决定。。共享检查提示在note checklist中，需要时读取；具体裁决与继续理由保留在所属帧。
  引用来源：note:累计原则.body

焦点：无；不自动选择暂停分支。

根目标 [W-01] 完成交付与验收
- `1.goal[0]`：候选版本通过直接测试
- `1.goal[1]`：用户确认约定的验收条件满足

## 暂停末端（不代表后台停止）
- [W-02] 澄清验收条件；show context W-02查看该分支。
- `2.status.reason`：[HumanReady] 等待用户选择验收条件的解释
- [W-03] 准备并直接测试候选版本；show context W-03查看该分支。
- `3.status.reason`：去向：新帧 W-05
- [W-05] 核对交付材料；show context W-05查看该分支。
- `5.status.reason`：去向：新帧 W-06
- [W-06] 保留另一条可选线索；show context W-06查看该分支。
- `6.status.reason`：[HumanReady] 等待模拟用户决定是否继续可选线索

使用边界：本视图提供复工材料，不代判继续理由、具体下一步或行动授权；缺失的新裁决和回执由调用方补齐，不把正文完整当全栈已校准证明。


```

## 改挂

```diff
--- previous.yaml
+++ current.yaml
@@ -90,8 +90,7 @@
     - 共通部分实现并完成直接测试，争议条件留下显式接口
     origin: []
     status:
-      kind: paused
-      reason: 去向：新帧 W-05
+      kind: active
     plan:
     - text: 实现共通部分
       children: []
@@ -122,7 +121,8 @@
       children: []
       progress:
         kind: ready
-      fork: []
+      fork:
+      - 5
     content:
       results: []
       side_effects: []
@@ -161,6 +161,12 @@
     - action: pause
       at: 2026-10-12T07:46:06.391014000Z
       detail: 去向：新帧 W-05
+    - action: resume
+      at: 2026-10-12T07:46:06.548394000Z
+      detail: 模拟改挂前恢复承担责任的父帧；Paused仅用于分支末端
+    - action: update
+      at: 2026-10-12T07:46:06.661781000Z
+      detail: 实施计划登记材料核对子帧，避免另建重复状态账；变更字段：plan[3].fork
   4:
     parent: 1
     title: 评估可选优化
@@ -192,14 +198,13 @@
       at: 2026-10-12T07:46:06.331236000Z
       detail: 按明确取消裁决退出，回到已暂停的实施帧
   5:
-    parent: 1
+    parent: 3
     title: 核对交付材料
     goal:
     - 核对候选版本的材料，并把核对依据交还实施帧
     origin: []
     status:
-      kind: paused
-      reason: 去向：新帧 W-06
+      kind: active
     plan:
     - text: 核对材料与版本对应关系
       children: []
@@ -225,6 +230,9 @@
     - action: pause
       at: 2026-10-12T07:46:06.451791000Z
       detail: 去向：新帧 W-06
+    - action: resume
+      at: 2026-10-12T07:46:06.631118000Z
+      detail: 模拟核对条件已齐备，进入改挂后的子帧
   6:
     parent: 1
     title: 保留另一条可选线索

```

## 验收前保持open

```diff
--- previous.yaml
+++ current.yaml
@@ -20,22 +20,24 @@
     - 用户确认约定的验收条件满足
     origin: []
     status:
-      kind: active
+      kind: paused
+      reason: '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
     plan:
     - text: 澄清验收条件
       children: []
       progress:
-        kind: in_progress
+        kind: item_done
       fork: []
     - text: 准备候选：先核对材料，再最终重测
       children: []
       progress:
-        kind: ready
+        kind: item_done
       fork: []
     - text: 请用户验收
       children: []
       progress:
-        kind: ready
+        kind: blocked
+        reason: 等待用户验收
       fork: []
     content:
       results: []
@@ -62,6 +64,18 @@
     - action: update
       at: 2026-10-12T07:46:06.763946000Z
       detail: 承接自 W-05 原则1（released）；变更字段：content.principles
+    - action: update
+      at: 2026-10-12T07:46:07.625178000Z
+      detail: 裁决子目标已完成；变更字段：plan[0].progress.kind
+    - action: update
+      at: 2026-10-12T07:46:07.658227000Z
+      detail: 候选版本与已明确条件一致且直接测试通过；变更字段：plan[1].progress.kind
+    - action: update
+      at: 2026-10-12T07:46:07.691103000Z
+      detail: 候选交付不等于验收；变更字段：plan[2].progress.kind、plan[2].progress.reason
+    - action: pause
+      at: 2026-10-12T07:46:07.724920000Z
+      detail: '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
   2:
     parent: 1
     title: 澄清验收条件
@@ -69,11 +83,16 @@
     - 取得用户对争议条件的明确选择
     origin: []
     status:
-      kind: paused
-      reason: '[HumanReady] 等待用户选择验收条件的解释'
+      kind: done
+      closure:
+        summary: 争议条件已有明确且适用的用户裁决
+        positive:
+        - result: 模拟用户选择已登记
+          evidence: ''
     plan: []
     content:
-      results: []
+      results:
+      - 模拟：用户明确选择第一种解释，候选版本的显式接口与之相符
       side_effects: []
       principles: []
       remarks:
@@ -88,6 +107,15 @@
     - action: pause
       at: 2026-10-12T07:46:05.812108000Z
       detail: '[HumanReady] 等待用户选择验收条件的解释'
+    - action: resume
+      at: 2026-10-12T07:46:07.529869000Z
+      detail: 模拟收到用户对争议条件的明确选择
+    - action: update
+      at: 2026-10-12T07:46:07.561355000Z
+      detail: 保存决定与适用范围；变更字段：content.results
+    - action: pop
+      at: 2026-10-12T07:46:07.592709000Z
+      detail: 返回根帧落实验收安排
   3:
     parent: 1
     title: 准备并直接测试候选版本

```

## 依据与边界

测试断言原native根目标不漂移，新的顺序裁决同步到父计划，原实施PUSH与改挂PUSH历史保留，每阶段notes MD往返相等，待验收仍open，原则/Correct履行现有合同；managed仅跟踪栈且clean，single不自动Git。结构测试不证明模拟业务事实。

既有MD目标段落编码会把两个goal合成一个多行字符串，因此不能称这些案例的整模型MD往返相等；本次直接测试区分native目标保持和notes往返。完整JSON/YAML供定位字段结构，未顺带修改目标编码。JSON候选frame字符串键解码、Correct只保存类别/摘要等既有边界也未修；改挂仍用YAML候选。

context呈现全部本链结果及外部执行事项、长正文和显式引用；不自动判断最新有效性或过滤过时正文。不是提速/减少token/准确率实证。
