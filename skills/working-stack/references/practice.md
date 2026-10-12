# 从 init 到验收：一条工作线如何随新信息变化

这是可运行的脱敏模拟，不是原项目的执行记录。只借鉴实际管理形态：根目标保留、独立问题下钻、等待与para、计划插入/重拟、用户取消可选支线、交付与验收分开。下面“用户消息”“测试结果”均为设定的模拟事实；执行工具命令不会真的完成业务或取得用户确认。

在一个空的练习目录中执行。`practice` 是托管名称，工具自动创建 `.working-stack/practice.yaml` 并限定Git提交；文件模式可将每条命令的 `practice` 换成 `./practice.yaml`，语义相同但不自动Git。详细参数以 `working-stack doc ENTRY` 的索引为准。

## 1. 初始委托：产物可用，并由用户按约定验收

先建立栈和根帧。根目标不是“写完文档”，也不是“工具运行成功”。

```sh
working-stack practice init --title '交付一个可用版本' --prefix W
working-stack practice note '累计原则' --body '模拟用户已确认：产物直接测试与用户验收分别建立；共通部分可先推进，最终验收仍由用户决定。' -r '登记已有模拟裁决，不由重要性自行造原则'
working-stack practice note description --body '本栈承接可用版本的交付与验收，不记录全部内部执行流水。全局范围约束：{{note:累计原则.body}}。共享检查提示在note checklist中，需要时读取；具体裁决与继续理由保留在所属帧。' -r '建立全局导览并引用单份已确认约束'
working-stack practice push --title '完成交付与验收' --goal '候选版本通过直接测试' --goal '用户确认约定的验收条件满足' -r '登记模拟委托的完整目标'
working-stack practice update W-01 --plan-add '澄清验收条件' --plan-add '准备并测试候选版本' --plan-add '请用户验收' --plan-set 0=in_progress -r '把根目标分成当前可执行计划并落实当前步骤'
working-stack practice show context
```

此时只有W-01，焦点在根帧。三个计划项只是执行安排，不是三个自动完成的目标。description在#标题后打印，context会展开累计原则正文；普通文字指向checklist不自动全文读取，不是新字段或强制note名。

## 2. 新信息：验收条件尚有一个独立裁决问题

模拟发现：某个条件有两种解释，需要用户选择。问题有自己的目标与返回条件，才建子帧；不是每条提问都push。

```sh
working-stack practice push W-01 --title '澄清验收条件' --goal '取得用户对争议条件的明确选择' -r '模拟发现独立裁决问题，解决后返回根计划'
working-stack practice update W-02 --remark '待裁决=两种解释都已列明，尚未取得选择' -r '记录已知事实与仍缺的判断'
working-stack practice pause W-02 -r '[HumanReady] 等待用户选择验收条件的解释'
working-stack practice show tree
```

W-02仍未完成，暂停原因可恢复。等待不是Expired，也不因为AI写好了两种解释就Done。

## 3. 不依赖该裁决的工作继续：建立para

模拟已获授权先准备共通部分。仍在同一根目标下push W-03，W-02保持暂停；不是把原问题移出范围。

```sh
working-stack practice push W-01 --title '准备并直接测试候选版本' --goal '共通部分实现并完成直接测试，争议条件留下显式接口' -r '共通部分不依赖等待中的裁决，可以并行推进'
working-stack practice update W-03 --plan-add '实现共通部分' --plan-add '直接测试' --plan-set 0=in_progress -r '建立本帧的实施计划与当前焦点'
working-stack practice note checklist --body '检查实现、直接测试与用户验收，三者不互相替代。' --meta appendix -r '登记一份共享检查提示，不复制到每个帧'
```

树上同父有两个未结束子帧，焦点只有W-03。para描述保留的上下文，不承诺两个进程同时运行。

## 4. 实施中发现缺口：插项、拆分、重拟，不滥建帧

模拟发现直接测试前还需要准备样例。这仍是W-03目标内的普通步骤。先插入；再把样例项细化成内嵌步骤。

```sh
working-stack practice insert 'W-03.plan[1]' --value '"准备测试样例"' -r '模拟发现测试缺样例，在直接测试前补项'
working-stack practice set 'W-03.plan[1]' --value '{"text":"准备测试样例","children":["常见输入","边界输入"]}' -r '把一个模糊步骤拆成可执行子项'
working-stack practice get 'W-03.plan' --json
working-stack practice show context
```

插入改变了位置：直接测试现在是`plan[2]`，不能照旧使用`plan[1]`。先query再编辑；不要用旧索引声称某项已完成。

模拟实施与样例已完成，测试暴露一个实际问题。更新事实并重拟剩余计划，已完成部分保留。

