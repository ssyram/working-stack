# 细化设计 — working-stack 实现框架（M1–M5 + import + AI 查询面）

> 规约源：`../playground.md`（核心语言，权威）+ `architecture.md`（P/M/R）。本文按 /workflow §4.3.3 与 /hoare-prompt 纪律给出函数级合同与论证；规约一律取自设计文档，不从直觉补设。错误模型：`WsError { Usage(String), Parse(String), Invalid(Vec<Problem>), Conflict, Locked, Io, Yaml }`；CLI exit：0 成功 / 2 Invalid+Usage+Parse / 3 Conflict / 4 Io+Locked+Yaml。

## 1. 范围

- 覆盖：text/model/view/resolve/validate/ops/persist/diff/print/import/cli/web。
- 不覆盖：git 自动提交（M5 后评估）、跨 store 结构化引用、SSE。
- 蓝本增补（打印保真与承载需要，均记录回 playground）：
  - `WorkingStack` 增 `title :: Maybe String`（文件头 `# …`）、`prefix :: Maybe String`（ID 打印形如 `T-01`）。二者为展示级元数据，不入帧数据。
  - `notes` 值由二元组改为同构结构 `{meta, body}`（YAML 可读性）。
  - `PlanItem.item :: (ST, [PlanItem])` 落为同构命名字段 `text + children`。

## 2. 复用与依赖

serde/serde_yaml/serde_json/chrono/clap/tiny_http。无其他运行时依赖（Q.I.5）。

## 3. 数据结构映射表（YAML 编码）

- ST ↔ 字符串（插值）；`Vec<ST>` ↔ 字符串列表（段落）。
- 和类型：`kind` 判别（见 architecture D.storage；`Action` 外标签：单元变体为字符串，`Other(s)` 为 `{other: s}`）。
- `Map` ↔ `BTreeMap`（确定性序）。`Maybe` ↔ `Option`（null）。

## 4. 模块细化（函数合同）

### 4.1 text

**parse_st(s) -> Result<ST>**
- Requires：s 为 UTF-8（Rust 保证）。
- Ensures：成功 ⇒ 每个 `{{ref}}` 成 `Segment::Ref(parse_ref)`；`\{{` 成字面 `{{`；输出为 norm 形（相邻 Text 已合并）；`{{` 无闭合或 ref 非法 ⇒ Err（fail closed，不静默吞）。
- 论证：单遍扫描，i 每轮严格前进（终止）；三分支（转义/引用/普通字符）按前缀互斥穷尽；norm 在出口统一执行 ⇒ 幂等。

**format_st(st) -> String**（trivial：Text 段转义 `{{`→`\{{`，Ref 段输出 `{{render_ref}}`）
- 性质：`parse_st(format_st(norm(x))) == Ok(norm(x))`（转义双射，见设计论证；测试钉住）。

**parse_ref(s) / render_ref(r)**（trivial 文法：头 `note:key`｜`N`｜`PREFIX-N`，尾 `(.field | [i])*`）
- Ensures：`render_ref(parse_ref(s)) == s` 对规范形成立。

### 4.2 view（全为纯派生）

**tips(ws)**：`live`（Active∪Paused）中无 live 子者。单遍：收集 live 集与 live-父集，差集即 tips（每帧入集一次 ⇒ O(n)）。
**focus(ws)**：Active tips 恰一 ⇒ Some；0 ⇒ None；>1（非法态）⇒ Some(最小 id) 保持确定性，validate 负责报错。
**focus_item(frame)**：前序遍历 plan 首个 `InProgress|Blocked` 项（等待也是当前工作项——实践 OP-23）。
**render_top(frame)**：`"TOP"`；focus_item 为 Blocked ⇒ `"TOP/PAUSED"`。
**print_id(ws,id)**：prefix 存在 ⇒ `{prefix}-{id:02}`（宽度取家族惯例 2，超长自然扩展）否则 `{id}`。

### 4.3 resolve / render

**resolve(ws, r) -> Option<ST>**
- Requires：任意已解析栈（可含悬挂引用）。
- Ensures：路径逐步核存在性与边界；Frame 头 ⇒ 文本 `[print_id title]`；字段表：title/goal/origin/status/plan/log/results/side_effects/principles/remarks；`Field+Index` 支持 `plan[i]`（该项 text）、`log[i]`（其后可再 `.detail/.at/.action`）、`results[i]`、`principles[i]`；`NoteK` 后可 `.body/.meta[i]`（meta 为普通字符串列表）。越界/未知 ⇒ None。
**render(ws, st, expanded)**：展开用深度上限 8（环安全）；未解析引用渲染 `⟨未解析:ref⟩`。validate 另做显式 visited-DFS 判环（不依赖上限）。

### 4.4 validate（I1–I10 → 机械检查）

