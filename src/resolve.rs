//! 引用解析与渲染（detailed-design §4.3；F04 修复：resolve 只取结构、不做内嵌渲染，
//! 展开循环防护用"当前展开路径引用不重复"（S10/F04 裁定），无固定深度截断）。

use crate::model::*;
use crate::text::{render_ref, Reference, Segment, SourcefulText};
use std::collections::BTreeSet;

/// 解析中的中间值（全部为结构引用或已组装 ST；不产生内嵌渲染字符串）。
enum Cur<'a> {
    Frame(&'a Frame),
    Note(&'a Note),
    Strings(&'a [String]),
    Plans(&'a [PlanItem]),
    Principles(&'a [Principle]),
    Events(&'a [Event]),
    Event(&'a Event),
    Sts(&'a [SourcefulText]),
    Txt(SourcefulText),
}

impl WorkingStack {
    /// resolve：路径逐步核存在性与边界；越界/未知 ⇒ None。
    /// 字段表（Frame）：title/goal/origin/status/plan/log/results/side_effects/principles/remarks；
    /// plan[i] → 项 text；log[i] → 事件（可再 .detail/.at/.action）；results[i]；principles[i] → rule；
    /// Note：body/meta[i]；Meta 是普通字符串列表。
    pub fn resolve(&self, r: &Reference) -> Option<SourcefulText> {
        let mut cur = match r.path.first()? {
            crate::text::RefSeg::Frame(id) => Cur::Frame(self.frames.get(id)?),
            crate::text::RefSeg::NoteK(k) => Cur::Note(self.notes.get(k)?),
            _ => return None,
        };
        for seg in &r.path[1..] {
            cur = self.step(cur, seg)?;
        }
        self.finish(cur)
    }

    fn finish(&self, cur: Cur<'_>) -> Option<SourcefulText> {
        match cur {
            Cur::Frame(f) => Some(SourcefulText(vec![Segment::Text(format!(
                "[{} {}]",
                self.frame_label(f),
                f.title
            ))])),
            Cur::Note(n) => Some(n.body.clone()),
            Cur::Txt(st) => Some(st),
            Cur::Principles(items) => Some(join_paragraphs(&items.iter().map(|p| p.rule.clone()).collect::<Vec<_>>())),
            Cur::Event(ev) => {
                let mut segs = vec![Segment::Text(format!("[{}] [{}] ", ev.action.label(), ev.at))];
                segs.extend(ev.detail.iter().cloned());
                Some(SourcefulText(segs))
            }
            _ => None, // 列表未终结（Plans/Events/Sts 直接结尾）→ 未解析
        }
    }

    /// 帧引用需要打印形 id：反查（单文件规模小，线性可接受）。
    fn frame_label(&self, f: &Frame) -> String {
        for (id, fr) in &self.frames {
            if std::ptr::eq(fr, f) {
                return self.print_id(*id);
            }
        }
        "?".into()
    }

    fn step<'a>(&'a self, cur: Cur<'a>, seg: &crate::text::RefSeg) -> Option<Cur<'a>> {
        use crate::text::RefSeg::*;
        match (cur, seg) {
            (Cur::Frame(f), Field(name)) => match name.as_str() {
                "title" => Some(Cur::Txt(SourcefulText(vec![Segment::Text(f.title.clone())]))),
                "goal" => Some(Cur::Txt(join_paragraphs(&f.goal))),
                "origin" => Some(Cur::Txt(join_paragraphs(&f.origin))),
                "status" => Some(Cur::Txt(self.status_st(f))),
                "plan" => Some(Cur::Plans(&f.plan)),
                "log" => Some(Cur::Events(&f.log)),
                "results" => Some(Cur::Sts(&f.content.results)),
                "side_effects" => Some(Cur::Sts(&f.content.side_effects)),
                "principles" => Some(Cur::Principles(&f.content.principles)),
                "remarks" => Some(Cur::Txt(join_paragraphs(
                    &f.content.remarks.values().cloned().collect::<Vec<_>>(),
                ))),
                _ => None,
            },
            (Cur::Plans(p), Index(i)) => p.get(*i).map(|it| Cur::Txt(it.text.clone())),
            (Cur::Principles(p), Index(i)) => p.get(*i).map(|it| Cur::Txt(it.rule.clone())),
            (Cur::Events(e), Index(i)) => e.get(*i).map(|ev| Cur::Event(ev)),
            (Cur::Event(ev), Field(name)) => match name.as_str() {
                "detail" => Some(Cur::Txt(ev.detail.clone())),
                "at" => Some(Cur::Txt(SourcefulText(vec![Segment::Text(ev.at.clone())]))),
                "action" => Some(Cur::Txt(SourcefulText(vec![Segment::Text(ev.action.label())]))),
                _ => None,
            },
            (Cur::Strings(m), Index(i)) => m.get(*i).map(|s| Cur::Txt(SourcefulText::text(s))),
            (Cur::Sts(s), Index(i)) => s.get(*i).map(|st| Cur::Txt(st.clone())),
            (Cur::Note(n), Field(name)) => match name.as_str() {
                "body" => Some(Cur::Txt(n.body.clone())),
                "meta" => Some(Cur::Strings(&n.meta)),
                _ => None,
            },
            _ => None,
        }
    }

    /// status 结构组装（不内嵌渲染 reason——外层 render 统一展开，F04）。
    fn status_st(&self, f: &Frame) -> SourcefulText {
        let mut segs: Vec<Segment> = Vec::new();
        match &f.status {
            FrameStatus::Active => segs.push(Segment::Text("active".into())),
            FrameStatus::Paused { reason } => {
                segs.push(Segment::Text("paused：".into()));
                segs.extend(reason.iter().cloned());
            }
            FrameStatus::Done { closure } => {
                segs.push(Segment::Text("done：".into()));
                segs.extend(closure.summary.iter().cloned());
            }
            FrameStatus::Expired { closure } => {
                segs.push(Segment::Text("expired：".into()));
                segs.extend(closure.summary.iter().cloned());
            }
        }
        SourcefulText(segs)
    }

    /// 展开渲染：引用替换为当前内容；循环防护 = 当前展开路径上引用不重复
    ///（无环栈全展开；有环显示 ⟨循环引用⟩——S10/F04 裁定，无深度截断）。
    pub fn render(&self, st: &SourcefulText) -> String {
        let mut seen: Vec<String> = Vec::new();
        self.render_inner(st, &mut seen)
    }

    fn render_inner(&self, st: &SourcefulText, seen: &mut Vec<String>) -> String {
        let mut out = String::new();
        for seg in st.iter() {
            match seg {
                Segment::Text(t) => out.push_str(t),
                Segment::Ref(r) => {
                    let key = render_ref(r);
                    if seen.contains(&key) {
                        out.push_str("⟨循环引用⟩");
                    } else if let Some(inner) = self.resolve(r) {
                        seen.push(key);
                        out.push_str(&self.render_inner(&inner, seen));
                        seen.pop();
                    } else {
                        out.push_str(&format!("⟨未解析:{}⟩", key));
                    }
                }
            }
        }
        out
    }

    /// 收集全文档引用（含位置），供 validate I8（经 view::walk_all_st 全覆盖——F03）。
    pub fn all_references(&self) -> Vec<(String, Reference)> {
        let mut out = Vec::new();
        crate::view::walk_all_st(self, |loc, st| {
            for seg in st.iter() {
                if let Segment::Ref(r) = seg {
                    out.push((loc.to_string(), r.clone()));
                }
            }
        });
        out
    }

    /// 引用图判环：从每个引用出发 DFS，路径上重复 ⇒ 环。返回环上引用串。
    pub fn ref_cycles(&self) -> Vec<String> {
        let mut cycles = Vec::new();
        let refs = self.all_references();
        let mut roots: BTreeSet<String> = BTreeSet::new();
        for (_, r) in &refs {
            roots.insert(render_ref(r));
        }
        for key in roots {
            let r = match crate::text::parse_ref(&key) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut path: Vec<String> = Vec::new();
            if self.dfs_cycle(&r, &mut path) {
                cycles.push(path.join(" → "));
            }
        }
        cycles
    }

    fn dfs_cycle(&self, r: &Reference, path: &mut Vec<String>) -> bool {
        let key = render_ref(r);
        if path.contains(&key) {
            path.push(key);
            return true;
        }
        path.push(key);
        if let Some(inner) = self.resolve(r) {
            for seg in inner.iter() {
                if let Segment::Ref(sub) = seg {
                    if self.dfs_cycle(sub, path) {
                        return true;
                    }
                }
            }
        }
        path.pop();
        false
    }
}

fn join_paragraphs(parts: &[SourcefulText]) -> SourcefulText {
    let mut out: Vec<Segment> = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if i > 0 {
            out.push(Segment::Text("\n\n".into()));
        }
        out.extend(p.iter().cloned());
    }
    SourcefulText(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{parse_ref, parse_st};

    fn mini() -> WorkingStack {
        let mut ws = WorkingStack::default();
        ws.prefix = Some("T".into());
        ws.frames.insert(
            1,
            Frame {
                parent: None,
                title: "根".into(),
                goal: vec![parse_st("完成 X").unwrap()],
                origin: vec![],
                status: FrameStatus::Active,
                plan: vec![PlanItem {
                    text: parse_st("第一步").unwrap(),
                    children: vec![],
                    progress: ItemState::InProgress,
                    fork: vec![],
                }],
                content: Content::default(),
                log: vec![Event {
                    action: Action::Push,
                    at: "2026-10-06T00:00:00Z".into(),
                    detail: parse_st("建帧").unwrap(),
                }],
            },
        );
        ws.notes.insert(
            "checklist".into(),
            Note {
                meta: vec!["appendix".into(), "order:10".into()],
                body: parse_st("全部实战").unwrap(),
            },
        );
        ws
    }

    #[test]
    fn resolves_fields() {
        let ws = mini();
        let want = |s: &str| ws.resolve(&parse_ref(s).unwrap());
        assert!(want("1").is_some());
        assert!(want("1.title").is_some());
        assert!(want("1.goal").is_some());
        assert!(want("1.plan[0]").is_some());
        assert!(want("1.log[0]").is_some());
        assert!(want("1.log[0].detail").is_some());
        assert!(want("note:checklist.body").is_some());
        assert_eq!(crate::text::flatten(&want("note:checklist.meta[0]").unwrap()), "appendix");
        assert!(want("note:checklist.meta[9]").is_none());
        assert!(want("1.nope").is_none());
        assert!(want("9").is_none());
        assert!(want("1.plan[5]").is_none());
        assert!(want("note:k.title").is_none()); // 蓝本无此字段
    }

    #[test]
    fn render_expands_and_cycle_safe() {
        let ws = mini();
        let r = ws.render(&parse_st("见 {{1.title}} 与 {{note:checklist.body}}").unwrap());
        assert!(r.contains("根") && r.contains("全部实战"));
        let bad = ws.render(&parse_st("{{42.goal}}").unwrap());
        assert!(bad.contains("未解析"));
        // F04 反例：status 自引用不再无限递归
        let mut w2 = mini();
        w2.frames.get_mut(&1).unwrap().status =
            FrameStatus::Paused { reason: parse_st("等 {{1.status}}").unwrap() };
        let out = w2.render(&parse_st("{{1.status}}").unwrap());
        assert!(out.contains("循环引用") || out.contains("paused"), "got: {out}");
    }

    #[test]
    fn detects_cycle() {
        let mut ws = mini();
        ws.notes.insert(
            "a".into(),
            Note { meta: Default::default(), body: parse_st("回 {{note:a.body}}").unwrap() },
        );
        assert!(!ws.ref_cycles().is_empty());
    }
}
