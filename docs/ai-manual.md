# Working Stack AI 使用手册

> 工作栈以本层owner为主体：记录其要求、已有判断/依据、认识缺口与下一责任，不以AI执行清单替代。
> 用户主导时AI协助用户理解与裁决；明确委派的子工作中，主线程AI可以在授权内作为owner。写文件/调度不自动授予裁决权。

## 工具内阅读入口与文档来源

裸命令及 `--help` 推荐 `working-stack doc ENTRY`。`working-stack doc` / `doc --list` 将 `ENTRY.md` 置顶标注，其他相对文档名称（含 .md）自动排序；读取时后缀可省略，支持嵌套路径。`doc ENTRY` 的正文介绍入口用法，末尾实时附加所选目录的全部其他文档索引；其他 doc 仍返回原文。附带 skill 位于 skills/working-stack/SKILL.md，只承接概念与语义管理，不重复本手册。

`WS_DOCUMENTATION_PATH` 优先覆盖；debug 默认读编译时 docs；release 默认读二进制旁的 documentation，而不是当前 cwd。跨平台安装和发布目录生成由 Rust 工具承担：`cargo run --locked --example distribute -- install` / `package target/working-stack-release`。裸 cargo install 只装二进制，须另带 documentation 或指定环境变量。完整合同统一见 `doc-command.md`，安装入门见 `ENTRY.md`。

## 1. 先恢复Owner的工作，而不是AI完成清单

Owner是本层有权确定目标、裁决取舍和判断完成的主体；来自实际委托，不来自自认、调度身份或文件作者。当前人机协作通常用户为owner，AI不得代签用户保留的范围/发布/验收；局部技术自治可以依既定授权作本层判断，不给每个步骤加人工关卡。

AI维护的依据应包括owner提出的要求/问题、已作选择与纠正、真实交付及回执、当前哪些认识已对齐或仍有缺口，以及双方尚欠什么。交付报告不等于owner已读懂/采用；owner待决定也不能隐藏AI仍欠的解释。

查询、更新、push/pop等操作都服务实际责任变化，并遵守已有授权。普通内部执行留在计划/evidence，不为每次调用造帧；修改必带reason。规则须本层有权主体确立，重要事实不自动成为原则。结构校验不证明理解充分或语义批准。完整约定及复杂项标记例子：`working-stack doc owner-collaboration`。

## 2. 快速开始

### 2.1 查看当前状态（最常用）

```bash
# 一次恢复方向：目标链、FOCUS、计划路径、结果/备注与暂停条件
working-stack <NAME> show context

# 更短总览：目标、焦点、Paused 分支
working-stack <NAME> show overview

# 树形视图（谁在栈上、谁暂停了）
working-stack <NAME> show tree

# 某个帧的完整详情
working-stack <NAME> show frame <ID>

# 全栈不变量校验
working-stack <NAME> validate
```

`show overview` 输出解读：
```
栈：Working Stack 工具化（WS-TOOL）      ← 栈标题
[T-01] WS 独立工具 —— 核心语言经用户…    ← 根帧 + 目标摘要
焦点路径：T-01 → T-03                    ← 当前工作路径
TOP：[T-03] 框架架构方案                  ← 当前焦点帧
  FOCUS：[in_progress] 用户审阅方案…      ← 焦点帧的当前工作项
Paused 分支：
  - [T-02] playground 核心语言迭代        ← 暂停中的分支 + 原因
已 pop：0 帧
```

### 2.2 操作是责任变化的表达，不是见到关键词就执行

| 已有授权/事实下的变化 | 操作例子 |
|---|---|
| 需要独立目的及回程的上下文 | push；普通步骤只改plan |
| 本层原目标已有依据全部满足 | pop Done；人类验收若为目标仍须明确验收 |
| 责任暂挂，另有可推进工作 | pause/恢复已有分支，或有依据push旁支 |
| 原等待条件已解除 | resume，不重建同一问题 |
| Owner取消未达回合 | pop --expired，保留成果与未完义务 |
| 新裁决改变当前安排 | 改所属计划/备注及受影响父责任，不只追加消息 |
| 当前归属明确改变 | YAML候选apply改parent，不倒写原PUSH |

表是用法示例，不是强制阶段枚举。操作不能替owner作新裁决，也不要求明确授权内的自治任务每步等待人类。