```sh
working-stack practice set 'W-03.plan' --value '[{"text":"实现共通部分","progress":"done"},{"text":"准备测试样例","progress":"done","children":[{"text":"常见输入","progress":"done"},{"text":"边界输入","progress":"done"}]},"修正直接测试发现的问题","重新运行直接测试"]' -r '模拟测试发现问题：保留已完成步骤，替换尚未完成的测试安排'
working-stack practice update W-03 --remark '测试发现=模拟：一个边界输入尚不符合约定，需要修正后重测' -r '登记具体失败，不把测试已运行写成通过'
```

计划改了，根目标没改；失败不能通过缩小目标或把测试项勾成Done消除。

## 4A. 用户裁决改变路线：更新父计划与继续理由

模拟用户明确新的顺序：先核对交付材料，再完成最终重测；用户验收权不变。不能只在实施子帧追加一句消息，父帧仍让接手者沿旧安排走。修改责任所属计划与备注，不把这次裁决倒写为原PUSH原因，也不改变根目标凑完成。

```sh
working-stack practice update W-01 --plan-text '1=准备候选：先核对材料，再最终重测' --remark '顺序裁决=模拟用户新增顺序：材料核对先于最终重测；验收仍留给用户' -r '按新的模拟裁决更新父计划，不缩根目标'
working-stack practice update W-03 --plan-text '3=核对材料后重新运行直接测试' --remark '继续理由=落实父帧候选准备责任；先修正问题并核对材料，再完成最终重测，不能代签验收' -r '同步实施路线及当前继续理由，保留原进入历史'
working-stack practice show context
```

这里“为何仍要推进”是当前解释，“原先为何进入”是历史；无需强制“当前判断”模板。实际改了哪些内容以本阶段diff为准。

## 5. 出现可选线索：独立支线与取消

模拟用户曾允许评估一个可选优化，所以进入W-04。随后用户明确取消，本轮不实施；保留线索不代表给根帧增加必须完成的义务。

```sh
working-stack practice push W-01 --title '评估可选优化' --goal '决定本轮是否实施可选优化' -r '模拟用户允许评估独立可选方案'
working-stack practice update W-04 --result-add '模拟：已列出收益与成本，尚未实施' -r '区分评估成果和实现成果'
working-stack practice update W-01 --remark '后续线索=模拟用户取消本轮可选优化；仅保留线索，不构成本轮验收义务' -r '根帧记录裁决边界，不偷偷承诺将来实施'
working-stack practice pop W-04 --expired --summary '模拟用户终止本轮可选优化回合' --pos '保留收益与成本比较' --focus-to W-03 -r '按明确取消裁决退出，回到已暂停的实施帧'
```

此处Expired表达用户终止该回合，不伪称优化实施完成。若只是暂等预算，则应pause而不是pop。

## 5A. 多个para：先保留上下文，再修正归属

模拟同时出现“交付材料核对”和“另一个可选线索”两个独立上下文。先挂到根帧，形成W-02/W-03/W-05/W-06四个未结束子帧，再暂停最后的线索；没有为了让树好看而宣告谁完成。

```sh
working-stack practice push W-01 --title '核对交付材料' --goal '核对候选版本的材料，并把核对依据交还实施帧' -r '模拟出现独立的材料核对上下文'
working-stack practice update W-05 --plan-add '核对材料与版本对应关系' --principle-add '模拟已确认规则：交付证据与用户验收必须分别登记' --principle-add '模拟局部规则：本次核对只使用当前候选版本材料' -r '登记核对计划与两条有来源的模拟规则'
working-stack practice push W-01 --title '保留另一条可选线索' --goal '取得用户是否继续此线索的决定' -r '模拟出现另一条独立旁支，保留回程'
working-stack practice pause W-06 -r '[HumanReady] 等待模拟用户决定是否继续可选线索'
working-stack practice show tree
```

## 5A1. 改挂责任归属：不重写最初PUSH

现在模拟明确：材料核对实际归属于W-03，而不是根帧的独立交付。改挂W-05到W-03，只改变当前责任，不重写最初PUSH记录。parent不属于路径编辑器写域，使用完整YAML候选apply；候选仅改parent，既有log全部保留。改挂前先恢复W-03：Paused只允许用于tip，接收live子帧后它将成为内部帧，不能仍标Paused。

