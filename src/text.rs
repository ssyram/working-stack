//! SourcefulText：段列表核心值 + 插值序列化（parse/format 往返，见 detailed-design §4.1）。
//! ST 是 newtype（serde orphan rule——`impl Serialize for Vec<Segment>` 不合法）。

use crate::model::Id;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};

/// 正文段。引用是结构段，不是字符串内语法（蓝本 SourcefulText = [Either Reference String]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Ref(Reference),
    Text(String),
}

/// 段列表 newtype。Deref 到 Vec<Segment>；serde 表现为单个插值字符串。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourcefulText(pub Vec<Segment>);

impl Deref for SourcefulText {
    type Target = Vec<Segment>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SourcefulText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Segment>> for SourcefulText {
    fn from(v: Vec<Segment>) -> Self {
        SourcefulText(v)
    }
}

impl SourcefulText {
    /// 单段纯文本快捷构造。trivial。
    pub fn text(s: &str) -> Self {
        SourcefulText(vec![Segment::Text(s.to_string())])
    }
}

/// 引用路径。文法：`12.goal` / `note:key.body` / `12.plan[1]` / `12.log[3].detail`。
/// 带前缀形式（`T-12`）解析时接受、规范形为纯数字（前缀仅展示元数据，不入身份——F14 裁定）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub path: Vec<RefSeg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefSeg {
    Frame(Id),
    NoteK(String),
    Field(String),
    Index(usize),
}

/// 规范形：合并相邻 Text 段、丢弃空 Text。幂等（测试钉住）。
pub fn norm(st: &SourcefulText) -> SourcefulText {
    let mut out: Vec<Segment> = Vec::new();
    for seg in st.iter() {
        match seg {
            Segment::Text(t) => {
                if t.is_empty() {
                    continue;
                }
                if let Some(Segment::Text(last)) = out.last_mut() {
                    last.push_str(t);
                } else {
                    out.push(Segment::Text(t.clone()));
                }
            }
            other => out.push(other.clone()),
        }
    }
    SourcefulText(out)
}

/// 解析引用路径。
/// Requires：s 为 UTF-8。
/// Ensures：头段 `note:key`｜`N`｜`PREFIX-N`（前缀丢弃，数字为身份）；尾段 `(.field | [i])*`；
/// 数字规范形满足 render_ref ∘ parse_ref = 恒等；空头/非数字/空字段/缺 `]` ⇒ Err（fail closed）。
pub fn parse_ref(s: &str) -> Result<Reference, String> {
    let mut path = Vec::new();
    let mut rest = s.trim();
    let head_end = rest.find(['.', '[']).unwrap_or(rest.len());
    let head = &rest[..head_end];
    if let Some(key) = head.strip_prefix("note:") {
        if key.is_empty() {
            return Err(format!("空的 note key: {s:?}"));
        }
        path.push(RefSeg::NoteK(key.to_string()));
    } else {
        let num = head.rsplit('-').next().unwrap_or(head);
        let id: Id = num.parse().map_err(|_| format!("引用头不是帧号或 note: {s:?}"))?;
        path.push(RefSeg::Frame(id));
    }
    rest = &rest[head_end..];
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('.') {
            let end = r.find(['.', '[']).unwrap_or(r.len());
            let f = &r[..end];
            if f.is_empty() {
                return Err(format!("空字段名: {s:?}"));
            }
            path.push(RefSeg::Field(f.to_string()));
            rest = &r[end..];
        } else if let Some(r) = rest.strip_prefix('[') {
            let end = r.find(']').ok_or_else(|| format!("引用下标缺少 ]: {s:?}"))?;
            let n: usize = r[..end].parse().map_err(|_| format!("下标非数字: {s:?}"))?;
            path.push(RefSeg::Index(n));
            rest = &r[end + 1..];
        } else {
            return Err(format!("引用语法错误: {s:?}"));
        }
    }
    Ok(Reference { path })
}

/// 渲染引用路径（数字规范形）。trivial。
pub fn render_ref(r: &Reference) -> String {
    let mut s = String::new();
    for seg in &r.path {
        match seg {
            RefSeg::Frame(id) => s.push_str(&id.to_string()),
            RefSeg::NoteK(k) => {
                s.push_str("note:");
                s.push_str(k);
            }
            RefSeg::Field(f) => {
                s.push('.');
                s.push_str(f);
            }
            RefSeg::Index(i) => s.push_str(&format!("[{i}]")),
        }
    }
    s
}

