# 导览与复工材料修订

## 目的与取舍

用户允许修订保存的PLAN，通过后直接执行。最新实战回复确认context与替换recipe有效，但旧正文未同步仍长，且不能把焦点/内容一起变化归因于工具提速。保留不改核心功能/数据结构的边界，不搭车加preview、细粒度回执或自动语义校准。

## 实际产物

description在#标题后打印，无共享内容/工作线说明包装，不重复打印。note_markdown的附件at_title只用于空导览与旧分节格式辨别，恢复原meta/空白/引用；不进入核心Note。import/print采用相同协议，native不变。

context呈现导览、全部本链已录入结果/外部执行事项、完整长正文与已有显式引用来源；不再以最近3条或240字符省略。只取导览、选中父链、暂停末端及显式引用，不无条件取全notes/关闭帧/log。最后的边界说明不冒充具体下一步。工具不判断记录效力，未收敛输入仍可导致长输出。

SKILL改为维护目的与归属，不强制每帧“当前判断”或五步模板；区分责任相关性、主张依据与行动授权。文档同步导览/取材/旧正文备份边界。独立recalibration文件没有修改。

## 运行依据与未闭合边界

- Rust72/72；JSON38、路径25、修复17、timeline9、doc25、分发15通过；context CLI/JSON只读及help测试通过；HTML JS syntax与7个note-meta局部断言通过。
- 新案例managed/single各77条命令、15阶段、6帧。断言新裁决修改父计划与继续理由、原目标及PUSH保持、notes各阶段往返、原则/Correct和验收边界；实物在references/practice-output-v2。旧版本证据保持原时间，新增版本用真实新时间。
- 真实WS-MAIN当场输入1365行/38帧，输出1361行，完整解析模型再导入差异0，notes/remarks相等、原件未改。证据位于target/ws-main-restructure。
- 10个核心边界文件hash与实施前完全相同：model/ops/view/resolve/validate/persist/diff/path_edit/commit_log/playground。sourceful引用展开复用原renderer，无新核心状态。

案例另暴露既有多goal输出合并成一个多行字符串，未顺带改变目标编码。案例断言明确是native目标保持与notes往返，不声称整模型MD相等；真实WS-MAIN本次解析模型相等是单独依据。JSON候选frame键与Correct完整快照边界也未修。

没有第二轮AI业务复工的量化实证，不报告提速、节省token或语义充分。全局发布目录为target/working-stack-restructure，仍需保留该目录及伴随文档。

## 度量

| 指标 | 数值 |
|---|---|
| 新增代码行数 | 约150（导览协议及测试） |
| 修改代码行数 | 约80（context呈现/既有测试） |
| 删除代码行数 | 约40（固定截断/结果切片） |
| 涉及文件数 | 约16，另224个新模拟证据文件与局部PLAN/学习 |
| 新增测试用例数 | Rust4，既有测试加强，双模式场景扩展 |
| 测试通过率 | Rust72/72；JSON38/38、路径25/25、修复17/17、timeline9/9、doc25/25、分发15/15 |
| 发现 bug 数 | 既有多goal结构保真边界1（未修） |
| 修复 bug 数 | 导览位置/身份往返及context材料省略的展示缺口 |
| 迭代轮次 | PLAN修订1，实现1，测试调整1 |

经验同步.pi/AI-learning.md：完整呈现与实际复工效果分别交付，测试范围不能扩大结论。
