//! md 家族解析器（WS-TOOL / WS-MAIN 家族；detailed-design §4.9）。
//! fail-loud：不认识的形状进 warnings，不静默丢；FOCUS 粗体 = 显示层 → 导入为普通状态项。

use crate::model::*;
use crate::text::{parse_st, Segment, SourcefulText};

pub struct ImportOut {
    pub ws: WorkingStack,
    pub warnings: Vec<String>,
}

struct TreeLine {
    indent: usize,
    dashed: bool,
    id: Id,
    marker: String,
}

pub fn import_md(md: &str) -> Result<ImportOut, WsError> {
    let mut warn: Vec<String> = Vec::new();
    let mut ws = WorkingStack::default();
    let mut cache_lines: Vec<String> = Vec::new();
    let mut note_headers = crate::note_markdown::attachments(md).map_err(WsError::Parse)?;

    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0usize;

    // 标题 + 描述
    while i < lines.len() && !lines[i].starts_with("# ") {
        if !lines[i].trim().is_empty() {
            warn.push(format!("标题前非空内容保留到全局备注：{:?}", lines[i]));
            cache_lines.push(lines[i].to_string());
        }
        i += 1;
    }
    if i >= lines.len() {
        return Err(WsError::Parse("未找到 # 标题行".into()));
    }
    ws.title = Some(lines[i][2..].trim().to_string());
    i += 1;
    let mut desc: Vec<&str> = Vec::new();
    while i < lines.len() && !lines[i].starts_with("## ") {
        desc.push(lines[i]);
        i += 1;
    }
    let desc_txt: String = desc.join("\n").trim().to_string();
    if !desc_txt.is_empty() {
        ws.notes.insert(
            "description".into(),
            Note { meta: vec!["before:tree".into()], body: parse_st(&desc_txt).map_err(WsError::Parse)? },
        );
    }

    // 分节扫描
    let mut tree: Vec<TreeLine> = Vec::new();
    let mut popped_index: Vec<(Id, String, Option<Id>)> = Vec::new();
    let mut sections: Vec<(Id, String, String, Vec<&str>)> = Vec::new();

    while i < lines.len() {
        let l = lines[i];
        if l.starts_with("## ") {
            let head = l[3..].trim().to_string();
            let mut body: Vec<&str> = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("## ") {
                body.push(lines[i]);
                i += 1;
            }
            if head == "栈状态" {
                for bl in &body {
                    let t = bl.trim();
                    if t.starts_with("```") || t.is_empty() || t == "空" {
                        continue;
                    }
                    match parse_tree_line(bl) {
                        Some(tl) => tree.push(tl),
                        None => {
                            warn.push(format!("栈状态行未解析：{t:?}"));
                            cache_lines.push((*bl).to_string());
                        },
                    }
                }
            } else if head == "共享内容" || head == "Appendix" {
                crate::note_markdown::read_section(&body, &mut ws.notes, &mut note_headers).map_err(WsError::Parse)?;
            } else if head == "格式信息" && body.iter().any(|line|line.starts_with("> notes: ")) {
            } else if head == "栈变化记录" {
                for bl in &body {
                    if let Some(rest) = bl.trim().strip_prefix("- [") {
                        if let Some(e) = parse_event(rest) {
                            ws.log.push(e);
                        }
                    }
                }
            } else if head == "已 pop 节点" {
                for bl in &body {
                    let t = bl.trim();
                    if let Some(rest) = t.strip_prefix("- [") {
                        if let Some(pi) = parse_popped_index(rest) {
                            popped_index.push(pi);
                        }
                    }
                }
            } else if head == "待归属缓存" {
                for bl in &body {
                    let display_key = bl.trim().strip_prefix("- ").and_then(|s| s.strip_suffix("（全文见共享内容节）"));
                    if display_key.is_some_and(|k| ws.notes.contains_key(k)) {
                        continue;
                    }
                    if !bl.trim().is_empty() {
                        cache_lines.push((*bl).to_string());
                    }
                }
            } else if head.starts_with('[') {
                let (id, marker, title) = parse_frame_head(&head);
                sections.push((id, marker, title, body));
            } else {
                warn.push(format!("未知小节 ## {head}（内容移入待归属缓存）"));
                cache_lines.push(format!("## {head}"));
                cache_lines.extend(body.iter().map(|s| (*s).to_string()));
            }
        } else {
            i += 1;
        }
    }

    // 树 → parent（dash 行父=缩进严格更小的最近帧，退化取最近非 dash 链行）
    let mut parents: Vec<(Id, Option<Id>)> = Vec::new();
    for (idx, tl) in tree.iter().enumerate() {
        let parent = if !tl.dashed {
            if idx == 0 { None } else { Some(tree[idx - 1].id) }
        } else {
            let mut cand = None;
            for prev in tree[..idx].iter().rev() {
                if prev.indent < tl.indent {
                    cand = Some(prev.id);
                    break;
                }
            }
            cand.or_else(|| tree[..idx].iter().rev().find(|p| !p.dashed).map(|p| p.id))
        };
        parents.push((tl.id, parent));
    }
    let tree_ids: Vec<Id> = tree.iter().map(|t| t.id).collect();

    // 帧节解析
    for (id, marker, title, body) in &sections {
        if ws.frames.contains_key(id) {
            warn.push(format!("帧 id {} 重复，后者覆盖前者", id));
        }
        let mut parsed = parse_frame_body(body, title, &mut warn).map_err(WsError::Parse)?;
        let tree_pos = tree_ids.iter().position(|x| x == id);
        if parsed.parent.is_none() {
            parsed.parent = tree_pos.and_then(|p| parents.get(p).and_then(|(_, pa)| *pa));
        }
        if let Some(tp) = tree_pos {
            if tree[tp].marker.contains("digression")
                && !parsed.origin.iter().any(|o| crate::text::flatten(o).contains("[digression]"))
            {
                parsed.origin.insert(0, parse_st("[digression]").unwrap());
            }
        }
        let status = status_from_marker(marker, &parsed, &mut warn, *id);
        ws.frames.insert(
            *id,
            Frame {
                parent: parsed.parent,
                title: parsed.title,
                goal: parsed.goal,
                origin: parsed.origin,
                status,
                plan: parsed.plan,
                content: parsed.content,
                log: parsed.log,
            },
        );
    }

    // 仅见于已 pop 索引的帧 → 兜底
    for (id, title, parent) in &popped_index {
        if ws.frames.contains_key(id) {
            continue;
        }
        warn.push(format!("帧 {} 仅见于已 pop 节点索引，按 Expired 建兜底帧", id));
        ws.frames.insert(
            *id,
            Frame {
                parent: *parent,
                title: title.clone(),
                goal: vec![],
                origin: vec![],
                status: FrameStatus::Expired {
                    closure: Closure { summary: parse_st("（仅见于索引，导入兜底）").unwrap(), ..Default::default() },
                },
                plan: vec![],
                content: Content::default(),
                log: vec![],
            },
        );
    }

    if let Some(p) = detect_prefix(md) {
        ws.prefix = Some(p);
    }

    // 已关闭帧原则兜底结算（I9）
    for (id, f) in ws.frames.iter_mut() {
        if f.status.is_closed() && f.content.principles.iter().any(|p| p.fate.is_none()) {
            for p in f.content.principles.iter_mut() {
                if p.fate.is_none() { p.fate = Some(Fate::KeptLocal); }
            }
            warn.push(format!("已关闭帧 {} 的原则机械默认 kept-local", id));
        }
    }

    // 焦点唯一化：多 Active tip → 保留树 TOP 标记者（否则最后一个），其余 Paused
    let active_tips: Vec<Id> = ws
        .tips()
        .into_iter()
        .filter(|t| matches!(ws.frames[t].status, FrameStatus::Active))
        .collect();
    if active_tips.len() > 1 {
        let keep = tree
            .iter()
            .find(|t| t.marker.starts_with("TOP"))
            .map(|t| t.id)
            .unwrap_or(*active_tips.last().unwrap());
        for t in active_tips {
            if t != keep {
                if let Some(f) = ws.frames.get_mut(&t) {
                    f.status = FrameStatus::Paused { reason: parse_st("（导入：多 Active tip 归一为非焦点）").unwrap() };
                }
                warn.push(format!("帧 {} 多 Active tip，归一为 Paused", t));
            }
        }
    }

    if !cache_lines.is_empty() {
        let note = ws.notes.entry("import-cache".into()).or_insert_with(|| Note {
            meta: vec!["appendix".into()], body: SourcefulText::default(),
        });
        if !note.body.is_empty() { note.body.push(Segment::Text("\n".into())); }
        note.body.extend(literal_text(&cache_lines.join("\n")).iter().cloned());
    }
    crate::note_markdown::metadata_complete(&note_headers).map_err(WsError::Parse)?;
    Ok(ImportOut { ws, warnings: warn })
}

