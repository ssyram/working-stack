# Owner中心的完整模拟：从委托到可用认识与验收

这是工具实际可执行的脱敏模拟，**不是原项目执行记录**。用户消息H01–H09、技术回执R1/R2均为下面明确设定的模拟材料；运行命令只登记它们，不会产生真实用户确认或业务测试。真实MD/YAML/context/diff和回执由案例脚本生成，见末尾索引。

本例根owner为人类用户，AI协助其判断候选是否可用。H01明确委派AI主线程在技术准备/核对标准内自行判断，W-03/W-05因此是局部自治，不逐项等待人审；范围变化与整体验收仍归用户。执行工具的人不因执行自动成为owner。

在空练习目录使用托管名`practice`；single模式把所有命令的practice替换成`./practice.yaml`，同接口但不自动Git。写入按既有核心操作完成，不新增owner/human_status字段或理解等级。准确格式见`working-stack doc ENTRY`，协作态度见`working-stack doc owner-collaboration`。

## 1. H01：用户委托与有界自治，不把目标变成AI清单

H01设定：用户要一个通过直接测试的候选，并保留是否满足约定验收条件的裁决；明确允许主线程自主准备/核对共通部分。用户尚未对未来报告给出审阅反馈，应记未知，不能声称未读或完全理解。

```sh
working-stack practice init --title 'Owner判断候选是否可用' --prefix W
working-stack practice note '累计原则' --body '模拟H01：用户是根owner；AI主线程在已委派技术标准内自主准备/核对共通部分。用户保留范围变化、发布与整体验收；AI交付不表示用户已理解或接受。' -r '登记明确模拟授权来源，不由调度身份取得裁决权'
working-stack practice note description --body '本栈承接用户关于候选可用性的工作与认识进程，不是AI完成清单。Owner及授权：{{note:累计原则.body}}。当前用户提问/对齐/采用与AI欠项在根帧所属备注；技术成果在各子责任，共享checklist按需读取。' -r '建立owner范围与材料导览，不复制动态进度账'
working-stack practice push --title '支持用户判断并完成交付' --goal '候选版本通过直接测试' --goal '用户确认约定的验收条件满足' -r '依据H01登记完整委托，技术交付不替人类裁决'
working-stack practice update W-01 --plan-add '与用户澄清验收条件' --plan-add '按授权准备并直接测试候选' --plan-add '协助用户判断候选是否满足约定条件' --plan-set 0=in_progress --remark 'Owner与边界=据模拟H01：根owner为用户；AI可自主技术准备，但不能扩大范围、发布或代签整体验收' --remark '候选报告｜人类协作=对象：将交付的候选及报告；据H01。用户提出可用性要求，尚无报告审阅反馈，审阅/具体采信状态未知；AI仍欠可用材料及解释' -r '把owner要求、AI责任与保留裁决登记到所属工作'
working-stack practice show context
```

根计划描述owner需要完成的工作，而不是“AI写代码/跑命令/交文件”就结束。导览引用已授权边界，普通checklist指路不自动全文读取。原目标仍有两项，不以owner当前疑问替换它们。

## 2. H02：接住用户问题，AI先履行解释义务

H02设定：用户问两种条件解释是否覆盖不同输入，不表示用户已经相信候选有错。AI需先讲清对象与差别，不能立即把整项写成“等待用户批准”。W-02有独立裁决目的与回程，所以push；普通解释步骤仍在plan。

```sh
working-stack practice push W-01 --title '协助用户澄清条件选择' --goal '取得用户对争议条件的明确选择' -r 'H02涉及保留给用户的条件裁决，需要独立上下文'
working-stack practice update W-02 --plan-add '解释两种条件的对象、差异与依据' --plan-set 0=in_progress --remark '用户协作=据模拟H02：用户询问两种条件的覆盖差别；尚未选择，AI仍欠清楚解释。问题不等于用户误信候选错误' -r '保留用户具体问题及AI欠项，不给理解评分'
working-stack practice show context
working-stack practice update W-02 --result-add '模拟：AI已说明两种条件的适用对象与覆盖差别，尚无用户选择' --plan-set 0=done --remark '用户协作=对象：条件解释v1；据模拟H02后AI说明。解释材料已交，用户是否审阅/采信无反馈，状态未知；选择仍归用户，后续提问AI继续负责回答' -r '登记解释交付，不把交付变成用户已理解'
working-stack practice pause W-02 -r '[HumanReady] 解释材料已提供，当前等待用户选择条件；不代签选择'
```

