# 案例实际输出与逐阶段差异

这是2026-10-12由工具真实执行生成的输出；业务场景为模拟。原时间戳与原因保留，不补造旧时间。每种模式72条案例命令，14阶段、6帧；额外只读导出/检查未计入案例命令数。

[案例过程与全部命令](../practice.md)。重跑：`python3 tests/practice-case.py target/practice-replay --binary target/debug/working-stack`（先cargo build；输出目录须未存在）。重跑时间/版本指纹会自然变化，不承诺字节相等。

## managed 模式

| 阶段 | 完整MD | 完整YAML | MD变更 | YAML变更 | 方向视图 | 命令回执 |
|---|---|---|---|---|---|---|
| 1. 初始委托：产物可用，并由用户按约定验收 | [MD](managed/01.md) | [YAML](managed/01.yaml) | [diff](managed/01.md.diff) | [diff](managed/01.yaml.diff) | [context](managed/01.context.md) | [回执](managed/01.commands.json) |
| 2. 新信息：验收条件尚有一个独立裁决问题 | [MD](managed/02.md) | [YAML](managed/02.yaml) | [diff](managed/02.md.diff) | [diff](managed/02.yaml.diff) | [context](managed/02.context.md) | [回执](managed/02.commands.json) |
| 3. 不依赖该裁决的工作继续：建立para | [MD](managed/03.md) | [YAML](managed/03.yaml) | [diff](managed/03.md.diff) | [diff](managed/03.yaml.diff) | [context](managed/03.context.md) | [回执](managed/03.commands.json) |
| 4. 实施中发现缺口：插项、拆分、重拟，不滥建帧 | [MD](managed/04.md) | [YAML](managed/04.yaml) | [diff](managed/04.md.diff) | [diff](managed/04.yaml.diff) | [context](managed/04.context.md) | [回执](managed/04.commands.json) |
| 5. 出现可选线索：独立支线与取消 | [MD](managed/05.md) | [YAML](managed/05.yaml) | [diff](managed/05.md.diff) | [diff](managed/05.yaml.diff) | [context](managed/05.context.md) | [回执](managed/05.commands.json) |
| 5A. 多个para：先保留上下文，再修正归属 | [MD](managed/06.md) | [YAML](managed/06.yaml) | [diff](managed/06.md.diff) | [diff](managed/06.yaml.diff) | [context](managed/06.context.md) | [回执](managed/06.commands.json) |
| 5A1. 改挂责任归属：不重写最初PUSH | [MD](managed/07.md) | [YAML](managed/07.yaml) | [diff](managed/07.md.diff) | [diff](managed/07.yaml.diff) | [context](managed/07.context.md) | [回执](managed/07.commands.json) |
| 5A2. 子帧收尾与原则归属 | [MD](managed/08.md) | [YAML](managed/08.yaml) | [diff](managed/08.md.diff) | [diff](managed/08.yaml.diff) | [context](managed/08.context.md) | [回执](managed/08.commands.json) |
| 5B. 计划重排与移除：先读路径，再按新位置操作 | [MD](managed/09.md) | [YAML](managed/09.yaml) | [diff](managed/09.md.diff) | [diff](managed/09.yaml.diff) | [context](managed/09.context.md) | [回执](managed/09.commands.json) |
| 6. 候选产物完成：只关闭对应子目标 | [MD](managed/10.md) | [YAML](managed/10.yaml) | [diff](managed/10.md.diff) | [diff](managed/10.yaml.diff) | [context](managed/10.context.md) | [回执](managed/10.commands.json) |
| 6A. 误关闭更正：保留错误历史，不倒写成功 | [MD](managed/11.md) | [YAML](managed/11.yaml) | [diff](managed/11.md.diff) | [diff](managed/11.yaml.diff) | [context](managed/11.context.md) | [回执](managed/11.commands.json) |
| 7. 等待结束：恢复旧帧，不重建问题 | [MD](managed/12.md) | [YAML](managed/12.yaml) | [diff](managed/12.md.diff) | [diff](managed/12.yaml.diff) | [context](managed/12.context.md) | [回执](managed/12.commands.json) |
| 8. 只有收到验收事实，才关闭根帧 | [MD](managed/13.md) | [YAML](managed/13.yaml) | [diff](managed/13.md.diff) | [diff](managed/13.yaml.diff) | [context](managed/13.context.md) | [回执](managed/13.commands.json) |
| 9. 同一操作的JSON输入与只读浏览 | [MD](managed/14.md) | [YAML](managed/14.yaml) | [diff](managed/14.md.diff) | [diff](managed/14.yaml.diff) | [context](managed/14.context.md) | [回执](managed/14.commands.json) |

