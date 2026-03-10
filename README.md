# URL Shortener — Rust

A high-throughput URL shortener built in Rust, designed to handle **100 million URLs per day** (~11,600 read RPS, ~1,160 write RPS). Built as a performance challenge to test Rust's viability as a production backend for extreme read workloads.

## Results

Load tested on **Windows Docker Desktop** (no bare-metal advantage):

| Metric        | Result                                  |
| ------------- | --------------------------------------- |
| Throughput    | **18,745 RPS**                          |
| P50 latency   | 2.6 ms                                  |
| P99 latency   | 5.1 ms                                  |
| P99.9 latency | 6.2 ms                                  |
| Success rate  | 100%                                    |
| Test duration | 30 s, 50 concurrent connections         |
| Response      | `301 Moved Permanently` (562,520 total) |

**Target was ~11,600 RPS — achieved 1.6× over target.**

Test command:

```
oha -z 30s -c 50 --no-tui --redirect 0 http://localhost:3000/{shortcode}
```

## Architecture

```
POST /shorten                                         GET /{shortcode}
      │                                                      │
      ▼                                                      ▼
  Validate URL                                        Redis cache lookup
  (http/https only)                                         │
      │                                              hit ───┤─── miss
      ▼                                               │         │
  Redis INCR (counter)                                │    ScyllaDB lookup
      │                                               │         │
      ▼                                               │    Redis SET (backfill)
  Hashids encode → shortcode                          │
      │                                               ▼
      ▼                                         301 Redirect
  ScyllaDB INSERT IF NOT EXISTS
      │
      ▼
  Redis SET EX 86400 (warm cache)
      │
      ▼
  201 { short_url, shortcode }
```

**Write path:** Redis `INCR` generates a monotonically increasing integer ID → Hashids encodes it to a 7-char alphanumeric shortcode → persisted to ScyllaDB, cached in Redis with 24h TTL.

**Read path:** Cache-first. Redis hit returns immediately (sub-millisecond). Cache miss falls through to ScyllaDB and back-fills the cache.

## Technology Stack

| Layer              | Technology                                   |
| ------------------ | -------------------------------------------- |
| Language           | Rust 2024 edition                            |
| Web framework      | axum 0.8.6                                   |
| Async runtime      | tokio 1.48 (full)                            |
| Persistent storage | ScyllaDB 6.2 (Cassandra-compatible)          |
| Counter            | Redis 7 (`INCR`)                             |
| Read cache         | Redis 7 (`SET EX 86400`, `allkeys-lru`)      |
| Redis client       | fred 10.1.0                                  |
| ScyllaDB client    | scylla 1.3.1                                 |
| Shortcode encoding | harsh 0.2.2 (Hashids)                        |
| Middleware         | tower-http 0.6.6 (tracing, timeout, gzip)    |
| Rate limiting      | tower_governor 0.8.0 (token bucket per IP)   |
| Observability      | tracing + tracing-subscriber                 |
| Container build    | clux/muslrust → scratch (musl static binary) |

## Requirements

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (or Docker Engine + Compose on Linux)
- No Rust toolchain required — the build runs entirely inside Docker

For load testing:

