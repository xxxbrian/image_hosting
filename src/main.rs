use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    routing::post,
    Router,
};
use reqwest::multipart as reqwest_multipart;
use std::net::SocketAddr;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_appender;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    let app = Router::new()
        .route("/", get(serve_static))
        .route("/upload", post(upload))
        .route("/file/:fileName", get(file))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn serve_static() -> impl IntoResponse {
    // println!("Serving static file");
    // match fs::read_to_string("public/index.html") {
    //     Ok(contents) => Html(contents).into_response(),
    //     Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Unable to read index.html").into_response(),
    // }
    Html(include_str!("../public/index.html"))
}

async fn upload(mut multipart: Multipart) -> impl IntoResponse {
    let host = "https://telegra.ph";
    let client = reqwest::Client::new();

    if let Some(field) = multipart.next_field().await.unwrap() {
        let file = field.bytes().await.unwrap();

        let form = reqwest_multipart::Form::new().part(
            "file",
            reqwest_multipart::Part::stream(file).file_name("rust.png"),
        );

        match client
            .post(format!("{}/upload", host))
            .multipart(form)
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(text) => text.into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            },
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

async fn file(Path(file_name): Path<String>) -> impl IntoResponse {
    let host = "https://telegra.ph";
    let client = reqwest::Client::new();

    match client
        .get(format!("{}/file/{}", host, file_name))
        .send()
        .await
    {
        Ok(response) => match response.bytes().await {
            Ok(bytes) => bytes.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
