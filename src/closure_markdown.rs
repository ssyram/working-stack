use crate::model::{Closure, ClosureItem};

pub fn emit(closure: &Closure, out: &mut String) {
    if closure.positive.is_empty() && closure.negative.is_empty()
        && closure.tangential.is_empty() && closure.residuals.is_empty() { return; }
    out.push_str("  - 对应：\n");
    for (label, items) in [("正向", &closure.positive), ("反向", &closure.negative),
        ("正交", &closure.tangential), ("未完义务", &closure.residuals)] {
        for item in items {
            out.push_str(&format!("    - {}：\n      - 结果：{}\n      - 依据：{}\n", label,
                serde_json::to_string(&item.result).unwrap(),
                serde_json::to_string(&item.evidence).unwrap()));
        }
    }
}

#[derive(Default)]
pub struct Rows {
    pub closure: Closure,
    active: Option<usize>,
}

impl Rows {
    fn items(&mut self, direction: usize) -> &mut Vec<ClosureItem> {
        match direction {
            0 => &mut self.closure.positive,
            1 => &mut self.closure.negative,
            2 => &mut self.closure.tangential,
            3 => &mut self.closure.residuals,
            _ => unreachable!(),
        }
    }

    pub fn consume(&mut self, indent: usize, text: &str) -> Result<bool, String> {
        if indent == 4 {
            self.active = match text {
                "正向：" => Some(0), "反向：" => Some(1),
                "正交：" => Some(2), "未完义务：" => Some(3),
                _ => None,
            };
            if let Some(direction) = self.active {
                self.items(direction).push(ClosureItem::default());
                return Ok(true);
            }
        }
        if indent == 6 {
            if let Some(direction) = self.active {
                let item = self.items(direction).last_mut().unwrap();
                if let Some(value) = text.strip_prefix("结果：") {
                    item.result = serde_json::from_str(value.trim()).map_err(|e| format!("结果字段：{e}"))?;
                    return Ok(true);
                }
                if let Some(value) = text.strip_prefix("依据：") {
                    item.evidence = serde_json::from_str(value.trim()).map_err(|e| format!("依据字段：{e}"))?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn append_to(self, closure: &mut Closure) {
        closure.positive.extend(self.closure.positive);
        closure.negative.extend(self.closure.negative);
        closure.tangential.extend(self.closure.tangential);
        closure.residuals.extend(self.closure.residuals);
    }
}