- [oha](https://github.com/hatoo/oha) — `cargo install oha` or download a binary release

## Running

**1. Clone and configure**

```bash
git clone https://github.com/DevShiba/url-shortener-rust
cd url-shortener-rust
```

Copy `.env` and adjust if needed (defaults work out of the box for local Docker):

```bash
# .env — no changes needed for local dev
SERVER_PORT=3000
SHORT_DOMAIN=http://localhost:3000
SCYLLA_NODES=scylladb:9042
SCYLLA_KEYSPACE=url_shortener
REDIS_COUNTER_URL=redis://redis-counter:6379
REDIS_CACHE_URL=redis://redis-cache:6379
HASHIDS_SALT=change-me-in-production-use-a-long-random-secret
HASHIDS_MIN_LENGTH=7
```

> **Important:** Change `HASHIDS_SALT` to a long random secret before deploying to production.

**2. Build and start the full stack**

```bash
docker-compose up --build
```

This starts:

- ScyllaDB (waits for healthy status before continuing)
- Two Redis instances (counter + cache)
- A migrator container that runs `migrations/init.cql`
- The Rust backend on port 3000

First startup takes a few minutes — ScyllaDB needs ~90 seconds to become healthy.

**3. Verify it's running**

```bash
curl -s http://localhost:3000/shorten  # should return 422/405 (not 502)
```

## Testing

**Shorten a URL:**

```bash
# Linux / macOS
curl -s -X POST http://localhost:3000/shorten \
  -H "Content-Type: application/json" \
  -d '{"long_url": "https://www.example.com/some/long/path"}' | jq

# Windows PowerShell
Invoke-RestMethod -Method Post -Uri http://localhost:3000/shorten `
  -ContentType "application/json" `
  -Body '{"long_url": "https://www.example.com/some/long/path"}'
```

Response:

```json
{
  "short_url": "http://localhost:3000/aLqBVqR",
  "shortcode": "aLqBVqR"
}
```

**Follow a redirect:**

```bash
# Linux / macOS (curl follows 301 by default with -L)
curl -L http://localhost:3000/aLqBVqR

# Windows PowerShell
Invoke-RestMethod -Uri http://localhost:3000/aLqBVqR
```

**Inspect the redirect without following it:**

```bash
curl -v --max-redirs 0 http://localhost:3000/aLqBVqR
# HTTP/1.1 301 Moved Permanently
# location: https://www.example.com/some/long/path
```

## Load Testing

Install oha:

```bash
cargo install oha
```

First, shorten a URL and capture the shortcode:

```bash
# Linux / macOS
SHORTCODE=$(curl -s -X POST http://localhost:3000/shorten \
  -H "Content-Type: application/json" \
  -d '{"long_url": "https://www.example.com"}' | jq -r '.shortcode')

oha -z 30s -c 50 --no-tui --redirect 0 "http://localhost:3000/$SHORTCODE"
```

```powershell
# Windows PowerShell
$r = Invoke-RestMethod -Method Post -Uri http://localhost:3000/shorten `
     -ContentType "application/json" `
     -Body '{"long_url": "https://www.example.com"}'

oha -z 30s -c 50 --no-tui --redirect 0 "http://localhost:3000/$($r.shortcode)"
```

> `--redirect 0` tells oha not to follow the `301`. Without it, oha chases the redirect to `example.com` and measures that roundtrip instead.

## Project Structure

```
src/
├── main.rs           # Bootstrap: connections, tracing, axum::serve
├── routes.rs         # AppState, router assembly, middleware layers
├── models.rs         # Request/response types (ShortenRequest, ShortenResponse)
├── errors.rs         # AppError with thiserror, IntoResponse impl
├── config.rs         # Config struct loaded from environment
├── handlers/
│   ├── shorten.rs    # POST /shorten
│   └── redirect.rs   # GET /{shortcode}
├── db/
│   └── scylla.rs     # ScyllaDB connection pool, UrlRepository
├── cache/
│   └── redis.rs      # Redis clients (counter + cache), CacheStore
└── utils/
    └── shortener.rs  # Hashids wrapper
migrations/
└── init.cql          # Keyspace + table DDL
Dockerfile            # clux/muslrust builder → scratch final image
docker-compose.yml    # Full local stack
```

## Security

- **Input validation:** `long_url` must be a valid `http://` or `https://` URL. All other schemes (e.g. `javascript:`, `file://`, `ftp://`) are rejected with `422 Unprocessable Entity`, preventing SSRF and open redirect abuse.
- **No open redirects:** Only URLs persisted by the service itself are ever served as redirect targets.
- **Rate limiting:** `POST /shorten` is limited to 10 requests/second per IP with a burst of 20 (token bucket via `tower_governor`). Excess requests receive `429 Too Many Requests`.
- **Static binary in `scratch`:** The final Docker image contains only the binary and TLS certificates — no shell, no package manager, minimal attack surface.
- **Non-root user:** The container runs as uid `10001` (non-root).

## ScyllaDB Schema

```cql
CREATE KEYSPACE IF NOT EXISTS url_shortener
    WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

CREATE TABLE IF NOT EXISTS url_shortener.url (
    shortcode  TEXT PRIMARY KEY,
    long_url   TEXT,
    created_at TIMESTAMP
);
```

Partition key = `shortcode` → every redirect lookup is a **single-partition read (O(1))**, regardless of dataset size. For production, use `NetworkTopologyStrategy` with a replication factor per datacenter.
