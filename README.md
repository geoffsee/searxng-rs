# SearXNG-RS

A privacy-respecting metasearch engine written in Rust. This project is a Rust-based implementation inspired by [SearXNG](https://github.com/searxng/searxng).

**Status:** Early development - core functionality works but expect rough edges.

## Features

- **Multi-Engine Search** - Aggregates results from multiple search engines
- **Privacy-First** - No user tracking or profiling, tracker URL removal
- **Query Syntax** - Language filters, engine bangs, time ranges
- **Built-in Plugins** - Calculator, unit converter, hash generator, tracker remover
- **Autocomplete** - Search suggestions from configurable backends
- **Basic i18n** - UI translations for English, German, and French

## Supported Search Engines

| Engine | Categories |
|--------|------------|
| Google | General |
| Bing | General |
| DuckDuckGo | General |
| Brave | General |
| Wikipedia | General |
| GitHub | IT |
| Stack Overflow | IT |
| YouTube | Videos |
| arXiv | Science |

## Installation

### Prerequisites

- Rust 1.70+ (with Cargo)

### Build

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release
```

### Run

```bash
# Using cargo
cargo run --release

# Or run the binary directly
./target/release/searxng-rs
```

The server starts at `http://127.0.0.1:8888` by default.

### Docker

```bash
docker run -p 8888:8888 ghcr.io/seemueller-io/searxng-rs:latest
```

Multi-arch images (amd64/arm64) are available on GHCR.

## Configuration

The application looks for `settings.yml` in these locations (in order):

1. `$SEARXNG_SETTINGS_PATH` environment variable
2. `./settings.yml`
3. `./config/settings.yml`
4. `/etc/searxng/settings.yml`
5. `~/.config/searxng-rs/settings.yml`

### Example Configuration

```yaml
general:
  debug: false
  instance_name: "SearXNG"
  enable_metrics: true

search:
  safe_search: 0              # 0=None, 1=Moderate, 2=Strict
  autocomplete: "duckduckgo"
  default_lang: "auto"

server:
  port: 8888
  bind_address: "127.0.0.1"
  secret_key: "change-me-in-production"

engines:
  - name: google
    disabled: false
  - name: duckduckgo
    disabled: false
  - name: brave
    disabled: false
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SEARXNG_SETTINGS_PATH` | Path to settings.yml | - |
| `SEARXNG_DEBUG` | Enable debug mode | `false` |
| `SEARXNG_PORT` | Server port | `8888` |
| `SEARXNG_BIND_ADDRESS` | Bind address | `127.0.0.1` |
| `SEARXNG_SECRET_KEY` | Secret key for sessions | - |

## Query Syntax

| Syntax | Example | Description |
|--------|---------|-------------|
| `:lang` | `rust :en` | Filter by language |
| `!engine` | `rust !github` | Search specific engine |
| `!category` | `cats !images` | Search category |
| `<timeout` | `query <10` | Custom timeout (seconds) |
| `!safesearch` | `query !safesearch` | Enable safe search |
| `!nosafesearch` | `query !nosafesearch` | Disable safe search |
| `!day/week/month/year` | `news !week` | Time range filter |
| `!!` | `!! query` | Redirect to first result |

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Home page |
| `GET /search` | Search results |
| `GET /autocomplete` | Search suggestions |
| `GET /preferences` | User preferences |
| `GET /stats` | Instance statistics |
| `GET /health` | Health check |

## Project Structure

```
src/
├── engines/        # Search engine implementations
├── web/            # HTTP server and routes
├── search/         # Search orchestration
├── query/          # Query parsing
├── results/        # Result types
├── plugins/        # Built-in plugins
├── autocomplete/   # Autocomplete backends
├── config/         # Configuration
├── network/        # HTTP client
├── cache/          # Caching layer
├── locales/        # Translations
└── templates/      # HTML templates
```

## Tech Stack

- **Runtime**: Tokio
- **Web Framework**: Axum
- **HTTP Client**: Reqwest
- **Templates**: Tera
- **Caching**: Moka

## Development

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.