fn detect_prefix(md: &str) -> Option<String> {
    for l in md.lines() {
        let t = l.trim_start();
        let t = t.strip_prefix("- ").unwrap_or(t);
        if let Some(rest) = t.strip_prefix('[') {
            let id_txt = rest.split(']').next()?;
            let (p, n) = split_id(id_txt.trim());
            if p.is_some() && n > 0 {
                return p;
            }
        }
    }
    None
}

/// `T-01` → (Some("T"), 1)；`12` → (None, 12)。
fn split_id(s: &str) -> (Option<String>, Id) {
    match s.rsplit_once('-') {
        Some((p, n)) => (Some(p.to_string()), n.parse().unwrap_or(0)),
        None => (None, s.parse().unwrap_or(0)),
    }
}

fn parse_frame_head(head: &str) -> (Id, String, String) {
    let rest = head.strip_prefix('[').unwrap_or(head);
    let (id_txt, rest) = rest.split_once(']').unwrap_or((rest, ""));
    let id = split_id(id_txt.trim()).1;
    let rest = rest.trim();
    if let Some(r) = rest.strip_prefix('[') {
        match r.split_once(']') {
            Some((m, t)) => (id, m.to_string(), t.trim().to_string()),
            None => (id, String::new(), rest.to_string()),
        }
    } else {
        (id, String::new(), rest.to_string())
    }
}

