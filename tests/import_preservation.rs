use working_stack::{import::import_md, print::export_markdown, text::flatten};

const SOURCE: &str = "# 示例\n\n## 栈状态\n\n```text\n[X-01] [TOP] 工作\n```\n\n## [X-01] [TOP] 工作\n\n- 上帧：无。\n- 弹帧目标：完成。\n- 计划：\n  - [ready] 正常步骤\n  - [resumed · parallel] 未知状态正文\n  - [ready] 模板 {{ skill_dir }} 不应变引用\n- 你确立的原则（已确认）：\n  P1 第一原则｜P2 第二原则\n- 待办（用户自审）：\n  1. 第一项\n  2. 第二项\n### 实战 checklist（共享：完整标题）\n\n| 项目 | 证据 |\n|---|---|\n| 检查 | 已测 |\n\n- 备注 · 原有说明：第一段\n  第二段\n  - 内嵌列表\n\n### 帧变化记录\n\n- [PUSH] [2026-10-07] 建栈\n";

#[test]
fn unknown_content_keeps_frame_ownership_and_roundtrips() {
    let first = import_md(SOURCE).unwrap();
    let f = &first.ws.frames[&1];
    assert_eq!(f.plan.len(), 1);
    assert_eq!(flatten(&f.plan[0].text), "正常步骤");
    assert!(f.content.principles.is_empty());
    let remarks = &f.content.remarks;
    assert!(flatten(&remarks["导入原文 · 计划"]).contains("[resumed · parallel] 未知状态正文"));
    assert!(flatten(&remarks["导入原文 · 计划"]).contains("{{ skill_dir }}"));
    assert!(flatten(&remarks["导入原文 · 你确立的原则（已确认）"]).contains("P1 第一原则｜P2 第二原则"));
    assert!(flatten(&remarks["导入原文 · 待办（用户自审）"]).contains("1. 第一项\n  2. 第二项"));
    assert!(flatten(&remarks["导入原文 · 实战 checklist（共享：完整标题）"]).contains("| 检查 | 已测 |"));
    assert_eq!(flatten(&remarks["原有说明"]), "第一段\n第二段\n- 内嵌列表");
    assert!(first.ws.notes.is_empty());
    let output = export_markdown(&first.ws);
    let second = import_md(&output).unwrap();
    assert_eq!(second.ws.frames[&1].content.remarks, *remarks);
    assert!(working_stack::validate::validate(&second.ws).is_empty());
    let third = import_md(&export_markdown(&second.ws)).unwrap();
    assert_eq!(third.ws.frames[&1].content.remarks, *remarks);
}

#[test]
fn global_unknown_sections_keep_headings_tables_and_literal_templates() {
    let input = format!("前置说明\n{}\n## 全局附录\n\n表格：\n| A | B |\n|---|---|\n| 模板 | {{{{ skill_dir }}}} |\n", SOURCE);
    let first = import_md(&input).unwrap();
    let body = flatten(&first.ws.notes["import-cache"].body);
    assert!(body.contains("前置说明"));
    assert!(body.contains("## 全局附录"));
    assert!(body.contains("| 模板 | {{ skill_dir }} |"));
    let second = import_md(&export_markdown(&first.ws)).unwrap();
    assert_eq!(second.ws.notes["import-cache"], first.ws.notes["import-cache"]);
    assert_eq!(second.ws.frames[&1].content.remarks, first.ws.frames[&1].content.remarks);
    let third = import_md(&export_markdown(&second.ws)).unwrap();
    assert_eq!(third.ws.notes["import-cache"], first.ws.notes["import-cache"]);
}

#[test]
fn failed_rows_keep_their_original_column() {
    let input = SOURCE.replace("- 你确立的原则（已确认）：", "- 确立原则：")
        .replace("- 待办（用户自审）：", "- 已得结果：");
    let parsed = import_md(&input).unwrap();
    let remarks = &parsed.ws.frames[&1].content.remarks;
    assert!(flatten(&remarks["导入原文 · 确立原则"]).contains("P1 第一原则｜P2 第二原则"));
    assert!(flatten(&remarks["导入原文 · 已得结果"]).contains("1. 第一项"));
}
