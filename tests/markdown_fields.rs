use working_stack::{import::import_md, print::export_markdown, model::*, text::{parse_st, SourcefulText}};

fn sample() -> WorkingStack {
    let mut ws = WorkingStack::default();
    ws.title = Some("结构往返".into());
    ws.push_root("t", "根".into(), vec![], parse_st("建根").unwrap()).unwrap();
    ws.push("t", 1, "子".into(), vec![], vec![], parse_st("下钻").unwrap()).unwrap();
    ws.frames.get_mut(&2).unwrap().status = FrameStatus::Done {
        closure: Closure { summary: parse_st("完成").unwrap(), ..Default::default() },
    };
    ws
}

fn roundtrips(ws: &WorkingStack) {
    assert!(working_stack::validate::validate(ws).is_empty());
    let mut current = ws.clone();
    for _ in 0..3 {
        let parsed = import_md(&export_markdown(&current)).unwrap();
        assert_eq!(parsed.ws, *ws);
        assert!(working_stack::validate::validate(&parsed.ws).is_empty());
        current = parsed.ws;
    }
}

#[test]
fn principle_rule_and_every_fate_remain_separate() {
    let mut ws = sample();
    ws.frames.get_mut(&1).unwrap().content.principles.push(Principle {
        rule: SourcefulText::default(), fate: None,
    });
    let fates = vec![
        Fate::KeptLocal,
        Fate::ReleasedTo { id: 1 },
        Fate::Invalidated { reason: parse_st("前提 {{1.title}} 已变\n不能沿用").unwrap() },
        Fate::Reclassified { target: ReclassTarget::Results },
        Fate::Reclassified { target: ReclassTarget::Remark { title: "新备注（标题）".into() } },
    ];
    let child = ws.frames.get_mut(&2).unwrap();
    for fate in fates {
        child.content.principles.push(Principle {
            rule: parse_st("正文自己的（kept-local）不应剥掉\n引用 {{1.title}}").unwrap(),
            fate: Some(fate),
        });
    }
    child.content.principles.push(Principle {
        rule: SourcefulText::text("模板 {{ skill_dir }}\n真实括号（invalidated：文本）"),
        fate: Some(Fate::KeptLocal),
    });
    roundtrips(&ws);
}

#[test]
fn all_closure_directions_keep_per_item_evidence_and_result() {
    let mut ws = sample();
    let items = vec![
        ClosureItem {
            result: parse_st("结果自己的（依据：不是独立字段）\n引用 {{1.title}}").unwrap(),
            evidence: parse_st("证据甲\n来源 {{1.title}}").unwrap(),
        },
        ClosureItem { result: SourcefulText::default(), evidence: SourcefulText::text("模板 {{ unknown }}；证据乙") },
        ClosureItem { result: parse_st("结果丙").unwrap(), evidence: SourcefulText::default() },
    ];
    let closure = Closure {
        summary: parse_st("回合停止，结果保留").unwrap(),
        positive: items.clone(), negative: items.clone(), tangential: items.clone(), residuals: items,
    };
    ws.frames.get_mut(&2).unwrap().status = FrameStatus::Expired { closure: closure.clone() };
    roundtrips(&ws);
    let done = Closure { residuals: vec![], ..closure };
    ws.frames.get_mut(&2).unwrap().status = FrameStatus::Done { closure: done };
    roundtrips(&ws);
}

#[test]
fn legacy_missing_fate_does_not_overwrite_explicit_fate() {
    let mut ws = sample();
    ws.frames.get_mut(&2).unwrap().content.principles = vec![
        Principle { rule: parse_st("第一条").unwrap(), fate: Some(Fate::KeptLocal) },
        Principle { rule: parse_st("第二条").unwrap(), fate: Some(Fate::ReleasedTo { id: 1 }) },
    ];
    let md = export_markdown(&ws).replacen("    - 处置：{\"kind\":\"kept_local\"}\n", "", 1);
    let parsed = import_md(&md).unwrap();
    assert_eq!(parsed.ws.frames[&2].content.principles, ws.frames[&2].content.principles);
}
