//! 网页视图：GET 只读，POST 复用 JSON 参数转换及 CLI dispatch。
use crate::model::*;
use crate::persist;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Response};

const INDEX_HTML: &str = include_str!("../assets/index.html");

pub fn serve(stack_arg: &str, file: &Path, port: u16) -> ! {
    let addr = format!("127.0.0.1:{port}");
    let server = tiny_http::Server::http(&addr).unwrap_or_else(|e| {
        eprintln!("无法监听 {addr}：{e}");
        std::process::exit(4);
    });
    let file: PathBuf = file.to_path_buf();
    println!("working-stack web: http://{addr}  （栈：{}）", file.display());
    for mut request in server.incoming_requests() {
        let method = request.method().as_str().to_string();
        let url = request.url().to_string();
        let (status, ctype, body) = match (method.as_str(), url.as_str()) {
            ("GET", "/") => (200u16, "text/html; charset=utf-8", INDEX_HTML.to_string()),
            ("GET", "/api/version") => api_version(&file),
            ("GET", "/api/state") => api_state(&file),
            ("POST", "/api/op") => {
                let mut buf = String::new();
                let _ = request.as_reader().read_to_string(&mut buf);
                handle_op(stack_arg, &file, &buf)
            }
            _ => (404, "text/plain; charset=utf-8", "not found".into()),
        };
        let header = Header::from_bytes("Content-Type", ctype).unwrap();
        let _ = request.respond(Response::from_string(body).with_status_code(status).with_header(header));
    }
    unreachable!()
}

fn json_resp(v: serde_json::Value) -> (u16, &'static str, String) {
    (200, "application/json", serde_json::to_string(&v).unwrap_or_default())
}

fn api_version(file: &Path) -> (u16, &'static str, String) {
    match persist::load(file) {
        Ok(l) => json_resp(serde_json::json!({"version": crate::cli::fnv_public(&l.raw)})),
        Err(e) => json_resp(serde_json::json!({"error": e.message()})),
    }
}

fn api_state(file: &Path) -> (u16, &'static str, String) {
    match persist::load(file) {
        Ok(l) => {
            let ws = &l.ws;
            let paused: Vec<serde_json::Value> = ws.tips().into_iter().filter_map(|t| match &ws.frames[&t].status {
                FrameStatus::Paused { reason } => Some(serde_json::json!({
                    "id": ws.print_id(t), "title": ws.frames[&t].title, "reason": ws.render(reason),
                })),
                _ => None,
            }).collect();
            let focus_path: Vec<String> = ws.focus().map(|f| ws.path(f).iter().map(|i| ws.print_id(*i)).collect()).unwrap_or_default();
            json_resp(serde_json::json!({
                "version": crate::cli::fnv_public(&l.raw), "title": ws.title, "prefix": ws.prefix,
                "focus": ws.focus().map(|f| ws.print_id(f)), "focusPath": focus_path,
                "tips": ws.tips().iter().map(|t| ws.print_id(*t)).collect::<Vec<_>>(), "paused": paused,
                "tree": crate::print::tree_text(ws), "markdown": crate::print::export_markdown(ws),
                "stack": ws, "warnings": l.warnings,
            }))
        }
        Err(e) => json_resp(serde_json::json!({"error": e.message()})),
    }
}

fn handle_op(stack_arg: &str, file: &Path, body: &str) -> (u16, &'static str, String) {
    let mut get_path = None;
    let prepared = crate::json_input::Request::parse(body).and_then(|mut request| {
        if request.op == "get" {
            get_path = Some(request.args.get("path").and_then(|v| v.as_str())
                .ok_or_else(|| WsError::Usage("get 需要 path 字符串".into()))?.to_string());
        }
        request.stack = Some(stack_arg.into());
        request.json = false;
        request.cli_args()
    });
    let args = match prepared {
        Ok(args) => args,
        Err(e) => return json_resp(serde_json::json!({"ok":false,"error":e.message(),"exit":e.exit_code()})),
    };
    if let Some(path) = get_path {
        let value = persist::load(file).and_then(|loaded| {
            let value = crate::path_edit::get(&loaded.ws, &path)?;
            Ok(serde_json::json!({"ok":true,"path":path,"value":value,
                "version":crate::cli::fnv_public(&loaded.raw),"warnings":loaded.warnings}))
        });
        return match value {
            Ok(value) => json_resp(value),
            Err(e) => json_resp(serde_json::json!({"ok":false,"error":e.message(),"exit":e.exit_code()})),
        };
    }
    let result = crate::cli::run(&args);
    match result {
        Ok(code) if code == 0 => {
            let (_, _, state) = api_state(file);
            let mut resp: serde_json::Value = serde_json::from_str(&state).unwrap();
            resp["ok"] = serde_json::json!(resp.get("error").is_none());
            json_resp(resp)
        }
        Ok(code) => json_resp(serde_json::json!({"ok": false, "exit": code})),
        Err(e) => json_resp(serde_json::json!({"ok": false, "error": e.message(), "exit": e.exit_code()})),
    }
}
