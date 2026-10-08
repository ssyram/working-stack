use working_stack::{import::import_md, model::*, note_layout::{layout, Position}, print::export_markdown, text::{parse_ref, parse_st, flatten}};

fn note(hints: &[&str], body: &str) -> Note {
    Note { meta: hints.iter().map(|s| s.to_string()).collect(), body: parse_st(body).unwrap() }
}

#[test]
fn hints_are_optional_ordered_and_non_validating() {
    assert_eq!(layout(&vec![]), (Position::Appendix, 0));
    let hints = vec!["before:frames", "不认识", "order:-5", "after:missing", "order:oops"]
        .into_iter().map(String::from).collect();
    assert_eq!(layout(&hints), (Position::BeforeFrames, -5));
    let hints = vec!["before:tree", "order:1", "appendix", "order:2"]
        .into_iter().map(String::from).collect();
    assert_eq!(layout(&hints), (Position::Appendix, 2));
    assert_eq!(hints.len(), 4);
}

#[test]
fn positions_order_defaults_and_hints_roundtrip() {
    let mut ws = WorkingStack::default();
    ws.title = Some("测试".into());
    ws.push_root("t", "FRAME_MARKER".into(), vec![], parse_st("r").unwrap()).unwrap();
    ws.notes.insert("bt".into(), note(&["before:tree"], "BEFORE_TREE"));
    ws.notes.insert("at".into(), note(&["after:tree"], "AFTER_TREE"));
    ws.notes.insert("bf".into(), note(&["before:frames"], "BEFORE_FRAME"));
    ws.notes.insert("af".into(), note(&["after:frames"], "AFTER_FRAME"));
    ws.notes.insert("z".into(), note(&["order:-1"], "FIRST_APPENDIX"));
    ws.notes.insert("a".into(), note(&[], "APPENDIX_A"));
    ws.notes.insert("b".into(), note(&["appendix", "after:nothing", "order:bad", "标签（未知）"], "APPENDIX_B"));
    ws.notes.insert("引号\"及换行\n".into(), note(&["x\ny", "标签，[]"], "## 字面标题\n  - 字面列表"));
    let original = ws.clone();
    let md = export_markdown(&ws);
    let at = |s: &str| md.find(s).unwrap();
    assert!(at("BEFORE_TREE") < at("## 栈状态"));
    assert!(at("## 栈状态") < at("AFTER_TREE"));
    assert!(at("AFTER_TREE") < at("BEFORE_FRAME"));
    assert!(at("BEFORE_FRAME") < at("## [1]"));
    assert!(at("## [1]") < at("AFTER_FRAME"));
    assert!(at("AFTER_FRAME") < at("## Appendix"));
    assert!(at("FIRST_APPENDIX") < at("APPENDIX_A"));
    assert!(at("APPENDIX_A") < at("APPENDIX_B"));
    for body in ["BEFORE_TREE", "AFTER_TREE", "BEFORE_FRAME", "AFTER_FRAME", "FIRST_APPENDIX", "APPENDIX_A", "APPENDIX_B"] {
        assert_eq!(md.matches(body).count(), 1);
    }
    assert_eq!(ws, original);
    assert!(!md.contains("### note:"));
    let footer = md.find("## 格式信息").unwrap();
    assert!(footer > at("APPENDIX_B"));
    assert!(!md[..footer].contains("\"meta\":"));
    assert!(md[footer..].contains("> notes: "));
    let parsed = import_md(&md).unwrap();
    assert_eq!(parsed.ws.notes, ws.notes);
    assert_eq!(import_md(&export_markdown(&parsed.ws)).unwrap().ws.notes, ws.notes);
    let json = serde_json::to_string(&ws).unwrap();
    assert_eq!(serde_json::from_str::<WorkingStack>(&json).unwrap(), ws);
    let yaml = serde_yaml::to_string(&ws).unwrap();
    assert_eq!(serde_yaml::from_str::<WorkingStack>(&yaml).unwrap(), ws);
}

#[test]
fn meta_index_resolves_as_literal_string() {
    let mut ws = WorkingStack::default();
    ws.notes.insert("k".into(), note(&["appendix", "{{not-a-reference}}", ""], "正文"));
    assert_eq!(flatten(&ws.resolve(&parse_ref("note:k.meta[1]").unwrap()).unwrap()), "{{not-a-reference}}");
    assert!(ws.resolve(&parse_ref("note:k.meta[3]").unwrap()).is_none());
    assert!(ws.resolve(&parse_ref("note:k.meta[1].body").unwrap()).is_none());
    assert!(ws.resolve(&parse_ref("note:k.meta").unwrap()).is_none());
    assert!(working_stack::validate::validate(&ws).is_empty());
}
