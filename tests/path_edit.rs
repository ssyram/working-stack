use serde_json::json;
use working_stack::{model::*, path_edit::{edit, get, Edit}, text::parse_st};

fn sample() -> WorkingStack {
    let mut ws = WorkingStack::default();
    ws.push_root("t", "根".into(), vec![], parse_st("建根").unwrap()).unwrap();
    for text in ["A", "B", "C"] {
        ws.plan_add("t", 1, parse_st(text).unwrap(), ItemState::Ready, parse_st("加计划").unwrap()).unwrap();
    }
    ws
}
fn texts(ws: &WorkingStack) -> Vec<String> {
    ws.frames[&1].plan.iter().map(|p| working_stack::text::flatten(&p.text)).collect()
}

#[test]
fn inserts_before_first_middle_and_at_end_or_list() {
    let ws = sample();
    for (path, expected) in [
        ("X-01.plan[0]", vec!["new", "A", "B", "C"]),
        ("1.plan[1]", vec!["A", "new", "B", "C"]),
        ("1.plan[3]", vec!["A", "B", "C", "new"]),
        ("1.plan", vec!["A", "B", "C", "new"]),
    ] {
        let candidate = edit(&ws, Edit::Insert { path:path.into(), value:json!("new") }).unwrap();
        assert_eq!(texts(&candidate), expected);
        assert_eq!(texts(&ws), vec!["A", "B", "C"]);
    }
}

#[test]
fn nested_set_string_preserves_item_metadata() {
    let mut ws = sample();
    ws.push("t",1,"子帧".into(),vec![],vec![],parse_st("分支").unwrap()).unwrap();
    let ws = edit(&ws, Edit::Set {path:"1.plan[0]".into(),value:json!({"text":"parent","progress":"in_progress","children":["child"],"fork":[2]})}).unwrap();
    let candidate = edit(&ws, Edit::Set {path:"1.plan[0]".into(),value:json!("new parent")}).unwrap();
    assert_eq!(candidate.frames[&1].plan[0].progress, ItemState::InProgress);
    assert_eq!(candidate.frames[&1].plan[0].children, ws.frames[&1].plan[0].children);
    assert_eq!(candidate.frames[&1].plan[0].fork, vec![2]);
    assert!(working_stack::validate::validate(&candidate).is_empty());
    let candidate = edit(&candidate, Edit::Set {path:"1.plan[0].children[0].text".into(),value:json!("new child")}).unwrap();
    assert_eq!(get(&candidate,"1.plan[0].children[0].text").unwrap(),json!("new child"));
    let candidate = edit(&candidate, Edit::Set {path:"1.plan[0].children[0].progress".into(),value:json!("blocked:等待")}).unwrap();
    assert_eq!(get(&candidate,"1.plan[0].children[0].progress").unwrap(),json!({"kind":"blocked","reason":"等待"}));
}

#[test]
fn replace_plan_split_children_and_remove() {
    let ws = sample();
    let ws = edit(&ws, Edit::Set {path:"1.plan".into(),value:json!([{"text":"完成部分","progress":"done"},"剩余事项"])}).unwrap();
    assert_eq!(ws.frames[&1].plan[0].progress,ItemState::ItemDone);
    let ws = edit(&ws, Edit::Set {path:"1.plan[1].children".into(),value:json!(["子项一",{"text":"子项二","children":["深层"]}])}).unwrap();
    assert_eq!(get(&ws,"1.plan[1].children[1].children[0].text").unwrap(),json!("深层"));
    let ws = edit(&ws, Edit::Remove {path:"1.plan[1].children[0]".into()}).unwrap();
    assert_eq!(ws.frames[&1].plan[1].children.len(),1);
}

#[test]
fn move_uses_destination_after_removal_and_keeps_original_on_error() {
    let ws = sample();
    let moved = edit(&ws, Edit::Move {from:"1.plan[0]".into(),path:"1.plan[2]".into()}).unwrap();
    assert_eq!(texts(&moved),vec!["B","C","A"]);
    let old = ws.clone();
    assert!(edit(&ws, Edit::Move {from:"1.plan[0]".into(),path:"1.plan[8]".into()}).is_err());
    assert_eq!(ws,old);
    assert!(edit(&ws, Edit::Insert {path:"1.plan[4]".into(),value:json!("bad")}).is_err());
    assert!(edit(&ws, Edit::Set {path:"1.plan[3].text".into(),value:json!("bad")}).is_err());
}

#[test]
fn supports_content_aliases_notes_and_non_plan_progress_remark() {
    let mut ws = sample();
    ws.frames.get_mut(&1).unwrap().content.remarks.insert("progress".into(),parse_st("旧备注").unwrap());
    ws.notes.insert("k".into(),Note{meta:vec![],body:parse_st("共享").unwrap()});
    let ws = edit(&ws,Edit::Insert{path:"1.results".into(),value:json!("结果")}).unwrap();
    assert_eq!(get(&ws,"1.content.results[0]").unwrap(),json!("结果"));
    let ws = edit(&ws,Edit::Set{path:"1.remarks.progress".into(),value:json!("新备注")}).unwrap();
    assert_eq!(get(&ws,"1.remarks.progress").unwrap(),json!("新备注"));
    let ws = edit(&ws,Edit::Insert{path:"note:k.meta".into(),value:json!("unknown-hint")}).unwrap();
    assert_eq!(get(&ws,"note:k.meta[0]").unwrap(),json!("unknown-hint"));
}

#[test]
fn identity_lifecycle_and_history_remain_owned_by_existing_operations() {
    let ws = sample();
    for path in ["1", "1.status", "1.parent", "1.log", "1.principles"] {
        assert!(edit(&ws,Edit::Set{path:path.into(),value:json!(null)}).is_err());
    }
    assert!(get(&ws,"1.status.kind").is_ok());
    assert!(get(&ws,"1.plan[0].nope").is_err());
}
