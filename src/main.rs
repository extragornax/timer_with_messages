use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::State,
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct Message {
    author: String,
    text: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct NewMessage {
    author: String,
    text: String,
}

type AppState = Arc<Mutex<Vec<Message>>>;

#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        .route("/", get(index))
        .route("/send", get(send_page))
        .route("/api/messages", get(get_messages))
        .route("/api/messages", post(post_message))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    println!("Timer running at http://{addr}");
    println!("Send messages at http://{addr}/send");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn send_page() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("send.html"),
    )
}

async fn get_messages(State(state): State<AppState>) -> Json<Vec<Message>> {
    let messages = state.lock().unwrap();
    Json(messages.clone())
}

async fn post_message(
    State(state): State<AppState>,
    Json(payload): Json<NewMessage>,
) -> Json<Message> {
    let msg = Message {
        author: payload.author,
        text: payload.text,
        created_at: Utc::now(),
    };
    let mut messages = state.lock().unwrap();
    messages.push(msg.clone());
    Json(msg)
}
