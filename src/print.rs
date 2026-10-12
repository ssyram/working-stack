//! 打印：树视图 + Markdown 导出（保真目标——与既有家族文件仅排版差；映射见 detailed-design §4.8）。
//! F13/F16/F23 修复：暂停原因入节、notes 全文走共享内容节、live 根过滤、[digression]/[TOP?] 概览标记。

use crate::model::*;
use crate::text::{flatten, format_st, SourcefulText};
use crate::note_layout::Position;
use crate::note_markdown::emit as emit_notes;
use std::collections::HashSet;

/// 栈状态树（仅此块）。dash/缩进只出现在兄弟分叉层；单链自然向下；live 根过滤（F16）。
pub fn tree_text(ws: &WorkingStack) -> String {
    let tips: HashSet<Id> = ws.tips().into_iter().collect();
    let focus = ws.focus();
    let mut out = String::new();
    for r in ws.live_roots() {
        emit_tree(ws, r, 0, false, &tips, focus, &mut out);
    }
    if out.is_empty() {
        out.push_str("空");
    }
    out
}

fn origin_is_digression(f: &Frame) -> bool {
    f.origin.first().map(|s| flatten(s).trim().starts_with("[digression]")).unwrap_or(false)
}

fn emit_tree(ws: &WorkingStack, id: Id, indent: usize, dashed: bool, tips: &HashSet<Id>, focus: Option<Id>, out: &mut String) {
    let f = &ws.frames[&id];
    let dig = if origin_is_digression(f) { "/digression" } else { "" };
    let marker = if tips.contains(&id) {
        match &f.status {
            FrameStatus::Active if focus == Some(id) => format!(" [{}{}]", WorkingStack::render_top(f), dig),
            FrameStatus::Active => format!(" [TOP?{dig}]"), // 多焦点/无焦点的诊断标记，非权威
            FrameStatus::Paused { .. } => format!(" [PAUSED{dig}]"),
            _ => String::new(),
        }
    } else {
        String::new()
    };
    let dash = if dashed { "- " } else { "" };
    out.push_str(&" ".repeat(indent));
    out.push_str(&format!("{dash}[{}]{} {}\n", ws.print_id(id), marker, f.title));
    let kids = ws.live_children(id);
    // 家族缩进规则：dash 子代 = 父 dash ? 父缩进+2 : 同层；单链延续无 dash（W-04 形态）
    let child_indent = if dashed { indent + 2 } else { indent };
    if kids.len() == 1 {
        emit_tree(ws, kids[0], child_indent, false, tips, focus, out);
    } else if kids.len() > 1 {
        for k in kids {
            emit_tree(ws, k, child_indent, true, tips, focus, out);
        }
    }
}

/// 整份 Markdown 导出（canonical 形；md 是变换管道的 re-print，非承载）。
pub fn export_markdown(ws: &WorkingStack) -> String {
    let mut s = String::new();
    let mut note_headers = Vec::new();
    if let Some(t) = &ws.title {
        s.push_str(&format!("# {t}\n\n"));
    }
    crate::note_markdown::emit_description(ws, &mut s, &mut note_headers);
    emit_notes(ws, Position::BeforeTree, &mut s, &mut note_headers);
    s.push_str("## 栈状态\n\n```text\n");
    s.push_str(&tree_text(ws));
    s.push_str("\n```\n\n");
    emit_notes(ws, Position::AfterTree, &mut s, &mut note_headers);

    let mut popped_desc: Vec<Id> = ws
        .frames
        .iter()
        .filter(|(_, f)| f.status.is_closed())
        .map(|(id, _)| *id)
        .collect();
    popped_desc.sort_unstable_by(|a, b| b.cmp(a));
    if !popped_desc.is_empty() {
        s.push_str("## 已 pop 节点\n\n");
        for id in &popped_desc {
            let f = &ws.frames[&id];
            let up = match f.parent {
                None => "无".to_string(),
                Some(p) => format!("[{}]", ws.print_id(p)),
            };
            s.push_str(&format!("- [{}] [POPPED] {}；上帧：{}。\n", ws.print_id(*id), f.title, up));
        }
        s.push('\n');
    }

    emit_notes(ws, Position::BeforeFrames, &mut s, &mut note_headers);

    for id in ws.live_tree_order() {
        s.push_str(&frame_section(ws, id));
    }
    for id in popped_desc {
        s.push_str(&frame_section(ws, id));
    }
    emit_notes(ws, Position::AfterFrames, &mut s, &mut note_headers);
    // 栈级事件（note/meta 变更记录）
    if !ws.log.is_empty() {
        s.push_str("## 栈变化记录\n\n");
        for e in &ws.log {
            s.push_str(&format!("- [{}] [{}] {}\n", e.action.label(), e.at, ws.render(&e.detail)));
        }
        s.push('\n');
    }
    emit_notes(ws, Position::Appendix, &mut s, &mut note_headers);
    crate::note_markdown::emit_attachments(&note_headers, &mut s);
    s
}