暂停发生在本轮解释已提供之后；新疑问到来仍可恢复。材料齐全是执行事实，不是用户理解或目标达成。

## 3. 共通部分可推进：AI技术owner不被人工关卡堵住

H01已授权共通部分，W-03由AI主线程按技术标准作本层判断；它的Done将只代表技术准备达成，根用户验收仍open。W-02保持暂停，para表示上下文并存，不表示两个进程必然同时运行。

```sh
working-stack practice push W-01 --title '在授权内准备并直接测试候选' --goal '共通部分实现并完成直接测试，争议条件留下显式接口' --origin '据H01技术委派，成果支持根owner的可用性判断；不承担用户验收' -r '共通部分不依赖等待中的选择，主线程可在授权内推进'
working-stack practice update W-03 --plan-add '实现共通部分' --plan-add '直接测试' --plan-set 0=in_progress --remark '本层Owner=AI主线程；据模拟H01的技术委派，以实际回执判断本层达成，范围与根验收仍归用户' -r '明确局部自治责任，不为每个技术步骤新增人审'
working-stack practice note checklist --body '技术回执、AI采用的结论、用户理解/采用与根验收分别登记；不因计划全勾推断后者。' --meta appendix -r '单份检查提示，不复制到每个帧'
```

AI自身“已采用”不是结论正确认证，仍需真实依据。权限只来自H01设定，不能由这条备注自授权。

## 4. 技术缺口改plan：不把发现问题推成用户认识有错

模拟实施中缺测试样例，再发现一个边界问题。这是AI技术责任内的普通步骤，补项/拆分/重拟即可，不为工具调用造帧，也不缩小根目标。

```sh
working-stack practice insert 'W-03.plan[1]' --value '"准备测试样例"' -r '技术准备发现缺样例，按既有授权补必要步骤'
working-stack practice set 'W-03.plan[1]' --value '{"text":"准备测试样例","children":["常见输入","边界输入"]}' -r '把样例步骤细化，不增加人类审阅关卡'
working-stack practice get 'W-03.plan' --json
working-stack practice show context
working-stack practice set 'W-03.plan' --value '[{"text":"实现共通部分","progress":"done"},{"text":"准备测试样例","progress":"done","children":[{"text":"常见输入","progress":"done"},{"text":"边界输入","progress":"done"}]},"修正直接测试发现的问题","重新运行直接测试"]' -r '模拟技术回执暴露问题：保留已完成部分并重拟剩余工作'
working-stack practice update W-03 --remark '测试发现=模拟：边界输入尚不符合约定，AI需修正/重测；不是用户提出问题就证明用户错误或候选全域不可用' -r '记录具体对象与技术欠项，不迁怒用户'
```

插入改变位置，直接测试已在plan[2]；query后再操作，不用旧下标假装改对了项。

## 4A. H03：用户裁决改变路线，父责任同步

H03设定：用户要求材料核对先于最终重测，但没有反馈最终候选的审阅/接受。这项顺序许可不等于验收。更新父计划和实施继续理由，不倒写原PUSH。

```sh
working-stack practice update W-01 --plan-text '1=准备候选：先核对材料，再最终重测' --remark '顺序裁决=据模拟H03：用户新增材料核对先于最终重测的顺序；仅改变路线，整体验收仍保留' --remark '候选报告｜人类协作=据模拟H03：用户已明确选择准备顺序；最终候选审阅尚无反馈，状态未知；AI仍欠材料核对和可靠重测，不将顺序许可当验收' -r '从用户实际选择更新父责任，不仅记AI收到消息'
working-stack practice update W-03 --plan-text '3=核对材料后重新运行直接测试' --remark '继续理由=落实H03后的父帧候选准备责任；AI先修正问题并核对材料，再最终重测；不代签用户验收' -r '同步局部自治路线及当前理由，保留原进入历史'
working-stack practice show context
```

Owner认识与采用范围的变化在其所属内容中呈现；无须给所有条目打“完全理解/理解大部分”标签。

## 5. Owner取消可选回合：成果不等于已采用

H04设定：用户允许初步评估优化，随后取消本轮深入评估。AI只列了收益/成本，收益尚未核实，独立目标未完成。Expired不是“用户没看，所以AI的工作失败”，而是owner取消该回合。