/// 插值字符串 → 段列表。
/// Requires：s 为 UTF-8。
/// Ensures：`{{ref}}` → Ref(parse_ref)；转义 `\\`→`\`、`\{`→`{`（与 format 全转义双射，测试钉住）；
/// 输出为 norm 形；`{{` 无闭合或 ref 非法 ⇒ Err（fail closed，不静默吞）。
/// 论证：单遍扫描 i 每轮严格前进 ⇒ 终止；分支按前缀互斥穷尽（转义前缀码性质保证无歧义）；norm 出口统一。
pub fn parse_st(s: &str) -> Result<SourcefulText, String> {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut segs: Vec<Segment> = Vec::new();
    let mut buf = String::new();
    let mut i = 0usize;
    while i < n {
        if chars[i] == '\\' && i + 1 < n {
            match chars[i + 1] {
                '\\' => {
                    buf.push('\\');
                    i += 2;
                }
                '{' => {
                    buf.push('{');
                    i += 2;
                }
                _ => {
                    buf.push('\\');
                    i += 1;
                }
            }
        } else if chars[i] == '{' && i + 1 < n && chars[i + 1] == '{' {
            if !buf.is_empty() {
                segs.push(Segment::Text(std::mem::take(&mut buf)));
            }
            let mut j = i + 2;
            while j + 1 < n && !(chars[j] == '}' && chars[j + 1] == '}') {
                j += 1;
            }
            if j + 1 >= n {
                return Err(format!("未闭合的引用: {s:?}"));
            }
            let inner: String = chars[i + 2..j].iter().collect();
            segs.push(Segment::Ref(parse_ref(&inner)?));
            i = j + 2;
        } else {
            buf.push(chars[i]);
            i += 1;
        }
    }
    if !buf.is_empty() {
        segs.push(Segment::Text(buf));
    }
    Ok(norm(&SourcefulText(segs)))
}

/// 段列表 → 插值字符串。全转义：`\\`→`\\\\`，再 `{{`→`\{{`；
/// Text 末尾的 `{` 若后接 Ref 段则转义为 `\{`（防段边界拼出 `{{`——F05 反例）。
/// 性质：parse_st(format_st(norm(x))) == Ok(norm(x))（测试钉住）。
pub fn format_st(st: &SourcefulText) -> String {
    let mut s = String::new();
    let n = st.len();
    for (i, seg) in st.iter().enumerate() {
        match seg {
            Segment::Text(t) => {
                let mut e = t.replace('\\', "\\\\").replace("{{", "\\{{");
                if e.ends_with('{') && i + 1 < n && matches!(st[i + 1], Segment::Ref(_)) {
                    e.pop();
                    e.push_str("\\{");
                }
                s.push_str(&e);
            }
            Segment::Ref(r) => {
                s.push_str("{{");
                s.push_str(&render_ref(r));
                s.push_str("}}");
            }
        }
    }
    s
}

/// 裸视图：串接 Text 段，忽略引用。trivial。
pub fn flatten(st: &SourcefulText) -> String {
    let mut s = String::new();
    for seg in st.iter() {
        if let Segment::Text(t) = seg {
            s.push_str(t);
        }
    }
    s
}

// —— serde：ST 在 YAML/JSON 中表现为单个字符串 ——

impl Serialize for SourcefulText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format_st(self))
    }
}

impl<'de> Deserialize<'de> for SourcefulText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_st(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_plain() {
        let x = SourcefulText::text("普通文本");
        assert_eq!(parse_st(&format_st(&x)).unwrap(), x);
    }

    #[test]
    fn roundtrip_escape() {
        // 全转义双射（oracle S4 / auditor F05：反斜杠邻引用、'{' 邻引用、字面 {{、孤立反斜杠）
        let cases: Vec<SourcefulText> = vec![
            SourcefulText(vec![Segment::Text("a\\".into()), Segment::Ref(parse_ref("12.goal").unwrap())]),
            SourcefulText(vec![Segment::Text("a{".into()), Segment::Ref(parse_ref("12.goal").unwrap())]),
            SourcefulText(vec![Segment::Text("a\\{{b".into()), Segment::Text("c".into())]),
            SourcefulText(vec![Segment::Text("b\\\\c".into())]),
            SourcefulText(vec![Segment::Text("{{".into())]),
            SourcefulText(vec![Segment::Ref(parse_ref("note:k.body").unwrap()), Segment::Text("}x".into())]),
        ];
        for x in cases {
            let nx = norm(&x);
            let f = format_st(&nx);
            assert_eq!(parse_st(&f).unwrap(), nx, "往返失败: {f:?}");
        }
    }

    #[test]
    fn ref_grammar() {
        for s in ["12", "12.goal", "note:k.body", "12.plan[1]", "12.log[3].detail", "3.origin", "note:k.meta[0]"] {
            let r = parse_ref(s).unwrap();
            assert_eq!(render_ref(&r), s);
        }
        // 带前缀：解析接受，规范形为数字（F14 裁定）
        assert_eq!(render_ref(&parse_ref("T-12.goal").unwrap()), "12.goal");
        assert!(parse_ref("").is_err());
        assert!(parse_ref("note:").is_err());
        assert!(parse_ref("12[").is_err());
        assert!(parse_ref("12..goal").is_err());
    }

    #[test]
    fn fail_closed_unclosed() {
        assert!(parse_st("见 {{12.goal").is_err());
        assert!(parse_st("坏 ref {{9++}}").is_err());
    }

    #[test]
    fn norm_idempotent_merges() {
        let x = SourcefulText(vec![
            Segment::Text("a".into()),
            Segment::Text("b".into()),
            Segment::Text(String::new()),
        ]);
        let n1 = norm(&x);
        assert_eq!(n1.0, vec![Segment::Text("ab".into())]);
        assert_eq!(norm(&n1), n1);
    }
}