fn parse_tree_line(l: &str) -> Option<TreeLine> {
    let indent = l.len() - l.trim_start().len();
    let t = l.trim_start();
    let dashed = t.starts_with("- ");
    let t = if dashed { &t[2..] } else { t };
    let rest = t.strip_prefix('[')?;
    let (id_txt, rest) = rest.split_once(']')?;
    let id = split_id(id_txt.trim()).1;
    let rest = rest.trim();
    let marker = if let Some(r) = rest.strip_prefix('[') {
        r.split_once(']').map(|(m, _)| m.to_string()).unwrap_or_default()
    } else {
        String::new()
    };
    Some(TreeLine { indent, dashed, id, marker })
}

fn parse_popped_index(rest: &str) -> Option<(Id, String, Option<Id>)> {
    let (id_txt, rest) = rest.split_once(']')?;
    let id = split_id(id_txt.trim()).1;
    let rest = rest.trim();
    let title_part = rest.strip_prefix("[POPPED]").unwrap_or(rest).trim();
    let (title, parent) = match title_part.split_once("；上帧：") {
        Some((t, p)) => {
            let p = p.trim().trim_end_matches('。').trim();
            let pid = p
                .strip_prefix('[')
                .and_then(|x| x.split_once(']'))
                .map(|(a, _)| split_id(a.trim()).1);
            (t.trim().to_string(), pid)
        }
        None => (title_part.trim_end_matches('。').to_string(), None),
    };
    Some((id, title, parent))
}

struct ParsedFrame {
    title: String,
    parent: Option<Id>,
    goal: Vec<SourcefulText>,
    origin: Vec<SourcefulText>,
    plan: Vec<PlanItem>,
    content: Content,
    log: Vec<Event>,
    pause_reason: Option<SourcefulText>,
    closure: Option<(bool, Closure)>, // (is_done, closure)
}

type W = Vec<String>;