```sh
working-stack practice push W-01 --title '形成可供Owner判断的优化方案' --goal '形成有依据的完整优化比较，供owner决定是否采用' -r 'H04允许独立可选评估，不取得本轮实施许可'
working-stack practice update W-04 --result-add '模拟：AI已列初步收益与成本，收益未核实，尚未形成完整可采用方案' -r '交付边界明确，不把AI草案变成owner已采用'
working-stack practice update W-01 --remark '后续线索=据模拟H04：用户取消本轮深入优化，保留初步材料而不增加交付义务；没有表示接受优化结论' -r '登记owner取消及成果使用范围'
working-stack practice pop W-04 --expired --summary 'H04终止未完成的可选评估回合' --pos '保留初步收益/成本材料' --focus-to W-03 -r '按owner取消退出，返回已暂停的技术责任'
```

暂等输入应pause；明确取消才Expired。未来新要求不自动复活这次回合。

## 5A. 多para与局部技术owner

出现材料核对与另一条线索两个独立上下文。H01授权AI作材料对应检查；H04同时明确规则：证据交付和owner采用分别登记。四个未结束子帧并存，焦点唯一，不因用户待选择就把所有技术工作停止。

```sh
working-stack practice push W-01 --title '核对供Owner使用的交付材料' --goal '核对候选与材料对应关系，将依据交还实施责任' -r '独立核对目的与回程，按H01委派的技术标准执行'
working-stack practice update W-05 --plan-add '核对材料与版本对应关系' --principle-add '据模拟H04确认：证据交付与owner采用必须分别登记' --principle-add '本层技术规则：只核对当前候选的材料对应关系' --remark '本层Owner=据H01，AI主线程负责技术核对达成；不判断用户已读报告，根验收仍归用户' -r '登记有效来源及局部判断边界，不只列AI待办'
working-stack practice push W-01 --title '形成另一条线索的取证安排' --goal '形成有依据的线索取证安排，供owner判断是否继续' -r '模拟Owner允许暂留独立线索，不承诺实施'
working-stack practice pause W-06 -r '[HumanReady] 模拟Owner暂未决定继续此可选方向；不伪造接受'
working-stack practice show tree
```

## 5A1. 改挂责任，不重写原owner授权或PUSH

核对实际属于W-03技术准备，而不是根的独立交付；parent编辑改变责任关系，不自动转交owner权力。先恢复接收live子帧的父帧，再用YAML候选只改parent，原log不动。

```sh
working-stack practice resume W-03 -r '恢复承担核对责任的技术父帧；Paused只用于tip'
working-stack practice export yaml --out candidate.yaml
python3 -c 'from pathlib import Path; p=Path("candidate.yaml"); t=p.read_text(); p.write_text(t.replace("\n  5:\n    parent: 1\n","\n  5:\n    parent: 3\n",1))'
working-stack practice apply candidate.yaml -r '核对归实施责任，原授权范围与PUSH历史保留'
working-stack practice resume W-05 -r '按已委派核对标准继续本层技术工作'
working-stack practice set 'W-03.plan[3]' --value '{"text":"核对材料后重新运行直接测试","fork":[5]}' -r '技术计划定位核对子责任，不复制另一份进度账'
working-stack practice get 'W-03.plan[3]' --json
```

## 5A2. 技术owner据证据收尾，不等待用户理解每个底层细节

模拟对应检查通过；W-05技术目标已满足，主线程可在H01授权内判Done，不要求用户先读懂每个核对步骤。规则按真实责任结算，原原则通过引用承接，不复制另一份权威。

```sh
working-stack practice set 'W-05.plan[0].progress' --value '"done"' -r '模拟对应检查通过，属于本层技术判断'
working-stack practice update W-05 --result-add '模拟：当前候选与交付材料对应关系已核对，供上层采用；不证明用户已读/接受' --settle 0=release:W-01 --settle 1=kept_local -r '技术目标达成，通用规则归根责任、局部规则留本帧'
working-stack practice pop W-05 --summary '技术owner在H01范围内确认材料核对达成' --pos '模拟材料对应检查依据，交还上层使用' -r '关闭已委派技术子目标，不代签用户验收'
working-stack practice update W-03 --remark '材料核对依据=见 {{W-05.results[0]}}；用户阅读/采用状态仍由根责任登记' -r '明确技术成果与人类协作不是同一事实'
working-stack practice pop W-06 --expired --summary '模拟Owner取消尚未完成的线索取证回合' --residual '保留线索，不构成当前交付义务' -r '据明确取消收尾，不包装成已完成方案'
```

