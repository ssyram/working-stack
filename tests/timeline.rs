use working_stack::{model::*, timeline, text::{parse_st, SourcefulText}};

fn event(at: &str, detail: &str) -> Event {
    Event {action:Action::Update,at:at.into(),detail:parse_st(detail).unwrap()}
}
fn sample() -> WorkingStack {
    let mut ws = WorkingStack::default();
    ws.prefix = Some("T".into());
    ws.push_root("old", "根".into(),vec![],parse_st("建根").unwrap()).unwrap();
    ws.push("old",1,"子".into(),vec![],vec![],parse_st("子").unwrap()).unwrap();
    ws.frames.get_mut(&1).unwrap().log.clear();
    let child=ws.frames.get_mut(&2).unwrap();
    child.log.clear();
    child.status=FrameStatus::Done{closure:Closure{summary:parse_st("完成").unwrap(),..Default::default()}};
    ws
}

#[test]
fn sorts_actual_instants_across_offsets_and_fraction_precision() {
    let mut ws=sample();
    ws.frames.get_mut(&1).unwrap().log=vec![
        event("2026-10-07T10:00:00+08:00","早"),
        event("2026-10-07T02:00:00.100000001Z","稍后"),
    ];
    ws.frames.get_mut(&2).unwrap().log=vec![event("2026-10-07T03:00:00Z","晚")];
    ws.log.push(event("2026-10-07T02:00:00.100000000Z","全局"));
    let before=ws.clone();
    let rows=timeline::collect(&ws);
    assert_eq!(rows.iter().map(|r|working_stack::text::flatten(&r.detail)).collect::<Vec<_>>(),vec!["早","全局","稍后","晚"]);
    assert_eq!(rows[1].frame,None);
    assert_eq!(rows[3].frame_id.as_deref(),Some("T-02"));
    assert!(rows.iter().all(|r|r.time_known));
    assert_eq!(ws,before);
}

#[test]
fn equal_times_preserve_owner_order_unknown_times_stay_raw_at_end() {
    let mut ws=sample();
    ws.frames.get_mut(&1).unwrap().log=vec![event("2026-10-07T02:00:00Z","一"),event("2026-10-07T02:00:00Z","二"),event("","无时间")];
    ws.frames.get_mut(&2).unwrap().log=vec![event("2026-10-07","只有日期")];
    let rows=timeline::collect(&ws);
    assert_eq!(rows[0].log_index,0);
    assert_eq!(rows[1].log_index,1);
    assert_eq!(rows[2].at,"");
    assert_eq!(rows[3].at,"2026-10-07");
    assert!(!rows[2].time_known && !rows[3].time_known);
    assert!(timeline::markdown(&ws).contains("时间不完整"));
}

#[test]
fn preserves_references_instead_of_expanding_current_objects_as_history() {
    let mut ws=sample();
    ws.frames.get_mut(&1).unwrap().log=vec![event("2026-10-07T02:00:00Z","依据 {{1.title}}")];
    assert_eq!(timeline::collect(&ws)[0].detail,parse_st("依据 {{1.title}}").unwrap());
    assert!(timeline::markdown(&ws).contains("{{1.title}}"));
    ws.frames.get_mut(&1).unwrap().log[0].detail=SourcefulText::text("第一行\n第二行");
    assert!(timeline::markdown(&ws).contains("\n  第二行"));
}