```sh
working-stack practice resume W-03 -r '模拟改挂前恢复承担责任的父帧；Paused仅用于分支末端'
working-stack practice export yaml --out candidate.yaml
python3 -c 'from pathlib import Path; p=Path("candidate.yaml"); t=p.read_text(); p.write_text(t.replace("\n  5:\n    parent: 1\n","\n  5:\n    parent: 3\n",1))'
working-stack practice apply candidate.yaml -r '模拟归属澄清：材料核对由实施帧承担；不改最初PUSH历史'
working-stack practice resume W-05 -r '模拟核对条件已齐备，进入改挂后的子帧'
working-stack practice set 'W-03.plan[3]' --value '{"text":"核对材料后重新运行直接测试","fork":[5]}' -r '实施计划登记材料核对子帧，避免另建重复状态账'
working-stack practice get 'W-03.plan[3]' --json
```

## 5A2. 子帧收尾与原则归属

父W-03有live子帧，不能先pop父帧。模拟核对完成后，局部规则留本帧，通用规则按已确认责任上移至严格祖先W-01。原则上移物化引用，不是抄一份看似相同的文字。

```sh
working-stack practice set 'W-05.plan[0].progress' --value '"done"' -r '模拟材料与版本对应关系核对完成'
working-stack practice update W-05 --result-add '模拟：当前候选版本与交付材料一致' --settle 0=release:W-01 --settle 1=kept_local -r '核对目标达成；通用规则归根帧、局部规则保留本帧'
working-stack practice pop W-05 --summary '模拟材料核对完成，依据交还实施帧' --pos '材料与版本对应关系已核对' -r '关闭子目标并返回实施帧'
working-stack practice update W-03 --remark '材料核对依据=见 {{W-05.results[0]}}' -r '引用核对依据，不把所有原文再复制一遍'
working-stack practice pop W-06 --expired --summary '模拟用户决定本轮不继续另一条可选线索' --residual '线索保留为建议，不构成当前交付义务' -r '模拟用户取消可选回合，恢复实施焦点'
```

## 5B. 计划重排与移除：先读路径，再按新位置操作

新增一个模拟重复检查步骤，改名后移到末尾；确认它与已完成核对重复，再移除。普通步骤不push，移除的是当前计划，不擦除历史。

```sh
working-stack practice insert 'W-03.plan[2]' --value '"重复核对候选材料"' -r '模拟暂列一项补充检查'
working-stack practice set 'W-03.plan[2].text' --value '"再核对候选材料"' -r '明确此计划项的对象'
working-stack practice move 'W-03.plan' --from 'W-03.plan[2]' -r '模拟调整步骤顺序，重复检查移到末尾'
working-stack practice get 'W-03.plan' --json
working-stack practice remove 'W-03.plan[4]' -r '模拟确认与W-05的核对重复，无新增贡献，移除当前冗余计划'
```

## 6. 候选产物完成：只关闭对应子目标

模拟实际问题已修正，重测通过。W-03目标满足，但整体用户验收仍未发生。

```sh
working-stack practice set 'W-03.plan[2].progress' --value '"done"' -r '模拟修正已完成'
working-stack practice set 'W-03.plan[3].progress' --value '"done"' -r '模拟直接重测通过'
working-stack practice update W-03 --result-add '模拟：共通部分直接测试通过，争议条件保留显式接口' -r '记录本帧实际达成范围，不代签用户验收'
working-stack practice pop W-03 --summary '共通候选版本准备完成且直接测试通过' --pos '模拟测试结果满足本帧目标' -r '本帧目标达成；根帧仍需裁决与验收'
working-stack practice show overview
```

W-02仍暂停，W-01没有因为兄弟完成而Done。

## 6A. 误关闭更正：保留错误历史，不倒写成功

上节的第一次Done在这里是**故意注入的模拟误判**：后来核实，某次重测实际未完成却被登记为通过。因此恢复同一个W-03，纠正计划和事实。不是正常新任务复活旧帧，也不是证明工具发现了业务错误。

```sh
working-stack practice update W-03 --reopen active -r '模拟核实首次Done依据有误：一次重测未完成，需纠正关闭判断'
working-stack practice set 'W-03.plan[3].progress' --value '"in_progress"' -r '模拟纠正计划误记，保留材料核对fork'
working-stack practice update W-03 --remark '误记更正=模拟：上次Done误把未完成重测当成通过；Correct保留原关闭摘要' --principle-add '模拟事实误列为原则：本轮直接测试已经通过' -r '记录原判断为何失效，不抹去第一次Done'
working-stack practice update W-03 --settle 0=reclass:results -r '模拟识别这是待依据支撑的事实，不是约束后续行为的规则，转存结果'
working-stack practice set 'W-03.plan[3].progress' --value '"done"' -r '模拟补跑直接测试，此次确实通过'
working-stack practice update W-03 --result-add '模拟：补跑直接测试通过；第一次Done是误记，不能倒写成当时已证实' -r '区分原错误判断与后来真实依据'
working-stack practice pop W-03 --summary '模拟补跑完成且直接测试通过，误关闭已更正' --pos '此次补跑测试结果与材料核对依据' -r '按补齐后的依据重新关闭同一子目标'
```