1) parent 存在、无环（DFS 三色）、从 roots 可达；fork/ReleasedTo 指向存在；2) para 结构（派生自明，不查）；3) Active tips ≤1；4) Paused ⇒ tip（无 live 子）；5) 终态不复活由 ops 保证（fresh_id 单调）；6) notes 单份（结构自明）；7) fork 目标 parent==本帧；8) 全部引用 resolve 成功且引用图无环（visited-DFS）；9) 已关闭帧 fate 全 Some ∧ Done ⇒ residuals 空；10) live 帧的 parent 为 live（或根）。问题串带定位（帧 id / note key / ref 串）。

### 4.5 ops（`impl WorkingStack`；全部 `&mut self`，事件自动追加）

通式 Requires：目标帧存在；reason 非空（宿主填）；操作前置由本函数自查并 `Result` 报错；保存时再整体 validate（双层：前置拦语义错，validate 拦结构错）。

- **fresh_id**：max+1（空 ⇒1）。Ensures：新 id 必不在 frames（I5 支持）。
- **push(now,parent,fields,reason) -> Result<Id>**：parent 存在∧live。旧 Active tip（若有且≠parent 链上新帧）→ Paused（机械理由"去向：新帧 {id}"）+ Pause 事件；新帧 Active + Push 事件。Ensures：新帧为唯一 Active tip；frames +1。
- **pop(now,id,outcome,focus_to,reason)**：id 为 live tip；无 live 子（I10）；principles 全结算（未结算 ⇒ Err 列出）；closure.summary 非空。status←Done/Expired + Pop 事件；焦点：focus_to=Some(t)（t 须 Paused tip）⇒ t 转 Active+Resume 事件；否则父帧若无其他 live 子自动成为派生焦点（无需写状态——父本就 Active），有则整线暂停。
- **pause(now,id,reason)**：id 为当前 Active tip ⇒ Paused+Pause。Ensures：Active tips=0。
- **resume(now,id,reason)**（CLI 别名 switch）：id 为 Paused tip；旧 Active tip（若有）→ Paused+机械事件；id→Active+Resume。Ensures：id 为唯一 Active tip。
- **set_goal/origin/title、plan_add、plan_set(path,state)、result_add、side_effect_add、principle_add、settle(idx,fate)、remark_set(title,body)**：库单次调用净变化才记 Update（空转不记——I6/信息增量原则）。plan_set 沿 indices 下钻 children，越界 Err。
- **note_put(key,meta,body)**：单份更新；不产生帧事件（I6），差异由 diff 呈现。
- **reopen(now,id,live_state,reason)**：仅 Done/Expired → Active/Paused；Correct 事件保留原关闭记录语义（历史事件不删）。

### 4.6 persist（三后端）

**resolve_target(arg) -> Target**：含路径分隔符或以 .yaml/.yml/.json/.md 结尾 ⇒ 文件目标；否则 ⇒ 托管目标（`<cwd>/.working-stack/<name>.yaml`，不存在且非 init/list ⇒ Err 列可用栈）。**native 承载仅 yaml/json**（codec 双向稳定）；**md 是变换管道非承载**（parse=import → Tx → re-print=export，每次写重印为 canonical 导出形，首次写提示差异，只读零副作用）。
**load(path)**：yaml ⇒ codec；md ⇒ import_md（warnings 随载入返回）。Ensures：raw 保留供冲突检查。
**save(path, ws, base_raw)**：validate 空 ⇒ 否则 Invalid；锁内重读 == base_raw 否则 Conflict；按后端序列化（yaml=format；md=export_markdown）→ tmp → fsync → rename。托管模式成功后 git add+commit 仅该文件（缺 git ⇒ 警告跳过）。
**FileLock**：`<path>.lock` create_new+PID；存在且 mtime>60s 可抢占（重试一次）；Drop 删除。

### 4.7 diff(old,new) -> Vec<String>

帧级：新增/删除 id；共有帧比较 title/goal/origin/status/plan/content/log-len → "F-12: goal 改变" 等；notes 增删改。仅摘要（不复制正文）。

### 4.8 print（保真目标：与 WS-TOOL/WS-MAIN 仅排版差）

**export_markdown(ws)**：`# {title}` + 描述段；`## 栈状态`（树算法：dash/缩进仅出现在兄弟分叉层；单链自然向下——父缩进不增、无 dash；分叉层子代缩进+2 加 dash）；`## 已 pop 节点`（id 降序）；帧节（live 树序后 popped 降序）：`## [id]{标记} title`，标记=tip? TOP/TOP/PAUSED/PAUSED : POPPED : 无；栏目序=上帧/来源/弹帧目标/〔达成情况〕/计划/已得结果/外部执行事项/确立原则/备注/### 帧变化记录；`## 待归属缓存`（未被引用 notes）。
- 保真映射（实质性差异，均为设计语义）：`**[FOCUS/X]**` → `[X]`（FOCUS 性由导出器对派生 focus_item 加粗还原，非存储）；达成情况 free-text → Closure（结果=summary；对应/依据 → positive 逐条）；`- 暂停原因：` 行 → Paused.reason（tip）/remark（焦点帧）；`P1 …` 编号导出时自动重编；计划嵌套缩进+2。
Principle 与 ClosureItem 的可重读字段格式及对应解码合同统一见 `markdown-fields.md`：rule/fate、result/evidence 分栏，值复用现有 JSON codec，不从自然语言后缀猜结构。

