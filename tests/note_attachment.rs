use working_stack::{import::import_md, print::export_markdown, model::*, text::parse_st};

#[test]
fn footer_preserves_raw_keys_hints_whitespace_and_readable_body() {
    let mut ws=WorkingStack::default();
    ws.title=Some("干净输出".into());
    ws.notes.insert("description".into(),Note{meta:vec!["before:tree".into(),"unknown".into()],body:parse_st(" \n说明\n").unwrap()});
    ws.notes.insert("import-cache".into(),Note{meta:vec!["appendix".into()],body:parse_st("## 原小节\n正文").unwrap()});
    let md=export_markdown(&ws);
    assert!(md.contains("### 工作线说明\n"));
    assert!(md.contains("### 补充内容\n"));
    assert!(!md.contains("### note:"));
    let boundary=md.find("## 格式信息").unwrap();
    assert!(!md[..boundary].contains("before:tree"));
    assert!(md[boundary..].contains("> notes: "));
    assert_eq!(import_md(&md).unwrap().ws,ws);
}

#[test]
fn reordered_readable_sections_match_their_metadata_by_title() {
    let mut ws=WorkingStack::default();ws.title=Some("重排".into());
    ws.notes.insert("a".into(),Note{meta:vec!["appendix".into(),"order:1".into()],body:parse_st("A").unwrap()});
    ws.notes.insert("b".into(),Note{meta:vec!["appendix".into(),"order:2".into()],body:parse_st("B").unwrap()});
    let md=export_markdown(&ws).replace("### a\n\n  A\n\n### b\n\n  B\n\n","### b\n\n  B\n\n### a\n\n  A\n\n");
    assert_eq!(import_md(&md).unwrap().ws.notes,ws.notes);
}

#[test]
fn prior_machine_note_headers_still_parse() {
    let md="# 旧格式\n\n## 栈状态\n\n空\n\n## Appendix\n\n### note:{\"key\":\"k\",\"meta\":[\"unknown\"]}\n\n  内容\n";
    let ws=import_md(md).unwrap().ws;
    assert_eq!(ws.notes["k"].meta,vec!["unknown"]);
    assert_eq!(working_stack::text::flatten(&ws.notes["k"].body),"内容");
}