/// 单帧节（含变化记录）。
pub fn frame_section(ws: &WorkingStack, id: Id) -> String {
    let f = &ws.frames[&id];
    let tip = ws.tips().contains(&id);
    let marker = match &f.status {
        FrameStatus::Active if tip && ws.focus() == Some(id) => format!(" [{}]", WorkingStack::render_top(f)),
        FrameStatus::Active => String::new(),
        FrameStatus::Paused { .. } => " [PAUSED]".into(),
        FrameStatus::Done { .. } | FrameStatus::Expired { .. } => " [POPPED]".into(),
    };
    let mut s = format!("## [{}]{} {}\n\n", ws.print_id(id), marker, f.title);
    s.push_str(&format!(
        "- 上帧：{}\n",
        match f.parent {
            None => "无（根帧）".to_string(),
            Some(p) => format!("[{}]", ws.print_id(p)),
        }
    ));
    if !f.origin.is_empty() {
        s.push_str(&format!("- 来源：{}\n", render_paragraphs(ws, &f.origin)));
    }
    if !f.goal.is_empty() {
        s.push_str(&format!("- 弹帧目标：{}\n", render_paragraphs(ws, &f.goal)));
    }
    if let FrameStatus::Paused { reason } = &f.status {
        s.push_str(&format!("- 暂停原因：{}\n", ws.render(reason)));
    }
    if let FrameStatus::Done { closure } | FrameStatus::Expired { closure } = &f.status {
        let kind = f.status.kind_str();
        s.push_str(&format!("- 达成情况：\n  - 结果：[{}] {}\n", kind, ws.render(&closure.summary)));
        crate::closure_markdown::emit(closure, &mut s);
    }
    if !f.plan.is_empty() {
        s.push_str("- 计划：\n");
        let focus_frame = ws.focus() == Some(id);
        let focus_item = WorkingStack::focus_item_of(f);
        emit_plan(ws, &f.plan, 1, focus_frame, focus_item, &mut s);
    }
    if !f.content.results.is_empty() {
        s.push_str("- 已得结果：\n");
        for r in &f.content.results {
            s.push_str(&format!("  - {}\n", format_st(r)));
        }
    }
    s.push_str("- 外部执行事项：");
    if f.content.side_effects.is_empty() {
        s.push_str("无。\n");
    } else {
        s.push('\n');
        for r in &f.content.side_effects {
            s.push_str(&format!("  - {}\n", ws.render(r)));
        }
    }
    if !f.content.principles.is_empty() {
        s.push_str("- 确立原则：\n");
        for (i, p) in f.content.principles.iter().enumerate() {
            crate::principle_markdown::emit(p, i, &mut s);
        }
    }
    for (k, v) in &f.content.remarks {
        if k == "备注" {
            s.push_str("- 备注：\n");
        } else {
            s.push_str(&format!("- 备注 · {}：\n", k));
        }
        for line in format_st(v).split('\n') {
            s.push_str(&format!("  {line}\n"));
        }
    }
    s.push_str("\n### 帧变化记录\n\n");
    for e in &f.log {
        s.push_str(&format!("- [{}] [{}] {}\n", e.action.label(), e.at, ws.render(&e.detail)));
    }
    s.push('\n');
    s
}

/// FOCUS 加粗还原：仅焦点帧的派生 focus 项打印 **[FOCUS/state]**，其余同族格式。
fn emit_plan(
    ws: &WorkingStack,
    plan: &[PlanItem],
    depth: usize,
    focus_frame: bool,
    focus_item: Option<&PlanItem>,
    out: &mut String,
) {
    for it in plan {
        let state = match &it.progress {
            ItemState::Ready => "[ready] ".into(),
            ItemState::InProgress => "[in_progress] ".into(),
            ItemState::Blocked { reason } => {
                if reason.is_empty() {
                    "[blocked] ".into()
                } else {
                    format!("[blocked]（{}）", ws.render(reason))
                }
            }
            ItemState::ItemDone => "[done] ".into(),
            ItemState::ItemExpired { reason } => format!("[expire: {}] ", ws.render(reason)),
        };
        let bullet = if focus_frame && focus_item.is_some_and(|x| std::ptr::eq(x, it)) {
            let plain = state.trim_matches(|c| c == '[' || c == ' ' || c == ']');
            format!("**[FOCUS/{}]** ", plain)
        } else {
            state
        };
        out.push_str(&" ".repeat(2 * depth));
        out.push_str(&format!("- {}{}\n", bullet, ws.render(&it.text)));
        emit_plan(ws, &it.children, depth + 1, focus_frame, focus_item, out);
    }
}

