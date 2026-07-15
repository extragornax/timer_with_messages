# API

The server runs on `http://0.0.0.0:3000`.

## Add a message

`POST /api/messages`

Adds a message that will appear on the timer display.

### Request

- **Content-Type:** `application/json`
- **Body:**

| Field         | Type   | Description                          |
|---------------|--------|--------------------------------------|
| `author`      | string | Who sent the message                 |
| `text`        | string | The message content                  |
| `bib`         | string | Runner's bib number                  |
| `runner_name` | string | Runner's name                        |

All fields are required.

### Example

```bash
curl -X POST http://localhost:3000/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "author": "Maman",
    "text": "Tu vas y arriver, on est fiers de toi !",
    "bib": "42",
    "runner_name": "Jean"
  }'
```

### Response

`200 OK` with the created message as JSON, including a server-set `created_at` (RFC 3339 timestamp):

```json
{
  "author": "Maman",
  "text": "Tu vas y arriver, on est fiers de toi !",
  "bib": "42",
  "runner_name": "Jean",
  "created_at": "2026-07-25T10:15:30.123456+00:00"
}
```

## List messages

`GET /api/messages`

Returns all messages ordered by insertion, as a JSON array of the same shape as above.

## Web form

A browser form to add messages is available at `GET /send`.
