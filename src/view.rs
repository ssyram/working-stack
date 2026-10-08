//! 派生视图（heads 等聚合不入存储——蓝本与 Q.E.2；全为纯函数）。
//! `walk_all_st` 是全文档 ST 遍历器（F03/F17：状态负载、item reason、closure、fate 全覆盖），供引用扫描与待归属共用。

use crate::model::*;
use crate::text::{Segment, SourcefulText};
use std::collections::{BTreeMap, HashSet};

impl WorkingStack {
    /// live 帧集合（Active ∪ Paused），按 id 升序。trivial。
    pub fn live_ids(&self) -> Vec<Id> {
        self.frames
            .iter()
            .filter(|(_, f)| f.status.is_live())
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn is_live(&self, id: Id) -> bool {
        self.frames.get(&id).map(|f| f.status.is_live()).unwrap_or(false)
    }

    /// 同父全部子帧（含已 pop；打印按需过滤）。BTreeMap 迭代序 ⇒ 确定性。
    pub fn children(&self, id: Id) -> Vec<Id> {
        self.frames
            .iter()
            .filter(|(_, f)| f.parent == Some(id))
            .map(|(i, _)| *i)
            .collect()
    }

    pub fn live_children(&self, id: Id) -> Vec<Id> {
        self.children(id).into_iter().filter(|c| self.is_live(*c)).collect()
    }

    /// 根 → … → 该帧。环保护：步数上限 = 帧数+1（validate 负责报环）。
    pub fn path(&self, id: Id) -> Vec<Id> {
        let mut chain = vec![id];
        let mut cur = id;
        let mut guard = self.frames.len() + 1;
        while let Some(p) = self.frames.get(&cur).and_then(|f| f.parent) {
            if guard == 0 {
                break;
            }
            chain.push(p);
            cur = p;
            guard -= 1;
        }
        chain.reverse();
        chain
    }

    pub fn roots(&self) -> Vec<Id> {
        self.frames
            .iter()
            .filter(|(_, f)| f.parent.is_none())
            .map(|(i, _)| *i)
            .collect()
    }

    /// live 根（树/节序只从 live 根出发——F16）。
    pub fn live_roots(&self) -> Vec<Id> {
        self.roots().into_iter().filter(|r| self.is_live(*r)).collect()
    }

    /// tips = live 中无 live 子者（每 live 帧入集一次 ⇒ O(n)）。
    pub fn tips(&self) -> Vec<Id> {
        let live: Vec<Id> = self.live_ids();
        let mut has_live_child: HashSet<Id> = HashSet::new();
        for id in &live {
            if let Some(p) = self.frames[id].parent {
                has_live_child.insert(p);
            }
        }
        live.into_iter().filter(|id| !has_live_child.contains(id)).collect()
    }

    /// 唯一 Active tip = 焦点；0 或多个（非法态）⇒ None——不伪造权威焦点（metis F9），validate 负责 I3。
    pub fn focus(&self) -> Option<Id> {
        let mut act: Vec<Id> = self
            .tips()
            .into_iter()
            .filter(|&t| matches!(self.frames[&t].status, FrameStatus::Active))
            .collect();
        if act.len() == 1 {
            act.sort_unstable();
            Some(act.remove(0))
        } else {
            None
        }
    }

    /// 帧内 FOCUS：前序第一个 InProgress 或 Blocked（等待也是当前工作项——实践 OP-23）。
    /// 关闭帧不产生当前 FOCUS（D2=B：计划为关闭时快照）。
    pub fn focus_item_of(frame: &Frame) -> Option<&PlanItem> {
        if frame.status.is_closed() {
            return None;
        }
        fn walk(items: &[PlanItem]) -> Option<&PlanItem> {
            for it in items {
                match it.progress {
                    ItemState::InProgress | ItemState::Blocked { .. } => return Some(it),
                    _ => {
                        if let Some(x) = walk(&it.children) {
                            return Some(x);
                        }
                    }
                }
            }
            None
        }
        walk(&frame.plan)
    }

    /// 焦点帧渲染标记：FOCUS 项为 Blocked ⇒ "TOP/PAUSED"（等待语义），否则 "TOP"。
    pub fn render_top(frame: &Frame) -> &'static str {
        match Self::focus_item_of(frame).map(|it| &it.progress) {
            Some(ItemState::Blocked { .. }) => "TOP/PAUSED",
            _ => "TOP",
        }
    }

