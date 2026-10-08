//! 结构不变量 I1–I10 的机械检查（detailed-design §4.4）。结构绿 ≠ 语义真（P.global.4）。
//! F02 修复：可达性 = 每帧沿 parent 上行可到根；环 = 上行重访自身路径。

use crate::model::*;

pub fn validate(ws: &WorkingStack) -> Vec<Problem> {
    let mut ps = Vec::new();
    check_tree(ws, &mut ps); // I1
    check_fork_and_fate(ws, &mut ps); // I7 + ReleasedTo 存在性
    check_tips(ws, &mut ps); // I3 / I4
    check_closure(ws, &mut ps); // I9
    check_live_parent(ws, &mut ps); // I10
    check_refs(ws, &mut ps); // I8
    ps
}

/// I1：parent 存在、无环、每帧沿 parent 可达某根。
/// 论证：每帧至多一个 parent ⇒ 上行链唯一；带步数上限的上行重访即环，到达 None 即可达根。
fn check_tree(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    if ws.frames.is_empty() {
        return; // 空栈合法（init 后）
    }
    if ws.roots().is_empty() {
        ps.push("I1: 存在帧但无根（全部帧都有 parent？）".into());
    }
    for (id, f) in &ws.frames {
        if let Some(p) = f.parent {
            if !ws.frames.contains_key(&p) {
                ps.push(format!("I1: {} 的 parent {} 不存在", ws.print_id(*id), ws.print_id(p)));
            }
        }
    }
    for id in ws.frames.keys() {
        let mut cur = *id;
        let mut walked: Vec<Id> = vec![cur];
        let guard = ws.frames.len() + 1;
        while let Some(p) = ws.frames.get(&cur).and_then(|f| f.parent) {
            if !ws.frames.contains_key(&p) {
                break; // 悬挂 parent 已由上面报告
            }
            if walked.contains(&p) {
                ps.push(format!("I1: 父链成环，含 {}", ws.print_id(p)));
                break;
            }
            walked.push(p);
            cur = p;
            if walked.len() > guard {
                ps.push(format!("I1: 父链异常长，含 {}", ws.print_id(*id)));
                break;
            }
        }
        // 走到 parent=None 即到达根；未到达且无环/悬挂报告 ⇒ 不可达（理论上由环覆盖）
        if ws.frames.get(&cur).and_then(|f| f.parent).is_none() {
            continue;
        }
    }
}

/// I7：fork 目标存在且 parent == 本帧；ReleasedTo 目标存在。
fn check_fork_and_fate(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    for (id, f) in &ws.frames {
        let pid = ws.print_id(*id);
        for it in walk_plan(&f.plan) {
            for fid in &it.fork {
                match ws.frames.get(fid) {
                    None => ps.push(format!("I7: {pid} 计划项 fork 指向不存在的 {}", ws.print_id(*fid))),
                    Some(t) if t.parent != Some(*id) => {
                        ps.push(format!("I7: {pid} 计划项 fork 的 {} parent 不是本帧", ws.print_id(*fid)))
                    }
                    _ => {}
                }
            }
        }
        for p in &f.content.principles {
            if let Some(Fate::ReleasedTo { id: to }) = p.fate {
                if !ws.frames.contains_key(&to) {
                    ps.push(format!("I7: {pid} 原则 released → {} 不存在", ws.print_id(to)));
                }
            }
        }
    }
}

/// I3：Active tips ≤ 1；I4：Paused 必为 tip（无 live 子）且理由非空。
fn check_tips(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    let tips: std::collections::HashSet<Id> = ws.tips().into_iter().collect();
    let active_tips: Vec<Id> = tips
        .iter()
        .copied()
        .filter(|t| matches!(ws.frames[t].status, FrameStatus::Active))
        .collect();
    if active_tips.len() > 1 {
        let names: Vec<String> = active_tips.iter().map(|t| ws.print_id(*t)).collect();
        ps.push(format!("I3: 多个 Active tip（{}），焦点不唯一", names.join("、")));
    }
    for (id, f) in &ws.frames {
        if let FrameStatus::Paused { reason } = &f.status {
            if !tips.contains(id) {
                ps.push(format!("I4: {} 为 Paused 但不是 tip（存在 live 子帧）", ws.print_id(*id)));
            }
            if reason.is_empty() {
                ps.push(format!("I4: {} Paused 但理由为空", ws.print_id(*id)));
            }
        }
    }
}

