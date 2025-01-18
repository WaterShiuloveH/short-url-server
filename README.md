# URL Shortener Service

This is a simple URL shortener service built using Rust with the `actix-web` framework. It allows you to shorten long URLs and redirect users using a generated short code.

## Features

- Generate a short URL for a given long URL with the expiration time (3 days).
- Redirect users to the original URL using the short URL.
- Automatically delete expired URLs from the database.
- SQLite database for persistence.

## Requirements

- Rust (latest stable version)
- SQLite
- `cargo` (Rust package manager)

## Getting Started

### 1. Clone the Repository

```bash
git clone <repository-url>
cd url-shortener
```

### 2. Install Dependencies

Ensure you have the required dependencies specified in Cargo.toml.

### 3. Create the Database

Create a SQLite database named urls.db in the project root:

```bash
sqlite3 urls.db
```

Inside the SQLite shell, create the required table:

```bash
CREATE TABLE urls (
    id INTEGER PRIMARY KEY,
    short TEXT UNIQUE NOT NULL,
    long TEXT NOT NULL,
    expires_at TEXT
);
```

Exit the shell:

```bash
.exit
```

### 4. Run the Application

Run the server using cargo:

```bash
cargo run
```

The server will start on http://127.0.0.1:8080.

## API Endpoints

### 1. Shorten URL

Endpoint: POST /
Request Body:

```bash
{
  "long_url": "https://example.com",
}
```

- long_url: The URL to shorten.

Response:

```bash
{
  "short_url": "http://localhost:8080/abc123"
}
```
### 2. Redirect URL
Endpoint: GET /{short_code}

Redirects to the original URL if the short code is valid and not expired.
Returns 410 Gone if the URL is expired.

### 3. Automatic Cleanup
Expired URLs are automatically deleted every hour.