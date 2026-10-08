//! 操作面：`impl WorkingStack`（Tx 的 Rust 形态——全部 &mut self，环境即 self）。
//! 合同见 detailed-design §4.5：操作前置自查 + 保存时整体 validate（双层）；
//! 事件自动追加，净变化才记；焦点归一由操作保持 I3（metis F3 / auditor F08–F10）。

use crate::model::*;
use crate::text::{parse_ref, Reference, RefSeg, Segment, SourcefulText};

fn ev(action: Action, at: &str, detail: SourcefulText) -> Event {
    Event { action, at: at.to_string(), detail }
}

fn mech(txt: String) -> SourcefulText {
    SourcefulText(vec![Segment::Text(txt)])
}

impl WorkingStack {
    /// 新帧 Id：max+1（空 ⇒1；u64 上限检查——F22 裁定）。Ensures：新 id 必不在 frames。
    pub fn fresh_id(&self) -> Result<Id, WsError> {
        self.frames
            .keys()
            .max()
            .map_or(Ok(1), |m| m.checked_add(1).ok_or_else(|| WsError::Usage("Id 域耗尽".into())))
    }

    fn require_reason(reason: &SourcefulText) -> Result<(), WsError> {
        if reason.is_empty() {
            Err(WsError::Usage("reason 不能为空（每个操作必带原因——Q.I.3）".into()))
        } else {
            Ok(())
        }
    }

    /// 建根帧（工作线起点）。
    /// Requires：reason/title 非空。Ensures：新根为唯一 Active tip；旧 Active tip（若有）→ Paused。
    pub fn push_root(
        &mut self,
        at: &str,
        title: String,
        goal: Vec<SourcefulText>,
        reason: SourcefulText,
    ) -> Result<Id, WsError> {
        Self::require_reason(&reason)?;
        if title.trim().is_empty() {
            return Err(WsError::Usage("title 不能为空".into()));
        }
        let id = self.fresh_id()?;
        if let Some(old) = self.focus() {
            let msg = format!("去向：新根帧 {}", self.print_id(id));
            if let Some(f) = self.frames.get_mut(&old) {
                f.status = FrameStatus::Paused { reason: mech(msg.clone()) };
                f.log.push(ev(Action::Pause, at, mech(msg)));
            }
        }
        self.frames.insert(
            id,
            Frame {
                parent: None,
                title,
                goal,
                origin: vec![],
                status: FrameStatus::Active,
                plan: vec![],
                content: Content::default(),
                log: vec![ev(Action::Push, at, reason)],
            },
        );
        Ok(id)
    }

    /// push：父下新建 Active 帧（下钻/resume 语义）。
    /// Requires：parent 存在 ∧ live；title/reason 非空。
    /// Ensures（F08/metis F5）：新帧为唯一 Active tip；parent 若为 Paused tip ⇒ 转 Active（回祖先链）；
    /// 旧焦点 tip 若 ≠ parent ⇒ Paused（= parent 时它成为祖先，保持 Active）。
    pub fn push(
        &mut self,
        at: &str,
        parent: Id,
        title: String,
        goal: Vec<SourcefulText>,
        origin: Vec<SourcefulText>,
        reason: SourcefulText,
    ) -> Result<Id, WsError> {
        Self::require_reason(&reason)?;
        match self.frames.get(&parent) {
            None => return Err(WsError::Usage(format!("父帧不存在：{}", self.print_id(parent)))),
            Some(p) if !p.status.is_live() => {
                return Err(WsError::Usage(format!(
                    "父帧 {} 已关闭（I10：live 帧的 parent 必为 live）",
                    self.print_id(parent)
                )))
            }
            _ => {}
        }
        if title.trim().is_empty() {
            return Err(WsError::Usage("title 不能为空".into()));
        }
        let id = self.fresh_id()?;
        // 旧焦点 ≠ parent：分支仍为 tip ⇒ 暂停；= parent：它将成祖先，保持 Active（F08）
        if let Some(old) = self.focus() {
            if old != parent {
                let msg = format!("去向：新帧 {}", self.print_id(id));
                if let Some(f) = self.frames.get_mut(&old) {
                    f.status = FrameStatus::Paused { reason: mech(msg.clone()) };
                    f.log.push(ev(Action::Pause, at, mech(msg)));
                }
            }
        }
        // parent 若为 Paused（tip）⇒ 转 Active 回祖先链（metis F5）
        let parent_paused = matches!(
            self.frames.get(&parent).map(|f| &f.status),
            Some(FrameStatus::Paused { .. })
        );
        if parent_paused {
            let msg = format!("push 子帧 {} 使本帧回到祖先链", self.print_id(id));
            if let Some(pf) = self.frames.get_mut(&parent) {
                pf.status = FrameStatus::Active;
                pf.log.push(ev(Action::Resume, at, mech(msg)));
            }
        }
        self.frames.insert(
            id,
            Frame {
                parent: Some(parent),
                title,
                goal,
                origin,
                status: FrameStatus::Active,
                plan: vec![],
                content: Content::default(),
                log: vec![ev(Action::Push, at, reason)],
            },
        );
        Ok(id)
    }