fn parse_frame_body(body: &[&str], head_title: &str, warn: &mut W) -> Result<ParsedFrame, String> {
    let mut p = ParsedFrame {
        title: head_title.to_string(),
        parent: None,
        goal: vec![],
        origin: vec![],
        plan: vec![],
        content: Content::default(),
        log: vec![],
        pause_reason: None,
        closure: None,
    };
    let mut in_log = false;
    let mut field: Option<&str> = None;
    let mut remark_title = String::new();
    let mut buf: Vec<String> = Vec::new();
    let mut closure_kind_done: Option<bool> = None;
    let mut closure_summary = String::new();
    let mut correspond: Vec<String> = vec![];
    let mut evidence: Vec<String> = vec![];
    let mut canonical_principle: Option<usize> = None;
    let mut closure_rows = crate::closure_markdown::Rows::default();

    macro_rules! flush {
        () => {{
            if let Some(f) = field {
                let joined = buf.join("\n");
                let txt = if matches!(f, "备注" | "原文") {
                    joined.trim_matches('\n').to_string()
                } else {
                    joined.trim().to_string()
                };
                if !txt.is_empty() {
                    if matches!(f, "备注" | "原文") {
                        let st = if f == "原文" { literal_text(&txt) } else { parse_st(&txt)? };
                        append_remark(&mut p.content, &remark_title, &st);
                    } else {
                        let st = parse_st(&txt)?;
                        match f {
                            "来源" => p.origin.push(st),
                            "弹帧目标" => p.goal.push(st),
                            "暂停原因" => p.pause_reason = Some(st),
                            _ => {}
                        }
                    }
                }
            }
            buf.clear();
            #[allow(unused_assignments)]
            {
                field = None;
            }
        }};
    }

    for line in body {
        let l = line.trim_end();
        if l.starts_with("### ") {
            flush!();
            in_log = l.trim() == "### 帧变化记录";
            if !in_log {
                remark_title = format!("导入原文 · {}", l[4..].trim());
                field = Some("原文");
                buf.push(l.to_string());
            }
            continue;
        }
        if in_log {
            if let Some(rest) = l.trim().strip_prefix("- [") {
                if let Some(e) = parse_event(rest) {
                    p.log.push(e);
                    continue;
                }
            }
            if !l.trim().is_empty() {
                warn.push(format!("帧 {} 变化记录行未解析：{:?}", p.title, l));
                preserve_line(&mut p.content, "帧变化记录", l);
            }
            continue;
        }
        let t = l.trim_start();
        let indent = l.len() - t.len();
        if indent == 0 && t.starts_with("- ") {
            let item = &t[2..];
            if let Some(rest) = item.strip_prefix("上帧：") {
                flush!();
                let r = rest.trim().trim_end_matches('。');
                p.parent = if r.starts_with("无") {
                    None
                } else {
                    r.strip_prefix('[')
                        .and_then(|x| x.split_once(']'))
                        .map(|(a, _)| split_id(a.trim()).1)
                };
                continue;
            }
            let mut matched_kw = false;
            for (kw, fname) in [("来源：", "来源"), ("弹帧目标：", "弹帧目标"), ("暂停原因：", "暂停原因")] {
                if let Some(rest) = item.strip_prefix(kw) {
                    flush!();
                    field = Some(fname);
                    buf.push(rest.to_string());
                    matched_kw = true;
                    break;
                }
            }
            if matched_kw {
                continue;
            }
            let bare = item.trim();
            if let Some(rest) = bare.strip_prefix("外部执行事项：") {
                flush!();
                field = Some("外部执行事项");
                let r = rest.trim();
                if !r.is_empty() && r != "无" && r != "无。" {
                    p.content.side_effects.push(parse_st(r)?);
                }
                continue;
            }
            if matches!(bare, "达成情况：" | "计划：") {
                flush!();
                field = Some(if bare == "达成情况：" { "达成情况" } else { "计划" });
                continue;
            }
            if let Some(rest) = bare.strip_prefix("已得结果：") {
                flush!();
                field = Some("已得结果");
                let r = rest.trim();
                if !r.is_empty() && r != "无" && r != "无。" {
                    p.content.results.push(parse_st(r)?);
                }
                continue;
            }
            if let Some(rest) = bare.strip_prefix("确立原则：") {
                flush!();
                field = Some("确立原则");
                canonical_principle = None;
                let r = rest.trim();
                let stripped = strip_principle_no(r);
                if !stripped.is_empty() && stripped != "无" {
                    p.content.principles.push(Principle { rule: parse_st(stripped)?, fate: None });
                }
                continue;
            }
            if let Some(rest) = item.strip_prefix("备注 · ") {
                flush!();
                let title_body = if rest.ends_with('：') { rest.rsplit_once('：') } else { rest.split_once('：') };
                if let Some((t2, v)) = title_body {
                    remark_title = t2.trim().to_string();
                    field = Some("备注");
                    buf.push(v.trim().to_string());
                } else {
                    warn.push(format!("帧 {} 备注行无冒号：{:?}", p.title, item));
                    remark_title = "导入原文 · 备注".into();
                    field = Some("原文");
                    buf.push(l.to_string());
                }
                continue;
            }
            if let Some(rest) = item.strip_prefix("备注：") {
                flush!();
                remark_title = "备注".into();
                field = Some("备注");
                buf.push(rest.trim().to_string());
                continue;
            }
            if let Some((label, _)) = item.split_once('：') {
                flush!();
                remark_title = format!("导入原文 · {}", label.trim());
                field = Some("原文");
                buf.push(l.to_string());
                warn.push(format!("帧 {} 未识别栏目保留为备注：{}", p.title, label));
                continue;
            }
            // 其他顶层 bullet：按当前 field 归档
            match field {
                Some("备注") | Some("原文") => buf.push(l.to_string()),
                Some("外部执行事项") => {
                    let r = item.trim();
                    if !r.is_empty() && r != "无" && r != "无。" {
                        p.content.side_effects.push(parse_st(r)?);
                    }
                }
                Some("来源") | Some("弹帧目标") | Some("暂停原因") => buf.push(t.to_string()),
                Some("确立原则") => {
                    let stripped = bare
                        .split_once(char::is_whitespace)
                        .map(|(_, x)| x)
                        .unwrap_or(bare);
                    if !stripped.is_empty() {
                        p.content.principles.push(Principle { rule: parse_st(stripped)?, fate: None });
                    }
                }
                Some("已得结果") => {
                    if !bare.is_empty() {
                        p.content.results.push(parse_st(bare)?);
                    }
                }
                _ => {
                    if !bare.is_empty() {
                        warn.push(format!("帧 {} 未知顶层行：{:?}", p.title, l));
                        preserve_line(&mut p.content, "未归属栏目", l);
                    }
                }
            }
            continue;
        }
        // 缩进行
        match field {
            Some("备注") => buf.push(l.strip_prefix("  ").unwrap_or(l).to_string()),
            Some("原文") => buf.push(l.to_string()),
            Some("计划") => {
                if t.starts_with("- ") {
                    match parse_plan_item(&t[2..]) {
                        Ok(Some(it)) => insert_plan(&mut p.plan, it, (indent / 2).saturating_sub(1)),
                        Ok(None) => {}
                        Err(e) => {
                            warn.push(format!("帧 {} 计划项解析失败，原文保留为备注：{e}", p.title));
                            preserve_line(&mut p.content, "计划", l);
                        }
                    }
                } else if !t.is_empty() {
                    preserve_line(&mut p.content, "计划", l);
                }
            }
            Some("达成情况") => {
                let tt = t.strip_prefix("- ").unwrap_or(t).trim();
                if closure_rows.consume(indent, tt)? { continue; }
                if let Some(rest) = tt.strip_prefix("结果：") {
                    parse_closure_result(rest.trim(), &mut closure_kind_done, &mut closure_summary);
                } else if let Some(rest) = tt.strip_prefix("依据：") {
                    evidence.push(rest.trim().to_string());
                } else if let Some(rest) = tt.strip_prefix("对应：") {
                    correspond.push(rest.trim().to_string());
                } else if let Some((label, rest)) = tt.split_once('：') {
                    if matches!(label.trim(), "正向" | "反向" | "正交" | "未完义务") {
                        correspond.push(format!("{}：{}", label.trim(), rest.trim()));
                    } else if !tt.is_empty() {
                        warn.push(format!("帧 {} 达成情况行未识别：{:?}", p.title, tt));
                        preserve_line(&mut p.content, "达成情况", l);
                    }
                } else if !tt.is_empty() {
                    warn.push(format!("帧 {} 达成情况行未识别：{:?}", p.title, tt));
                    preserve_line(&mut p.content, "达成情况", l);
                }
            }
            Some("来源") | Some("弹帧目标") | Some("暂停原因") => buf.push(t.to_string()),
            Some("外部执行事项") => {
                if t.starts_with("- ") {
                    let r = t[2..].trim();
                    if !r.is_empty() && r != "无" && r != "无。" {
                        p.content.side_effects.push(parse_st(r)?);
                    }
                } else if !t.is_empty() {
                    warn.push(format!("帧 {} 外部执行事项行未识别：{:?}", p.title, l));
                    preserve_line(&mut p.content, "外部执行事项", l);
                }
            }
            Some("已得结果") => {
                if t.starts_with("- ") {
                    let r = t[2..].trim();
                    if !r.is_empty() && r != "无" {
                        p.content.results.push(parse_st(r)?);
                    }
                } else if !t.is_empty() {
                    warn.push(format!("帧 {} 已得结果行未识别：{:?}", p.title, l));
                    preserve_line(&mut p.content, "已得结果", l);
                }
            }
            Some("确立原则") => {
                let text = t.strip_prefix("- ").unwrap_or(t).trim();
                if indent == 4 {
                    if let Some(index) = canonical_principle {
                        if crate::principle_markdown::field(text, &mut p.content.principles[index])? { continue; }
                    }
                }
                if t.starts_with("- ") {
                    let r = t[2..].trim();
                    if indent == 2 {
                        if let Some(principle) = crate::principle_markdown::header(r) {
                            canonical_principle = Some(p.content.principles.len());
                            p.content.principles.push(principle);
                            continue;
                        }
                    }
                    canonical_principle = None;
                    let stripped = strip_principle_no(r);
                    if !stripped.is_empty() && stripped != "无" {
                        p.content.principles.push(Principle { rule: parse_st(stripped)?, fate: None });
                    }
                } else if !t.is_empty() {
                    warn.push(format!("帧 {} 确立原则行未识别：{:?}", p.title, l));
                    preserve_line(&mut p.content, "确立原则", l);
                }
            }
            _ => {
                if !t.is_empty() {
                    warn.push(format!("帧 {} 帧内自由行（未归属）：{:?}", p.title, l));
                    preserve_line(&mut p.content, "未归属栏目", l);
                }
            }
        }
    }
    flush!();

    if let Some(done) = closure_kind_done {
        let mut cl = Closure { summary: parse_st(&closure_summary)?, ..Default::default() };
        let ev_text = evidence.join("；");
        for c in &correspond {
            if let Some((label, rest)) = c.split_once('：') {
                let item = ClosureItem {
                    result: parse_st(rest.trim())?,
                    evidence: if ev_text.is_empty() { SourcefulText::default() } else { parse_st(&ev_text)? },
                };
                match label.trim() {
                    "反向" => cl.negative.push(item),
                    "正交" => cl.tangential.push(item),
                    "未完义务" => cl.residuals.push(item),
                    _ => cl.positive.push(item),
                }
            } else if !c.is_empty() {
                cl.positive.push(ClosureItem { result: parse_st(c)?, evidence: SourcefulText::default() });
            }
        }
        closure_rows.append_to(&mut cl);
        p.closure = Some((done, cl));
    }
    Ok(p)
}

