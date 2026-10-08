use crate::model::WsError;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub stack: Option<String>,
    pub op: String,
    #[serde(default)]
    pub args: Map<String, Value>,
    #[serde(default)]
    pub reason: String,
    #[serde(default = "default_json")]
    pub json: bool,
}

fn default_json() -> bool { true }
fn usage(message: impl Into<String>) -> WsError { WsError::Usage(message.into()) }

fn strings(value: Value, key: &str) -> Result<Vec<String>, WsError> {
    match value {
        Value::String(s) => Ok(vec![s]),
        Value::Array(xs) => xs.into_iter().map(|x| match x {
            Value::String(s) => Ok(s),
            _ => Err(usage(format!("{key} 的元素必须是字符串"))),
        }).collect(),
        _ => Err(usage(format!("{key} 必须是字符串或字符串数组"))),
    }
}

impl Request {
    pub fn parse(text: &str) -> Result<Self, WsError> {
        serde_json::from_str(text).map_err(|e| WsError::Parse(e.to_string()))
    }

    pub fn cli_args(mut self) -> Result<Vec<String>, WsError> {
        let mut out = Vec::new();
        if let Some(stack) = self.stack { out.push(stack); }
        out.push(self.op.clone());
        let (positional, flags, lists, boolean, reason): (&[&str], &[&str], &[&str], &[&str], bool) =
            match self.op.as_str() {
                "init" => (&[], &["title", "prefix"], &[], &[], false),
                "push" => (&["parent"], &["title"], &["goal", "origin"], &[], true),
                "pop" => (&["id"], &["summary", "focus_to"], &["residual", "pos"], &["expired"], true),
                "pause" | "resume" | "switch" => (&["id"], &[], &[], &[], true),
                "update" => (&["id"], &["title", "reopen"], &[
                    "goal", "origin", "result_add", "side_add", "principle_add", "settle",
                    "remark", "plan_add", "plan_set", "plan_text", "plan_add_child",
                ], &[], true),
                "note" => (&["key"], &["body"], &["meta"], &[], self.args.contains_key("body")),
                "get" => (&["path"], &[], &[], &[], false),
                "set" | "insert" => (&["path"], &["value"], &[], &[], true),
                "remove" => (&["path"], &[], &[], &[], true),
                "move" => (&["path"], &["from"], &[], &[], true),
                "show" => (&["what", "id"], &[], &[], &[], false),
                "doc" => (&["name"], &[], &[], &["list"], false),
                "export" => (&["format"], &["out"], &[], &[], false),
                "import" => (&["md"], &["out"], &[], &[], false),
                "apply" => (&["candidate"], &[], &[], &[], true),
                "edit" => (&[], &[], &[], &[], true),
                "web" => (&[], &["port"], &[], &[], false),
                "list" | "validate" | "timeline" => (&[], &[], &[], &[], false),
                _ => return Err(usage(format!("未知操作：{}", self.op))),
            };
        for key in positional {
            if let Some(value) = self.args.remove(*key) {
                let s = value.as_str().ok_or_else(|| usage(format!("{key} 必须是字符串")))?;
                out.push(s.into());
            }
        }
        for key in flags {
            if let Some(value) = self.args.remove(*key) {
                let s = if *key == "value" {
                    serde_json::to_string(&value).map_err(|e| WsError::Parse(e.to_string()))?
                } else if *key == "port" {
                    value.as_u64().map(|n| n.to_string()).ok_or_else(|| usage("port 必须是非负整数"))?
                } else {
                    value.as_str().ok_or_else(|| usage(format!("{key} 必须是字符串")))?.to_string()
                };
                let flag = if self.op == "import" && *key == "out" { "-o".into() }
                    else { format!("--{}", key.replace('_', "-")) };
                out.push(format!("{flag}={s}"));
            }
        }
        for key in lists {
            if let Some(value) = self.args.remove(*key) {
                let mut values = strings(value, key)?;
                if self.op == "update" && matches!(*key, "goal" | "origin") && values.is_empty() {
                    values.push(String::new());
                }
                for s in values { out.push(format!("--{}={s}", key.replace('_', "-"))); }
            }
        }
        for key in boolean {
            if let Some(value) = self.args.remove(*key) {
                match value.as_bool() {
                    Some(true) => out.push(format!("--{key}")),
                    Some(false) => (),
                    None => return Err(usage(format!("{key} 必须是布尔值"))),
                }
            }
        }
        if let Some(key) = self.args.keys().next() { return Err(usage(format!("{} 未知参数：{key}", self.op))); }
        if reason {
            if self.reason.trim().is_empty() { return Err(usage("reason 必填")); }
            out.push(format!("-r={}", self.reason));
        }
        if self.json { out.push("--json".into()); }
        Ok(out)
    }
}

pub fn process_args(args: &[String]) -> Result<Vec<String>, WsError> {
    use std::io::{IsTerminal, Read};
    if std::io::stdin().is_terminal() { return Ok(args.to_vec()); }
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).map_err(WsError::Io)?;
    if text.trim().is_empty() { return Ok(args.to_vec()); }
    Request::parse(&text)?.cli_args()
}
