# Exom

Hall-based collaboration platform. Discord's feature set without the garbage — clean core, plugin marketplace monetization.

## Architecture

```
crates/
  core/       - 19 models, 17 storage stores, permissions, events, client networking
  protocol/   - MessagePack wire protocol (25 client + 28 server message types)
  relay/      - WebSocket relay server (auth, routing, file uploads, rate limiting)
  app/        - Desktop UI (overhaul in progress — migrating to Electron + React)
```

### Core Concepts

- **Hall**: A workspace with channels, members, roles, chat, and hosting state
- **Channels**: Text, voice, announcement, stage — organized under categories
- **Roles**: Five-tier permission system (52 actions)
  - Hall Builder (Owner) — full control
  - Hall Prefect (Admin) — management
  - Hall Moderator — chat and member moderation
  - Hall Agent (Member) — standard participation
  - Hall Fellow (Guest) — limited access
- **Hosting**: Dynamic host election with epoch-based anti-split-brain
- **Hall Chest**: Local file storage per Hall (sync planned)
- **Parlors**: Plugin marketplace (planned — revenue share model)

### Networking

Hybrid relay architecture. Thin WebSocket relay routes messages between clients without storing history or running business logic.

```
[Client] ──WebSocket──> [Relay] <──WebSocket── [Client]
   │                      │                       │
[SQLite]            [Queue DB]              [SQLite]
```

- HMAC-SHA256 token auth
- MessagePack binary wire format
- Offline message queuing (7-day TTL)
- File upload with SHA-256 dedup (25 MB limit)
- WebRTC signaling for voice/video (SDP + ICE relay)
- Token bucket rate limiting
- Exponential backoff reconnection on client side

### Backend Features

| Domain | Status |
|--------|--------|
| Channels (text/voice/category/announcement/stage) | Complete |
| User profiles (avatar, bio, status, custom status) | Complete |
| DMs and group DMs | Complete |
| Friend system (requests, blocks) | Complete |
| Reactions (unicode + custom emoji) | Complete |
| Attachments and embeds | Complete |
| Threads and replies | Complete |
| Pins | Complete |
| Bans (timed + permanent) | Complete |
| Audit log (28 action types) | Complete |
| Notifications (read state, mentions, mute) | Complete |
| Custom emoji per Hall | Complete |
| Webhooks | Complete |
| Voice state tracking | Complete |
| Hall discovery and vanity invites | Complete |
| Per-channel permission overrides (bitfield) | Complete |
| Event bus (tokio broadcast) | Complete |
| Relay server | Complete |
| Wire protocol | Complete |
| Client networking | Complete |
| Desktop UI | In progress |
| E2E encryption | Planned |
| WebRTC media | Planned |
| Parlor plugin system | Planned |

## Building

### Prerequisites

- Rust 1.75+
- Linux: `sudo apt-get install libfontconfig1-dev libfreetype6-dev`

### Build

```bash
cargo build --workspace
```

### Run Relay

```bash
cargo run -p exom-relay
# or with config:
cargo run -p exom-relay -- --config relay.toml
# generate default config:
cargo run -p exom-relay -- --init
```

Relay defaults to port 9400. Set `EXOM_RELAY_SECRET` env var for production.

### Test

```bash
cargo test --workspace
```

## Development

```bash
cargo fmt --all
cargo clippy --workspace
cargo test --workspace
```

## Tech Stack

- Language: Rust
- Storage: SQLite (rusqlite)
- Networking: Axum (relay), tokio-tungstenite (client)
- Serialization: MessagePack (rmp-serde)
- Auth: Argon2 + HMAC-SHA256
- Logging: tracing
- UI: Electron + React + shadcn/ui + Tailwind (in progress)

## License

MIT
