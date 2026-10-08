//! 数据模型（蓝本 1:1；YAML 编码见 architecture D.storage；未知键 fail closed——F07）。

use crate::text::SourcefulText;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 帧身份。蓝本 Integer 在此收窄为 u64（F22 裁定，入 detailed-design）；负数不在域内。
pub type Id = u64;
pub type Title = String;
pub type Time = String;
pub type Reason = SourcefulText;
pub type Problem = String;

/// 整个 WS。heads 不存储（派生，见 view）。`log` 为栈级事件（note/meta 等无帧归属变更——oracle S6）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct WorkingStack {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    #[serde(default)]
    pub notes: BTreeMap<String, Note>,
    #[serde(default)]
    pub frames: BTreeMap<Id, Frame>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Note {
    #[serde(default)]
    pub meta: Meta,
    #[serde(default)]
    pub body: SourcefulText,
}

pub type Meta = Vec<String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    #[serde(default)]
    pub parent: Option<Id>,
    #[serde(default)]
    pub title: Title,
    #[serde(default)]
    pub goal: Vec<SourcefulText>,
    #[serde(default)]
    pub origin: Vec<SourcefulText>,
    pub status: FrameStatus,
    #[serde(default)]
    pub plan: Vec<PlanItem>,
    #[serde(default)]
    pub content: Content,
    #[serde(default)]
    pub log: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FrameStatus {
    Active,
    Paused { reason: Reason },
    Done { closure: Closure },
    Expired { closure: Closure },
}

impl FrameStatus {
    /// trivial：live = Active ∪ Paused（"在栈内"定义）。
    pub fn is_live(&self) -> bool {
        matches!(self, FrameStatus::Active | FrameStatus::Paused { .. })
    }
    pub fn is_closed(&self) -> bool {
        matches!(self, FrameStatus::Done { .. } | FrameStatus::Expired { .. })
    }
    pub fn kind_str(&self) -> &'static str {
        match self {
            FrameStatus::Active => "active",
            FrameStatus::Paused { .. } => "paused",
            FrameStatus::Done { .. } => "done",
            FrameStatus::Expired { .. } => "expired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanItem {
    #[serde(default)]
    pub text: SourcefulText,
    #[serde(default)]
    pub children: Vec<PlanItem>,
    #[serde(default)]
    pub progress: ItemState,
    #[serde(default)]
    pub fork: Vec<Id>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ItemState {
    Ready,
    InProgress,
    Blocked { reason: Reason },
    ItemDone,
    ItemExpired { reason: Reason },
}

impl Default for ItemState {
    fn default() -> Self {
        ItemState::Ready
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Content {
    #[serde(default)]
    pub results: Vec<SourcefulText>,
    #[serde(default)]
    pub side_effects: Vec<SourcefulText>,
    #[serde(default)]
    pub principles: Vec<Principle>,
    #[serde(default)]
    pub remarks: BTreeMap<Title, SourcefulText>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Principle {
    #[serde(default)]
    pub rule: SourcefulText,
    #[serde(default)]
    pub fate: Option<Fate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Fate {
    KeptLocal,
    ReleasedTo { id: Id },
    Invalidated { reason: Reason },
    Reclassified { target: ReclassTarget },
}

/// 判别字段统一为 kind（F21 裁定）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReclassTarget {
    Results,
    Remark { title: Title },
}

/// 关闭报告：Done/Expired 共享（命名取中立概念，蓝本 Closure）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Closure {
    #[serde(default)]
    pub summary: SourcefulText,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positive: Vec<ClosureItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative: Vec<ClosureItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tangential: Vec<ClosureItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residuals: Vec<ClosureItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ClosureItem {
    #[serde(default)]
    pub result: SourcefulText,
    #[serde(default)]
    pub evidence: SourcefulText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub action: Action,
    pub at: Time,
    #[serde(default)]
    pub detail: SourcefulText,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Push,
    Pop,
    Pause,
    Resume,
    Update,
    Correct,
    Other(String),
}

impl Action {
    /// 打印名（与既有家族大写形式一致）。trivial。
    pub fn label(&self) -> String {
        match self {
            Action::Push => "PUSH".into(),
            Action::Pop => "POP".into(),
            Action::Pause => "PAUSE".into(),
            Action::Resume => "RESUME".into(),
            Action::Update => "UPDATE".into(),
            Action::Correct => "CORRECT".into(),
            Action::Other(s) => s.to_uppercase(),
        }
    }
}

/// pop 的两种关闭（操作参数，不入存储）。
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    DoneO(Closure),
    ExpiredO(Closure),
}

/// 统一错误模型 → CLI exit 码（detailed-design §错误模型）。
#[derive(Debug)]
pub enum WsError {
    Usage(String),
    Parse(String),
    Invalid(Vec<Problem>),
    Conflict,
    Locked,
    Io(std::io::Error),
    Yaml(String),
}

impl WsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            WsError::Usage(_) | WsError::Parse(_) | WsError::Invalid(_) => 2,
            WsError::Conflict => 3,
            WsError::Locked | WsError::Io(_) | WsError::Yaml(_) => 4,
        }
    }
    pub fn message(&self) -> String {
        match self {
            WsError::Usage(s) => format!("用法/前置错误：{s}"),
            WsError::Parse(s) => format!("解析错误：{s}"),
            WsError::Invalid(ps) => format!("结构校验失败（{} 项）：\n{}", ps.len(), ps.join("\n")),
            WsError::Conflict => "基准冲突：文件已被其他写者修改".into(),
            WsError::Locked => "文件被锁（另一写者在运行，或锁残留）".into(),
            WsError::Io(e) => format!("IO 错误：{e}"),
            WsError::Yaml(s) => format!("YAML 错误：{s}"),
        }
    }
}

impl From<std::io::Error> for WsError {
    fn from(e: std::io::Error) -> Self {
        WsError::Io(e)
    }
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for WsError {}
