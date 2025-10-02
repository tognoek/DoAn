use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{
    fs,
    fs::File,
    io::{self, Write},
    path::Path,
};
use tokio::net::TcpListener;
use zip::write::FileOptions;

#[derive(Deserialize)]
struct Params {
    id: String,
    name: String,
}

async fn download_zip(Query(params): Query<Params>) -> impl IntoResponse {
    let code_path = format!("data/code/{}.cpp", params.id);
    let test_path = format!("data/test/{}", params.name);
    let zip_path = format!("/tmp/{}_{}.zip", params.id, params.name);

    if let Err(e) = create_zip(&zip_path, &code_path, &test_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error creating zip: {}", e),
        )
            .into_response();
    }

    match tokio::fs::read(&zip_path).await {
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
            format!("Read zip error: {}", e),
        )
            .into_response(),
    }
}

fn create_zip(zip_path: &str, code_path: &str, test_path: &str) -> io::Result<()> {
    let file = File::create(Path::new(zip_path))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // add code file
    if Path::new(code_path).exists() {
        let content = fs::read(code_path)?;
        zip.start_file(
            format!("code/{}", Path::new(code_path).file_name().unwrap().to_string_lossy()),
            options,
        )?;
        zip.write_all(&content)?;
    }

    // add test folder
    if Path::new(test_path).exists() {
        for entry in walkdir::WalkDir::new(test_path) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let relative = path.strip_prefix("data").unwrap();
                zip.start_file(relative.to_string_lossy(), options)?;
                let content = fs::read(path)?;
                zip.write_all(&content)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/download", get(download_zip));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server chạy tại http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
