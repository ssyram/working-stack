use working_stack::{context_view::render, model::*, text::SourcefulText};
fn t(value: &str) -> SourcefulText { SourcefulText::text(value) }

fn sample() -> WorkingStack {
    let mut ws = WorkingStack::default();
    ws.title = Some("方向".into()); ws.prefix = Some("T".into());
    let root = ws.push_root("t0", "根".into(), vec![t("产物可用"), t("用户验收")], t("委托")).unwrap();
    ws.plan_add("t1", root, t("等用户决定"), ItemState::Ready, t("计划")).unwrap();
    ws.frames.get_mut(&root).unwrap().content.remarks.insert("审批边界".into(), t("未批准不得执行"));
    let side = ws.push("t2", root, "旁支".into(), vec![t("独立问题")], vec![], t("调查")).unwrap();
    ws.pause("t3", side, t("等回执")).unwrap();
    let focus = ws.push("t4", root, "实施".into(), vec![t("直接测试")], vec![t("共通部分已获授权")], t("进入")).unwrap();
    ws.plan_add("t5", focus, t("准备输入"), ItemState::Ready, t("计划")).unwrap();
    ws.plan_add_child("t6", focus, &[0], t("具体下一步"), t("细化")).unwrap();
    ws.plan_set("t7", focus, &[0, 0], ItemState::Blocked {reason:t("等真实输入")}, t("等待")).unwrap();
    ws.frames.get_mut(&focus).unwrap().content.results = (0..5).map(|i|t(&format!("收件{i}未确认"))).collect();
    ws
}

#[test]
fn context_links_all_goals_exact_paths_and_existing_boundaries_without_history_dump() {
    let ws = sample(); let before = serde_json::to_string(&ws).unwrap();
    let out = render(&ws, None);
    for part in ["T-01 → T-03", "用户验收", "未批准不得执行", "3.plan[0].children[0]",
                 "计划 [blocked]", "3.plan[0].children[0].progress.reason", "等真实输入", "FOCUS：具体下一步", "3.results[4]", "等回执",
                 "可能不含最新对话裁决", "事件时间不等于事实观察时间"] { assert!(out.contains(part), "{part}"); }
    assert!(out.contains("收件0未确认"));
    assert!(!out.contains("省略前"));
    assert!(!out.contains("[PUSH]"));
    assert_eq!(serde_json::to_string(&ws).unwrap(), before);
}

#[test]
fn no_focus_keeps_roots_and_waits_explicit_selection_does_not_resume() {
    let mut ws = sample(); ws.pause("t8", 3, t("等批准")).unwrap();
    let out = render(&ws, None);
    assert!(out.contains("焦点：无")); assert!(out.contains("用户验收"));
    assert!(out.contains("等回执")); assert!(out.contains("等批准"));
    let chosen = render(&ws, Some(3));
    assert!(chosen.contains("所选分支（不切换焦点）"));
    assert!(chosen.contains("3.status.reason"));
    assert_eq!(ws.focus(), None);
}

#[test]
fn long_boundaries_are_complete_and_closed_selection_is_a_snapshot() {
    let mut ws = sample();
    let boundary = format!("{}；仍须用户批准", "长".repeat(300));
    ws.frames.get_mut(&3).unwrap().content.remarks.insert("长内容".into(), t(&boundary));
    let context = render(&ws, None);
    assert!(context.contains(&boundary));
    assert!(!context.contains("截断"));
    assert!(!context.contains("\n下一步："));
    ws.pop("t9", 3, Outcome::DoneO(Closure {summary:t("完成子目标"), positive:vec![], negative:vec![], tangential:vec![], residuals:vec![]}), None, t("返回")).unwrap();
    let out = render(&ws, Some(3));
    assert!(out.contains("[done]")); assert!(out.contains("FOCUS：未指定"));
    assert!(out.contains("3.status.closure.summary"));
}