## 5B. 计划重排/删除服务Owner目的，不以忙碌证明进展

新增检查，改名、移动；核实与已完成W-05重复后删除当前冗余步骤，保留原过程证据。普通步骤不push，也不以加更多检查冒充充分性。

```sh
working-stack practice insert 'W-03.plan[2]' --value '"重复核对候选材料"' -r '模拟暂列一项与目标相关的检查'
working-stack practice set 'W-03.plan[2].text' --value '"再核对候选材料"' -r '明确对象后再判断必要性'
working-stack practice move 'W-03.plan' --from 'W-03.plan[2]' -r '调整执行顺序，不改Owner目标'
working-stack practice get 'W-03.plan' --json
working-stack practice remove 'W-03.plan[4]' -r '已有对应核对足够，重复项无新增贡献，移除当前冗余'
```

## 6. 技术交付v1不推出人类已经理解或采用

这里先登记模拟R1“通过”并关闭技术帧；下节明确注入一次技术误判。即使这次判断当时被登记为通过，也不允许把根目标或人类状态一同写Done。

```sh
working-stack practice set 'W-03.plan[2].progress' --value '"done"' -r '模拟修正已完成'
working-stack practice set 'W-03.plan[3].progress' --value '"done"' -r '先登记模拟R1重测通过；下一节将核实其误记'
working-stack practice update W-03 --result-add '模拟R1：v1共通候选被登记为直接测试通过，争议条件仍显式；不表示用户已接受' -r '技术判断只适用于本层范围'
working-stack practice pop W-03 --summary '先按R1登记技术准备完成，根用户判断仍open' --pos '当时登记的R1结果与材料核对' -r '本层技术owner收尾，不将交付当用户认识完成'
working-stack practice update W-01 --remark '候选报告｜人类协作=对象：候选v1；AI已交技术材料，用户审阅/采信未反馈，状态未知；根责任仍欠可用性判断，不声称用户未读或完全理解' -r '成果事实与最新人类状态分别登记'
working-stack practice show context
```

## 6A. 技术误关闭更正：AI承担错误，不归成用户偏差

模拟核实：R1没有包含一次必要重测，AI曾混答完成状态。因此更正原关闭，保存关闭前完整快照。不是用户有疑问就“用户理解错了”，也不是正常新要求复活旧帧。

```sh
working-stack practice export yaml --out before-correct.yaml
working-stack practice update W-03 --reopen active -r 'AI核实R1遗漏必要重测，原技术Done依据不成立；在原授权内纠正'
working-stack practice set 'W-03.results[0]' --value '"v1候选及材料已交，但原R1通过判断撤回：一次必要重测未完成。原判断保存在before-correct.yaml及阶段快照。"' -r '替换旧当前成果判断，保留错误依据，不无限append更正'
working-stack practice set 'W-03.plan[3].progress' --value '"in_progress"' -r '恢复真实未完重测，保留fork'
working-stack practice update W-03 --remark '误记更正=AI曾将R1未完成重测混答为通过；更正责任在AI，不能将其写成用户认识错误' --principle-add '模拟事实误列为规则：v1候选材料对应关系已核对' -r '保留错误及责任，识别事实并非行为规则'
working-stack practice update W-03 --settle 0=reclass:results -r '将已核对的材料事实从原则转存，不从重要性产生规范效力'
working-stack practice set 'W-03.plan[3].progress' --value '"done"' -r '设定模拟R2实际补跑通过'
working-stack practice set 'W-03.results[0]' --value '"模拟R2：v2补跑直接测试通过；v1通过判断曾被撤回，不能倒写成当时已证实。"' -r '当前技术成果指向可靠补跑依据，保留必要反例'
working-stack practice pop W-03 --summary '技术owner据R2确认v2准备目标达成，原误关已更正' --pos '模拟R2与材料对应证据；不声明用户已经采用' -r '在已委派范围内据补齐依据收尾'
working-stack practice update W-01 --remark '候选报告｜人类协作=对象：v2及R1更正说明；AI已提供补跑依据，用户是否阅读更正尚无反馈，状态未知；AI需在沟通时解释v1/v2差别' -r '更新最新人类协作对象，不反写过去理解'
```

Correct自动保存旧关闭类别/摘要，不是完整Closure快照；本例额外导出原YAML并保存逐阶段实物。成果事实保留在技术帧，最新用户阅读/理解/采用留在根责任。