## 3. 核心操作详解

### 3.1 push — 进入新工作

```bash
# 在 T-01 下钻新帧
working-stack <NAME> push T-01 \
  --title "调查性能瓶颈" \
  --goal "找到根因并提出修复方案" \
  -r "用户要求先调查再修复"

# 省略 parent = 建根帧（第一项工作）
working-stack <NAME> push --title "项目启动" --goal "完成 MVP" -r "初始化"
```

**规则**：
- `--title` 必填，`--goal` 建议（弹帧目标 = 完成判据）
- `-r`（reason）必填：为什么进入这个上下文
- push 后新帧自动成为 TOP；旧焦点自动 Paused
- 同一父帧下多次 push = para（并行分支）
- goal不等于当前FOCUS；push后用 `update T-02 --plan-add '第一步' --plan-set 0=in_progress -r '落实当前步骤'` 填首项。

### 3.2 pop — 结束工作

```bash
# 完成
working-stack <NAME> pop T-03 \
  --summary "性能瓶颈定位为 N+1 查询，索引优化后降低 80%" \
  -r "在已委派的技术判据内据实际测量判断达成；不代签整体用户验收"

# 放弃/失效（partial）
working-stack <NAME> pop T-04 \
  --expired \
  --summary "方案 B 因依赖不兼容放弃" \
  --residual "剩余调研材料转入 T-05" \
  -r "上游 API 不支持所需接口"
```

**前置条件**（不满足会被拒绝）：
- 该帧必须是 tip（没有进行中的子帧）
- 所有原则必须已结算（`--settle`）
- `--summary` 必填（一句话结论）

**关闭后**：焦点自动回到父帧（或用户指定的 `--focus-to`）。Done 是默认模式，不传完成开关；未达而结束回合才传 `--expired`。

### 3.3 pause / resume — 暂停与恢复

```bash
# 暂停当前焦点
working-stack <NAME> pause T-03 -r "等待外部 API 文档更新"

# 恢复已暂停的分支
working-stack <NAME> resume T-03 -r "API 文档已更新"
```

**规则**：
- `pause` 只作用于当前 TOP
- `resume` 自动暂停旧焦点、激活目标帧
- 理由必填：暂停写"在等什么"，恢复写"为什么现在回来"

### 3.4 update — 修改帧内容

```bash
# 改标题
working-stack <NAME> update T-02 --title "新标题" -r "更正名称"

# 改目标（多段用多个 --goal）
working-stack <NAME> update T-02 --goal "第一段" --goal "第二段" -r "目标细化"

# 加计划项
working-stack <NAME> update T-02 --plan-add "新步骤" -r "增加步骤"

# 改计划状态（路径如 0=顶层第一项、0.1=第一项的子项）
working-stack <NAME> update T-02 --plan-set 0=in_progress -r "开始第一步"
working-stack <NAME> update T-02 --plan-set '0=blocked:等待具体依赖输入' -r "登记实际等待条件"
working-stack <NAME> update T-02 --plan-set 0=done -r "完成"
working-stack <NAME> update T-02 --plan-set 0=expire:"不再需要" -r "方案变更"

# 改计划正文
working-stack <NAME> update T-02 --plan-text 0="更新后的步骤描述" -r "细化步骤"

# 添加内嵌子项
working-stack <NAME> update T-02 --plan-add-child 0="子步骤" -r "拆分子步骤"

# 添加原则
working-stack <NAME> update T-02 --principle-add "每次修改必须跑测试" -r "用户确立"

# 结算原则
working-stack <NAME> update T-02 --settle 0=kept_local -r "保留在本帧"
working-stack <NAME> update T-02 --settle 0=release:T-01 -r "上移到根帧"
working-stack <NAME> update T-02 --settle 0=invalidated:"前提已不成立" -r "推翻"
working-stack <NAME> update T-02 --settle 0=reclass:results -r "误列，转结果"

# 添加备注/结果
working-stack <NAME> update T-02 --remark "待核=某假设尚未验证" -r "标注待核"
working-stack <NAME> update T-02 --result-add "实测加速 3.2x" -r "记录数据"
```

**计划状态**：`ready` → `in_progress` → `done`；可转 `blocked`（等待）或 `expire:理由`（不再做）。

### 3.5 note — 共享内容

