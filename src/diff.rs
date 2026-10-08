//! 机械差异摘要（提交返回；不复制正文）。F12 修复：status 比较整个和类型值（含理由/关闭报告）；
//! C2 修复：补 parent 与 log 内容比较。

use crate::model::*;
use std::collections::BTreeSet;

/// diff(old, new) -> 人读摘要行。用途：CLI/Web 提交回执与 apply 的事件生成依据。
pub fn diff(old: &WorkingStack, new: &WorkingStack) -> Vec<String> {
    let mut out = Vec::new();
    if old.title != new.title || old.prefix != new.prefix {
        out.push("元信息（title/prefix）改变".into());
    }
    let old_ids: BTreeSet<Id> = old.frames.keys().copied().collect();
    let new_ids: BTreeSet<Id> = new.frames.keys().copied().collect();
    for id in new_ids.difference(&old_ids) {
        out.push(format!("新增帧 {} {}", new.print_id(*id), new.frames[id].title));
    }
    for id in old_ids.difference(&new_ids) {
        out.push(format!("移除帧 {} {}", old.print_id(*id), old.frames[id].title));
    }
    for id in old_ids.intersection(&new_ids) {
        let a = &old.frames[id];
        let b = &new.frames[id];
        let pid = new.print_id(*id);
        if a.parent != b.parent {
            out.push(format!("{pid}: 改挂（parent 变更）"));
        }
        if a.title != b.title {
            out.push(format!("{pid}: title 改变"));
        }
        if a.goal != b.goal {
            out.push(format!("{pid}: goal 改变"));
        }
        if a.origin != b.origin {
            out.push(format!("{pid}: origin 改变"));
        }
        if a.status != b.status {
            out.push(format!("{pid}: status {} → {}", a.status.kind_str(), b.status.kind_str()));
        }
        if a.plan != b.plan {
            out.push(format!("{pid}: plan 改变"));
        }
        if a.content != b.content {
            out.push(format!("{pid}: content（结果/原则/备注）改变"));
        }
        let (la, lb) = (a.log.len(), b.log.len());
        if a.log != b.log {
            if lb > la {
                out.push(format!("{pid}: log +{}", lb - la));
            } else {
                out.push(format!("{pid}: log 改写（长度 {} → {}）", la, lb));
            }
        }
    }
    let old_notes: BTreeSet<&String> = old.notes.keys().collect();
    let new_notes: BTreeSet<&String> = new.notes.keys().collect();
    for k in new_notes.difference(&old_notes) {
        out.push(format!("新增 note {k}"));
    }
    for k in old_notes.difference(&new_notes) {
        out.push(format!("移除 note {k}"));
    }
    for k in old_notes.intersection(&new_notes) {
        if old.notes[*k] != new.notes[*k] {
            out.push(format!("note {k} 更新"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::SourcefulText;

    #[test]
    fn catches_status_payload_change() {
        // F12 反例：同 kind 不同 reason 必须报差异
        let mk = |r: &str| {
            let mut ws = WorkingStack::default();
            ws.frames.insert(
                1,
                Frame {
                    parent: None,
                    title: "t".into(),
                    goal: vec![],
                    origin: vec![],
                    status: FrameStatus::Paused { reason: SourcefulText::text(r) },
                    plan: vec![],
                    content: Content::default(),
                    log: vec![],
                },
            );
            ws
        };
        let d = diff(&mk("等 A"), &mk("等 B"));
        assert!(d.iter().any(|x| x.contains("status")), "{d:?}");
    }
}
