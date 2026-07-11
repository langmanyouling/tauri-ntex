use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::sync::oneshot;

struct ServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

type SharedHandle = Arc<Mutex<ServerHandle>>;

use ntex::web;

async fn index() -> web::HttpResponse {
    web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>ntex</title></head>
<body style="font-family:sans-serif;text-align:center;padding-top:60px">
  <h1>Hello from ntex!</h1>
  <p>This page is served by the embedded ntex web server.</p>
</body></html>"#,
        )
}

async fn health() -> web::HttpResponse {
    let body = serde_json::json!({
        "status": "ok",
        "service": "tauri-ntex-app"
    });
    web::HttpResponse::Ok().json(&body)
}

async fn api_info() -> web::HttpResponse {
    let body = serde_json::json!({
        "endpoints": [
            { "method": "GET", "path": "/", "desc": "首页" },
            { "method": "GET", "path": "/health", "desc": "健康检查" },
            { "method": "GET", "path": "/api/info", "desc": "API 信息" }
        ]
    });
    web::HttpResponse::Ok().json(&body)
}

#[tauri::command]
fn start_ntex_server(handle: State<'_, SharedHandle>) -> Result<String, String> {
    let mut h = handle.lock().map_err(|e| e.to_string())?;

    if h.shutdown_tx.is_some() {
        return Err("服务器已经在运行中".into());
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let thread_handle = std::thread::spawn(move || {
        let sys = ntex::rt::System::new("ntex-server");

        sys.block_on(async move {
            let server = web::server(|| {
                web::App::new()
                    .service(web::resource("/").route(web::get().to(index)))
                    .service(web::resource("/health").route(web::get().to(health)))
                    .service(web::resource("/api/info").route(web::get().to(api_info)))
            })
            .disable_signals()
            .bind("127.0.0.1:9000")
            .expect("Failed to bind ntex server on 127.0.0.1:9000");

            let srv = server.run();

            let _ = tokio::select! {
                _ = srv => {},
                _ = shutdown_rx => {
                    println!("[ntex] 收到停止信号，正在关闭...");
                }
            };
        });
    });

    h.shutdown_tx = Some(shutdown_tx);
    h.thread_handle = Some(thread_handle);

    Ok("ntex 服务器已启动 — http://127.0.0.1:9000".into())
}

#[tauri::command]
fn stop_ntex_server(handle: State<'_, SharedHandle>) -> Result<String, String> {
    let mut h = handle.lock().map_err(|e| e.to_string())?;

    if let Some(tx) = h.shutdown_tx.take() {
        let _ = tx.send(());
    }

    if let Some(jh) = h.thread_handle.take() {
        let _ = jh.join();
    }

    Ok("ntex 服务器已停止".into())
}

fn main() {
    let server_handle: SharedHandle = Arc::new(Mutex::new(ServerHandle {
        shutdown_tx: None,
        thread_handle: None,
    }));

    tauri::Builder::default()
        .manage(server_handle)
        .invoke_handler(tauri::generate_handler![
            start_ntex_server,
            stop_ntex_server,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