fn parse_closure_result(r: &str, kind: &mut Option<bool>, summary: &mut String) {
    if let Some(k) = r.strip_prefix("[done]") {
        *kind = Some(true);
        *summary = k.trim().to_string();
    } else if let Some(k) = r.strip_prefix("[expired]") {
        *kind = Some(false);
        *summary = k.trim().to_string();
    } else if let Some(k) = r.strip_prefix("[expire") {
        if let Some((inner, after)) = k.split_once(']') {
            let reason = inner.trim_start_matches(':').trim();
            let after = after.trim();
            *kind = Some(false);
            *summary = if after.is_empty() { reason.to_string() } else { format!("{reason}——{after}") };
        }
    } else {
        // 无判别标记：保守 Expired
        *kind = Some(false);
        *summary = r.to_string();
    }
}


/// 剥原则编号前缀（P1 / P12 / B/G25 形式）；无编号则原样。
fn strip_principle_no(r: &str) -> &str {
    let t = r.trim();
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    let mut it = t.splitn(2, char::is_whitespace);
    let first = it.next().unwrap_or("");
    let rest = match it.next() { Some(x) => x, None => return t };
    let core = first.trim_end_matches(['｜', ':', '：']);
    let num = core.trim_start_matches(|c: char| c.is_ascii_alphabetic() || c == '/');
    if (core.starts_with('P') || core.starts_with("B/G")) && digits(num) && !rest.is_empty() {
        rest.trim_start()
    } else {
        t
    }
}