最终完整输出：[managed/final.md](managed/final.md)、[YAML](managed/final.yaml)。完整时间线：[MD](managed/timeline.md)、[JSON](managed/timeline.json)。[全部命令回执](managed/commands.json)、[断言结果](managed/report.json)。

## single 模式

| 阶段 | 完整MD | 完整YAML | MD变更 | YAML变更 | 方向视图 | 命令回执 |
|---|---|---|---|---|---|---|
| 1. 初始委托：产物可用，并由用户按约定验收 | [MD](single/01.md) | [YAML](single/01.yaml) | [diff](single/01.md.diff) | [diff](single/01.yaml.diff) | [context](single/01.context.md) | [回执](single/01.commands.json) |
| 2. 新信息：验收条件尚有一个独立裁决问题 | [MD](single/02.md) | [YAML](single/02.yaml) | [diff](single/02.md.diff) | [diff](single/02.yaml.diff) | [context](single/02.context.md) | [回执](single/02.commands.json) |
| 3. 不依赖该裁决的工作继续：建立para | [MD](single/03.md) | [YAML](single/03.yaml) | [diff](single/03.md.diff) | [diff](single/03.yaml.diff) | [context](single/03.context.md) | [回执](single/03.commands.json) |
| 4. 实施中发现缺口：插项、拆分、重拟，不滥建帧 | [MD](single/04.md) | [YAML](single/04.yaml) | [diff](single/04.md.diff) | [diff](single/04.yaml.diff) | [context](single/04.context.md) | [回执](single/04.commands.json) |
| 5. 出现可选线索：独立支线与取消 | [MD](single/05.md) | [YAML](single/05.yaml) | [diff](single/05.md.diff) | [diff](single/05.yaml.diff) | [context](single/05.context.md) | [回执](single/05.commands.json) |
| 5A. 多个para：先保留上下文，再修正归属 | [MD](single/06.md) | [YAML](single/06.yaml) | [diff](single/06.md.diff) | [diff](single/06.yaml.diff) | [context](single/06.context.md) | [回执](single/06.commands.json) |
| 5A1. 改挂责任归属：不重写最初PUSH | [MD](single/07.md) | [YAML](single/07.yaml) | [diff](single/07.md.diff) | [diff](single/07.yaml.diff) | [context](single/07.context.md) | [回执](single/07.commands.json) |
| 5A2. 子帧收尾与原则归属 | [MD](single/08.md) | [YAML](single/08.yaml) | [diff](single/08.md.diff) | [diff](single/08.yaml.diff) | [context](single/08.context.md) | [回执](single/08.commands.json) |
| 5B. 计划重排与移除：先读路径，再按新位置操作 | [MD](single/09.md) | [YAML](single/09.yaml) | [diff](single/09.md.diff) | [diff](single/09.yaml.diff) | [context](single/09.context.md) | [回执](single/09.commands.json) |
| 6. 候选产物完成：只关闭对应子目标 | [MD](single/10.md) | [YAML](single/10.yaml) | [diff](single/10.md.diff) | [diff](single/10.yaml.diff) | [context](single/10.context.md) | [回执](single/10.commands.json) |
| 6A. 误关闭更正：保留错误历史，不倒写成功 | [MD](single/11.md) | [YAML](single/11.yaml) | [diff](single/11.md.diff) | [diff](single/11.yaml.diff) | [context](single/11.context.md) | [回执](single/11.commands.json) |
| 7. 等待结束：恢复旧帧，不重建问题 | [MD](single/12.md) | [YAML](single/12.yaml) | [diff](single/12.md.diff) | [diff](single/12.yaml.diff) | [context](single/12.context.md) | [回执](single/12.commands.json) |
| 8. 只有收到验收事实，才关闭根帧 | [MD](single/13.md) | [YAML](single/13.yaml) | [diff](single/13.md.diff) | [diff](single/13.yaml.diff) | [context](single/13.context.md) | [回执](single/13.commands.json) |
| 9. 同一操作的JSON输入与只读浏览 | [MD](single/14.md) | [YAML](single/14.yaml) | [diff](single/14.md.diff) | [diff](single/14.yaml.diff) | [context](single/14.context.md) | [回执](single/14.commands.json) |

