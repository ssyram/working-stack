//! working-stack 核心库。
//! 规约源：`playground.md`（核心语言）+ `docs/architecture.md`（P 合同）+ `docs/detailed-design.md`（函数合同）。

pub mod text;
pub mod model;
pub mod view;
pub mod timeline;
pub mod commit_log;
pub mod resolve;
pub mod validate;
pub mod ops;
pub mod stack_path;
pub mod path_edit;
pub mod persist;
pub mod diff;
pub mod print;
pub mod principle_markdown;
pub mod closure_markdown;
pub mod note_layout;
pub mod note_markdown;
pub mod import;
pub mod cli;
pub mod doc_catalog;
pub mod json_input;
pub mod web;

pub use model::*;
pub use text::{Reference, Segment, SourcefulText};