fn literal_text(text: &str) -> SourcefulText {
    SourcefulText(vec![Segment::Text(text.to_string())])
}

fn preserve_line(c: &mut Content, column: &str, line: &str) {
    append_remark(c, &format!("导入原文 · {column}"), &literal_text(line));
}

fn append_remark(c: &mut Content, title: &str, body: &SourcefulText) {
    let e = c.remarks.entry(title.to_string()).or_default();
    if !e.is_empty() {
        e.push(Segment::Text("\n".into()));
    }
    e.extend(body.iter().cloned());
    *e = crate::text::norm(e);
}

fn parse_event(rest: &str) -> Option<Event> {
    let (action_txt, rest) = rest.split_once(']')?;
    let rest = rest.trim();
    let (at, detail) = if let Some(r) = rest.strip_prefix('[') {
        match r.split_once(']') {
            Some((a, d)) => (a.trim().to_string(), d.trim().to_string()),
            None => (String::new(), rest.to_string()),
        }
    } else {
        (String::new(), rest.to_string())
    };
    let action = match action_txt.trim().to_uppercase().as_str() {
        "PUSH" => Action::Push,
        "POP" => Action::Pop,
        "PAUSE" => Action::Pause,
        "RESUME" => Action::Resume,
        "UPDATE" => Action::Update,
        "CORRECT" => Action::Correct,
        other => Action::Other(other.to_string()),
    };
    Some(Event { action, at, detail: parse_st(&detail).ok()? })
}