现有Correct保存原关闭类别和摘要，不是全部旧Closure的结构化快照；真实需要完整旧证据时，应在更正前另存原导出快照。本案例各阶段完整快照见末尾证据索引。

## 7. 等待结束：恢复旧帧，不重建问题

模拟用户回复明确选择，同时共通版本的接口与选择一致，无需额外实现。

```sh
working-stack practice resume W-02 -r '模拟收到用户对争议条件的明确选择'
working-stack practice update W-02 --result-add '模拟：用户明确选择第一种解释，候选版本的显式接口与之相符' -r '保存决定与适用范围'
working-stack practice pop W-02 --summary '争议条件已有明确且适用的用户裁决' --pos '模拟用户选择已登记' -r '返回根帧落实验收安排'
working-stack practice set 'W-01.plan[0].progress' --value '"done"' -r '裁决子目标已完成'
working-stack practice set 'W-01.plan[1].progress' --value '"done"' -r '候选版本与已明确条件一致且直接测试通过'
working-stack practice set 'W-01.plan[2].progress' --value '"blocked:等待用户验收"' -r '候选交付不等于验收'
working-stack practice pause W-01 -r '[HumanReady] 候选版本已交付，等待用户确认验收条件满足'
working-stack practice show context W-01
```

这是适合停下来汇报的状态：实施与裁决完成，根目标仍open。真实使用时没有验收消息，就应停在这里，不执行下一节。

## 8. 只有收到验收事实，才关闭根帧

以下继续设定一条**模拟用户确认**：已检查候选版本，约定验收条件全部满足。不是AI看计划全勾选后自己认定。

```sh
working-stack practice resume W-01 -r '模拟用户确认约定验收条件全部满足'
working-stack practice set 'W-01.plan[2].progress' --value '"done"' -r '登记模拟用户验收事实'
working-stack practice update W-01 --result-add '模拟：直接测试通过，且用户确认全部约定验收条件满足' -r '两项根目标的依据均已具备'
working-stack practice update W-01 --settle 0=kept_local -r '模拟已确认的通用规则由根帧持有，退出前明确处置'
working-stack practice pop W-01 --summary '模拟委托的两项根目标均满足' --pos '直接测试结果与模拟用户验收确认' -r '对照完整原目标收尾'
working-stack practice validate
working-stack practice timeline
working-stack practice export md --out practice.md
```

最终W-01/W-02/W-03/W-05为Done，W-04/W-06为Expired；没有open焦点，取消的可选优化不被包装成完成成果。时间线由工具聚合所有帧和栈级事件，不能用事件数量当业务步骤数。

## 9. 同一操作的JSON输入与只读浏览

正文案例默认用CLI便于跟读，AI可直接传一个JSON请求。这里查询最终快照，无open焦点，不制造额外事件：

```json
{"stack":"practice","op":"show","args":{"what":"context"}}
```

先前的插项也可用`op:insert`、`args:{path,value}`；reason仍解释为什么改，不能拿格式示例里的模拟事实替代真实依据。

## 10. 全部实际输出与diff

[本版完整输出索引](practice-output-v2/README.md)包含managed/single各15阶段的真实MD、YAML、逐阶段diff、context视图和完整命令回执。索引还直接展示初始完整MD/YAML、多para状态、改挂diff、待验收与收尾diff；最终文件和时间线也完整保存，不只给模型自述。

重跑：先`cargo build --locked`，再`python3 tests/practice-case.py target/practice-replay --binary target/debug/working-stack`。案例命令从本文代码块读取，测试断言根目标不漂移、改挂不改PUSH历史、验收前保持open、原则承接与Correct存在；managed仅跟踪栈文件、single不自动Git。模拟事实不是这些结构测试能证明的业务真相。

## 从案例迁移到自己的任务

- 套用管理判断，不照抄虚构的业务事实、Frame ID或验收确认。
- 新信息先归类：帧内步骤→改plan；独立目的/裁决与回程→push；外部等待→pause；回复到来→resume；取消→Expired；原目标有实际依据全部满足→Done。
- 误把交付当验收时，有据更正原判断：帧使用`reopen active|paused`，计划项使用路径`set ...progress`；不要把正常新回合当成旧帧复活。工具目前保留旧关闭类别及摘要，不是完整Closure快照。
- JSON输入与上述CLI使用同一操作。例如给`W-03.plan[1]`插入字符串，见`working-stack doc path-edit/README`；先查询实际路径，不能假定旧索引仍表示同一项。