最终完整输出：[single/final.md](single/final.md)、[YAML](single/final.yaml)。完整时间线：[MD](single/timeline.md)、[JSON](single/timeline.json)。[全部命令回执](single/commands.json)、[断言结果](single/report.json)。

## 初始完整输出（managed）

```markdown
# 交付一个可用版本

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

- [PUSH] [2026-10-12T04:15:54.932936000Z] 登记模拟委托的完整目标
- [UPDATE] [2026-10-12T04:15:54.997068000Z] 把根目标分成当前可执行计划并落实当前步骤；变更字段：plan



```

```yaml
title: 交付一个可用版本
prefix: W
notes: {}
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
      at: 2026-10-12T04:15:54.932936000Z
      detail: 登记模拟委托的完整目标
    - action: update
      at: 2026-10-12T04:15:54.997068000Z
      detail: 把根目标分成当前可执行计划并落实当前步骤；变更字段：plan


```

## 多para暂停快照

```markdown
# 工作上下文：交付一个可用版本

仅为已录入快照，可能不含最新对话裁决；记录不等于已验证/已接受，事件时间不等于事实观察时间。

焦点：无；不自动选择暂停分支。

根目标 [W-01] 完成交付与验收
- `1.goal[0]`：候选版本通过直接测试
- `1.goal[1]`：用户确认约定的验收条件满足

## 暂停末端（不代表后台停止）
- [W-02] 澄清验收条件：[HumanReady] 等待用户选择验收条件的解释；show context W-02查看该分支。
- [W-03] 准备并直接测试候选版本：去向：新帧 W-05；show context W-03查看该分支。
- [W-05] 核对交付材料：去向：新帧 W-06；show context W-05查看该分支。
- [W-06] 保留另一条可选线索：[HumanReady] 等待模拟用户决定是否继续可选线索；show context W-06查看该分支。

下一步：对照最新用户裁决与以上计划/备注；只补读取影响当前判断的缺口，不把本视图当执行授权或全栈已校准证明。


```

## 改挂帧：只改当前parent并保留原PUSH

```diff
--- previous.yaml
+++ current.yaml
@@ -80,8 +80,7 @@
     - 共通部分实现并完成直接测试，争议条件留下显式接口
     origin: []
     status:
-      kind: paused
-      reason: 去向：新帧 W-05
+      kind: active
     plan:
     - text: 实现共通部分
       children: []
@@ -108,11 +107,12 @@
       progress:
         kind: ready
       fork: []
-    - text: 重新运行直接测试
-      children: []
-      progress:
-        kind: ready
-      fork: []
+    - text: 重新运行直接测试并核对材料
+      children: []
+      progress:
+        kind: ready
+      fork:
+      - 5
     content:
       results: []
       side_effects: []
@@ -147,6 +147,12 @@
     - action: pause
       at: 2026-10-12T04:15:55.976749000Z
       detail: 去向：新帧 W-05
+    - action: resume
+      at: 2026-10-12T04:15:56.239148000Z
+      detail: 模拟改挂前恢复承担责任的父帧；Paused仅用于分支末端
+    - action: update
+      at: 2026-10-12T04:15:56.456693000Z
+      detail: 实施计划登记材料核对子帧，避免另建重复状态账；变更字段：plan[3].fork、plan[3].text
   4:
     parent: 1
     title: 评估可选优化
@@ -178,14 +184,13 @@
       at: 2026-10-12T04:15:55.890677000Z
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
@@ -211,6 +216,9 @@
     - action: pause
       at: 2026-10-12T04:15:56.082317000Z
       detail: 去向：新帧 W-06
+    - action: resume
+      at: 2026-10-12T04:15:56.401678000Z
+      detail: 模拟核对条件已齐备，进入改挂后的子帧
   6:
     parent: 1
     title: 保留另一条可选线索

```

## 交付后等待验收，不提前关闭根

