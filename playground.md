# Working Stack 核心语言 Playground

以用户骨架为基底补全；字段语义见注释，用户自行调整。与 `docs/design.md`（Work 统一提案）冲突处，以本文件为准。

```haskell
type Id = Integer
type Title = String
type Time = String        -- 带时区的真实记录时刻，不倒推

type SourcefulText = [Either Reference String]  -- 正文段列表；引用是结构段，不是字符串内语法

type Reason      = SourcefulText  -- 暂停/退出/阻断理由（可引用承接帧等）

-- 文件层仍是插值语法（如 "见 {{12.goal}}"）：人/AI 直接编辑的是插值形式；
-- parse 产出段列表，format 还原规范形，parse(format(x)) = parse(x)。
-- 合法性检查、引用级 diff、免转义都是段列表的结构性质，不靠字符串扫描。

data Reference = Reference { path :: [RefSeg] }  -- 例：12.goal / note:checklist-07.body / 12.plan[1]

data RefSeg
    = Frame Id       -- 进入某帧；整帧渲染为其 "[Id 标题]"
    | NoteK String   -- 进入某 note（按 key）
    | Field Text     -- 字段名：goal / origin / status / title / ...
    | Index Int      -- 列表下标：plan[2] / principles[0] / log[n]

flatten :: SourcefulText -> String         -- 裸视图：串接 String 段，忽略引用（纯函数）
norm    :: SourcefulText -> SourcefulText  -- 合并相邻 String 段，规范形（纯函数）
-- resolve / render 见下：走 View，隐式携带 WorkingStack

data WorkingStack = WorkingStack {
    title   :: Maybe Title,        -- 文件头打印（工作线名称）；展示级元数据，不入帧数据
    prefix  :: Maybe String,       -- 帧打印前缀（如 "T" → T-01）；展示级，不参与身份
    notes   :: Map String Note,    -- 全局共享内容：单份正文，引用只放 key；Meta 只提示打印位置与顺序
    frames  :: Map Id Frame,       -- 全部帧：在栈 + 已 pop（Done/Expired 即已退出）；ID 稳定不重用
    log     :: [Event]             -- 栈级事件：无帧归属的变更（note/meta 等）记录处
}  -- heads 不存储：tip/焦点/暂停头全部由 frames 派生（见派生视图与 I2–I4）

data Note = Note { meta :: Meta, body :: SourcefulText }  -- 蓝本二元组的可读承载（同构）

data Frame = Frame {
    parent  :: Maybe Id,     -- Nothing = 根帧；父子关系只存这一份，children/path 派生
    title   :: Title,        -- 概览短名
    goal    :: [SourcefulText],  -- 弹帧目标 = 完成判据；多条为合取：全部达成才 Done。Closure 对应它写结果/依据
    origin  :: [SourcefulText],  -- 来源：为何进入本帧；digression 定性写这里（首段 "[digression]" 约定，打印层识别概览标记），不另设类型
    status  :: FrameStatus,
    plan    :: [PlanItem],   -- 本帧计划；某项 push 成子帧时经 fork 挂接
    content :: Content,      -- 正文栏目
    log     :: [Event]       -- 帧变化记录：只追加；更正 = Correct 新条目，保留原文
}

data FrameStatus
    = Active                 -- live 非末帧（祖先，一律 Active，见 I4）与唯一 Active tip（= 焦点，见 I3）
    | Paused Reason          -- 未选中的分支头（必为 tip）；Reason = 在等什么/去向。特殊标记（如首段 "[HumanReady]"）由打印层识别 Reason 文本并 Mark/截取，不入类型
    | Done Closure           -- 已 pop 且弹帧目标达成（goal 合取全部成立）
    | Expired Closure        -- 已 pop 本回合结束：Closure = 成果、未完义务与承接

data Closure = Closure {
    summary    :: SourcefulText,  -- 一句话结论：整体如何对应弹帧目标（done / expire 理由先行）
    positive   :: [ClosureItem],  -- 正向：原目标自身实现了什么
    negative   :: [ClosureItem],  -- 反向：目标本体或其认知前提被怎样推翻（非“未完成”；未完义务在 residuals）
    tangential :: [ClosureItem],  -- 正交：原目标外、有来源依据符合原意的额外收获
    residuals  :: [ClosureItem]   -- 未完义务与承接（引用承接帧）；Done 应为空
}

data ClosureItem = ClosureItem {
    result   :: SourcefulText,  -- 结果描述
    evidence :: SourcefulText   -- 对应弹帧目标的依据
}
```

```haskell
-- 任务项与帧的关系只此一处：fork
data PlanItem = PlanItem {
    text     :: SourcefulText,   -- 条目正文（原 item 元组分量一）
    children :: [PlanItem],      -- 嵌套子列表（原 item 元组分量二；叶子为空）
    progress :: ItemState,   -- 本条自身状态，不从子项聚合（避免双份记账）；帧内 FOCUS = 前序第一个 InProgress 或 Blocked（等待也是当前工作项，实践 OP-23 [FOCUS/blocked]）
    fork     :: [Id]         -- 本条被 push 成子帧时的承接，可一抵多；空列表 = 无承接
}

data ItemState = Ready | InProgress | Blocked Reason | ItemDone | ItemExpired Reason
-- ItemExpired：计划项未完成而本回合不再做（实践如 [expire: partial, see OP-17]），Reason 写承接去向

data Content = Content {
    results     :: [SourcefulText],         -- 已得结果：对应弹帧目标的非平凡收获
    sideEffects :: [SourcefulText],         -- 外部执行事项：真实落盘/提交/启停进程；只读不算
    principles  :: [Principle],             -- 用户确认的原则；pop 时逐条结算 fate
    remarks     :: Map Title SourcefulText  -- 备注 · <名称>：待核判断/背景，可多条
}

data Principle = Principle {
    rule :: SourcefulText,
    fate :: Maybe Fate  -- Nothing = 未结 fate，还在栈内
}

data Fate
    = KeptLocal               -- 留在本帧
    | ReleasedTo Id           -- 上移到接收帧
    | Invalidated Reason      -- 被推翻
    | Reclassified ReclassTarget  -- 误列原则 → 移入结果/备注（保留原文与去向）

data ReclassTarget = ToResults | ToRemark Title

data Event = Event {
    action :: Action,
    at     :: Time,
    detail :: SourcefulText  -- 信息增量：不重复帧内内容，用 Reference 段引用；改 XX→YY 写明出处
}

data Action = Push | Pop | Pause | Resume | Update | Correct | Other String

type Meta = [String]  -- 开放打印提示；认识且适用的应用，其余忽略并保留
-- 例：["appendix", "order:10"]；具体解释由打印层定义，见 docs/note-meta.md
```

