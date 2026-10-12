use crate::model::*;
use crate::text::format_st;

fn short(text: &str) -> String {
    let mut chars = text.chars();
    let first: String = chars.by_ref().take(240).collect();
    if chars.next().is_some() { format!("{first} …（截断；get路径读取全文）") } else { first }
}

fn text(out: &mut String, path: &str, value: &crate::text::SourcefulText) {
    out.push_str(&format!("- `{path}`：{}\n", short(&format_st(value))));
}

fn plans(out: &mut String, path: &str, items: &[PlanItem]) {
    for (i, item) in items.iter().enumerate() {
        let p = format!("{path}[{i}]");
        match &item.progress {
            ItemState::ItemDone | ItemState::ItemExpired { .. } => {}
            state => {
                let label = match state {
                    ItemState::Ready => "ready".into(),
                    ItemState::InProgress => "in_progress".into(),
                    ItemState::Blocked { reason } => format!("blocked：{}", short(&format_st(reason))),
                    _ => unreachable!(),
                };
                out.push_str(&format!("- `{p}` [{label}] {}\n", short(&format_st(&item.text))));
            }
        }
        plans(out, &format!("{p}.children"), &item.children);
    }
}

pub fn render(ws: &WorkingStack, selected: Option<Id>) -> String {
    let mut out = format!("# 工作上下文：{}\n\n", ws.title.as_deref().unwrap_or("未命名"));
    out.push_str("仅为已录入快照，可能不含最新对话裁决；记录不等于已验证/已接受，事件时间不等于事实观察时间。\n\n");
    let selected = selected.or_else(|| ws.focus());
    if let Some(id) = selected {
        let chain = ws.path(id);
        out.push_str(&format!("责任路径：{}\n", chain.iter().map(|i| ws.print_id(*i)).collect::<Vec<_>>().join(" → ")));
        out.push_str(&format!("{}：[{}] {}\n", if ws.focus() == Some(id) {"TOP"} else {"所选分支（不切换焦点）"}, ws.print_id(id), ws.frames[&id].title));
        if let Some(item) = WorkingStack::focus_item_of(&ws.frames[&id]) {
            out.push_str(&format!("FOCUS：{}\n", short(&format_st(&item.text))));
        } else {
            out.push_str("FOCUS：未指定；ready项只是待做计划，不自动选择下一步。\n");
        }
        for fid in chain {
            let f = &ws.frames[&fid];
            out.push_str(&format!("\n## [{}] {} [{}]\n", ws.print_id(fid), f.title, f.status.kind_str()));
            if f.goal.is_empty() { out.push_str("目标：未录入\n"); }
            for (i, v) in f.goal.iter().enumerate() { text(&mut out, &format!("{fid}.goal[{i}]"), v); }
            for (i, v) in f.origin.iter().enumerate() { text(&mut out, &format!("{fid}.origin[{i}]"), v); }
            out.push_str(if f.status.is_closed() { "关闭时的未完成计划快照（不代表当前待做）：\n" } else { "未完成计划：\n" });
            plans(&mut out, &format!("{fid}.plan"), &f.plan);
            if f.plan.is_empty() { out.push_str("- 无计划；目标不能替代当前步骤。\n"); }
            let skip = f.content.results.len().saturating_sub(3);
            if skip > 0 { out.push_str(&format!("结果：省略前{skip}条，get `{fid}.results`读取完整列表。\n")); }
            for (i, v) in f.content.results.iter().enumerate().skip(skip) { text(&mut out, &format!("{fid}.results[{i}]"), v); }
            for (key, v) in &f.content.remarks {
                out.push_str(&format!("备注 {key}（get `{fid}.remarks`）：{}\n", short(&format_st(v))));
            }
            for (i, p) in f.content.principles.iter().enumerate() {
                let fate = serde_json::to_string(&p.fate).unwrap();
                out.push_str(&format!("原则 `{fid}.content.principles[{i}]` [{fate}]：{}\n", short(&format_st(&p.rule))));
            }
            if let FrameStatus::Paused { reason } = &f.status { text(&mut out, &format!("{fid}.status.reason"), reason); }
            if let FrameStatus::Done { closure } | FrameStatus::Expired { closure } = &f.status {
                text(&mut out, &format!("{fid}.status.closure.summary"), &closure.summary);
            }
            if let Some(event) = f.log.last() {
                out.push_str(&format!("最近记录：{} [{}]；历史用show log {}查看。\n", event.at, event.action.label(), ws.print_id(fid)));
            }
        }
    } else {
        out.push_str("焦点：无；不自动选择暂停分支。\n");
        for id in ws.live_roots() {
            out.push_str(&format!("\n根目标 [{}] {}\n", ws.print_id(id), ws.frames[&id].title));
            for (i, v) in ws.frames[&id].goal.iter().enumerate() { text(&mut out, &format!("{id}.goal[{i}]"), v); }
        }
    }
    out.push_str("\n## 暂停末端（不代表后台停止）\n");
    for id in ws.tips() {
        if let FrameStatus::Paused { reason } = &ws.frames[&id].status {
            out.push_str(&format!("- [{}] {}：{}；show context {}查看该分支。\n", ws.print_id(id), ws.frames[&id].title, short(&format_st(reason)), ws.print_id(id)));
        }
    }
    out.push_str("\n下一步：对照最新用户裁决与以上计划/备注；只补读取影响当前判断的缺口，不把本视图当执行授权或全栈已校准证明。\n");
    out
}
