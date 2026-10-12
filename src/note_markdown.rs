use crate::model::{Meta, Note, WorkingStack};
use crate::note_layout::{layout, Position};
use crate::text::{format_st, parse_st};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct Header {
    key: String,
    meta: Meta,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    at_title: bool,
}

fn title(key: &str) -> String {
    match key {
        "description" => "工作线说明".into(),
        "import-cache" => "补充内容".into(),
        _ => key.replace(['\n','\r'], " "),
    }
}

pub fn parse_header(rest: &str) -> Result<(String, Meta), String> {
    if rest.starts_with('{') {
        let header: Header = serde_json::from_str(rest).map_err(|e| e.to_string())?;
        return Ok((header.key, header.meta));
    }
    let (key, meta) = match rest.trim().split_once('（') {
        Some((key, hints)) => (key.trim(), hints.trim_end_matches('）').split('，')
            .filter(|s| !s.is_empty()).map(String::from).collect()),
        None => (rest.trim(), vec![]),
    };
    Ok((key.into(), meta))
}

pub fn attachments(md: &str) -> Result<Vec<Header>, String> {
    let mut info=false;
    for line in md.lines() {
        if let Some(head)=line.strip_prefix("## ") { info=head=="格式信息"; }
        else if info {
            if let Some(value)=line.strip_prefix("> notes: ") {
                return serde_json::from_str(value).map_err(|e|e.to_string());
            }
        }
    }
    Ok(vec![])
}

pub fn read_description(body: &[&str], headers: &mut Vec<Header>) -> Result<Option<Note>, String> {
    if let Some(index) = headers.iter().position(|h| h.key == "description" && h.at_title) {
        let header = headers.remove(index);
        let text = body.iter().filter_map(|line| {
            line.strip_prefix("  ").or_else(|| (!line.is_empty()).then_some(*line))
        }).collect::<Vec<_>>().join("\n");
        return Ok(Some(Note { meta: header.meta, body: parse_st(&text)? }));
    }
    let text = body.join("\n");
    let text = text.trim();
    if text.is_empty() { Ok(None) }
    else { Ok(Some(Note { meta: vec!["before:tree".into()], body: parse_st(text)? })) }
}

pub fn emit_description(ws: &WorkingStack, out: &mut String, headers: &mut Vec<Header>) {
    if let Some(note) = ws.notes.get("description") {
        headers.push(Header { key: "description".into(), meta: note.meta.clone(), at_title: true });
        for line in format_st(&note.body).split('\n') { out.push_str(&format!("  {line}\n")); }
        out.push('\n');
    }
}

pub fn read_section(body: &[&str], notes: &mut BTreeMap<String,Note>, headers: &mut Vec<Header>) -> Result<(),String> {
    let mut current: Option<(String,Meta,bool)>=None;
    let mut buffer=Vec::new();
    for line in body {
        if let Some(head)=line.strip_prefix("### ") {
            finish(current.take(),&mut buffer,notes)?;
            if let Some(rest)=head.strip_prefix("note:") {
                let (key,meta)=parse_header(rest)?;
                current=Some((key,meta,true));
            } else if let Some(index)=headers.iter().position(|h|title(&h.key)==head) {
                let header=headers.remove(index);
                current=Some((header.key,header.meta,false));
            } else {
                current=Some((head.to_string(),vec![],false));
            }
        } else if current.is_some() {
            if let Some(text)=line.strip_prefix("  ") { buffer.push(text.to_string()); }
            else if !line.is_empty() { buffer.push((*line).to_string()); }
        }
    }
    finish(current,&mut buffer,notes)
}

fn finish(current: Option<(String,Meta,bool)>, buffer: &mut Vec<String>, notes: &mut BTreeMap<String,Note>) -> Result<(),String> {
    if let Some((key,meta,legacy))=current {
        let text=buffer.join("\n");
        let text=if legacy {text.trim()} else {&text};
        notes.insert(key,Note{meta,body:parse_st(text)?});
    }
    buffer.clear();
    Ok(())
}

pub fn emit(ws: &WorkingStack, position: Position, out: &mut String, headers: &mut Vec<Header>) {
    let mut notes: Vec<_> = ws.notes.iter().filter_map(|(key, note)| {
        let (p, order) = layout(&note.meta);
        (key != "description" && p == position).then_some((order, key, note))
    }).collect();
    notes.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)));
    if notes.is_empty() { return; }
    out.push_str(if position == Position::Appendix { "## Appendix\n\n" } else { "## 共享内容\n\n" });
    for (_, key, note) in notes {
        headers.push(Header{key:key.clone(),meta:note.meta.clone(),at_title:false});
        out.push_str(&format!("### {}\n\n",title(key)));
        for line in format_st(&note.body).split('\n') { out.push_str(&format!("  {line}\n")); }
        out.push('\n');
    }
}

pub fn emit_attachments(headers: &[Header], out: &mut String) {
    if headers.is_empty() { return; }
    out.push_str("## 格式信息\n\n> 以下仅用于再次解析，非业务内容。\n");
    out.push_str(&format!("> notes: {}\n",serde_json::to_string(headers).unwrap()));
}

pub fn metadata_complete(headers: &[Header]) -> Result<(),String> {
    if headers.is_empty() { Ok(()) } else { Err("格式附录中存在未匹配的note标题".into()) }
}
