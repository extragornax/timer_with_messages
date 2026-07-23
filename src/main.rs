use std::env;
use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct Message {
    id: i64,
    author: String,
    text: String,
    bib: String,
    runner_name: String,
    /// Avatar of the runner the message is addressed to, supplied by the
    /// sender (the Strava picture from the registration site). Absent when the
    /// runner has no Strava picture; the display falls back to initials.
    profile_url: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct NewMessage {
    author: String,
    text: String,
    bib: String,
    runner_name: String,
    #[serde(default)]
    profile_url: Option<String>,
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
            profile_url TEXT,
            created_at TEXT NOT NULL
        )",
    )
    .unwrap();

    // Databases created before profile_url existed keep their old schema, so
    // add the column in place rather than losing the messages already stored.
    let has_profile_url = conn
        .prepare("SELECT 1 FROM pragma_table_info('messages') WHERE name = 'profile_url'")
        .unwrap()
        .exists([])
        .unwrap();
    if !has_profile_url {
        conn.execute("ALTER TABLE messages ADD COLUMN profile_url TEXT", [])
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let conn = Connection::open("messages.db").unwrap();
    init_db(&conn);
    let db: Db = Arc::new(Mutex::new(conn));

    let app = Router::new()
        .route("/", get(index))
        .route("/simulate", get(simulate))
        .route("/send", get(send_page))
        .route("/admin", get(admin_page))
        .route("/api/messages", get(get_messages))
        .route("/api/messages", post(post_message))
        .route("/api/messages/{id}", delete(delete_message))
        .with_state(db);

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("Timer running at http://{addr}");
    println!("Send messages at http://{addr}/send");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn render_index(simulate: bool) -> Html<String> {
    let target = env::var("TARGET_DATE").unwrap_or_else(|_| "2026-07-25T12:00:00Z".to_string());
    let html = include_str!("index.html")
        .replace("{{TARGET_DATE}}", &target)
        .replace("{{SIMULATE}}", if simulate { "true" } else { "false" });
    Html(html)
}

async fn index() -> Html<String> {
    render_index(false)
}

async fn simulate() -> Html<String> {
    render_index(true)
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
        .prepare(
            "SELECT id, author, text, bib, runner_name, profile_url, created_at
             FROM messages ORDER BY id",
        )
        .unwrap();
    let messages = stmt
        .query_map([], |row| {
            let created_at_str: String = row.get(6)?;
            Ok(Message {
                id: row.get(0)?,
                author: row.get(1)?,
                text: row.get(2)?,
                bib: row.get(3)?,
                runner_name: row.get(4)?,
                profile_url: row.get(5)?,
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
        "INSERT INTO messages (author, text, bib, runner_name, profile_url, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &payload.author,
            &payload.text,
            &payload.bib,
            &payload.runner_name,
            &payload.profile_url,
            &now.to_rfc3339(),
        ),
    )
    .unwrap();
    Json(Message {
        id: conn.last_insert_rowid(),
        author: payload.author,
        text: payload.text,
        bib: payload.bib,
        runner_name: payload.runner_name,
        profile_url: payload.profile_url,
        created_at: now,
    })
}

/// Returns true only when `ADMIN_TOKEN` is set, non-empty, and the request
/// carries a matching `Authorization: Bearer <token>` header.
fn check_auth(headers: &HeaderMap) -> bool {
    let token = match env::var("ADMIN_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return false,
    };
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v == token)
        .unwrap_or(false)
}

async fn admin_page() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        include_str!("admin.html"),
    )
}

async fn delete_message(
    State(db): State<Db>,
    Path(id): Path<i64>,
    headers: HeaderMap,
) -> StatusCode {
    if !check_auth(&headers) {
        return StatusCode::UNAUTHORIZED;
    }
    let conn = db.lock().unwrap();
    let affected = conn
        .execute("DELETE FROM messages WHERE id = ?1", [id])
        .unwrap();
    if affected == 0 {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::NO_CONTENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn with_auth(value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(header::AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn check_auth_gate() {
        // No token configured => always deny, even with a bearer header.
        unsafe { env::remove_var("ADMIN_TOKEN") };
        assert!(!check_auth(&with_auth("Bearer anything")));

        // Empty token => deny.
        unsafe { env::set_var("ADMIN_TOKEN", "") };
        assert!(!check_auth(&with_auth("Bearer ")));

        // Configured token: only an exact matching bearer is accepted.
        unsafe { env::set_var("ADMIN_TOKEN", "s3cret") };
        assert!(check_auth(&with_auth("Bearer s3cret")));
        assert!(!check_auth(&with_auth("Bearer wrong")));
        assert!(!check_auth(&with_auth("s3cret"))); // missing "Bearer " prefix
        assert!(!check_auth(&HeaderMap::new())); // no header at all
    }
}
