use crate::{model::*, text::{norm, Segment, SourcefulText}};
use serde_json::Value;
use std::collections::BTreeSet;

fn paths(old: &Value, new: &Value, path: &str, out: &mut Vec<String>) {
    if old == new { return; }
    match (old, new) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                let next = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                match (a.get(key), b.get(key)) {
                    (Some(a), Some(b)) => paths(a,b,&next,out),
                    _ => out.push(next),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i,(a,b)) in a.iter().zip(b).enumerate() { paths(a,b,&format!("{path}[{i}]"),out); }
        }
        _ => out.push(path.into()),
    }
}

pub fn fields(old: &Frame, new: &Frame, content_only: bool) -> Vec<String> {
    let mut old = serde_json::to_value(old).unwrap();
    let mut new = serde_json::to_value(new).unwrap();
    let keys: &[&str] = if content_only { &["log","status","parent"] } else { &["log"] };
    for key in keys {
        old.as_object_mut().unwrap().remove(*key);
        new.as_object_mut().unwrap().remove(*key);
    }
    let mut changed=Vec::new();
    paths(&old,&new,"",&mut changed);
    changed
}

pub fn notes(old: &WorkingStack, new: &WorkingStack) -> Vec<String> {
    let mut changed=Vec::new();
    paths(&serde_json::to_value(&old.notes).unwrap(),&serde_json::to_value(&new.notes).unwrap(),"notes",&mut changed);
    changed
}

pub fn detail(reason: &SourcefulText, changed: &[String]) -> SourcefulText {
    let mut detail=reason.clone();
    if !changed.is_empty() { detail.push(Segment::Text(format!("；变更字段：{}",changed.join("、")))); }
    norm(&detail)
}

pub fn coalesce_update(ws: &mut WorkingStack, old: &WorkingStack, at: &str) {
    for (id, before) in &old.frames {
        let frame=ws.frames.get_mut(id).unwrap();
        let changed=fields(before,frame,true);
        let fresh=frame.log.split_off(before.log.len());
        let mut recorded=false;
        for mut event in fresh {
            if matches!(event.action,Action::Update) && event.at == at {
                if recorded || changed.is_empty() { continue; }
                event.detail=detail(&event.detail,&changed);
                recorded=true;
            }
            frame.log.push(event);
        }
    }
}