    /// 打印 id：prefix 存在 ⇒ `T-01`（宽度 2，超长自然扩展）。
    pub fn print_id(&self, id: Id) -> String {
        match &self.prefix {
            Some(p) => format!("{}-{:02}", p, id),
            None => id.to_string(),
        }
    }

    /// 解析用户输入的 id（接受 `T-12` 或 `12`）。trivial。
    pub fn parse_id(&self, s: &str) -> Option<Id> {
        s.trim().rsplit('-').next()?.parse().ok()
    }

    /// live 树序：live 根升序 + 先序遍历（live 子）——打印节顺序用（F16：关闭根不入）。
    pub fn live_tree_order(&self) -> Vec<Id> {
        let mut order = Vec::new();
        for r in self.live_roots() {
            self.pre_order_live(r, &mut order);
        }
        order
    }

    fn pre_order_live(&self, id: Id, out: &mut Vec<Id>) {
        out.push(id);
        for c in self.live_children(id) {
            self.pre_order_live(c, out);
        }
    }
}

/// 全文档 SourcefulText 遍历（F03/F17 修复：含状态负载与全部 note 正文）。
/// 位置标签用于问题定位。
pub fn walk_all_st(ws: &WorkingStack, mut f: impl FnMut(&str, &SourcefulText)) {
    for (id, fr) in &ws.frames {
        let pid = ws.print_id(*id);
        for st in fr.goal.iter().chain(fr.origin.iter()) {
            f(&format!("{pid} goal/origin"), st);
        }
        plan_walk(&fr.plan, &pid, &mut f);
        match &fr.status {
            FrameStatus::Paused { reason } => f(&format!("{pid} paused.reason"), reason),
            FrameStatus::Done { closure } | FrameStatus::Expired { closure } => {
                f(&format!("{pid} closure.summary"), &closure.summary);
                for (label, items) in [
                    ("positive", &closure.positive),
                    ("negative", &closure.negative),
                    ("tangential", &closure.tangential),
                    ("residuals", &closure.residuals),
                ] {
                    for it in items.iter() {
                        f(&format!("{pid} closure.{label}"), &it.result);
                        f(&format!("{pid} closure.{label}"), &it.evidence);
                    }
                }
            }
            FrameStatus::Active => {}
        }
        for st in fr.content.results.iter().chain(fr.content.side_effects.iter()) {
            f(&format!("{pid} content"), st);
        }
        for p in &fr.content.principles {
            f(&format!("{pid} principles"), &p.rule);
            if let Some(Fate::Invalidated { reason }) = &p.fate {
                f(&format!("{pid} fate"), reason);
            }
        }
        for (k, st) in &fr.content.remarks {
            f(&format!("{pid} remark:{k}"), st);
        }
        for e in &fr.log {
            f(&format!("{pid} log"), &e.detail);
        }
    }
    for (k, n) in &ws.notes {
        f(&format!("note:{k}"), &n.body);
    }
}

fn plan_walk(items: &[PlanItem], loc: &str, f: &mut impl FnMut(&str, &SourcefulText)) {
    for it in items {
        f(loc, &it.text);
        match &it.progress {
            ItemState::Blocked { reason } | ItemState::ItemExpired { reason } => f(loc, reason),
            _ => {}
        }
        plan_walk(&it.children, loc, f);
    }
}

/// 保留 key：description 承载文件头散文（打印层约定，auditor DESIGN_DOC_INCOMPLETE 裁定入文档）。
pub const RESERVED_NOTE_KEYS: [&str; 1] = ["description"];