```bash
# 创建/更新
working-stack <NAME> note checklist --body "全部 FSM 已实战测试" --meta appendix --meta order:10 -r "更新清单"

# 查看
working-stack <NAME> note checklist
```

Note 是全局共享内容（单份正文，多帧引用）。`meta` 是打印提示字符串列表：认识且适用的直接应用，其余忽略但保留。默认放末尾 Appendix；支持 `before:tree`、`after:tree`、`before:frames`、`after:frames`、`appendix` 及 `order:N`。同类提示后一个适用值生效。具体合同见 `note-meta.md`。文档可直接通过 `working-stack doc note-meta` 读取，完整列表用 `working-stack doc` 或 `working-stack doc --list`；嵌套文档使用列表中的名称。

JSON 写入例：`{"op":"note","args":{"key":"checklist","body":"正文","meta":["appendix","order:10","自定义标签"]},"reason":"更新"}`（另提供 stack）。引用提示可用 `{{note:checklist.meta[0]}}`。

### 3.5.1 全局导览与Owner复工内容

`description`约定为本栈全局导览，MD中直接打印在#标题后，不显示内部key/meta；其meta仍保留但不搬动导览位置。填写范围与材料定位，不另抄一份目标/动态状态。例子（其中用户确认须有真实来源）：

```bash
working-stack <NAME> note '累计原则' --body '用户已确认：本轮只准备候选，验收由用户决定。' -r '登记已有明确裁决'
working-stack <NAME> note description --body '本栈承接交付目标。复工约束：{{note:累计原则.body}}。长实验依据按需读取，不记录全部内部工具流水。' -r '建立范围与材料导览'
working-stack <NAME> note '累计原则'  # 只读该note，不需要show note子命令
```

累计原则不是强制note名，也不能与已有权威材料重复；可指向正确归属。导览可说明本层owner的委派来源及保留裁决，不作为权限引擎或自授权依据。普通文字指路不自动展开，显式引用在context中展开；MD保持原引用。继任者能恢复owner目的、认识依据与下一责任才是目的，不要求每帧统一“当前判断”。原进入原因不倒写；新裁决影响父计划时同步归属。

当前事实被替代时用get/set或按标题覆盖remark。log仅保存原因/变更字段，不自动备份旧正文；仍支撑当前判断的历史成果、反例及推翻依据须保留。执行进展、主张可信、行动授权不是同一阶段轴。

### 3.6 引用 — 在正文中链接栈内容

在 `--goal`、`--plan-text`、`--remark` 等正文参数中，用 `{{路径}}` 插入结构化引用：

```bash
# 引用帧目标
--goal "参见 {{T-01.goal}} 的总体要求"

# 引用计划项
--remark "依赖=被 {{T-02.plan[0]}} 阻塞"

# 引用事件记录
--result-add "据 {{T-02.log[0].detail}} 记录"
```

**引用路径**：`帧ID.字段`、`帧ID.列表字段[下标]`、`note:键名.body`
- `T-01.title` — 标题
- `T-01.goal` — 弹帧目标全文
- `T-01.plan[0]` — 第一个计划项正文
- `T-01.log[3].detail` — 第四条事件的内容
- `note:checklist.body` — 共享 note 正文

**校验**：保存时自动检查所有引用可解析。无效引用会被拒绝。

### 3.7 按路径编辑计划（推荐）

用 `get` 读取结构，不必从 Markdown 再提取计划：

```bash
working-stack <NAME> get 'T-01.plan' --json
```

JSON 请求的 value 可以直接是字符串、对象或数组：

```json
{"stack":"mytask","op":"insert","args":{"path":"T-01.plan[1]","value":"插在第二项之前"},"reason":"补步骤"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan[0].text","value":"新正文"},"reason":"细化"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan[0].progress","value":"done"},"reason":"完成"}
{"stack":"mytask","op":"set","args":{"path":"T-01.plan[0].children","value":["子步骤一","子步骤二"]},"reason":"拆分"}
{"stack":"mytask","op":"remove","args":{"path":"T-01.plan[1]"},"reason":"不再需要"}
{"stack":"mytask","op":"move","args":{"from":"T-01.plan[0]","path":"T-01.plan"},"reason":"移到末尾"}
```