fn parse_plan_item(txt: &str) -> Result<Option<PlanItem>, String> {
    let mut t = txt.trim();
    if let Some(r) = t.strip_prefix("**") {
        t = r.trim();
        if let Some(r2) = t.strip_suffix("**") {
            t = r2.trim();
        }
    }
    let (bracket, rest) = if let Some(r) = t.strip_prefix('[') {
        match r.split_once(']') {
            Some((inner, rest)) => (inner.trim().to_string(), rest.trim_start_matches('*').trim().to_string()),
            None => return Err(format!("计划项括号未闭合：{t:?}")),
        }
    } else {
        (String::new(), t.to_string())
    };
    let (state_word, label) = match bracket.split_once('·') {
        Some((w, l)) => (w.trim().to_string(), l.trim().to_string()),
        None => {
            let w = bracket.strip_prefix("FOCUS/").unwrap_or(&bracket).to_string();
            (w, String::new())
        }
    };
    let (progress, reason_in_bracket) = match state_word.as_str() {
        "" => (ItemState::Ready, false),
        "done" => (ItemState::ItemDone, false),
        "in_progress" => (ItemState::InProgress, false),
        "ready" => (ItemState::Ready, false),
        "blocked" => (ItemState::Blocked { reason: SourcefulText::default() }, false),
        w if w.starts_with("expire") => {
            let reason = w.trim_start_matches("expire").trim_start_matches(':').trim();
            (ItemState::ItemExpired { reason: parse_st(reason)? }, true)
        }
        w => return Err(format!("计划项状态未识别：{w:?}")),
    };
    let _ = reason_in_bracket;
    let text = if label.is_empty() { rest } else { format!("（{label}）{rest}") };
    Ok(Some(PlanItem { text: parse_st(&text)?, children: vec![], progress, fork: vec![] }))
}

fn insert_plan(plan: &mut Vec<PlanItem>, item: PlanItem, depth: usize) {
    if depth == 0 {
        plan.push(item);
        return;
    }
    if let Some(last) = plan.last_mut() {
        insert_plan(&mut last.children, item, depth - 1);
    } else {
        plan.push(item);
    }
}