fn render_paragraphs(ws: &WorkingStack, parts: &[SourcefulText]) -> String {
    parts
        .iter()
        .map(|p| ws.render(p))
        .collect::<Vec<_>>()
        .join("\n  ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::parse_st;

    #[test]
    fn export_shape() {
        let mut ws = WorkingStack::default();
        ws.title = Some("测试栈（WS-T）".into());
        ws.prefix = Some("T".into());
        let r = ws
            .push_root("2026-10-06T00:00:00Z", "根任务".into(), vec![parse_st("完成 X").unwrap()], parse_st("建栈").unwrap())
            .unwrap();
        ws.plan_add("t", r, parse_st("第一步").unwrap(), ItemState::InProgress, parse_st("开始").unwrap())
            .unwrap();
        let md = export_markdown(&ws);
        assert!(md.contains("# 测试栈（WS-T）"));
        assert!(md.contains("## 栈状态"));
        assert!(md.contains("[T-01] [TOP] 根任务"));
        assert!(md.contains("**[FOCUS/in_progress]**"));
        assert!(md.contains("### 帧变化记录"));
        assert!(md.contains("- [PUSH] [2026-10-06T00:00:00Z]"));
    }

    #[test]
    fn tree_para_shape_and_markers() {
        let mut ws = WorkingStack::default();
        ws.prefix = Some("W".into());
        let r = ws.push_root("t", "根".into(), vec![], parse_st("r").unwrap()).unwrap();
        let _a = ws.push("t", r, "A".into(), vec![], vec![], parse_st("r").unwrap()).unwrap();
        let b = ws.push("t", r, "B".into(), vec![], vec![], parse_st("r").unwrap()).unwrap();
        let _c = ws.push("t", b, "C".into(), vec![], vec![], parse_st("r").unwrap()).unwrap();
        let t = tree_text(&ws);
        let lines: Vec<&str> = t.lines().collect();
        assert_eq!(lines[0], "[W-01] 根");
        assert_eq!(lines[1], "- [W-02] [PAUSED] A");
        assert_eq!(lines[2], "- [W-03] B");
        assert_eq!(lines[3], "  [W-04] [TOP] C");
    }

    #[test]
    fn paused_reason_exported() {
        // F13 反例：Paused 理由必须出现在导出
        let mut ws = WorkingStack::default();
        ws.prefix = Some("P".into());
        ws.push_root("t", "根".into(), vec![], parse_st("r").unwrap()).unwrap();
        ws.pause("t2", 1, parse_st("等待审批").unwrap()).unwrap();
        let md = export_markdown(&ws);
        assert!(md.contains("等待审批"), "{md}");
        assert!(md.contains("- 暂停原因：等待审批"));
    }

    #[test]
    fn shared_notes_full_body_exported() {
        // S9 反例：被引用 note 全文必须可往返
        let mut ws = WorkingStack::default();
        ws.prefix = Some("S".into());
        let mut goal = parse_st("见 {{note:checklist.body}}").unwrap();
        let _ = &mut goal;
        ws.push_root("t", "根".into(), vec![goal], parse_st("r").unwrap()).unwrap();
        ws.note_put(
            "t",
            "checklist",
            vec!["appendix".into(), "checklist".into()],
            parse_st("第一行\n第二行关键义务").unwrap(),
            parse_st("建清单").unwrap(),
        )
        .unwrap();
        let md = export_markdown(&ws);
        assert!(md.contains("### checklist\n"));
        assert!(md.contains("> notes: [{\"key\":\"checklist\",\"meta\":[\"appendix\",\"checklist\"]}]"));
        assert!(!md.contains("### note:"));
        assert!(md.contains("第二行关键义务"), "{md}");
    }

    #[test]
    fn closed_root_not_in_tree_or_dup() {
        // F16 反例
        let mut ws = WorkingStack::default();
        ws.push_root("t", "根".into(), vec![], parse_st("r").unwrap()).unwrap();
        let cl = Closure { summary: parse_st("完成").unwrap(), ..Default::default() };
        ws.pop("t2", 1, Outcome::DoneO(cl), None, parse_st("收").unwrap()).unwrap();
        let md = export_markdown(&ws);
        assert_eq!(md.matches("## [1] [POPPED]").count(), 1);
        assert!(tree_text(&ws).contains("空"));
    }
}