直接向 plan/children 列表 insert 表示末尾追加；带 [i] 表示在该下标之前插入。set 计划项为字符串仅改正文，保留状态/子项/fork；set 为对象替换整项，set plan 为数组重拟整段。move 的目标按移除来源后的结构解释。索引是位置，不会自动调整旧引用；无效候选不会保存。

重新推进已 done 的计划项时，设置对应 `.progress` 为 `in_progress`；这不是帧 reopen。`update <ID> --reopen active|paused` 只用于更正已误关的整个帧，参数不是计划下标。

新增备注用 `update --remark '标题=正文'`；路径 set 只覆盖已存在的映射成员，不能把新备注正文当作该参数的格式。

完整操作域、其他文本/note 编辑及例子：`working-stack doc path-edit/README`。get 是结构查询，新的深层读取路径不自动扩张 `{{引用}}` 的 resolve 支持范围。

## 4. JSON stdin 通道（AI 推荐方式）

除了 CLI 参数，你可以通过 **stdin 传一个完整 JSON 请求**（允许多行）。CLI 和 Web 共用参数转换；CLI 默认返回 JSON，Web 返回更新后的状态快照，响应格式不同。

```bash
# 查询
echo '{"stack":"mytask","op":"show","args":{"what":"overview"}}' | working-stack

# push
echo '{"stack":"mytask","op":"push","args":{"parent":"T-03","title":"调查","goal":"找到根因"},"reason":"用户要求"}' | working-stack

# update 计划状态
echo '{"stack":"mytask","op":"update","args":{"id":"T-03","plan_set":["0=in_progress"]},"reason":"开始"}' | working-stack

# pop
echo '{"stack":"mytask","op":"pop","args":{"id":"T-04","summary":"瓶颈定位为 N+1"},"reason":"完成"}' | working-stack
```

**JSON 格式**：`{"stack":"栈名","op":"动词","args":{...},"reason":"..."}`
- `stack`：栈名或文件路径（同 CLI 第一个位置参数）
- `op`：操作动词（push/pop/pause/resume/switch/update/note/show/validate/export/init/web/list）
- `args`：操作参数，字段见下表。文本列表字段接受字符串或字符串数组。
- `reason`：push/pop/pause/resume/switch/update/apply/edit 和 note 写入必填；查询不需要。
- `json`：可选，默认 true；false 使用原 CLI 文本输出。
- 成功返回 JSON：`{"ok":true,"diff":[...],"version":"...","warnings":[],"text":null}`；错误仍写 stderr，并返回非零退出码。
- 字段缺席表示不修改；update 的 `goal:[]`、`origin:[]` 清空；`title:""`、note 的 `body:""` 传入空文本，由原操作及不变量决定是否允许。
- 非空 stdin 无法解析、字段拼错或类型不匹配会报错，不回退执行原 CLI 参数。

| op | args 字段 |
|---|---|
| init | title、prefix |
| push | parent（省略建根）、title、goal、origin |
| pop | id、summary、expired（bool）、focus_to、residual、pos |
| pause / resume / switch | id |
| update | id、title、goal、origin、result_add、side_add、principle_add、settle、remark、plan_add、plan_set、plan_text、plan_add_child、reopen |
| note | key、body（省略为读取）、meta（字符串数组） |
| show | what、id（frame/log 时需要） |
| validate / list / timeline | 无 |
| doc | name（可省略 .md，可嵌套）、list（bool） |
| get | path |
| set / insert | path、value（JSON 值，无需转成字符串） |
| remove | path |
| move | path（目标）、from（来源） |
| export | format、out |
| import | md、out |
| apply | candidate |
| edit | 无（使用 `$EDITOR`，仍为交互式操作） |
| web | port（整数） |

`get` 直接返回 `{ok,path,value,version}`，value 为选中结构。`show` 的 JSON 响应中 `text` 仍是人读文本；要获取完整结构化栈，使用 `export`，`format:"json"`，再解析响应的 `text`。请求转换及空值合同见 `json-channel.md`。

两种方式并存——CLI 参数适合快速手敲，JSON stdin 适合 AI 编程调用。

## 5. 查询面（AI 工作流）

### 4.1 快速恢复方向（供调用方校准）

