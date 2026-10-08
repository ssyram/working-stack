use crate::model::Principle;
use crate::text::SourcefulText;

pub fn emit(principle: &Principle, index: usize, out: &mut String) {
    out.push_str(&format!("  - P{}\n    - 原则：{}\n    - 处置：{}\n", index + 1,
        serde_json::to_string(&principle.rule).unwrap(),
        serde_json::to_string(&principle.fate).unwrap()));
}

pub fn header(text: &str) -> Option<Principle> {
    text.strip_prefix('P')?.parse::<usize>().ok()?;
    Some(Principle { rule: SourcefulText::default(), fate: None })
}

pub fn field(text: &str, principle: &mut Principle) -> Result<bool, String> {
    if let Some(value) = text.strip_prefix("原则：") {
        principle.rule = serde_json::from_str(value.trim()).map_err(|e| format!("原则字段：{e}"))?;
        return Ok(true);
    }
    if let Some(value) = text.strip_prefix("处置：") {
        principle.fate = serde_json::from_str(value.trim()).map_err(|e| format!("处置字段：{e}"))?;
        return Ok(true);
    }
    Ok(false)
}