**print_tree(ws)**：仅树块。

### 4.9 import（解析 WS-TOOL/WS-MAIN 家族）

**import_md(md) -> Result<(WorkingStack, Vec<String> warnings)>**
- 文法（按家族固定，fail-loud：不认识的形状进 warnings，不静默丢）：
  - `# X` 首个 → title；至下个 `##` 前散文 → description（ST，加粗保留）。
  - `## 栈状态` fenced block：行 `[ID]{ [标记]} title` / `- [ID]…`；父=缩进严格小于当前 i 的最近帧（无则前一帧——覆盖根下首层 dash）。标记 TOP/TOP/PAUSED/PAUSED/TOP/digression 记录（digression → origin 首段加 `[digression]`，与打印约定闭环）。
  - `## [ID] {标记} title` 节：标记 POPPED ⇒ 关闭（由达成情况定 Done/Expired；缺达成情况 ⇒ 保守 Expired+warning）；TOP/TOP-PAUSED ⇒ Active（TOP/PAUSED 的等待差异不伪造数据，派生渲染差异记 warning）；PAUSED ⇒ Paused。
  - 栏目行：`上帧：无|ID`；`来源：`→origin；`弹帧目标：`→goal；`达成情况：`→Closure（结果行 `[done]X`/`[expire:R]X` → kind+summary；对应/依据 bullets → positive 条目）；`计划：` bullets：`[state( · label)?]` / `**[FOCUS/state]**` → state（label 并入文本 `（label）` 前缀），嵌套缩进 → children；`已得结果：`/`外部执行事项：` → 列表；`确立原则：` `P\d+ …` → 去编号入 rule；`备注 · T：`/`备注：` → remarks（同名 title 追加段落）；`暂停原因：` → Paused.reason（Active 焦点帧则入 remark + warning）。
  - `### 帧变化记录` → 事件（action 词、时间串原样保留、detail 原文）。
  - `## 已 pop 节点` 交叉核对；`## 待归属缓存` → 单条 note `import-cache`。
- Ensures：产出栈或 Err；成功 ⇒ validate 通过或 warnings 含未通过项（已关闭帧 principles 未结算 ⇒ 机械默认 KeptLocal+warning）。
- 论证：家族格式由 check_working_stacks 系约束（行前缀/标记枚举固定），故按固定文法解析；所有分支覆盖标记全集（穷尽 match，未知进 warnings）。
- FOCUS 处理（用户裁决"自行处理"）：`**[FOCUS/X]**` = 显示层强调 ⇒ 导入为状态 X 的普通项；导出器对派生 focus_item 重新加粗 ⇒ 往返不丢语义，FOCUS 不入存储（与蓝本一致）。

### 4.10 cli（动词 = 操作面；AI 查询面；三模式目标解析）

目标解析见 §4.6；`list` 动词（无栈参）列 `<cwd>/.working-stack/` 下栈名。

`doc` 和帮助不走栈目标解析。读取、列表、环境变量与 debug/release 来源及伴随安装合同见 `doc-command.md`（D.docs 权威）；doc_catalog 实现目录选择/扫描/原文返回，安装与发布脚本建立 release 布局。

变更动词（均 `-r/--reason` 必填）：init/push/pop/pause/resume(switch)/update(子旗标+reopen)/note/apply/edit。
查询动词（只读，`--json` 供 AI）：`show tree|frame|log|timeline|notes|pops`，加：
- `timeline` / `show timeline`：全帧及栈级事件按解析后的RFC3339时刻稳定排序，带来源/日志下标，未知时间不补造；CLI新事件用纳秒表示。完整合同见 `timeline.md`。
- `show overview`：/recalibration Print A——每根行：goal 首段、path(root→focus)、paused tips+理由、FOCUS 项、下一 ready 项。
- `show focus`：焦点帧全节 + FOCUS 项 + 阻塞点。
- `import <md> -o out.yaml`。
路径 get 与 set/insert/remove/move 见 `path-edit/README.md`；候选在内存完成后复用 apply_events/apply_checks/save。apply_events 对发生变化的帧追加原因事件，对 note 变化追加栈级事件；无净变化不追加。生命周期及日志不属于路径编辑器写域。

