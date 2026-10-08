# WS 工具框架架构（D.root）

> 本文件回答：允许什么交互、每个操作对核心语言数据结构的含义、打印/Web 视图如何设计、存储与并发合同、实现选型。Q 见 `principles.md`；核心语言蓝本见 `../playground.md`（冲突处以蓝本为准）。不含 I（代码）。

## D.root — 实现框架

### Upstream
- root：Q.I.1–Q.I.7
- 无父层；本层 P 中带全局效力的声明为 P.global。

### P

**P.global.1 权威源唯一**：scope=全部组件。一切状态出自 ws 文件经核心库解析的值；其余表示（概览、MD 导出、Web DOM、缓存）皆为派生投影，不承担权威。excludes：第二份人工同步的状态表示。

**P.global.2 变更统一通道**：scope=全部前端与核心。任何修改 = 一次 Tx 提交：操作必带 reason；提交前 validate 全量不变量（I1–I10）+ 基准指纹冲突检测；通过后由工具自动生成 Event（action/at/detail含reason及必要净变更摘要）追加相关帧 log 并原子落盘。excludes：绕过通道直写文件后被当作已提交、手工编写 Event。

**P.global.3 同一核心库**：scope=CLI、Web、导出、测试。四个前端只做参数/呈现转换，全部调用同一 ws-core 的 View/Tx；不存在第二套状态逻辑。excludes：Web 端私有写路径。

**P.global.4 语义不判**：scope=全部组件。工具不判目标达成、原则确认、验收；结构合法仅证明形式。excludes：从 validate 绿灯推出语义结论。

**P.local.1 原子持久化**：落盘 = 临时文件 + fsync + rename；读者永见完整版本；提交失败不报成功。
**P.local.2 单写者串行**：同 store 并发写以文件锁串行化；基准（读入时内容指纹）不符即冲突返回，不静默覆盖。
**P.local.3 可移植**：单二进制零运行时依赖（Q.I.5）；时间源=宿主时钟 ISO8601 带时区，不倒推。
**P.local.4 Git 限域可选**：store 位于 git 仓库内时自动 commit **仅本次变更的 ws 文件**；绝 `add -A`/`-am`；仓库外静默跳过。（Q.E.1）

### M — Operational Model

**components**：
- `D.storage`：ws.yaml 编解码。contract：P.global.1；parse∘format 幂等。
- `D.core`：类型 + View/Tx + resolve/render/diff/validate + 操作语义。contract：蓝本 1:1；P.global.2/4。
- `D.cli`：动词层（人读 + `--json`）。contract：P.global.2/3；面向 AI。
- `D.web`：内嵌 HTTP，同一 API。contract：P.global.3。
- `D.print`：tree/frame/timeline/export。contract：只读派生，标记识别约定。
- `D.runtime`：锁/原子写/时间/git。contract：P.local.1–4。
- `D.docs`：帮助入口、动态文档读取、伴随安装/发布。contract：Q.I.7；P.global.3（CLI 只读，不依赖栈）。

**composition**：CLI/Web → ws-core Tx（reason 必填）→ validate(I1–I10) → diff 生成 → runtime 原子落盘（可选 git commit）→ 返回（版本指纹 + diff 摘要 + Event 清单）→ 各视图按新版本刷新。读路径：View 函数直接跑在解析值上，无隐藏状态。

### R

**Derivation**：Q.I.1→P.global.1；Q.I.2→P.global.3；Q.I.3→P.global.2；Q.I.4→P.global.4；Q.I.5→P.local.3；Q.E.1→P.local.1/2/4；Q.E.2→全局"派生优先"贯穿 P.global.1。
**Satisfaction**：组件 contract 覆盖 P 全部条目；D.docs 的发现→读取→分发组合承接 Q.I.7；组合链路（提交→验证→落盘→返回）无绕行点；未 discharge：无。
**Optimality**（Q.A.3）：Rust——enum+serde 无损映射蓝本和类型（FrameStatus/Fate/SourcefulText 的 match 穷尽性由编译器保证）、单二进制含 CLI+HTTP、纯函数 validate/diff 易测。Go：部署同级但和类型经 interface 表达笨拙；TS/Python：运行时依赖违反 Q.I.5；Haskell：用户排除。**选 Rust**。

### Expansion

`D.storage`/`D.cli`/`D.web`/`D.print`/`D.runtime` 展开如下；`D.core` 无独立架构问题（蓝本即规格），`implemented-by` 实现轮。

---

## D.storage — ws 文件规范（三后端）

### Upstream
- parent obligation：P.global.1。
- applicable global P：P.global.1。