/// I9：已关闭帧 principles.fate 全 Some；Done ⇒ residuals 空。
///（F15 修复：不再禁止关闭帧保留"暂停原因"备注——蓝本无此禁令。）
fn check_closure(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    for (id, f) in &ws.frames {
        let pid = ws.print_id(*id);
        if f.status.is_closed() {
            let unsettled = f.content.principles.iter().any(|p| p.fate.is_none());
            if unsettled {
                ps.push(format!("I9: 已关闭帧 {pid} 存在未结算原则"));
            }
            if let FrameStatus::Done { closure } = &f.status {
                if !closure.residuals.is_empty() {
                    ps.push(format!("I9: Done 帧 {pid} 存在 residuals（应为空）"));
                }
            }
        }
    }
}

/// I10：live 帧的 parent 必为 live（或根）。
fn check_live_parent(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    for id in ws.live_ids() {
        if let Some(p) = ws.frames[&id].parent {
            if !ws.is_live(p) {
                ps.push(format!(
                    "I10: live 帧 {} 的 parent {} 已关闭（先处置子帧或改挂）",
                    ws.print_id(id),
                    ws.print_id(p)
                ));
            }
        }
    }
}

/// I8：全部引用可解析（含状态负载——经 view::walk_all_st 全覆盖，F03）；引用图无环。
fn check_refs(ws: &WorkingStack, ps: &mut Vec<Problem>) {
    for (loc, r) in ws.all_references() {
        if ws.resolve(&r).is_none() {
            ps.push(format!("I8: {} 的引用 {{{}}} 未解析", loc, crate::text::render_ref(&r)));
        }
    }
    for cyc in ws.ref_cycles() {
            ps.push(format!("I8: 引用环：{cyc}"));
    }
}

pub fn walk_plan(plan: &[PlanItem]) -> impl Iterator<Item = &PlanItem> {
    let mut out = Vec::new();
    fn rec<'a>(plan: &'a [PlanItem], out: &mut Vec<&'a PlanItem>) {
        for it in plan {
            out.push(it);
            rec(&it.children, out);
        }
    }
    rec(plan, &mut out);
    out.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::parse_st;

    fn f(status: FrameStatus, parent: Option<Id>) -> Frame {
        Frame {
            parent,
            title: String::new(),
            goal: vec![],
            origin: vec![],
            status,
            plan: vec![],
            content: Content::default(),
            log: vec![],
        }
    }

    #[test]
    fn normal_parent_child_passes() {
        // F02 反例：最普通的父子栈必须通过
        let mut ws = WorkingStack::default();
        ws.frames.insert(1, f(FrameStatus::Active, None));
        ws.frames.insert(2, f(FrameStatus::Active, Some(1)));
        assert!(validate(&ws).is_empty(), "{:?}", validate(&ws));
    }

    #[test]
    fn catches_two_active_tips() {
        let mut ws = WorkingStack::default();
        ws.frames.insert(1, f(FrameStatus::Active, None));
        ws.frames.insert(2, f(FrameStatus::Active, Some(1)));
        ws.frames.insert(3, f(FrameStatus::Active, Some(1)));
        let ps = validate(&ws);
        assert!(ps.iter().any(|p| p.starts_with("I3:")));
    }

    #[test]
    fn catches_cycle_and_dangling() {
        let mut ws = WorkingStack::default();
        let mut a = f(FrameStatus::Active, None);
        a.parent = Some(2); // 1→2→1
        let mut b = f(FrameStatus::Active, Some(1));
        b.goal = vec![parse_st("见 {{7.title}}").unwrap()];
        ws.frames.insert(1, a);
        ws.frames.insert(2, b);
        let ps = validate(&ws);
        assert!(ps.iter().any(|p| p.starts_with("I1:") && p.contains("环")), "{ps:?}");
        assert!(ps.iter().any(|p| p.starts_with("I8:")));
    }

    #[test]
    fn catches_i10_and_i9() {
        let mut ws = WorkingStack::default();
        ws.frames.insert(1, f(FrameStatus::Done { closure: Closure::default() }, None));
        ws.frames.insert(2, f(FrameStatus::Active, Some(1))); // I10：父已关
        let p1 = ws.frames.get_mut(&1).unwrap();
        p1.content
            .principles
            .push(Principle { rule: parse_st("R").unwrap(), fate: None }); // I9 未结算
        let ps = validate(&ws);
        assert!(ps.iter().any(|p| p.starts_with("I9:")));
        assert!(ps.iter().any(|p| p.starts_with("I10:")));
    }

    #[test]
    fn catches_ref_in_paused_reason() {
        // F03 反例：Paused.reason 内的悬挂引用必须被 I8 报告
        let mut ws = WorkingStack::default();
        ws.frames.insert(
            1,
            f(FrameStatus::Paused { reason: parse_st("等 {{99.goal}}").unwrap() }, None),
        );
        let ps = validate(&ws);
        assert!(ps.iter().any(|p| p.starts_with("I8:")), "{ps:?}");
    }
}