优先`working-stack <NAME> show context`，先理解全局导览，再看所选责任链；显式分支用`show context T-02`。它呈现本链目标、未完成计划路径、已记录结果/实际执行事项、备注/原则与暂停条件，并展开显式引用；不按最近条数/字符数裁掉必要正文，也不无条件读全部notes或日志。旧正文未收敛时输出仍会长，完整不等于语义充分或工具提速。无焦点不会自动选分支；记录时间不等于事实时间，更不是最新用户意图。对照本次裁决与实际执行增量决定下一步，范围与维护例子见`working-stack doc context-view`。


```bash
working-stack <NAME> show context     # 一次恢复目标链/焦点/当前项及可编辑路径
working-stack <NAME> show overview    # 更短的树与焦点总览
working-stack <NAME> show focus       # 焦点帧全节 + FOCUS 项
working-stack <NAME> show tree        # 树结构
```

### 4.2 深入分析

```bash
working-stack <NAME> show frame T-02     # 完整帧内容
working-stack <NAME> show log T-02       # 变化记录
working-stack <NAME> show timeline       # 时间线 Markdown（兼容查询入口）
working-stack <NAME> timeline --json      # 全帧/栈级事件、来源下标及真实时间排序
working-stack <NAME> show notes          # 共享内容列表
working-stack <NAME> show pops           # 已关闭帧及结论
```

### 4.3 导出

```bash
working-stack <NAME> export md --out stack.md   # 人读 Markdown
working-stack <NAME> export json                 # 机器可读 JSON
```

### 4.4 JSON 输出（加 --json）

命令支持 `--json`；原查询/修改响应包含 `{ok, diff, version, warnings, text}`，路径 get 则直接带结构化 value：

```bash
working-stack <NAME> show overview --json
```

## 6. 三种目标模式

| 用法 | 模式 | 适用场景 |
|---|---|---|
| `working-stack mytask …` | 托管 | `<cwd>/.working-stack/mytask.yaml` + 自动 Git |
| `working-stack path/to/file.yaml …` | single / 独立文件 | 直接操作该文件，不自动托管 Git |
| `working-stack path/to/WS.md …` | md 管道 | 解析 → 变换 → 重印（非原生承载） |

托管模式每次操作自动 Git commit（仅该文件），可回溯全部变更历史。single 是同一个命令接口的独立文件模式，不需要额外开关：传文件名如 `task.yaml` 或完整路径即识别为文件；用相同 init 创建文件，再用同一路径查询/修改。single 不创建 .working-stack 或自动 Git 提交；保存会建立所需父目录，文件自己的仓库仍可由使用者管理。

时间线汇总全部帧（含关闭帧）和全局日志；`timeline` 输出 Markdown，`timeline --json` 输出带 frame/log_index 的结构化事件。同刻或时间不完整不能推断精确跨帧先后，规则见 `working-stack doc timeline`。

## 7. 与Owner协作形成工作理解

### 7.1 人类owner的当前认识不是进度评分

主动说明交付回答了什么、证据/适用范围哪里不足以及用户需要判断什么；接住提问与纠正。用户问“边界是否覆盖”不表示用户误信测试失效；若AI混答版本，应纠正自身。用户明确未读才写未审阅，无反馈写未知。具体已对齐A/B、C待解释有用，“理解80%/完全理解”无据无用。审阅、理解、采用不是线性等级，批准探索不等于接受结果。

### 7.2 复杂plan和achievement的最新人类状态

复杂plan保留任务名称，在正文附对象/版本及对话依据支持的短协作说明，使用set字符串保留progress/children/fork。长说明或跨项裁决放所属备注/note。AI尚欠解释时仍推进，不把任务全部挂成等待用户。

成果事实与依据独立；最新人类状态建议用所属当前帧按标题覆盖备注，例如`比较报告｜人类协作`，正文写报告v2、实际对话来源、已对齐/缺口与下一责任。不以结果下标充当新身份。ClosureItem仍是关闭时历史达成记录，不不断改成人类当前状态，也不让动态note反向改变过去验收。

```bash
working-stack mytask update T-01 --remark '比较报告｜人类协作=对象：报告v2；据H12。用户明确已看主结论、未看附录；A已确认，C待AI解释；仅允许试用，正式采用待裁决。' -r '据真实反馈覆盖当前协作状态；不代表已经验收'
```

H12是例子的来源代号，真实使用必须可定位。只展开影响当前动作的内容，不强制每项填表或人工关卡。详细推荐和JSON plan例子见`working-stack doc owner-collaboration`。

### 7.3 自治与部分委派