### 目标解析（三模式，用户零负担区分）
- `STACK-NAME`（无路径分隔符、无已知扩展名）⇒ **托管模式**：`<cwd>/.working-stack/<name>.yaml`；目录内自动 git（无 .git 则 init），每次成功写操作 commit 仅该栈文件，message = `ws: <op> <name>: <reason 摘要>`；git 缺失则警告跳过。
- `*.yaml/*.yml/*.json` 路径 ⇒ **文件模式**：无自动 git（用户自己的仓库管）；完整读写。
- `*.md` 路径 ⇒ **md 管道（非 native 承载）**：parse=import_md → Tx 变换 → re-print=export_markdown。不是存储后端：每次写都整份重印为 canonical 导出形，不保留原文件排版/家族变体（差异=已文档化的导出映射，首次写时提示）；只读查看零副作用。支持查看现有家族文件与 ad-hoc 变换。

### P
- 顶层四键：`title:`/`prefix:`（展示级元数据，可缺省）+ `notes:`（Map String Note）+ `frames:`（Map Id Frame）；另 `log:`（栈级事件，可缺省）。无 heads。未知键解析报错（deny_unknown_fields，fail closed）。
- 和类型 YAML 编码统一用判别字段 `kind`：

```yaml
# FrameStatus
{kind: active}                       # Active
{kind: paused, reason: <ST>}         # Paused Reason
{kind: done, closure: {…}}           # Done Closure
{kind: expired, closure: {…}}        # Expired Closure
# ItemState: ready | in_progress | blocked(reason) | item_done | item_expired(reason)
# Fate: kept_local | released_to(id) | invalidated(reason) | reclassified(target)
# Maybe: null / 值；List: YAML 列表；Map: YAML 映射
```

- **SourcefulText 编码**：文件里是普通字符串 + `{{引用路径}}` 插值；parse 展开为段列表，format 还原规范形（norm 后幂等）。`{{` 字面量以 `\{{` 转义。引用路径语法 = 蓝本 RefSeg：`12.goal`、`note:checklist-07.body`、`12.plan[1]`、`12.log[3].detail`。
- Time：ISO8601 带时区字符串；宿主生成。
- 扩展字段：未知键解析报错（fail closed），不留静默忽略。

### M
serde 类双向 codec：`parse :: Text -> WorkingStack`、`format :: WorkingStack -> Text`（canonical：字段定序、norm 后 SourcefulText）。
### R
幂等合同 `parse(format(x)) = parse(x)` 由编码表逐类型保证；`\{{` 转义消除插值歧义。JSON 是 API 交换格式，不是人编文件（AI 大段编辑时也可用 `ws apply` 提交 JSON/YAML 整文件，见 D.cli）。
**implemented-by** 实现轮。

---

## D.cli — 交互框架（操作 = 核心语言的 Tx 面）

### Upstream
- parent obligation：P.global.2/3；面向 AI 执行（Q.I.2）。

### P
- 每个动词 ↔ 蓝本 Tx 操作一一对应；reason 必填（`-r/--reason`，缺省报错）。
- 输出双模式：默认人读；`--json` 供 AI（含版本指纹、diff 摘要、生成 Event、Problem 列表）。
- exit code：0 成功；2 结构/校验失败（stderr 列 Problem 定位到帧/字段）；3 基准冲突；4 IO/锁失败。
- 路径内容编辑复用候选提交：get/set/insert/remove/move 的路径、支持域、简写及原子性合同见 `path-edit/README.md`；语法定位由 stack_path 承担，候选变换由 path_edit 承担。生命周期、身份和日志仍由原操作拥有。
- 批量原文编辑与单词元同等入口：`ws apply <file>`（整文件候选 → parse → validate → Replace 提交）；`ws edit` = 调 $EDITOR 生成候选后走同一通道。

### M — 动词表（语义 = 对数据结构的变换）

| 动词 | 参数（均含 `-r`） | Tx 语义 | 自动 Event |
|---|---|---|---|
| `ws init` | — | 建空栈（0 帧 0 note） | — |
| `ws push <parent>` | `--title --goal [--origin] [--from-plan-item i]` | 新帧 Active 入 parent.plan（或经 fork 挂接）；原 Active tip → Paused | 新帧 Push；旧 tip Pause |
| `ws pop <id>` | `--summary`，默认Done；可选`--expired/--residual/--pos/--focus-to` | status ← Done/Expired；principles 先经update结算，关闭前无live子帧；回parent或指定focus | 帧Pop |
| `ws pause <id>` | reason 即暂停理由 | tip → Paused | Pause |
| `ws resume <id>` | — | Paused tip → Active（原 Active tip 同批 Paused） | Resume + 旧 tip Pause |
| `ws switch <id>` | — | 仅切焦点：旧 Active tip Paused，目标 tip Active | 双方各一条 |
| `ws update <id>` | `--set goal/origin/title/...`、`--plan-add/--plan-set <i> <state>`、`--result-add`、`--principle-add/--settle <i> <fate>`、`--remark <title>` | 字段级更新；CLI按每帧/提交聚合净变化，保留其他语义事件 | Update含原因及变更字段 |
| `ws note <key>` | `put --body/--meta`、`get` | 单份正文更新，全部引用处生效 | 引用帧不记（I6）；note 版本在 diff |
| `ws get <path>` | `--json` | 实际 JSON 结构值，不展开引用 | — |
| `ws set/insert/remove/move` | path + JSON value/from + reason | 内容候选 → 同一 apply/validate/save | 净差异事件 |
| `ws apply <file>` / `ws edit` | 候选整文件 | Replace 通道：parse → I1–I10 → 净差异 | 按实际差异生成（不虚构中间操作） |
| `ws show tree\|frame <id>\|log <id>\|timeline\|checklist` | `--json` | 只读 View | — |
| `ws resolve <ref>` / `ws render <ref>` | — | 调试引用路径 | — |
| `ws validate` | — | 全量不变量 + 引用解析 | — |
| `ws export md\|json` | `--out` | 派生导出 | — |