## 7. H05：条件选择完成，不代表报告整体接受

H05设定：用户明确选择第一种条件解释，但尚未对候选v2的效果结论作接受。恢复同一条件帧，不重建；该独立裁决目标可以Done，根可用性判断仍须协助。

```sh
working-stack practice resume W-02 -r '收到模拟H05对条件解释的明确选择'
working-stack practice update W-02 --result-add '据模拟H05：用户选择第一种解释；这只闭合条件选择，不等于接受候选v2的整体效果' --remark '用户协作=H05明确选择第一种条件，选择目标已成立；不推定全部报告审阅或所有细节理解' -r '保存具体选择及适用范围，不提升为全域接受'
working-stack practice pop W-02 --summary '用户owner在H05完成条件选择' --pos 'H05的明确选择，只适用于该条件' -r '关闭本层已达裁决目标并返回根责任'
working-stack practice set 'W-01.plan[0].progress' --value '"done"' -r '条件选择已明确'
working-stack practice set 'W-01.plan[1].progress' --value '"done"' -r 'R2技术准备已达，不代签根用户判断'
working-stack practice set 'W-01.plan[2].progress' --value '"in_progress"' -r 'AI仍需帮助用户理解v2依据，不能直接全挂待用户'
working-stack practice update W-01 --remark '候选报告｜人类协作=据H05：条件选择明确，v2报告审阅/具体采信尚无反馈；AI欠v1/v2与效果边界说明，用户后续决定是否满足约定条件' -r '下一责任同时保留AI欠项与owner裁决'
```

## 7A. H06：复杂plan/result的人类状态，有对象与来源

H06设定：用户明确说已看主结论、未看附录，确认条件A/B，但问“C的证据是否只支持有限输入”。AI此前没有讲清适用范围，仍欠解释；不记录“用户误信结论错误”“理解80%”，也不因“等用户”隐藏AI欠项。

```sh
working-stack practice set 'W-01.plan[2]' --value '"协助用户判断候选：对象v2，据H06，A/B已确认，C待AI解释；详见候选报告人类协作备注，当前不能代签验收。"' -r '复杂plan保留任务并附短协作说明；字符串set不改状态/子项'
working-stack practice update W-01 --remark '候选报告｜人类协作=对象：报告v2；据模拟H06。用户明确已看主结论、未看附录，确认A/B，询问C适用范围；AI欠C反例与范围解释。尚未作整体接受决定，问题不是错误信念。' --remark 'AI解释责任=先说明C的证据只覆盖有限输入，解释v1/R1为何更正为v2/R2；用户判断不能替代AI提供这些材料' -r '按具体对话更新成果的最新人类状态，不改技术历史达成'
working-stack practice show context
```

计划仍in_progress，下一动作属于AI。技术帧W-03的Done不被写成人类接受；其Closure也不不断改成最新理解栏。

## 7B. H07/H08：解释、有限采用和剩余未知

H07设定：AI补充C的反例/适用边界及版本差异；用户复述确认A/B/C这几个具体点，明确未读附录，只批准约定输入范围的试用，不接受整体效果结论。H08设定用户开始试用，当前需真实试用反馈。未读附录不是不允许其授权；有限采用也不是根目标已达。

```sh
working-stack practice update W-01 --result-add '模拟：AI已交v2的C反例/范围说明及R1到R2的更正依据，技术结论仍只覆盖约定输入' --remark 'AI解释责任=据模拟H07，C解释与版本更正说明已提供；后续新疑问仍须响应，不声称交付自动证明理解' -r 'AI补齐自身解释义务，保留适用条件'
working-stack practice update W-01 --remark '候选报告｜人类协作=对象：报告v2及C补充说明；据模拟H07复述，A/B/C已具体对齐；用户明确未读附录，只批准约定输入试用，未接受整体效果。H08试用反馈未返回；AI继续回答新疑问，最终判断归用户。' -r '更新具体对齐、有限批准和未知，不用笼统完全理解'
working-stack practice set 'W-01.plan[2]' --value '"用户判断候选是否满足约定条件：H07已对齐A/B/C且仅批准试用；H08试用反馈待返回，AI已补本轮解释，不要求用户理解全部底层细节。"' -r '复杂项最新说明与成果备注同一来源，不把试用许可当验收'
working-stack practice set 'W-01.plan[2].progress' --value '"blocked:等待约定试用结果及用户裁决"' -r '当前AI本轮材料/解释已齐，实际缺试用与owner保留决定'
working-stack practice pause W-01 -r '[HumanReady] 据H08等待约定试用反馈/用户裁决；不表示后台自动停止，也不是推卸AI欠项'
working-stack practice show context W-01
```