```haskell
-- 全局唯一环境 = WorkingStack，经 Monad 隐式携带，不再逐函数显式传。
-- View：只读投影，类型级排除改动。Tx：操作事务——链内每步看到最新候选栈；
-- 边界处 runTx 得 (结果, 新栈)，原栈不消失（值不可变），diff/validate 用 old vs new 对比后提交。
type View a = Reader WorkingStack a
type Tx a   = State WorkingStack a   -- View 可直接提升进 Tx
runTx :: Tx a -> WorkingStack -> (a, WorkingStack)
-- 失败通道未定：可换 StateT WorkingStack Either 或经 validate 返回 [Problem]

-- 派生视图：不单独存（heads 整体可算，不入数据）
children  :: Id -> View [Id]      -- 同父全部子帧（含已 pop，打印时按需过滤）
path      :: Id -> View [Id]      -- 根 → … → 该帧
roots     :: View [Id]            -- parent = Nothing 的帧
live      :: View [Id]            -- status ∈ {Active, Paused}
tips      :: View [Id]            -- live 且无 live 子帧：各分支头
focus     :: View (Maybe Id)      -- 唯一 Active tip；Nothing = 整线暂停
renderTop :: Frame -> String      -- 焦点帧渲染 "TOP"；其 FOCUS 项为 Blocked 时 "TOP/PAUSED"（等待住 plan 项，实践同 [FOCUS/blocked] 等回信）

resolve  :: Reference -> View (Maybe SourcefulText) -- 路径解析，机械查合法性
render   :: SourcefulText -> View String            -- 展开：引用替换为当前内容；帧→"[Id 标题]"
validate :: View [Problem]                          -- 候选栈整体跑 I1–I8；空列表才可发布
diff     :: WorkingStack -> WorkingStack -> [Effect] -- 提交前 old vs new 的机械差异
```

```haskell
-- 结构不变量（机械可查）
-- I1 父链唯一、无环；每帧从某根可达；notes/fork 引用的 Id 全部存在
-- I2 para：同父可同时有多个未 pop 子帧；每个 live 分支沿唯一未完子链恰有一个头（tip）
-- I3 tip ∈ {Active, Paused}；Active tip 全栈至多一个 = 焦点，其余 tip 均 Paused 且带理由；无 Active tip = 整线暂停
-- I4 Paused 必为 tip；live 非末帧一律 Active（祖先不携带暂停理由，不逐层维护）
-- I5 Done/Expired 为终态：不作新活动回合复活（新承接用新帧）；误记经 Correct 更正（可恢复 live，旧 Closure 机械保存于 Correct 事件），原条目保留
-- I6 notes 单份正文；共享更新不改变引用帧的 goal/status
-- I7 plan.fork 中每个 Id 指向存在且 parent = 本帧 的子帧
-- I8 全部 SourcefulText 的 Reference 可解析（resolve 成功）；引用图无环，render 展开有限
-- I9 已 pop（Done/Expired）帧的 principles.fate 全部为 Just（pop 时结算义务）；Done 的 Closure.residuals 为空
-- I10 live 帧的 parent 必为 live 帧（等价：关闭帧无 live 子帧——关闭前子帧须已关闭或改挂）
```

```haskell
-- 操作语义草稿：Tx，环境隐式；Time 由宿主传入（时刻是数据不是环境），事件追加进相关帧 log
type Problem = String                                  -- validate 最小载体，结构化可后补
type Effect  = String                                  -- 机械差异条目最小载体，结构化后补
data Outcome = DoneO Closure | ExpiredO Closure    -- pop 的两种关闭；退出理由在 Closure.summary

freshId  :: Tx Id                          -- 新帧 Id 分配（这也是操作必须用 State 的原因之一）
push     :: Time -> Id -> Frame -> Tx ()   -- 父下新建 Active 帧成为新 tip/焦点；原 Active tip → Paused+Reason
                                           -- para push = 同一操作再用于同父，无专门构造
popFrame :: Time -> Id -> Outcome -> Tx () -- status ← Done/Expired；焦点默认回 parent（可显式指定去向）
                                           -- 父帧综合由人/AI 写入 content，不自动生成
pause    :: Time -> Id -> Reason -> Tx ()  -- 头转 Paused；焦点另选或 Nothing
resume   :: Time -> Id -> Tx ()            -- Paused 头恢复并成焦点；新建承接帧不是 resume
switch   :: Time -> Id -> Tx ()            -- 仅切焦点：旧头 Paused+去向，新头记 RESUME/续行
update   :: Time -> Id -> [Field] -> Tx () -- 字段级更新；净变化才生成 UPDATE，空转不记
```