CLI update 在提交边界按帧合并本次新增Update，原因后附净变更字段，其他动作和旧日志不动；path/apply的单帧Update也带摘要。合同见 `fixes/skill-trial.md`。

输出：人读默认；`--json`：{ok, diff, events, version, problems}。exit 码见错误模型。

### 4.11 web

GET `/`（内嵌页）、`/api/state`（栈 JSON+派生 focus/tips+渲染 MD）、`/api/version`（raw 哈希）；POST `/api/op` `{op,args,reason}` 与 CLI 同一 dispatch（load→op→validate→save，锁内）。轮询 version 刷新。绑定 127.0.0.1。

## 5. 测试计划

单元：parse_st/format 往返（含转义/空/未闭合）、parse_ref 文法、tips/focus 合成栈、push/pop/pause/resume 事件序与焦点派生、validate 各 I 反例（双 Active tip、Paused 非 tip、I10、悬挂 ref、引用环、未结算 pop）。集成：ops→save→load 等值；import WS-TOOL.md → export 与原文 diff 仅排版（人工核对映射项）；apply 净差异事件。性能：单文件千帧内交互式（无断言，观察级）。

## 6. 自检 checklist

- [x] 推导连续/分支穷尽（ops 各返回路径显式）、退出（Result 全覆盖）、callee 合同（resolve/validate 被引用处注明）、循环（扫描/DFS 终止论证）、假设链（规约源自 playground/architecture，无凭空新增）。
- [x] non-trivial 论证就位；trivial 标注。
- [x] 保真映射清单显式（FOCUS/达成情况/暂停原因/编号）。

## 7. 审查修订记录（2026-10-06，oracle/metis/spec-impl-auditor 三路审查后）

代码注释已就地标注来源编号；此处记实现与本文差异的权威裁定：

1. **S2/F1 裁定**：reopen 保留（实践 OP-19 原位撤回）；旧关闭类别与summary保存于Correct事件；完整Closure快照不在当前自动保存范围。蓝本 I5 措辞已同步澄清。
2. **S8 裁定**：import 对已关闭帧未结算原则默认 KeptLocal——"无迁移证据 ⇒ fate=留在原处"是忠实保守读法（原则文本物理在本帧），warning 提示复核。
3. **S3/F19/F20**：锁无自动抢占（删他者锁是竞态）；锁路径 = 追加 `.lock` 后缀；锁内基准检查兜底并发。
4. **S4/F05**：转义全量双射（`\\`→`\\\\` 再 `{{`→`\{{`；Text 末尾 `{` 邻 Ref 转 `\{`）。
5. **S5/F09/F10/metis F3/F4**：pop 焦点归一（选择序 focus_to > 未关闭原焦点 > 新暴露父 tip > 无；其余 Active tip 机械转 Paused）；focus_to ≠ 自身。
6. **F08/metis F5**：push 对 Paused 父转回 Active（祖先链）；旧焦点 = parent 时不暂停。
7. **S6**：栈级 `log :: [Event]`（蓝本已增补）；note_put 带 reason 记入。
8. **S7**：链路顺序修正为 Tx（含事件追加）→ validate → 落盘（实现即此序）。
9. **S9**：export 增 `## 共享内容` 节承载全部 notes 全文（description 除外）；import 解析之；md 自往返稳定。
10. **S10/F04**：render 循环防护 = 展开路径引用不重复（无深度截断）；resolve 只取结构、不内嵌渲染。
11. **C1/metis F2**：apply 转移检查（旧 id ⊆ 新、关闭不复活、log 前缀只增、新 ReleasedTo 严格祖先、Reclassified 物化引用链接）。
12. **C2/F12**：diff 比较整个 status 值 + parent + log 内容。
13. **F06/N1**：Note 路径 = body/meta[i]（无 .title；Meta 合同见 `note-meta.md`）。
14. **F07**：全结构 deny_unknown_fields。
15. **F14**：引用 canonical = 纯数字（带前缀形式仅解析便利）。
16. **F21**：ReclassTarget 判别字段统一 kind。
17. **F22**：Id = u64（蓝本 Integer 收窄，负数不在域）；fresh_id checked_add。
18. **F15**：删除"已关闭帧不得保留暂停原因备注"的伪不变量。
19. **F16**：树/节序仅 live 根；关闭根不重复。
20. **F18**：settle/remark_set 净变化才记。
21. **metis D2/D4**：关闭帧计划=关闭时快照（不产生当前 FOCUS）；位置引用=当前槽位语义。
22. **metis F6/F7/D3-A**：settle 物化承接——ReleasedTo 须严格祖先且接收帧追加引用型原则（fate=None）；Reclassified 目标同 Tx 追加对源原则的引用。
23. **metis F9**：focus() 在 0 或多个 Active tip 时返回 None（不伪造权威）；树显示 [TOP?] 诊断标记。
