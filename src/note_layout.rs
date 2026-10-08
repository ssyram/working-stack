use crate::model::Meta;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position { BeforeTree, AfterTree, BeforeFrames, AfterFrames, Appendix }

pub fn layout(meta: &Meta) -> (Position, i64) {
    let mut position = Position::Appendix;
    let mut order = 0;
    for hint in meta {
        match hint.as_str() {
            "before:tree" => position = Position::BeforeTree,
            "after:tree" => position = Position::AfterTree,
            "before:frames" => position = Position::BeforeFrames,
            "after:frames" => position = Position::AfterFrames,
            "appendix" => position = Position::Appendix,
            _ => {
                if let Some(n) = hint.strip_prefix("order:").and_then(|s| s.parse::<i64>().ok()) {
                    order = n;
                }
            }
        }
    }
    (position, order)
}