```diff
--- previous.yaml
+++ current.yaml
@@ -14,22 +14,24 @@
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
     - text: 准备并测试候选版本
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
@@ -52,6 +54,18 @@
     - action: update
       at: 2026-10-12T04:15:56.617078000Z
       detail: 承接自 W-05 原则1（released）；变更字段：content.principles
+    - action: update
+      at: 2026-10-12T04:15:58.006186000Z
+      detail: 裁决子目标已完成；变更字段：plan[0].progress.kind
+    - action: update
+      at: 2026-10-12T04:15:58.062234000Z
+      detail: 候选版本与已明确条件一致且直接测试通过；变更字段：plan[1].progress.kind
+    - action: update
+      at: 2026-10-12T04:15:58.118747000Z
+      detail: 候选交付不等于验收；变更字段：plan[2].progress.kind、plan[2].progress.reason
+    - action: pause
+      at: 2026-10-12T04:15:58.177293000Z
+      detail: '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
   2:
     parent: 1
     title: 澄清验收条件
@@ -59,11 +73,16 @@
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
@@ -78,6 +97,15 @@
     - action: pause
       at: 2026-10-12T04:15:55.190277000Z
       detail: '[HumanReady] 等待用户选择验收条件的解释'
+    - action: resume
+      at: 2026-10-12T04:15:57.839304000Z
+      detail: 模拟收到用户对争议条件的明确选择
+    - action: update
+      at: 2026-10-12T04:15:57.892405000Z
+      detail: 保存决定与适用范围；变更字段：content.results
+    - action: pop
+      at: 2026-10-12T04:15:57.948955000Z
+      detail: 返回根帧落实验收安排
   3:
     parent: 1
     title: 准备并直接测试候选版本

```

## 用户验收后收尾

```diff
--- previous.yaml
+++ current.yaml
@@ -14,8 +14,12 @@
     - 用户确认约定的验收条件满足
     origin: []
     status:
-      kind: paused
-      reason: '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
+      kind: done
+      closure:
+        summary: 模拟委托的两项根目标均满足
+        positive:
+        - result: 直接测试结果与模拟用户验收确认
+          evidence: ''
     plan:
     - text: 澄清验收条件
       children: []
@@ -30,15 +34,16 @@
     - text: 请用户验收
       children: []
       progress:
-        kind: blocked
-        reason: 等待用户验收
-      fork: []
-    content:
-      results: []
+        kind: item_done
+      fork: []
+    content:
+      results:
+      - 模拟：直接测试通过，且用户确认全部约定验收条件满足
       side_effects: []
       principles:
       - rule: '{{5.principles[0]}}'
-        fate: null
+        fate:
+          kind: kept_local
       remarks:
         后续线索: 模拟用户取消本轮可选优化；仅保留线索，不构成本轮验收义务
     log:
@@ -66,6 +71,21 @@
     - action: pause
       at: 2026-10-12T04:15:58.177293000Z
       detail: '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
+    - action: resume
+      at: 2026-10-12T04:15:58.282939000Z
+      detail: 模拟用户确认约定验收条件全部满足
+    - action: update
+      at: 2026-10-12T04:15:58.336921000Z
+      detail: 登记模拟用户验收事实；变更字段：plan[2].progress.kind、plan[2].progress.reason
+    - action: update
+      at: 2026-10-12T04:15:58.396428000Z
+      detail: 两项根目标的依据均已具备；变更字段：content.results
+    - action: update
+      at: 2026-10-12T04:15:58.453578000Z
+      detail: 模拟已确认的通用规则由根帧持有，退出前明确处置；变更字段：content.principles[0].fate
+    - action: pop
+      at: 2026-10-12T04:15:58.513526000Z
+      detail: 对照完整原目标收尾
   2:
     parent: 1
     title: 澄清验收条件

```

## 已知能力边界

本案例改挂使用YAML候选apply，不改core。工具目前native/apply解码走serde_yaml，JSON候选的字符串帧键可能被拒，不能把JSON export后直接apply称为已可用；完整JSON输入请求与JSON文件存储是两件事。改挂接收live子帧时，父帧必须符合既有Active内部帧不变量；先resume父帧，不造新规则。

案例中第一次Done是故意模拟的误判，不是测试证明用户已验收。Correct只保存旧关闭类别与摘要，完整阶段快照另外保存在上述文件。引用使用已有可解析路径，如`{{W-05.results[0]}}`，不是未支持的裸results列表引用。