    /// pop：关闭帧（Done/Expired）+ 关闭后焦点归一（metis F3 / auditor F09/F10）。
    /// Requires：id 为 live tip；无 live 子（I10）；principles 全结算；closure.summary 非空；
    /// focus_to（若给）≠ id 且为 Paused tip。
    /// Ensures：关闭后全栈恰 ≤1 个 Active tip——选择序：focus_to > 未被关闭的原焦点 > 新暴露的父 tip > 无；
    /// 其余 Active tip 机械转 Paused（机械理由 + Pause 事件）。
    pub fn pop(
        &mut self,
        at: &str,
        id: Id,
        outcome: Outcome,
        focus_to: Option<Id>,
        reason: SourcefulText,
    ) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        let f = self
            .frames
            .get(&id)
            .ok_or_else(|| WsError::Usage(format!("帧不存在：{}", self.print_id(id))))?;
        if !f.status.is_live() {
            return Err(WsError::Usage("帧已关闭（误记更正请用 reopen）".into()));
        }
        if !self.live_children(id).is_empty() {
            return Err(WsError::Usage(format!("{} 存在 live 子帧：先 pop/改挂（I10）", self.print_id(id))));
        }
        let unsettled: Vec<usize> = f
            .content
            .principles
            .iter()
            .enumerate()
            .filter(|(_, p)| p.fate.is_none())
            .map(|(i, _)| i)
            .collect();
        if !unsettled.is_empty() {
            return Err(WsError::Usage(format!(
                "原则未结算（I9）：{}——先 update --settle",
                unsettled.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")
            )));
        }
        let closure = match &outcome {
            Outcome::DoneO(c) | Outcome::ExpiredO(c) => c,
        };
        if closure.summary.is_empty() {
            return Err(WsError::Usage("closure.summary 不能为空（一句话结论）".into()));
        }
        if let Some(t) = focus_to {
            if t == id {
                return Err(WsError::Usage("focus_to 不能指向被 pop 的帧自身（F09）".into()));
            }
            match self.frames.get(&t).map(|f| &f.status) {
                Some(FrameStatus::Paused { .. }) => {}
                _ => return Err(WsError::Usage("focus_to 须指向 Paused tip".into())),
            }
        }
        let old_focus = self.focus();
        let parent = f.parent;
        let status = match outcome {
            Outcome::DoneO(c) => FrameStatus::Done { closure: c },
            Outcome::ExpiredO(c) => FrameStatus::Expired { closure: c },
        };
        let f = self.frames.get_mut(&id).unwrap();
        f.status = status;
        f.log.push(ev(Action::Pop, at, reason));
        // 焦点选择序（metis F3）
        let chosen = focus_to.or_else(|| old_focus.filter(|&o| o != id)).or_else(|| {
            parent.and_then(|p| {
                let pf = self.frames.get(&p)?;
                let active = matches!(pf.status, FrameStatus::Active);
                let now_tip = self.live_children(p).is_empty() && pf.status.is_live();
                if active && now_tip {
                    Some(p)
                } else {
                    None
                }
            })
        });
        if let Some(t) = chosen {
            let is_paused = matches!(
                self.frames.get(&t).map(|f| &f.status),
                Some(FrameStatus::Paused { .. })
            );
            if is_paused {
                let msg = format!("承接自 {} 的 pop；焦点转移", self.print_id(id));
                if let Some(tf) = self.frames.get_mut(&t) {
                    tf.status = FrameStatus::Active;
                    tf.log.push(ev(Action::Resume, at, mech(msg)));
                }
            }
        }
        // 其余 Active tip 机械转 Paused（I3 保持）
        let extras: Vec<Id> = self
            .tips()
            .into_iter()
            .filter(|t| {
                chosen != Some(*t) && matches!(self.frames[t].status, FrameStatus::Active)
            })
            .collect();
        for t in extras {
            let msg = format!("（pop {} 后归一：非选中焦点）", self.print_id(id));
            if let Some(tf) = self.frames.get_mut(&t) {
                tf.status = FrameStatus::Paused { reason: mech(msg.clone()) };
                tf.log.push(ev(Action::Pause, at, mech(msg)));
            }
        }
        Ok(())
    }

    /// pause：当前焦点 tip → Paused。
    /// Requires：id 为当前唯一 Active tip（focus）；reason 非空。Ensures：Active tips = 0。
    pub fn pause(&mut self, at: &str, id: Id, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        if self.focus() != Some(id) {
            return Err(WsError::Usage(format!("{} 不是当前焦点 tip（pause 只作用于 Active tip）", self.print_id(id))));
        }
        let f = self.frames.get_mut(&id).unwrap();
        f.status = FrameStatus::Paused { reason: reason.clone() };
        f.log.push(ev(Action::Pause, at, reason));
        Ok(())
    }

    /// resume / switch：Paused tip 恢复为焦点。
    /// Requires：id 为 Paused；reason 非空。Ensures：id 为唯一 Active tip；旧焦点 tip → Paused。
    pub fn resume(&mut self, at: &str, id: Id, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        match self.frames.get(&id).map(|f| &f.status) {
            Some(FrameStatus::Paused { .. }) => {}
            Some(_) => return Err(WsError::Usage("目标不是 Paused 帧".into())),
            None => return Err(WsError::Usage("帧不存在".into())),
        }
        if let Some(old) = self.focus() {
            if old != id {
                let msg = format!("去向：resume {}", self.print_id(id));
                if let Some(f) = self.frames.get_mut(&old) {
                    f.status = FrameStatus::Paused { reason: mech(msg.clone()) };
                    f.log.push(ev(Action::Pause, at, mech(msg)));
                }
            }
        }
        let f = self.frames.get_mut(&id).unwrap();
        f.status = FrameStatus::Active;
        f.log.push(ev(Action::Resume, at, reason));
        Ok(())
    }

    // —— 字段级更新（净变化才记——F18）——

    fn ensure_frame(&self, id: Id) -> Result<(), WsError> {
        self.frames.get(&id).ok_or_else(|| WsError::Usage(format!("帧不存在：{}", self.print_id(id))))?;
        Ok(())
    }

    pub fn set_title(&mut self, at: &str, id: Id, title: String, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        if self.frames[&id].title != title {
            let f = self.frames.get_mut(&id).unwrap();
            f.title = title;
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    pub fn set_goal(&mut self, at: &str, id: Id, goal: Vec<SourcefulText>, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        if self.frames[&id].goal != goal {
            let f = self.frames.get_mut(&id).unwrap();
            f.goal = goal;
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    pub fn set_origin(&mut self, at: &str, id: Id, origin: Vec<SourcefulText>, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        if self.frames[&id].origin != origin {
            let f = self.frames.get_mut(&id).unwrap();
            f.origin = origin;
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    /// plan_add：追加计划项（顶层）。
    pub fn plan_add(&mut self, at: &str, id: Id, text: SourcefulText, state: ItemState, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        f.plan.push(PlanItem { text, children: vec![], progress: state, fork: vec![] });
        f.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    /// plan_add_child：给 path 处的项追加内嵌子项（右键"添加内嵌列表"通道）。
    pub fn plan_add_child(&mut self, at: &str, id: Id, path: &[usize], text: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        let mut cur: &mut Vec<PlanItem> = &mut f.plan;
        for idx in path {
            cur = cur.get_mut(*idx).ok_or_else(|| WsError::Usage("plan 路径越界".into()))?.children.as_mut();
        }
        cur.push(PlanItem { text, children: vec![], progress: ItemState::Ready, fork: vec![] });
        let f = self.frames.get_mut(&id).unwrap();
        f.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    /// plan_text：按路径改计划项正文（内联编辑通道）。越界 Err；净变化才记。
    pub fn plan_text(&mut self, at: &str, id: Id, path: &[usize], text: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        if path.is_empty() {
            return Err(WsError::Usage("plan 路径不能为空".into()));
        }
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        let mut cur: &mut Vec<PlanItem> = &mut f.plan;
        for idx in &path[..path.len() - 1] {
            cur = cur.get_mut(*idx).ok_or_else(|| WsError::Usage("plan 路径越界".into()))?.children.as_mut();
        }
        let item = cur.get_mut(path[path.len() - 1]).ok_or_else(|| WsError::Usage("plan 路径越界".into()))?;
        if item.text != text {
            item.text = text;
            let f = self.frames.get_mut(&id).unwrap();
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    /// plan_set：按路径（下标序列，末位为目标项）改状态。越界 Err；净变化才记。
    pub fn plan_set(&mut self, at: &str, id: Id, path: &[usize], state: ItemState, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        if path.is_empty() {
            return Err(WsError::Usage("plan 路径不能为空".into()));
        }
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        let mut cur: &mut Vec<PlanItem> = &mut f.plan;
        for idx in &path[..path.len() - 1] {
            cur = cur
                .get_mut(*idx)
                .ok_or_else(|| WsError::Usage("plan 路径越界".into()))?
                .children
                .as_mut();
        }
        let item = cur
            .get_mut(path[path.len() - 1])
            .ok_or_else(|| WsError::Usage("plan 路径越界".into()))?;
        if item.progress != state {
            item.progress = state;
            let f = self.frames.get_mut(&id).unwrap();
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    pub fn result_add(&mut self, at: &str, id: Id, text: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        f.content.results.push(text);
        f.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    pub fn side_effect_add(&mut self, at: &str, id: Id, text: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        f.content.side_effects.push(text);
        f.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    pub fn principle_add(&mut self, at: &str, id: Id, rule: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let f = self.frames.get_mut(&id).unwrap();
        f.content.principles.push(Principle { rule, fate: None });
        f.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    /// settle：结算原则 fate；移交/转存同 Tx 物化（metis F6/F7/D3-A）。
    /// - ReleasedTo{to}：Requires to 为本帧**严格祖先**；接收帧追加引用型原则（fate=None，承接义务）。
    /// - Reclassified：ToResults/ToRemark 同 Tx 追加引用源原则的条目（转存可机械检查）。
    pub fn settle(&mut self, at: &str, id: Id, idx: usize, fate: Fate, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        if idx >= self.frames[&id].content.principles.len() {
            return Err(WsError::Usage("原则下标越界".into()));
        }
        match &fate {
            Fate::ReleasedTo { id: to } => {
                let path = self.path(id);
                if *to == id || !path.contains(to) {
                    return Err(WsError::Usage(format!(
                        "released 目标 {} 必须是本帧的严格祖先",
                        self.print_id(*to)
                    )));
                }
            }
            Fate::Reclassified { .. } | Fate::KeptLocal | Fate::Invalidated { .. } => {}
        }
        // 物化承接（D3-A）
        let src_ref = SourcefulText(vec![Segment::Ref(Reference {
            path: vec![RefSeg::Frame(id), RefSeg::Field("principles".into()), RefSeg::Index(idx)],
        })]);
        let mut extra: Option<(Id, SourcefulText)> = None;
        match &fate {
            Fate::ReleasedTo { id: to } => extra = Some((*to, src_ref.clone())),
            Fate::Reclassified { target } => match target {
                ReclassTarget::Results => {
                    let f = self.frames.get_mut(&id).unwrap();
                    f.content.results.push(src_ref.clone());
                }
                ReclassTarget::Remark { title } => {
                    let f = self.frames.get_mut(&id).unwrap();
                    let slot = f.content.remarks.entry(title.clone()).or_default();
                    slot.extend(src_ref.0.iter().cloned());
                }
            },
            _ => {}
        }
        let f = self.frames.get_mut(&id).unwrap();
        f.content.principles[idx].fate = Some(fate);
        f.log.push(ev(Action::Update, at, reason.clone()));
        if let Some((to, rule)) = extra {
            let msg = format!("承接自 {} 原则{}（released）", self.print_id(id), idx + 1);
            if let Some(tf) = self.frames.get_mut(&to) {
                tf.content.principles.push(Principle { rule, fate: None });
                tf.log.push(ev(Action::Update, at, mech(msg)));
            }
        }
        Ok(())
    }

    pub fn remark_set(&mut self, at: &str, id: Id, title: Title, body: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.ensure_frame(id)?;
        let changed = match self.frames[&id].content.remarks.get(&title) {
            Some(old) => *old != body,
            None => true,
        };
        if changed {
            let f = self.frames.get_mut(&id).unwrap();
            f.content.remarks.insert(title, body);
            f.log.push(ev(Action::Update, at, reason));
        }
        Ok(())
    }

    /// note_put：单份正文更新；栈级 Event 记录（oracle S6；I6 不受影响——不触帧字段）。
    pub fn note_put(&mut self, at: &str, key: &str, meta: Meta, body: SourcefulText, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        self.notes.insert(key.to_string(), Note { meta, body });
        self.log.push(ev(Action::Update, at, reason));
        Ok(())
    }

    /// reopen：误记关闭的更正（I5 澄清：不作新活动回合复活；误记 Correct 例外）。
    /// 旧 Closure 机械保存进 Correct 事件（结构化数据不丢——metis F1 裁定）。
    /// Requires：id 已关闭；reason 非空。
    pub fn reopen(&mut self, at: &str, id: Id, paused_reason: Option<SourcefulText>, reason: SourcefulText) -> Result<(), WsError> {
        Self::require_reason(&reason)?;
        let f = self.frames.get_mut(&id).ok_or_else(|| WsError::Usage("帧不存在".into()))?;
        if !f.status.is_closed() {
            return Err(WsError::Usage("帧未关闭（reopen 只用于更正误记 Done/Expired）".into()));
        }
        let (kind, summary) = match &f.status {
            FrameStatus::Done { closure } | FrameStatus::Expired { closure } => {
                (f.status.kind_str(), closure.summary.clone())
            }
            _ => unreachable!(),
        };
        f.status = match paused_reason {
            Some(r) => FrameStatus::Paused { reason: r },
            None => FrameStatus::Active,
        };
        let keep = mech(format!("\n原 Closure：[{}] ", kind));
        let mut detail = reason.0;
        detail.extend(keep.0);
        detail.extend(summary.0);
        let detail = crate::text::norm(&SourcefulText(detail));
        let f = self.frames.get_mut(&id).unwrap();
        f.log.push(ev(Action::Correct, at, detail));
        Ok(())
    }

    /// apply 转移检查（C1 / metis F2）：整文件替换必须保持身份与历史合同。
    /// 检查：旧帧 id ⊆ 新帧 id（不得删除）；旧关闭帧不得变 live（I5）；每帧旧 log 必须是新 log 前缀；
    /// 新/变更的 ReleasedTo 须为严格祖先；Reclassified 目标须含对源原则的引用（物化转存）。
    pub fn apply_checks(old: &WorkingStack, new: &WorkingStack) -> Vec<Problem> {
        let mut ps = Vec::new();
        for (id, of) in &old.frames {
            match new.frames.get(id) {
                None => ps.push(format!("I5: 帧 {} 被删除（退出必须保留关闭记录）", old.print_id(*id))),
                Some(nf) => {
                    if of.status.is_closed() && nf.status.is_live() {
                        ps.push(format!("I5: 关闭帧 {} 复活（误记更正请走 reopen/Correct）", old.print_id(*id)));
                    }
                    if nf.log.len() < of.log.len() || nf.log[..of.log.len()] != of.log[..] {
                        ps.push(format!("历史改写：{} 的既有 log 必须原样保留（只能追加）", old.print_id(*id)));
                    }
                }
            }
        }
        for (id, nf) in &new.frames {
            let old_fate = old
                .frames
                .get(id)
                .map(|of| of.content.principles.iter().map(|p| p.fate.clone()).collect::<Vec<_>>());
            for (i, p) in nf.content.principles.iter().enumerate() {
                let changed = match &old_fate {
                    Some(v) => v.get(i).cloned().flatten() != p.fate,
                    None => p.fate.is_some(),
                };
                if !changed {
                    continue;
                }
                match &p.fate {
                    Some(Fate::ReleasedTo { id: to }) => {
                        let path = new.path(*id);
                        if *to == *id || !path.contains(to) {
                            ps.push(format!("I7: {} 原则{} released → {} 不是严格祖先", new.print_id(*id), i + 1, new.print_id(*to)));
                        }
                    }
                    Some(Fate::Reclassified { target }) => {
                        let want = Reference {
                            path: vec![RefSeg::Frame(*id), RefSeg::Field("principles".into()), RefSeg::Index(i)],
                        };
                        let linked = match target {
                            ReclassTarget::Results => nf.content.results.iter().any(|st| has_ref(st, &want)),
                            ReclassTarget::Remark { title } => nf
                                .content
                                .remarks
                                .get(title)
                                .map(|st| has_ref(st, &want))
                                .unwrap_or(false),
                        };
                        if !linked {
                            ps.push(format!("I7: {} 原则{} reclassified 但目标未含对源原则的引用（转存未物化）", new.print_id(*id), i + 1));
                        }
                    }
                    _ => {}
                }
            }
        }
        ps
    }
}

fn has_ref(st: &SourcefulText, want: &Reference) -> bool {
    st.iter().any(|seg| matches!(seg, Segment::Ref(r) if *r == *want))
}

// parse_ref 引用（settle 的 src_ref 用字面构造，保留 parse_ref 导入以备 CLI）
#[allow(dead_code)]
fn _unused(r: &str) -> Result<Reference, String> {
    parse_ref(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::validate;

    fn reason(s: &str) -> SourcefulText {
        SourcefulText(vec![Segment::Text(s.into())])
    }

    fn ws3() -> (WorkingStack, Id, Id, Id) {
        let mut ws = WorkingStack::default();
        ws.prefix = Some("T".into());
        let r = ws.push_root("t0", "根".into(), vec![reason("完成 X")], reason("建栈")).unwrap();
        let a = ws.push("t1", r, "分支A".into(), vec![], vec![], reason("进入A")).unwrap();
        let b = ws.push("t2", r, "分支B".into(), vec![], vec![], reason("进入B")).unwrap();
        (ws, r, a, b)
    }

    #[test]
    fn push_switches_focus_and_pauses_old() {
        let (ws, _r, a, b) = ws3();
        assert_eq!(ws.focus(), Some(b));
        assert!(matches!(ws.frames[&a].status, FrameStatus::Paused { .. }));
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn push_under_focus_keeps_parent_active() {
        // F08 反例：向焦点帧下钻，父不 Paused
        let (mut ws, _r, _a, b) = ws3();
        let c = ws.push("t3", b, "下钻".into(), vec![], vec![], reason("下钻")).unwrap();
        assert!(matches!(ws.frames[&b].status, FrameStatus::Active)); // 祖先链 Active
        assert_eq!(ws.focus(), Some(c));
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn push_under_paused_parent_resumes_it() {
        // metis F5 反例：向 Paused tip 下钻 ⇒ parent 回 Active
        let (mut ws, _r, a, _b) = ws3(); // a 是 Paused tip
        let c = ws.push("t3", a, "恢复下钻".into(), vec![], vec![], reason("回A下钻")).unwrap();
        assert!(matches!(ws.frames[&a].status, FrameStatus::Active));
        assert_eq!(ws.focus(), Some(c));
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn pop_requires_settled_and_tip() {
        let (mut ws, _r, _a, b) = ws3();
        ws.principle_add("t3", b, reason("P1 测试"), reason("加原则")).unwrap();
        let cl = Closure { summary: reason("完成"), ..Default::default() };
        assert!(ws.pop("t4", b, Outcome::DoneO(cl.clone()), None, reason("收尾")).is_err());
        ws.settle("t5", b, 0, Fate::KeptLocal, reason("结算")).unwrap();
        ws.pop("t6", b, Outcome::DoneO(cl), None, reason("收尾")).unwrap();
        // 父有另一 Paused 子 a ⇒ 无自动焦点，a 保持 Paused、根非 tip
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn pop_paused_tip_keeps_other_focus() {
        // metis F3 场景 3：pop 非焦点 Paused tip，别处焦点保留
        let (mut ws, _r, _a, b) = ws3();
        let cl = Closure { summary: reason("关闭A线"), ..Default::default() };
        ws.pop("t7", _a, Outcome::ExpiredO(cl), None, reason("关A")).unwrap();
        assert_eq!(ws.focus(), Some(b));
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn pop_normalizes_exposed_parent() {
        // metis F3 场景 1：pop 暴露的父 tip 不产生第二焦点
        let (mut ws, r, _a, _b) = ws3();
        // 结构：root r → a(paused)…重排：直接构造 r→x(paused tip)、独立根 z(active)
        let mut ws2 = WorkingStack::default();
        ws2.prefix = Some("Z".into());
        let r2 = ws2.push_root("t0", "主根".into(), vec![], reason("建")).unwrap();
        let x = ws2.push("t1", r2, "支线".into(), vec![], vec![], reason("支")).unwrap();
        let _ = ws;
        let _ = r;
        // 先暂停 x 使其成为 Paused tip（此时无 Active tip），再 resume? 简化：直接用 ws2 结构测 pop(x)
        let cl = Closure { summary: reason("收线"), ..Default::default() };
        // x 是当前 Active tip；pop 后 r2 暴露为 tip（无其他 live 子）→ 自动成为焦点
        ws2.pop("t2", x, Outcome::ExpiredO(cl), None, reason("收线")).unwrap();
        assert_eq!(ws2.focus(), Some(r2));
        assert!(validate(&ws2).is_empty());
    }

    #[test]
    fn pop_focus_to_self_rejected() {
        // F09 反例
        let (mut ws, _r, _a, b) = ws3();
        let cl = Closure { summary: reason("完成"), ..Default::default() };
        assert!(ws.pop("t8", b, Outcome::DoneO(cl), Some(b), reason("x")).is_err());
    }

    #[test]
    fn pop_focus_to_normalizes_exposed_parent() {
        // metis F3 场景 2：pop 焦点 + focus_to 兄弟 ⇒ 暴露的父归 Paused
        let mut ws = WorkingStack::default();
        ws.prefix = Some("Q".into());
        let r = ws.push_root("t0", "根".into(), vec![], reason("建")).unwrap();
        let a = ws.push("t1", r, "A".into(), vec![], vec![], reason("A")).unwrap();
        let _ = a; // a 现为 Paused? push B 会让 a paused：
        let b = ws.push("t2", r, "B".into(), vec![], vec![], reason("B")).unwrap();
        let cl = Closure { summary: reason("完成B"), ..Default::default() };
        ws.pop("t3", b, Outcome::DoneO(cl), Some(a), reason("收B回A")).unwrap();
        assert_eq!(ws.focus(), Some(a));
        // r 无 live 子（a 是 live? a paused tip）——r 不是 tip；唯一 Active tip = a
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn settle_released_requires_ancestor_and_materializes() {
        let (mut ws, _r, a, _b) = ws3(); // a paused tip；焦点 b
        let z = _r;
        ws.principle_add("t3", a, reason("P 规则"), reason("加")).unwrap();
        // released 到兄弟（非祖先）拒绝
        let _ = z;
        let (ws_ref, _, _, _) = (ws3());
        let sibling = ws_ref.frames.keys().max().copied().unwrap();
        let _ = sibling;
        // 构造：a 的严格祖先 = 根
        let root = ws.frames.keys().copied().min().unwrap();
        assert!(ws
            .settle("t4", a, 0, Fate::ReleasedTo { id: sibling_of(&ws, a) }, reason("x"))
            .is_err());
        ws.settle("t5", a, 0, Fate::ReleasedTo { id: root }, reason("上移")).unwrap();
        // 接收帧获得引用型原则
        assert!(ws.frames[&root].content.principles.iter().any(|p| {
            matches!(p.rule.0.first(), Some(Segment::Ref(_))) && p.fate.is_none()
        }));
    }

    fn sibling_of(ws: &WorkingStack, id: Id) -> Id {
        ws.frames
            .keys()
            .find(|k| **k != id && ws.frames[k].parent == ws.frames[&id].parent)
            .copied()
            .unwrap()
    }

    #[test]
    fn reopen_preserves_closure_in_correct() {
        let (mut ws, _r, _a, b) = ws3();
        let cl = Closure {
            summary: reason("完成且含依据"),
            positive: vec![ClosureItem { result: reason("R"), evidence: reason("E") }],
            ..Default::default()
        };
        ws.pop("ta", b, Outcome::DoneO(cl), None, reason("误收")).unwrap();
        let log_len = ws.frames[&b].log.len();
        ws.reopen("tb", b, None, reason("用户未验收，撤回 pop")).unwrap();
        assert!(ws.frames[&b].status.is_live());
        let last = ws.frames[&b].log.last().unwrap();
        assert!(matches!(last.action, Action::Correct));
        assert!(crate::text::flatten(&last.detail).contains("原 Closure"));
        assert!(crate::text::flatten(&last.detail).contains("完成且含依据"));
        assert_eq!(ws.frames[&b].log.len(), log_len + 1);
        assert!(validate(&ws).is_empty());
    }

    #[test]
    fn update_noop_no_event() {
        let (mut ws, _r, _a, b) = ws3();
        let n = ws.frames[&b].log.len();
        ws.set_title("t9", b, "分支B".into(), reason("同名重设")).unwrap();
        assert_eq!(ws.frames[&b].log.len(), n);
    }

    #[test]
    fn apply_checks_guard_history_and_revive() {
        let (mut ws, _r, _a, b) = ws3();
        let cl = Closure { summary: reason("完成"), ..Default::default() };
        ws.pop("t9", b, Outcome::DoneO(cl), None, reason("先合法关闭")).unwrap();
        // 复活
        let mut revived = ws.clone();
        revived.frames.get_mut(&b).unwrap().status = FrameStatus::Active;
        let ps = WorkingStack::apply_checks(&ws, &revived);
        assert!(ps.iter().any(|p| p.contains("复活")));
        // 删除
        let mut deleted = ws.clone();
        deleted.frames.remove(&b);
        let ps = WorkingStack::apply_checks(&ws, &deleted);
        assert!(ps.iter().any(|p| p.contains("删除")));
        // log 改写
        let mut rewritten = ws.clone();
        rewritten.frames.get_mut(&b).unwrap().log[0].detail = reason("改写历史");
        let ps = WorkingStack::apply_checks(&ws, &rewritten);
        assert!(ps.iter().any(|p| p.contains("历史改写")));
    }
}
