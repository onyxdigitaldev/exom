# Exom Architecture

## Overview

Exom is a Hall-based collaboration platform — a Discord alternative with a clean core and plugin marketplace monetization. Built in Rust with a hybrid relay networking model.

## Crate Structure

```
exom/
├── crates/
│   ├── core/           # Core library (7,756 lines)
│   │   ├── models/     # 19 data models
│   │   ├── permissions/# Permission system (52 actions)
│   │   ├── hosting/    # Host election
│   │   ├── storage/    # 17 SQLite storage stores
│   │   ├── chest/      # Local file management
│   │   ├── events/     # Event bus (tokio broadcast)
│   │   └── network/    # Relay client (WebSocket + reconnect)
│   ├── protocol/       # Wire protocol (645 lines)
│   │   ├── messages    # 25 client + 28 server message types
│   │   └── codec       # MessagePack encode/decode
│   ├── relay/          # Relay server binary (2,024 lines)
│   │   ├── auth        # HMAC-SHA256 token validation
│   │   ├── connection  # WebSocket lifecycle
│   │   ├── router      # Message routing
│   │   ├── queue       # Offline message queue (SQLite)
│   │   ├── files       # File upload/download (HTTP)
│   │   ├── ratelimit   # Token bucket rate limiter
│   │   ├── config      # TOML config + env overrides
│   │   └── state       # Shared state (connections, halls, voice, DMs)
│   └── app/            # Desktop UI (Electron + React, in progress)
```

## Core Concepts

### Hall

A Hall is the primary workspace. It contains:
- Channels (text, voice, announcement, stage) organized by categories
- Members with assigned roles
- Chat messages with threads, reactions, pins, attachments
- Hosting state with election epochs
- Hall Chest (local file storage)
- Custom emoji
- Webhooks
- Discovery listing and vanity invite

### Models (19)

| Model | Purpose |
|-------|---------|
| Hall | Workspace with icon/banner/splash |
| Channel | Text/voice/category/announcement/stage |
| User + Session | Auth with Argon2 + 1-week sessions |
| UserProfile | Display name, avatar, bio, status |
| Membership + MemberInfo | User-Hall relationship with role |
| Message + MessageDisplay | Chat with reply_to, thread_id, pins |
| Invite | Token-based with expiry and use limits |
| DmChannel + DirectMessage | 1-on-1 and group DMs |
| Relationship | Friends, blocks, pending requests |
| Reaction | Unicode + custom emoji on messages |
| Attachment + Embed | Files and rich link previews |
| Ban | Timed or permanent, with reason |
| AuditLogEntry | 28 action types with JSON changes |
| ReadState + Mention + NotificationSettings | Per-channel unreads and mute |
| CustomEmoji | Per-Hall, static or animated |
| Webhook | Per-channel with token-based posting |
| VoiceState | Per-user mute/deaf/stream/video |
| HallDiscovery + VanityInvite | Server browser + custom slugs |
| PermissionOverride | Per-channel bitfield allow/deny |

### Roles

Five-tier hierarchy with 52 permission actions:

| Role | Level | Key Powers |
|------|-------|------------|
| Hall Builder | 5 | Owner — delete hall, transfer ownership |
| Hall Prefect | 4 | Admin — manage roles, ban, hall settings |
| Hall Moderator | 3 | Moderate — kick, delete messages, invite, manage emoji/webhooks |
| Hall Agent | 2 | Member — host, chest access, voice, chat |
| Hall Fellow | 1 | Guest — view and chat only |

### Hosting

Dynamic host election determines which member coordinates Hall activities.

- First eligible member (Agent+) entering an empty Hall becomes host
- Higher-role members joining receive takeover prompt
- Host leaving triggers cascade election to next highest-priority online member
- `election_epoch` counter prevents split-host scenarios

### Hall Chest

Local folder storage per Hall:
```
~/.local/share/dev.onyx.exom/chests/{hall-id}/
├── shared/
├── personal/
└── downloads/
```
Agent+ roles only. ChestSync trait defined for future sync implementation.

