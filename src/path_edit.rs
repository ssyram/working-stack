use crate::{model::{WorkingStack, WsError}, stack_path::{self, Part}};
use serde_json::{json, Value};

pub enum Edit {
    Set { path: String, value: Value },
    Insert { path: String, value: Value },
    Remove { path: String },
    Move { path: String, from: String },
}

fn usage(text: &str) -> WsError { WsError::Usage(text.into()) }
fn codec(error: serde_json::Error) -> WsError { WsError::Parse(error.to_string()) }

fn plan_list(parts: &[Part]) -> bool {
    matches!(parts.last(), Some(Part::Key(key)) if key == "plan" || key == "children")
}
fn plan_item(parts: &[Part]) -> bool {
    matches!(parts.last(), Some(Part::Index(_))) && plan_list(&parts[..parts.len()-1])
}

fn item(value: Value) -> Result<Value, WsError> {
    let mut value = match value { Value::String(text) => json!({"text":text}), other => other };
    if let Value::Object(fields) = &mut value {
        if let Some(children) = fields.get_mut("children") { *children = list(children.take())?; }
        if let Some(Value::String(state)) = fields.get("progress") {
            let state = crate::cli::parse_state(state)?;
            fields.insert("progress".into(), serde_json::to_value(state).map_err(codec)?);
        }
    }
    Ok(value)
}
fn list(value: Value) -> Result<Value, WsError> {
    match value {
        Value::Array(values) => values.into_iter().map(item).collect::<Result<Vec<_>,_>>().map(Value::Array),
        other => Ok(other),
    }
}

fn prepared(parts: &[Part], value: Value, inserting: bool, existing: Option<&Value>) -> Result<Value, WsError> {
    if plan_item(parts) {
        if !inserting && value.is_string() {
            let mut object = existing.unwrap().clone();
            object["text"] = value;
            return Ok(object);
        }
        return item(value);
    }
    if plan_list(parts) { return if inserting { item(value) } else { list(value) }; }
    if matches!(parts.last(), Some(Part::Key(key)) if key == "progress") && plan_item(&parts[..parts.len()-1]) {
        if let Value::String(state) = &value {
            return serde_json::to_value(crate::cli::parse_state(state)?).map_err(codec);
        }
    }
    Ok(value)
}

fn insert(root: &mut Value, parts: &[Part], value: Value) -> Result<(), WsError> {
    if let Some(Part::Index(index)) = parts.last() {
        let array = stack_path::at_mut(root, &parts[..parts.len()-1])?.as_array_mut()
            .ok_or_else(|| usage("insert 目标必须是列表"))?;
        if *index > array.len() { return Err(usage("insert 下标超出列表长度")); }
        array.insert(*index, value);
    } else {
        stack_path::at_mut(root, parts)?.as_array_mut()
            .ok_or_else(|| usage("insert 目标必须是列表"))?.push(value);
    }
    Ok(())
}

fn remove(root: &mut Value, parts: &[Part]) -> Result<Value, WsError> {
    let parent = stack_path::at_mut(root, &parts[..parts.len()-1])?;
    match parts.last().unwrap() {
        Part::Index(index) => {
            let array = parent.as_array_mut().ok_or_else(|| usage("remove 下标目标必须是列表"))?;
            if *index >= array.len() { return Err(usage("remove 下标越界")); }
            Ok(array.remove(*index))
        }
        Part::Key(key) => parent.as_object_mut().ok_or_else(|| usage("remove 字段目标必须是对象"))?
            .remove(key).ok_or_else(|| usage("remove 字段不存在")),
    }
}

pub fn get(ws: &WorkingStack, path: &str) -> Result<Value, WsError> {
    let value = serde_json::to_value(ws).map_err(codec)?;
    Ok(stack_path::read(&value, &stack_path::parse(path)?)?.clone())
}

pub fn edit(ws: &WorkingStack, operation: Edit) -> Result<WorkingStack, WsError> {
    let path = match &operation {
        Edit::Set {path,..} | Edit::Insert {path,..} | Edit::Remove {path} | Edit::Move {path,..} => path,
    };
    let parts = stack_path::parse(path)?;
    if !stack_path::editable(&parts) { return Err(usage("此路径由专用操作维护，不属于内容编辑范围")); }
    let mut candidate = serde_json::to_value(ws).map_err(codec)?;
    match operation {
        Edit::Set { value, .. } => {
            let old = stack_path::read(&candidate, &parts)?;
            let value = prepared(&parts, value, false, Some(old))?;
            *stack_path::at_mut(&mut candidate, &parts)? = value;
        }
        Edit::Insert { value, .. } => {
            let value = prepared(&parts, value, true, None)?;
            insert(&mut candidate, &parts, value)?;
        }
        Edit::Remove { .. } => { remove(&mut candidate, &parts)?; }
        Edit::Move { from, .. } => {
            let source = stack_path::parse(&from)?;
            if !stack_path::editable(&source) { return Err(usage("来源路径不属于内容编辑范围")); }
            let value = remove(&mut candidate, &source)?;
            insert(&mut candidate, &parts, value)?;
        }
    }
    serde_json::from_value(candidate).map_err(codec)
}