fn status_from_marker(marker: &str, p: &ParsedFrame, warn: &mut W, id: Id) -> FrameStatus {
    match marker {
        "PAUSED" => FrameStatus::Paused {
            reason: p.pause_reason.clone().unwrap_or_else(|| {
                warn.push(format!("帧 {} Paused 无暂停原因，填占位", id));
                parse_st("（导入：未提供原因）").unwrap()
            }),
        },
        "POPPED" => {
            let (done, cl) = p.closure.clone().unwrap_or_else(|| {
                warn.push(format!("帧 {} POPPED 无达成情况，按 Expired 兜底", id));
                (false, Closure { summary: parse_st("（导入：无达成情况）").unwrap(), ..Default::default() })
            });
            if done {
                FrameStatus::Done { closure: cl }
            } else {
                FrameStatus::Expired { closure: cl }
            }
        }
        "TOP" => FrameStatus::Active,
        m if m.starts_with("TOP") => {
            if m.contains("PAUSED") {
                warn.push(format!("帧 {} 为 {m}：等待语义导入后由计划项 Blocked 表达（渲染按派生）", id));
            }
            FrameStatus::Active
        }
        "" => FrameStatus::Active,
        other => {
            warn.push(format!("帧 {} 未知标记 {}，按 Active 导入", id, other));
            FrameStatus::Active
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# 示例栈（WS-X）

把示例做成可往返的最小家族文件。

## 栈状态

```text
[X-01] 根任务
- [X-02] [PAUSED] 分支A
- [X-03] [TOP] 当前工作
```

## 已 pop 节点

- [X-04] [POPPED] 旧帧；上帧：[X-01]。

## [X-01] 根任务

- 上帧：无（根帧）。
- 来源：建栈。
- 弹帧目标：完成 X。

- 计划：
  - [done] 第一步
  - **[FOCUS/in_progress]** 第二步

### 帧变化记录

- [PUSH] [2026-10-06T00:00:00Z] 建栈。

## [X-02] [PAUSED] 分支A

- 上帧：[X-01]。
- 来源：进入 A。
- 弹帧目标：A 完成。
- 暂停原因：注意力转到 X-03。

### 帧变化记录

- [PUSH] [2026-10-06T00:01:00Z] 进入 A。
- [PAUSE] [2026-10-06T00:02:00Z] 转向 X-03。

## [X-03] [TOP] 当前工作

- 上帧：[X-01]。
- 来源：进入 C。
- 弹帧目标：C 完成。

- 计划：
  - [blocked] 等回信

### 帧变化记录

- [PUSH] [2026-10-06T00:03:00Z] 进入 C。

## [X-04] [POPPED] 旧帧

- 上帧：[X-01]。
- 来源：旧工作。
- 弹帧目标：旧目标。
- 达成情况：
  - 结果：[done] 旧目标已达成。
  - 对应：正向交付。
  - 依据：记录。

### 帧变化记录

- [PUSH] [2026-10-06T00:00:30Z] 旧工作。
- [POP] [2026-10-06T00:00:59Z] 完成。
";

    #[test]
    fn imports_family_sample() {
        let out = import_md(SAMPLE).expect("解析失败");
        let ws = out.ws;
        assert_eq!(ws.title.as_deref(), Some("示例栈（WS-X）"));
        assert_eq!(ws.prefix.as_deref(), Some("X"));
        assert_eq!(ws.frames.len(), 4);
        assert_eq!(ws.focus(), Some(3));
        assert!(matches!(ws.frames[&2].status, FrameStatus::Paused { .. }));
        assert!(matches!(ws.frames[&4].status, FrameStatus::Done { .. }));
        // FOCUS 粗体 → InProgress 普通项
        assert!(ws.frames[&1].plan[1].progress == ItemState::InProgress);
        // 树父子
        assert_eq!(ws.frames[&2].parent, Some(1));
        assert_eq!(ws.frames[&4].parent, Some(1));
    }

    #[test]
    fn roundtrip_export_import() {
        let out = import_md(SAMPLE).unwrap();
        let md = crate::print::export_markdown(&out.ws);
        let again = import_md(&md).unwrap();
        // 关键结构稳定（文本排版允许差异）
        let (a, b) = (out.ws, again.ws);
        assert_eq!(a.frames.len(), b.frames.len());
        for id in a.frames.keys() {
            assert_eq!(a.frames[id].parent, b.frames[id].parent, "帧 {id} 父子不稳定");
            assert_eq!(a.frames[id].status.kind_str(), b.frames[id].status.kind_str(), "帧 {id} 状态不稳定");
        }
        assert_eq!(a.focus(), b.focus());
    }
}
