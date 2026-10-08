use crate::{model::*, text::{format_st, SourcefulText}};
use chrono::DateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub frame: Option<Id>,
    pub frame_id: Option<String>,
    pub log_index: usize,
    pub at: String,
    pub action: Action,
    pub detail: SourcefulText,
    pub time_known: bool,
}

pub fn collect(ws: &WorkingStack) -> Vec<Row> {
    let mut rows = Vec::new();
    for (frame, log) in ws.frames.iter().map(|(id, frame)| (Some(*id), &frame.log))
        .chain(std::iter::once((None, &ws.log))) {
        for (log_index, event) in log.iter().enumerate() {
            let instant = DateTime::parse_from_rfc3339(&event.at).ok();
            let row = Row {
                frame, frame_id: frame.map(|id| ws.print_id(id)), log_index,
                at: event.at.clone(), action: event.action.clone(), detail: event.detail.clone(),
                time_known: instant.is_some(),
            };
            rows.push((instant, row));
        }
    }
    rows.sort_by(|a, b| match (&a.0, &b.0) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    rows.into_iter().map(|(_, row)| row).collect()
}

pub fn markdown(ws: &WorkingStack) -> String {
    let mut output = String::from("# 事件时间线\n\n同刻事件仅保留稳定展示顺序；时间不完整的记录列在末尾，不推断因果。\n\n");
    for row in collect(ws) {
        let owner = row.frame_id.as_deref().unwrap_or("stack");
        let unknown = if row.time_known { "" } else { "（时间不完整）" };
        output.push_str(&format!("- [{}] [{} log[{}]] [{}]{} {}\n",
            row.at, owner, row.log_index, row.action.label(), unknown,
            format_st(&row.detail).replace('\n', "\n  ")));
    }
    output
}