这里适合停下来汇报：用户具体理解到哪些点有据可说，技术子目标已完成，根责任尚未达。真实工作没有H09，就停在这里。

## 8. H09：Owner明确验收，不要求全部细节理解才结束

H09设定：用户在约定输入试用后明确确认两项根目标满足、接受本次交付；没有扩大结论到未覆盖输入，也没有声明自己读完全部底层材料。

```sh
working-stack practice resume W-01 -r '收到模拟H09约定试用结果及明确验收'
working-stack practice update W-01 --remark '候选报告｜人类协作=对象：v2及约定范围试用；据模拟H09，用户明确确认约定条件满足并接受本次交付。A/B/C此前已对齐，附录仍未读；不推出全域效果或全部细节理解。' -r '据实际设定的owner裁决更新当前人类状态，不从计划勾选推出验收'
working-stack practice set 'W-01.plan[2].progress' --value '"done"' -r '根用户判断目标由H09明确闭合'
working-stack practice update W-01 --result-add '据模拟R2及H09：技术直接测试通过，用户确认约定范围验收条件满足；未覆盖输入仍不在结论中' --settle 0=kept_local -r '两项根目标各有依据，规则归根责任保留'
working-stack practice pop W-01 --summary '用户owner据H09接受约定范围交付，两项根目标满足' --pos 'R2技术依据与H09约定范围试用/明确验收，不声明全域结论' -r '依据owner目标与裁决收尾，而非AI完成清单'
working-stack practice validate
working-stack practice timeline
working-stack practice export md --out practice.md
```

最终W-01/W-02/W-03/W-05为Done，W-04/W-06为Expired。技术Done、人类具体认识与根验收分别有来源；取消的方向不包装成采用。复杂achievement的Closure保留关闭时口径，后续新反馈进入其当前所属责任，不借动态note倒写验收。

## 9. JSON只读继承：查询不制造认识或授权

同一CLI也支持JSON输入；最终无open焦点，查询仍可继承导览/既有记录，不自动恢复或判断owner心理。

```json
{"stack":"practice","op":"show","args":{"what":"context"}}
```

完整认识/采用记录在根帧正文与备注里，可显式query该关闭快照。自治主线程采用结论也应记录具体依据/未闭合点，不自述“完全理解”闭合。若整条工作被委派给AI owner，同一模型依授权完成技术判断，无须添加人类审阅阶段；超出委派或遇保留裁决才交上层。

## 10. 真实输出、diff与重跑

[Owner中心本版输出索引](practice-output-owner/README.md)包含managed/single各阶段完整MD/YAML/JSON、context、diff及命令回执。初始导览、有界技术自治、用户提出疑问时AI仍欠解释、有限采用、明确验收及历史Correct都可逐阶段定位。

重跑：`cargo build --locked`后执行`python3 tests/practice-case.py target/practice-owner-replay --binary target/debug/working-stack`。输出目录须未存在。时间戳来自实际运行，不能当模拟对话时间；重跑自然改变时间/指纹。

脚本断言授权内技术帧先Done但根用户目标仍open、H06时AI尚欠解释所以in_progress、H07/H08只批准试用所以仍blocked、H09才闭合根；同时覆盖原目标保持、改挂不改PUSH、规则/Correct、notes往返和两种Git行为。结构测试不证明模拟事实或用户实际理解。

## 从案例继承态度，而不复制答案

维护owner的要求、实际选择/纠正、已有/未采用依据、认识缺口及双方下一责任；不要把AI执行历史当工作本体。只展开影响接续的信息，字段齐全不是充分性。方法例子不是所有任务的强制步骤，简单执行项不必人审。

复杂plan附简短有来源的当前协作说明，长说明在所属备注/note；成果事实独立，最新人类状态按语义标题覆盖。未知不写成未审阅，疑问不写成错误信念；局部授权不自动扩大到根裁决。正常新要求不伪装成误关闭更正。

现有MD多goal可能合并成多行字符串，JSON候选frame键解码和Correct完整快照边界未修；本版不声称整模型MD无损。角色/协作说明都使用现有正文，核心类型、工具操作、状态枚举和独立recalibration方法文件不改。
