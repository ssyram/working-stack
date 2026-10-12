use crate::model::*;
use crate::text::{render_ref, Segment, SourcefulText};

fn text(ws: &WorkingStack, out: &mut String, path: &str, value: &SourcefulText) {
    out.push_str(&format!("- `{path}`：{}\n", ws.render(value)));
    let references: Vec<_> = value.iter().filter_map(|segment| match segment {
        Segment::Ref(reference) => Some(render_ref(reference)),
        Segment::Text(_) => None,
    }).collect();
    if !references.is_empty() {
        out.push_str(&format!("  引用来源：{}\n", references.join("、")));
    }
}

fn plans(ws: &WorkingStack, out: &mut String, path: &str, items: &[PlanItem]) {
    for (i, item) in items.iter().enumerate() {
        let p = format!("{path}[{i}]");
        match &item.progress {
            ItemState::ItemDone | ItemState::ItemExpired { .. } => {}
            state => {
                let label = match state {
                    ItemState::Ready => "ready",
                    ItemState::InProgress => "in_progress",
                    ItemState::Blocked { .. } => "blocked",
                    _ => unreachable!(),
                };
                out.push_str(&format!("计划 [{label}]\n"));
                text(ws, out, &p, &item.text);
                if let ItemState::Blocked { reason } = state {
                    text(ws, out, &format!("{p}.progress.reason"), reason);
                }
            }
        }
        plans(ws, out, &format!("{p}.children"), &item.children);
    }
}

pub fn render(ws: &WorkingStack, selected: Option<Id>) -> String {
    let mut out = format!("# 工作上下文：{}\n\n", ws.title.as_deref().unwrap_or("未命名"));
    out.push_str("仅为已录入快照，可能不含最新对话裁决；记录不等于已验证/已接受，事件时间不等于事实观察时间。\n\n");
    if let Some(note) = ws.notes.get("description") {
        text(ws, &mut out, "note:description.body", &note.body);
        out.push('\n');
    }
    let selected = selected.or_else(|| ws.focus());
    if let Some(id) = selected {
        let chain = ws.path(id);
        out.push_str(&format!("责任路径：{}\n", chain.iter().map(|i| ws.print_id(*i)).collect::<Vec<_>>().join(" → ")));
        out.push_str(&format!("{}：[{}] {}\n", if ws.focus() == Some(id) {"TOP"} else {"所选分支（不切换焦点）"}, ws.print_id(id), ws.frames[&id].title));
        if let Some(item) = WorkingStack::focus_item_of(&ws.frames[&id]) {
            out.push_str(&format!("FOCUS：{}\n", ws.render(&item.text)));
        } else {
            out.push_str("FOCUS：未指定；ready项只是待做计划，不自动选择下一步。\n");
        }
        for fid in chain {
            let f = &ws.frames[&fid];
            out.push_str(&format!("\n## [{}] {} [{}]\n", ws.print_id(fid), f.title, f.status.kind_str()));
            if f.goal.is_empty() { out.push_str("目标：未录入\n"); }
            for (i, v) in f.goal.iter().enumerate() { text(ws, &mut out, &format!("{fid}.goal[{i}]"), v); }
            for (i, v) in f.origin.iter().enumerate() { text(ws, &mut out, &format!("{fid}.origin[{i}]"), v); }
            out.push_str(if f.status.is_closed() { "关闭时的未完成计划快照（不代表当前待做）：\n" } else { "未完成计划：\n" });
            plans(ws, &mut out, &format!("{fid}.plan"), &f.plan);
            if f.plan.is_empty() { out.push_str("- 无计划；目标不能替代当前步骤。\n"); }
            if !f.content.results.is_empty() { out.push_str("已记录结果（不按写入顺序判断效力）：\n"); }
            for (i, v) in f.content.results.iter().enumerate() { text(ws, &mut out, &format!("{fid}.results[{i}]"), v); }
            if !f.content.side_effects.is_empty() { out.push_str("已登记的外部执行事项（不是实时监控）：\n"); }
            for (i, v) in f.content.side_effects.iter().enumerate() { text(ws, &mut out, &format!("{fid}.side_effects[{i}]"), v); }
            for (key, v) in &f.content.remarks {
                out.push_str(&format!("备注 {key}（get `{fid}.remarks`）：\n"));
                text(ws, &mut out, &format!("{fid}.remarks"), v);
            }
            for (i, p) in f.content.principles.iter().enumerate() {
                let fate = serde_json::to_string(&p.fate).unwrap();
                out.push_str(&format!("原则处置 [{fate}]\n"));
                text(ws, &mut out, &format!("{fid}.content.principles[{i}].rule"), &p.rule);
            }
            if let FrameStatus::Paused { reason } = &f.status { text(ws, &mut out, &format!("{fid}.status.reason"), reason); }
            if let FrameStatus::Done { closure } | FrameStatus::Expired { closure } = &f.status {
                text(ws, &mut out, &format!("{fid}.status.closure.summary"), &closure.summary);
            }
            if let Some(event) = f.log.last() {
                out.push_str(&format!("最近记录：{} [{}]；历史用show log {}查看。\n", event.at, event.action.label(), ws.print_id(fid)));
            }
        }
    } else {
        out.push_str("焦点：无；不自动选择暂停分支。\n");
        for id in ws.live_roots() {
            out.push_str(&format!("\n根目标 [{}] {}\n", ws.print_id(id), ws.frames[&id].title));
            for (i, v) in ws.frames[&id].goal.iter().enumerate() { text(ws, &mut out, &format!("{id}.goal[{i}]"), v); }
        }
    }
    out.push_str("\n## 暂停末端（不代表后台停止）\n");
    for id in ws.tips() {
        if let FrameStatus::Paused { reason } = &ws.frames[&id].status {
            out.push_str(&format!("- [{}] {}；show context {}查看该分支。\n", ws.print_id(id), ws.frames[&id].title, ws.print_id(id)));
            text(ws, &mut out, &format!("{id}.status.reason"), reason);
        }
    }
    out.push_str("\n使用边界：本视图提供复工材料，不代判继续理由、具体下一步或行动授权；缺失的新裁决和回执由调用方补齐，不把正文完整当全栈已校准证明。\n");
    out
}
