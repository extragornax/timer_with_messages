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
| `profile_url` | string | Runner's avatar URL (optional)       |
| `team_name`   | string | Runner's team (optional)             |
| `team_emoji`  | string | Emoji shown before the team (optional) |

`author`, `text`, `bib` and `runner_name` are required; the rest may be
omitted or `null`.

The display shows `<avatar> <runner_name> - <team_emoji> <team_name> (<bib>)`.
Without `profile_url` the avatar falls back to the runner's initials on a
colour derived from their name; without `team_name` the team is left out
entirely, which is what solo entrants get.

### Example

```bash
curl -X POST http://localhost:3000/api/messages \
  -H "Content-Type: application/json" \
  -d '{
    "author": "Maman",
    "text": "Tu vas y arriver, on est fiers de toi !",
    "bib": "42",
    "runner_name": "Jean",
    "profile_url": "https://dgalywyr863hv.cloudfront.net/pictures/athletes/1/2/3/large.jpg",
    "team_name": "Trianon",
    "team_emoji": "🤘"
  }'
```

### Response

`200 OK` with the created message as JSON, including a server-set `id` and
`created_at` (RFC 3339 timestamp):

```json
{
  "id": 1,
  "author": "Maman",
  "text": "Tu vas y arriver, on est fiers de toi !",
  "bib": "42",
  "runner_name": "Jean",
  "profile_url": "https://dgalywyr863hv.cloudfront.net/pictures/athletes/1/2/3/large.jpg",
  "team_name": "Trianon",
  "team_emoji": "🤘",
  "created_at": "2026-07-25T10:15:30.123456+00:00"
}
```

## List messages

`GET /api/messages`

Returns all messages ordered by insertion, as a JSON array of the same shape as above.

## Delete a message

`DELETE /api/messages/{id}`

Deletes a single message by `id`. **Requires authentication.**

- **Header:** `Authorization: Bearer <ADMIN_TOKEN>`
- `ADMIN_TOKEN` is read from the environment (see `.env.example`). If it is
  unset or empty, all delete requests are rejected.

### Responses

| Status           | Meaning                          |
|------------------|----------------------------------|
| `204 No Content` | Deleted                          |
| `401 Unauthorized` | Missing/invalid token          |
| `404 Not Found`  | No message with that `id`        |

### Example

```bash
curl -X DELETE http://localhost:3000/api/messages/1 \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

## Pages

- `GET /` — timer display.
- `GET /simulate` — same display with a date field that overrides the target date (client-side preview only).
- `GET /send` — browser form to add a message.
- `GET /admin` — admin page: log in with `ADMIN_TOKEN` to list and delete messages.
