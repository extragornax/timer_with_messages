use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::State,
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct Message {
    author: String,
    text: String,
    bib: String,
    runner_name: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct NewMessage {
    author: String,
    text: String,
    bib: String,
    runner_name: String,
}

type Db = Arc<Mutex<Connection>>;

fn init_db(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            author TEXT NOT NULL,
            text TEXT NOT NULL,
            bib TEXT NOT NULL,
            runner_name TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .unwrap();
}

#[tokio::main]
async fn main() {
    let conn = Connection::open("messages.db").unwrap();
    init_db(&conn);
    let db: Db = Arc::new(Mutex::new(conn));

    let app = Router::new()
        .route("/", get(index))
        .route("/send", get(send_page))
        .route("/api/messages", get(get_messages))
        .route("/api/messages", post(post_message))
        .with_state(db);

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

async fn get_messages(State(db): State<Db>) -> Json<Vec<Message>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT author, text, bib, runner_name, created_at FROM messages ORDER BY id")
        .unwrap();
    let messages = stmt
        .query_map([], |row| {
            let created_at_str: String = row.get(4)?;
            Ok(Message {
                author: row.get(0)?,
                text: row.get(1)?,
                bib: row.get(2)?,
                runner_name: row.get(3)?,
                created_at: created_at_str.parse::<DateTime<Utc>>().unwrap(),
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    Json(messages)
}

async fn post_message(State(db): State<Db>, Json(payload): Json<NewMessage>) -> Json<Message> {
    let now = Utc::now();
    let conn = db.lock().unwrap();
    conn.execute(
        "INSERT INTO messages (author, text, bib, runner_name, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&payload.author, &payload.text, &payload.bib, &payload.runner_name, &now.to_rfc3339()),
    )
    .unwrap();
    Json(Message {
        author: payload.author,
        text: payload.text,
        bib: payload.bib,
        runner_name: payload.runner_name,
        created_at: now,
    })
}
