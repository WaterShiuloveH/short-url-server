use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::{Duration, NaiveDateTime, Utc};
use rand::{distributions::Alphanumeric, Rng};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::thread;
use tokio::task;

struct AppState {
    db: Mutex<Connection>,
}

#[derive(Serialize, Deserialize)]
struct UrlRequest {
    long_url: String,
    expires_in: Option<i64>,
}

#[derive(Serialize)]
struct ShortenedUrl {
    short_url: String,
}

// Generate a random short code
fn generate_short_url() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect()
}

// Shorten URL Handler
async fn shorten_url(data: web::Data<AppState>, req: web::Json<UrlRequest>) -> impl Responder {
    let short_code = generate_short_url();
    let conn = data.db.lock().unwrap();
    let expires_at = Utc::now() + Duration::seconds(req.expires_in.unwrap_or(3 * 24 * 60 * 60));

    let result = conn.execute(
        "INSERT INTO urls (short, long, expires_at) VALUES (?1, ?2, ?3)",
        params![
            short_code,
            req.long_url,
            expires_at.format("%Y-%m-%d %H:%M:%S").to_string()
        ],
    );

    match result {
        Ok(_) => HttpResponse::Ok().json(ShortenedUrl {
            short_url: format!("http://localhost:8080/{}", short_code),
        }),
        Err(_) => HttpResponse::InternalServerError().body("Error storing URL"),
    }
}

// Redirect to Original URL
async fn redirect_url(data: web::Data<AppState>, short_code: web::Path<String>) -> impl Responder {
    let conn = data.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT long, expires_at FROM urls WHERE short = ?1")
        .unwrap();
    let mut rows = stmt.query(params![short_code.into_inner()]).unwrap();

    if let Some(row) = rows.next().unwrap() {
        let long_url: String = row.get(0).unwrap();
        let expires_at: Option<String> = row.get(1).unwrap();

        // Check expiration
        if let Some(expiry) = expires_at {
            match NaiveDateTime::parse_from_str(&expiry, "%Y-%m-%d %H:%M:%S") {
                Ok(expiry_time) => {
                    if expiry_time < Utc::now().naive_utc() {
                        return HttpResponse::Gone().body("This short URL has expired!");
                    }
                }
                Err(_) => {
                    return HttpResponse::InternalServerError().body("Error parsing expiry time");
                }
            }
        }
        HttpResponse::Found()
            .append_header(("Location", long_url))
            .finish()
    } else {
        HttpResponse::NotFound().body("URL not found")
    }
}

fn delete_expired_links(conn: &Connection) {
    conn.execute("DELETE FROM urls WHERE expires_at < datetime('now')", [])
        .unwrap();
}
// Start the Server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_clone = Connection::open("urls.db").expect("Failed to open database");
    task::spawn(async move {
        loop {
            thread::sleep(std::time::Duration::from_secs(3600)); // Runs every hour
            delete_expired_links(&db_clone);
        }
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: Mutex::new(Connection::open("urls.db").expect("Failed to open database")),
            }))
            .route("/", web::post().to(shorten_url))
            .route("/{short_code}", web::get().to(redirect_url))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
