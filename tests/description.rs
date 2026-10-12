use working_stack::{import::import_md, model::*, print::export_markdown, text::{parse_st, flatten}};

#[test]
fn intro_preserves_empty_whitespace_headers_refs_and_original_hints() {
    for body in ["", "  \n\t\n", "\n 导览 \n\n", "## 不是栈栏目\n### 不是note标题\n末行", "参见 {{note:规则.body}}，字面 \\{{模板}}"] {
        let mut ws = WorkingStack::default();
        ws.title = Some("导览".into());
        ws.notes.insert("description".into(), Note {meta:vec!["after:frames".into(), "未知提示".into()], body:parse_st(body).unwrap()});
        ws.notes.insert("规则".into(), Note {meta:vec![],body:parse_st("仅限当前范围").unwrap()});
        for _ in 0..3 {
            let md = export_markdown(&ws);
            assert!(!md.contains("### 工作线说明"));
            assert!(md.contains("\"at_title\":true"));
            let before = ws.clone();
            ws = import_md(&md).unwrap().ws;
            assert_eq!(ws, before);
        }
    }
}

#[test]
fn legacy_section_description_uses_footer_and_is_not_consumed_as_empty_intro() {
    let md = "# 旧输出\n\n## 栈状态\n\n空\n\n## 共享内容\n\n### 工作线说明\n\n  旧导览\n  \n\n## 格式信息\n\n> notes: [{\"key\":\"description\",\"meta\":[\"before:tree\",\"标签\"]}]\n";
    let ws = import_md(md).unwrap().ws;
    assert_eq!(flatten(&ws.notes["description"].body), "旧导览\n");
    assert_eq!(ws.notes["description"].meta, vec!["before:tree", "标签"]);
    let new_md = export_markdown(&ws);
    assert!(new_md.starts_with("# 旧输出\n\n  旧导览\n"));
    assert_eq!(import_md(&new_md).unwrap().ws, ws);
}

#[test]
fn ordinary_heading_prose_still_imports_and_absence_does_not_create_note() {
    let ws = import_md("# 手写\n\n这是导览。\n\n## 栈状态\n\n空\n").unwrap().ws;
    assert_eq!(flatten(&ws.notes["description"].body), "这是导览。");
    assert_eq!(ws.notes["description"].meta, vec!["before:tree"]);
    let empty = import_md("# 无导览\n\n## 栈状态\n\n空\n").unwrap().ws;
    assert!(!empty.notes.contains_key("description"));
    assert_eq!(import_md(&export_markdown(&empty)).unwrap().ws, empty);
}
