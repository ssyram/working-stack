use working_stack::{model::*, text::{parse_ref,parse_st,flatten}, commit_log};

fn sample() -> WorkingStack {
    let mut ws=WorkingStack::default();
    ws.push_root("t0","根".into(),vec![],parse_st("建根").unwrap()).unwrap();
    ws.push("t0",1,"子".into(),vec![],vec![],parse_st("下钻").unwrap()).unwrap();
    ws.principle_add("t0",2,parse_st("规则A").unwrap(),parse_st("规则").unwrap()).unwrap();
    ws.principle_add("t0",2,parse_st("规则B").unwrap(),parse_st("规则").unwrap()).unwrap();
    ws
}

#[test]
fn indexed_principles_and_aggregate_projection_both_resolve() {
    let ws=sample();
    assert_eq!(flatten(&ws.resolve(&parse_ref("2.principles[0]").unwrap()).unwrap()),"规则A");
    assert_eq!(flatten(&ws.resolve(&parse_ref("2.principles").unwrap()).unwrap()),"规则A\n\n规则B");
    assert!(ws.resolve(&parse_ref("2.principles[9]").unwrap()).is_none());
}

#[test]
fn release_and_reclassification_materialized_references_validate() {
    for fate in [Fate::ReleasedTo{id:1},Fate::Reclassified{target:ReclassTarget::Results},
        Fate::Reclassified{target:ReclassTarget::Remark{title:"事实".into()}}] {
        let mut ws=sample();
        ws.settle("t1",2,0,fate,parse_st("处置").unwrap()).unwrap();
        assert!(working_stack::validate::validate(&ws).is_empty());
        assert_eq!(ws.render(&parse_st("{{2.principles[0]}}").unwrap()),"规则A");
    }
}

#[test]
fn command_update_coalesces_only_fresh_events_and_adds_fields() {
    let mut ws=sample();let old=ws.clone();
    let reason=parse_st("补齐步骤").unwrap();
    ws.plan_add("t1",2,parse_st("一步").unwrap(),ItemState::Ready,reason.clone()).unwrap();
    ws.plan_add("t1",2,parse_st("二步").unwrap(),ItemState::Ready,reason.clone()).unwrap();
    ws.set_title("t1",2,"新标题".into(),reason).unwrap();
    commit_log::coalesce_update(&mut ws,&old,"t1");
    let prefix=old.frames[&2].log.len();
    assert_eq!(&ws.frames[&2].log[..prefix],old.frames[&2].log.as_slice());
    assert_eq!(ws.frames[&2].log.len(),prefix+1);
    let detail=flatten(&ws.frames[&2].log[prefix].detail);
    assert!(detail.starts_with("补齐步骤"));
    assert!(detail.contains("plan") && detail.contains("title"));
    assert!(WorkingStack::apply_checks(&old,&ws).is_empty());
}

#[test]
fn net_zero_updates_have_no_committed_event() {
    let mut ws=sample();
    ws.plan_add("t0",2,parse_st("一步").unwrap(),ItemState::Ready,parse_st("加").unwrap()).unwrap();
    let old=ws.clone();
    ws.plan_set("t1",2,&[0],ItemState::InProgress,parse_st("变更").unwrap()).unwrap();
    ws.plan_set("t1",2,&[0],ItemState::Ready,parse_st("变更").unwrap()).unwrap();
    commit_log::coalesce_update(&mut ws,&old,"t1");
    assert_eq!(ws,old);
}

#[test]
fn correct_event_keeps_original_detail_and_state_change_separate() {
    let mut ws=sample();
    ws.frames.get_mut(&2).unwrap().status=FrameStatus::Done{closure:Closure{summary:parse_st("误关结论").unwrap(),..Default::default()}};
    let old=ws.clone();
    ws.set_title("t1",2,"更正标题".into(),parse_st("修正").unwrap()).unwrap();
    ws.reopen("t1",2,None,parse_st("撤销误关").unwrap()).unwrap();
    let correct=ws.frames[&2].log.last().unwrap().clone();
    commit_log::coalesce_update(&mut ws,&old,"t1");
    assert_eq!(ws.frames[&2].log.last().unwrap(),&correct);
    assert!(flatten(&correct.detail).contains("误关结论"));
    assert!(!flatten(&ws.frames[&2].log[old.frames[&2].log.len()].detail).contains("status"));
}
