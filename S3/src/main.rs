mod monitor;
mod stats;
mod  pool;
use monitor::Monitor;
use dotenvy::dotenv;
use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse},
    routing::get,
    Router,
    extract::Extension
};
use serde::Deserialize;
use std::{path::{Path}, io};
use tokio::{fs::File, io::AsyncReadExt};
use zip::write::FileOptions;
use std::io::Cursor;
use std::io::Write;
use tokio::sync::mpsc;
use crate::stats::StatEvent;
use crate::pool::spawn_stats_pool;

const DATA_DIR: &str = "data";

#[derive(Deserialize)]
struct Params {
    id: String,
    name: String,
}

async fn download_zip(Query(params): Query<Params>, 
        Extension(tx): Extension<mpsc::Sender<StatEvent>>) -> impl IntoResponse {
    let monitor = Monitor::start();
    // let _ = tx.send(StatEvent { cpu: 0.0, ram: 0, disk: 0 }).await;

    if !is_safe_name(&params.id) || !is_safe_name(&params.name) {
        let res = monitor.end();
        let _ = tx.send(res);
        return (StatusCode::BAD_REQUEST, "Invalid id or name".to_string()).into_response();
    }

    let code_path = Path::new("data").join("code").join(format!("{}.cpp", params.id));
    let test_path = Path::new("data").join("test").join(&params.name);

    let resp = match create_zip_in_memory(&code_path, &test_path).await {
        Ok(bytes) => {
            let filename = format!("{}_{}.zip", params.id, params.name);
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, "application/zip".parse().unwrap());
            headers.insert(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename)
                    .parse()
                    .unwrap(),
            );
            (headers, bytes).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Zip error: {}", e),
        )
            .into_response(),
    };

    let res = monitor.end();
    let _ = tx.send(res).await;

    resp
}

async fn create_zip_in_memory(code_path: &Path, test_path: &Path) -> io::Result<Vec<u8>> {
    let buffer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buffer);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // add code file
    if code_path.exists() {
        let mut file = File::open(code_path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        zip.start_file(
            format!("code/{}", code_path.file_name().unwrap().to_string_lossy()),
            options,
        )?;
        zip.write_all(&buf)?;
    }

    // add test folder
    if test_path.exists() {
        for entry in walkdir::WalkDir::new(test_path) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let relative = path.strip_prefix(DATA_DIR).unwrap();
                let mut file = File::open(path).await?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await?;
                zip.start_file(relative.to_string_lossy(), options)?;
                zip.write_all(&buf)?;
            }
        }
    }

    let cursor: Cursor<Vec<u8>> = zip.finish()?; 
    Ok(cursor.into_inner())
}

fn is_safe_name(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let (tx, rx) = mpsc::channel::<StatEvent>(1000);
    spawn_stats_pool(rx, 500);
    let app = Router::new()
                .route("/download", get(download_zip))
    .layer(Extension(tx.clone()));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server chạy tại http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
