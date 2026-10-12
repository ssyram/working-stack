use working_stack::{context_view::render, model::*, text::{parse_st, SourcefulText}};
fn t(s: &str) -> SourcefulText { SourcefulText::text(s) }

#[test]
fn intro_and_explicit_sources_restore_material_without_mass_note_or_history_reading() {
    let mut ws = WorkingStack::default();
    ws.push_root("t0", "委托".into(), vec![t("达到用户目标")], t("用户委托")).unwrap();
    ws.push("t1", 1, "早期实验".into(), vec![], vec![], t("原进入原因")).unwrap();
    ws.frames.get_mut(&2).unwrap().content.results.push(t("否定旧方案的必要证据"));
    ws.pop("t2", 2, Outcome::DoneO(Closure { summary:t("实验完成"), ..Default::default() }), None, t("原依据")).unwrap();
    ws.push("t3", 1, "当前工作".into(), vec![], vec![], t("现在进入")).unwrap();
    ws.notes.insert("description".into(), Note {meta:vec![],body:parse_st("本栈只记录交付责任；遵循 {{note:规则.body}}。备查note长材料按需读取。").unwrap()});
    ws.notes.insert("规则".into(), Note {meta:vec![],body:t("结论可信不等于授权实施")});
    ws.notes.insert("长材料".into(), Note {meta:vec![],body:t("不应无条件出现的备查全文")});
    ws.frames.get_mut(&3).unwrap().content.remarks.insert("继续理由".into(), parse_st("新方案保留 {{2.results[0]}} 的限制").unwrap());
    ws.frames.get_mut(&3).unwrap().content.side_effects.push(t("真实回执：作业已派发，尚未返回"));
    let before = serde_json::to_string(&ws).unwrap();
    let context = render(&ws, None);
    assert!(context.find("本栈只记录").unwrap() < context.find("责任路径").unwrap());
    for part in ["结论可信不等于授权实施", "引用来源：note:规则.body", "否定旧方案的必要证据", "引用来源：2.results[0]", "3.side_effects[0]", "尚未返回"] {
        assert!(context.contains(part), "{part}");
    }
    assert!(!context.contains("不应无条件出现的备查全文"));
    assert!(!context.contains("原进入原因"));
    assert!(!context.contains("原依据"));
    assert!(!context.contains("### 帧变化记录"));
    assert_eq!(serde_json::to_string(&ws).unwrap(), before);
    ws.pause("t4", 3, t("待审批")).unwrap();
    let no_focus = render(&ws, None);
    assert!(no_focus.contains("本栈只记录交付责任"));
    assert!(no_focus.contains("焦点：无"));
}