## Networking

### Hybrid Relay Architecture

```
[Client A] ──WebSocket──> [Exom Relay] <──WebSocket── [Client B]
     │                        │                            │
  [SQLite]              [SQLite queue]                 [SQLite]
                     (offline msgs only)
```

The relay is thin and stateless (except the offline queue). It does NOT store message history or run business logic. Each client keeps its own SQLite database.

### What the Relay Does
- Routes messages between Hall members via WebSocket
- Tracks presence (online/offline)
- Queues messages for offline users (7-day TTL)
- Forwards WebRTC signaling for voice/video
- Hosts file upload/download (SHA-256 dedup, 25 MB limit)
- Rate limiting (token bucket per user per action)

### Wire Protocol
- Binary MessagePack format (compact, fast)
- HMAC-SHA256 token auth (5-minute window)
- 30-second heartbeat keepalive
- 90-second disconnect timeout
- 1 MB max message size
- Protocol version field for future compatibility

### Client Networking
- `RelayClient` with exponential backoff reconnection (1s → 30s cap)
- Auto-drains offline queue on reconnect
- `EventBus` trait with `LocalEventBus` (tokio broadcast) for dispatching real-time events
- Convenience methods: `join_hall()`, `send_message()`, `start_typing()`, `join_voice()`

## Permission Matrix (Expanded)

| Action | Builder | Prefect | Moderator | Agent | Fellow |
|--------|---------|---------|-----------|-------|--------|
| Delete Hall | Y | N | N | N | N |
| Edit Settings | Y | Y | N | N | N |
| Transfer Ownership | Y | N | N | N | N |
| Create/Edit Channel | Y | Y | Y | N | N |
| Delete Channel | Y | Y | N | N | N |
| Invite Members | Y | Y | Y | N | N |
| Kick Members | Y | Y | Y | N | N |
| Ban Members | Y | Y | N | N | N |
| Change Roles | Y | Y | N | N | N |
| Send Messages | Y | Y | Y | Y | Y |
| Delete Others' Messages | Y | Y | Y | N | N |
| Pin Messages | Y | Y | Y | N | N |
| Add Reactions | Y | Y | Y | Y | Y |
| Manage Reactions | Y | Y | Y | N | N |
| Create Threads | Y | Y | Y | Y | N |
| Become Host | Y | Y | Y | Y | N |
| View/Write Chest | Y | Y | Y | Y | N |
| Connect Voice | Y | Y | Y | Y | N |
| Mute/Deafen Others | Y | Y | Y | N | N |
| @everyone/@here | Y | Y | Y | N | N |
| Manage Emoji | Y | Y | Y | N | N |
| Create Webhooks | Y | Y | Y | N | N |
| Channel Perm Overrides | Y | Y | N | N | N |
| Activate Parlor | Y | Y | N | N | N |

## Data Storage

SQLite database with 23 tables:

**Core**: users, sessions, halls, memberships, channels, messages, invites
**Social**: dm_channels, dm_participants, direct_messages, relationships
**Engagement**: reactions, attachments, embeds, mentions, custom_emoji
**Moderation**: bans, audit_log
**State**: read_states, notification_settings, voice_states
**Discovery**: hall_discovery, vanity_invites
**Permissions**: permission_overrides

Schema version 3. Migration system with version tracking.

## Future

### Parlors (Plugin Marketplace)
Plugin modules that extend Hall functionality. Third-party developers build plugins, charge for them, Exom takes 50% revenue share (Roblox model).

```rust
pub trait ParlorModule: Send + Sync {
    fn parlor_type_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn on_activate(&mut self, hall_id: Uuid);
    fn on_deactivate(&mut self, hall_id: Uuid);
}
```

### E2E Encryption
Relay is already a dumb pipe — messages can be wrapped with encryption without relay changes.

### WebRTC Media
Signaling infrastructure is complete (SDP + ICE relay). Actual audio/video codec integration is next.
