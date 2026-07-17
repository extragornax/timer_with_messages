# Timer with Messages

A big-screen timer for a 24-hour endurance event. It counts down to the start,
then counts up the elapsed time, and displays encouragement messages sent by
families and friends so the runner can see them on screen.

## Features

- Countdown to the start date, then elapsed time after it passes.
- Rotating on-screen messages fetched from the server, targeted by runner
  bib number and name.
- A `/send` web form and a JSON API to add messages.
- A token-protected `/admin` page to list and delete messages.
- Selectable clock styles, shareable via URL.
- SQLite persistence and a Docker setup.

## Run

```bash
cargo run
```

Then open:

- http://localhost:3000 — the timer display
- http://localhost:3000/simulate — same display with a date field to preview any target date
- http://localhost:3000/send — form to add a message
- http://localhost:3000/admin — admin page (list and delete messages)

## Configuration

Configuration comes from environment variables, loaded from a `.env` file if
present. Copy `.env.example` to `.env` and adjust:

| Variable      | Description                                                        |
|---------------|--------------------------------------------------------------------|
| `TARGET_DATE` | Timer start (RFC 3339). Defaults to `2026-07-25T12:00:00Z`.        |
| `ADMIN_TOKEN` | Secret for the `/admin` page. If unset/empty, deletion is disabled.|

```bash
cp .env.example .env
# edit .env, then:
cargo run
```

The `/admin` page prompts for the `ADMIN_TOKEN`; it lists all messages and lets
you delete them. Deletion goes through `DELETE /api/messages/{id}` with an
`Authorization: Bearer <ADMIN_TOKEN>` header (see [API.md](API.md)).

### Docker

```bash
docker compose up --build
```

Serves on http://localhost:9018 (see `docker-compose.yml` for the port and
`TARGET_DATE`). Message data is persisted to `./data`.

## Clock styles

Pick a style from the dropdown (top-left). The choice is saved locally and
reflected in the URL as `?style=<name>`, so a link carries the style with it.
Available styles:

| Name          | Value      | Look                                   |
|---------------|------------|----------------------------------------|
| Neon          | `neon`     | Orbitron with a glowing halo (default) |
| Minimal       | `minimal`  | Thin Inter, no effects                 |
| LCD           | `lcd`      | Green segmented glow                   |
| Retro         | `retro`    | Italic gold with a drop shadow         |
| Red on Black  | `red`      | Plain red on black                     |
| LED Matrix    | `matrix`   | Red glowing 5×7 dot-matrix LEDs        |

Example shareable link: `http://localhost:3000/?style=matrix`

## API

Adding and listing messages is documented in [API.md](API.md).

## Development

```bash
cargo build            # debug build
cargo build --release  # release build
cargo test             # run tests
cargo clippy           # lint
cargo fmt              # format
```
