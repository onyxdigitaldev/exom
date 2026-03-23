/// Core data types mirroring exom-core Rust models.
/// These match the JSON shapes returned by the sidecar API.

export interface User {
  user_id: string
  username: string
}

export interface AuthResponse {
  user_id: string
  username: string
  session_id: string
}

export type RoleKey = 'Builder' | 'Prefect' | 'Moderator' | 'Agent' | 'Fellow'

export interface Hall {
  id: string
  name: string
  description: string | null
  owner_id: string
  icon_hash: string | null
  member_count: number
}

export interface Channel {
  id: string
  hall_id: string
  name: string
  channel_type: string
  topic: string | null
  parent_id: string | null
  position: number
  nsfw: boolean
}

export interface Message {
  id: string
  sender_id: string
  sender_username: string
  sender_role: string
  content: string
  timestamp: string
  is_edited: boolean
  reply_to: string | null
  thread_id: string | null
  is_pinned: boolean
  reaction_count: number
  thread_reply_count: number
}

export interface Member {
  user_id: string
  username: string
  role: string
  is_online: boolean
  is_host: boolean
}

export interface ReactionSummary {
  emoji: string
  is_custom: boolean
  custom_emoji_id: string | null
  count: number
  me: boolean
}

export interface Invite {
  id: string
  hall_id: string
  token: string
  created_by: string
  role: string
  created_at: string
  expires_at: string | null
  max_uses: number | null
  use_count: number
}

export interface Ban {
  id: string
  user_id: string
  banned_by: string
  reason: string | null
  created_at: string
  expires_at: string | null
}

export interface AuditEntry {
  id: string
  actor_id: string
  action_type: string
  target_id: string | null
  reason: string | null
  created_at: string
}

export interface DmChannel {
  id: string
  channel_type: string
  name: string | null
  participants: string[]
  last_message_at: string | null
}

export interface UnreadChannel {
  channel_id: string
  mention_count: number
  last_read_message_id: string | null
}

export interface RelayToken {
  user_id: string
  token: string
  timestamp: string
  protocol_version: number
}

export interface UserProfile {
  user_id: string
  display_name: string | null
  avatar_hash: string | null
  bio: string | null
  status: number
  custom_status_text: string | null
  custom_status_emoji: string | null
}

export interface DiscoveryHall {
  hall_id: string
  name: string
  description: string | null
  icon_hash: string | null
  category: string | null
  member_count: number
}
