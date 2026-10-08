use crate::{model::WsError, text::{parse_ref, RefSeg}};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Part { Key(String), Index(usize) }

pub fn parse(path: &str) -> Result<Vec<Part>, WsError> {
    let reference = parse_ref(path).map_err(WsError::Parse)?;
    let mut parts = match &reference.path[0] {
        RefSeg::Frame(id) => vec![Part::Key("frames".into()), Part::Key(id.to_string())],
        RefSeg::NoteK(key) => vec![Part::Key("notes".into()), Part::Key(key.clone())],
        _ => unreachable!(),
    };
    for (i, segment) in reference.path.iter().enumerate().skip(1) {
        match segment {
            RefSeg::Field(field) => {
                if i == 1 && matches!(reference.path[0], RefSeg::Frame(_))
                    && matches!(field.as_str(), "results" | "side_effects" | "remarks" | "principles") {
                    parts.push(Part::Key("content".into()));
                }
                parts.push(Part::Key(field.clone()));
            }
            RefSeg::Index(index) => parts.push(Part::Index(*index)),
            _ => unreachable!(),
        }
    }
    Ok(parts)
}

pub fn editable(parts: &[Part]) -> bool {
    match parts {
        [Part::Key(root), Part::Key(_), Part::Key(field), ..] if root == "notes" => {
            matches!(field.as_str(), "body" | "meta")
        }
        [Part::Key(root), Part::Key(_), Part::Key(field), tail @ ..] if root == "frames" => {
            matches!(field.as_str(), "title" | "goal" | "origin" | "plan")
                || field == "content" && matches!(tail.first(), Some(Part::Key(name))
                    if matches!(name.as_str(), "results" | "side_effects" | "remarks"))
        }
        _ => false,
    }
}

pub fn read<'a>(mut value: &'a Value, parts: &[Part]) -> Result<&'a Value, WsError> {
    for part in parts {
        value = match part {
            Part::Key(key) => value.get(key),
            Part::Index(index) => value.get(*index),
        }.ok_or_else(|| WsError::Usage(format!("路径不存在：{part:?}")))?;
    }
    Ok(value)
}

pub fn at_mut<'a>(mut value: &'a mut Value, parts: &[Part]) -> Result<&'a mut Value, WsError> {
    for part in parts {
        value = match part {
            Part::Key(key) => value.get_mut(key),
            Part::Index(index) => value.get_mut(*index),
        }.ok_or_else(|| WsError::Usage(format!("路径不存在：{part:?}")))?;
    }
    Ok(value)
}