/// 未被任何强引用使用的 notes（= 待归属缓存；排除保留 key）。
pub fn unattached_notes(ws: &WorkingStack) -> Vec<String> {
    let mut referenced: HashSet<String> = HashSet::new();
    walk_all_st(ws, |_, st| {
        for seg in st.iter() {
            if let Segment::Ref(r) = seg {
                if let Some(crate::text::RefSeg::NoteK(k)) = r.path.first() {
                    referenced.insert(k.clone());
                }
            }
        }
    });
    ws.notes
        .keys()
        .filter(|k| !RESERVED_NOTE_KEYS.contains(&k.as_str()) && !referenced.contains(k.as_str()))
        .cloned()
        .collect()
}

/// 收集全部 note 引用 key（BTreeMap 去重；保留下方旧签名兼容内部使用）。
#[allow(dead_code)]
fn collect_note_refs_btree(ws: &WorkingStack) -> BTreeMap<String, ()> {
    let mut set: BTreeMap<String, ()> = BTreeMap::new();
    walk_all_st(ws, |_, st| {
        for seg in st.iter() {
            if let Segment::Ref(r) = seg {
                if let Some(crate::text::RefSeg::NoteK(k)) = r.path.first() {
                    set.insert(k.clone(), ());
                }
            }
        }
    });
    set
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::parse_st;

    fn frame(parent: Option<Id>, status: FrameStatus) -> Frame {
        Frame {
            parent,
            title: "t".into(),
            goal: vec![],
            origin: vec![],
            status,
            plan: vec![],
            content: Content::default(),
            log: vec![],
        }
    }

    #[test]
    fn tips_and_focus() {
        let mut ws = WorkingStack::default();
        ws.prefix = Some("T".into());
        ws.frames.insert(1, frame(None, FrameStatus::Active));
        ws.frames.insert(2, frame(Some(1), FrameStatus::Active));
        ws.frames.insert(
            3,
            frame(Some(1), FrameStatus::Paused { reason: parse_st("等回信").unwrap() }),
        );
        assert_eq!(ws.tips(), vec![2, 3]);
        assert_eq!(ws.focus(), Some(2));
        assert_eq!(ws.path(2), vec![1, 2]);
    }

    #[test]
    fn focus_none_when_multiple_or_zero() {
        let mut ws = WorkingStack::default();
        ws.frames.insert(1, frame(None, FrameStatus::Active));
        ws.frames.insert(2, frame(Some(1), FrameStatus::Active));
        ws.frames.insert(3, frame(Some(1), FrameStatus::Active));
        assert_eq!(ws.focus(), None); // 双 Active tip：不伪造权威（F9）
        ws.frames.get_mut(&2).unwrap().status = FrameStatus::Paused { reason: parse_st("r").unwrap() };
        ws.frames.get_mut(&3).unwrap().status = FrameStatus::Paused { reason: parse_st("r").unwrap() };
        assert_eq!(ws.focus(), None); // 全 Paused = 整线暂停
    }

    #[test]
    fn walk_covers_status_payloads() {
        // F03 反例：Paused.reason 里的引用必须被扫描到
        let mut ws = WorkingStack::default();
        let mut f = frame(None, FrameStatus::Active);
        f.status = FrameStatus::Paused { reason: parse_st("等 {{99.goal}}").unwrap() };
        ws.frames.insert(1, f);
        let mut refs = 0;
        walk_all_st(&ws, |loc, _| {
            refs += 1;
            assert!(loc.contains("paused.reason"));
        });
        assert_eq!(refs, 1);
    }

    #[test]
    fn unattached_excludes_referenced_via_note_body() {
        // F17 反例：note 正文中的引用也算归属
        let mut ws = WorkingStack::default();
        ws.notes.insert("k".into(), Note { meta: Default::default(), body: parse_st("清单").unwrap() });
        ws.notes.insert("a".into(), Note { meta: Default::default(), body: parse_st("见 {{note:k.body}}").unwrap() });
        assert_eq!(unattached_notes(&ws), vec!["a".to_string()]); // k 被 a 引用⇒已归属；a 自身无入边
    }
}