### R
动词表覆盖蓝本全部 Tx 操作 + 只读面；`update` 的字段选择器与 RefSeg 字段名同一词汇，降低两套命名漂移风险。**implemented-by** 实现轮。

---

## D.web — 网页视图

### Upstream
- parent obligation：P.global.3；人可观看/操作（Q.I.2）。

### P
- 后端 = 同一 ws-core：`GET /api/*` 直接绑 View 函数，`POST /api/<verb>` 直接绑 Tx 动词（参数与 D.cli 一一对应，reason 必填）；目标解析同 D.storage 三模式（web 亦可用 STACK-NAME）。
- 只绑定本机回环；无账号体系（Q.I.6）。
- 同步：响应带版本指纹；前端轮询指纹（SSE 可后加），指纹变则重拉受影响视图。
- 写路径不绕过 P.global.2：POST 同样走 validate→原子落盘，返回同构结果（Problem/diff/Event）。

### M
页面 = 树视图（同 D.print tree 的 DOM 版，可点选帧）+ 帧详情卡（栏目/计划/原则/Closure/log）+ 时间线（跨帧 Event 合并）+ 内联编辑（正文 → `POST /api/update`；整帧编辑 → `ws apply` 同通道的 `/api/apply`）。静态资源内嵌二进制。
### R
无第二套状态逻辑（P.global.3）由"端点=动词/视图函数"的直射结构保证。**implemented-by** 实现轮。

---

## D.print — 打印视图

### Upstream
- parent obligation：P.global.1（只读派生）。

### P
- `tree`：live 森林；para 仅在兄弟分叉处缩进；末端帧标 `TOP` / `TOP/PAUSED`（焦点帧且其 FOCUS 项=Blocked 时）`[PAUSED]`；origin 首段 `[digression]` → 概览标 `[digression]`；Paused reason 首段 `[HumanReady]` 等约定标记 → Mark 并从显示文本截取；popped 折叠为计数行。
- `frame`：栏目化排版（goal/plan/FOCUS 高亮/content/principles+fate/Closure 三方向+residuals/log 尾部）。
- `timeline`：跨帧 Event 按时间合并；`export md`：人读文档（含已 pop 帧达成情况）。
- 打印永不回写数据；标记识别只影响显示。
- Note.meta 为字符串列表，不改变业务语义；默认 Appendix。打印器按顺序应用认识且适用的位置/排序提示，其余忽略但保留。具体合同见 `note-meta.md`。

### R
全部标记（TOP/PAUSED/digression/HumanReady）皆派生或文本约定，对应 Q.E.2/E.3 与蓝本 renderTop。**implemented-by** 实现轮。

---

## D.docs — 文档入口与分发

### Upstream / P / M / R

父层分配 Q.I.7 的完整使用入口与随工具可读职责。具体合同、来源选择、ENTRY 排序、profile 及安装/发布分工统一由 `doc-command.md` 定义；`ENTRY.md` 为实际入口，`ai-manual.md` 为操作手册。CLI 帮助负责发现，读取器负责唯一来源、动态扫描及入口全量索引，安装/发布脚本负责 release 所需的文档与附带语义 SKILL。组合后覆盖该职责，不另造内嵌副本或文档索引账。

---

## D.runtime — 持久化与并发

### Upstream
- parent obligation：P.local.1–4。

### P / M
文件锁（同 store 互斥）→ 读入+指纹 → 候选 Tx → validate → Event 追加 → canonical format → tmp+fsync+rename →（git 内则 commit 仅该文件，消息=动词+reason 摘要）→ 释放锁。时间源 UTC ISO8601。崩溃恢复：rename 前中断 = 旧版本完好，无半写状态。
### R
P.local.1/2/4 的最小实现面；git 集成限域直接编码 Q.E.1 教训。**implemented-by** 实现轮。

---

## 里程碑

M1 ws-core（类型/codec/validate/diff）→ M2 CLI 全动词 + 自动 Event + 原子落盘 → M3 打印视图 + export → M4 Web 只读 → M5 Web 写路径 + git 集成。

## 遗留决策点（不阻塞架构）

1. 跨 store 结构化引用：当前 Reference 无栈身份，跨文件引用退化为纯文本（实践亦如此）；要不要 `RefSeg` 加 `StackK` 待定。
2. `remarks :: Map Title SourcefulText`：key 唯一但展示序=字典序；若顺序重要改 `[Remark]` + 唯一性不变量。