帧是owner需要继承的理解/裁决上下文，普通实现过程留计划。技术子目标可由明确受托的主线程AI依据实际回执判Done；若发布/根验收仍归用户，就继续留在父责任，不因子任务Done自动通过。主线程已采用结论不证明客观正确，也不取得上层保留权。

权限或目标真正不清且影响当前动作时，向有权owner询问具体取舍；不是每步询问“可以继续吗”。没有新owner字段、两套模式开关或心理画像。

### 7.4 Web共同维护

人类用`working-stack <NAME> web`查看/编辑同一份数据；CLI与网页通过同一操作链更新。选中、颜色或结构绿只说明展示/形式，不能当用户理解、采用或验收。具体刷新行为与浏览器验收边界见现有界面文档，不由协作说明改变实现。

## 8. 不变量速查（保存前自动校验）

- I1 父链无环、每帧可达
- I3 全栈至多一个 Active tip（= TOP）
- I4 Paused 帧必须是 tip 且有理由
- I5 已关闭帧不复活（误记用 Correct 更正）
- I7 计划项 fork 指向存在的子帧
- I8 所有 `{{引用}}` 可解析且无环
- I9 已关闭帧的原则全部结算；Done 帧无剩余义务
- I10 活帧的父帧也必须是活帧

违反任何一条 → 保存被拒绝 → 修复后重试。

## 9. 常见错误与处理

| 错误 | 原因 | 处理 |
|---|---|---|
| `父帧不存在` | parent ID 拼错 | 用 `show tree` 确认 ID |
| `存在 live 子帧` | pop时子责任仍open | 先完成或按owner裁决退出子责任；pause仍live，不能因此pop父帧 |
| `原则未结算` | 有 `fate:null` 的原则 | 先 `--settle` |
| `不是当前焦点 tip` | pause 只作用于 TOP | 先 `resume` 到该帧 |
| `引用未解析` | `{{路径}}` 无效 | 检查帧 ID 和字段名 |
| `reason 必填` | 忘了 `-r` | 补上理由重试 |
| `基准冲突` | 另一端（网页/AI）已改 | 重新查询再操作 |

## 10. 最佳实践

1. **操作前先查**：`show context` 取回当前目标链与焦点，对照本次用户消息；只需更短树总览时用 `show overview`。
同次 `update` 在每个实际改变的帧只生成一条 Update，detail 保留原因并附净变更字段摘要；独立 Correct 等事件及历史记录不合并。无净变化不造 Update。

2. **写有信息量的 reason**："用户要求先调查再修复" > "进入"。理由是变更记录的灵魂。
3. **--summary 写结论不写过程**："性能瓶颈定位为 N+1 查询" > "花了两小时调查"。
4. **用引用避免重复**：`--goal "参见 {{T-01.goal}}"` 而非抄写整个目标。
5. **部分完成用 --expired + --residual**：不假装完成，如实记录剩余义务和承接。
6. **规则须有本层效力来源**：owner确立或明确委派内形成的规则可登记；观察到的“重要事实”不自动成为原则。用户保留的规则裁决不能代签。
7. **操作后确认**：看返回的 diff 确认变更符合预期。

## 11. 导入旧 Markdown 的内容保留

```bash
working-stack import path/to/WS-MAIN.md -o mytask
working-stack mytask export md --out reprinted.md
```

识别出的目标、计划等进入核心字段；帧内无法识别的栏目、checklist 表格和未知计划状态原文进入该帧 `remarks`，标题以 `导入原文 · 原栏目名` 标识。没有帧归属的内容进入全局 `import-cache` note。warning 表示没有语义化解析，不代表这些正文被丢弃。

这些备注是原文，不是工具确认过的计划状态或原则；不要从保存成功推断状态已完成。可通过 `show frame <ID>` 查看，或通过 `export json` 获取 `frames[id].content.remarks` / `notes`。

重印会调整缩进，并对字面模板花括号采用 SourcefulText 转义（如 `\{{ skill_dir }}`），避免误当栈引用；再次导入恢复同一正文。内容保留合同见 `import-preservation.md`。

原则与结论条目重印时，`原则/处置`、`结果/依据` 分开列出，字段值使用 JSON 编码保留正文及引用；处置标记不会追加进原则正文，证据不会并入结果。具体格式见 `working-stack doc markdown-fields`。
