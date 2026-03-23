/// HTTP client for the exom-sidecar API.
///
/// All methods return typed responses matching the sidecar's JSON output.
/// The sidecar runs on localhost:9401 — not exposed to the network.

import type {
  AuthResponse,
  Ban,
  AuditEntry,
  Channel,
  DiscoveryHall,
  DmChannel,
  Hall,
  Invite,
  Member,
  Message,
  ReactionSummary,
  RelayToken,
  UnreadChannel,
  User,
  UserProfile,
} from './types'

const BASE = 'http://localhost:9401'

/** Typed fetch wrapper with error handling. */
async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(body.error || `Request failed: ${res.status}`)
  }

  // 204 No Content
  if (res.status === 204) return undefined as T

  return res.json()
}

// ── Auth ──────────────────────────────────────────────

export const auth = {
  login: (username: string, password: string) =>
    request<AuthResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  register: (username: string, password: string) =>
    request<AuthResponse>('/api/auth/register', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    }),

  logout: () =>
    request<void>('/api/auth/logout', { method: 'POST' }),

  me: () =>
    request<User>('/api/auth/me'),
}

// ── Halls ─────────────────────────────────────────────

export const halls = {
  list: () =>
    request<Hall[]>('/api/halls'),

  get: (id: string) =>
    request<Hall>(`/api/halls/${id}`),

  create: (name: string, description?: string) =>
    request<Hall>('/api/halls', {
      method: 'POST',
      body: JSON.stringify({ name, description }),
    }),

  leave: (id: string) =>
    request<void>(`/api/halls/${id}/leave`, { method: 'DELETE' }),
}

// ── Channels ──────────────────────────────────────────

export const channels = {
  list: (hallId: string) =>
    request<Channel[]>(`/api/halls/${hallId}/channels`),

  create: (hallId: string, name: string, channelType: string, topic?: string, parentId?: string) =>
    request<Channel>(`/api/halls/${hallId}/channels`, {
      method: 'POST',
      body: JSON.stringify({
        name,
        channel_type: channelType,
        topic,
        parent_id: parentId,
      }),
    }),

  delete: (id: string) =>
    request<void>(`/api/channels/${id}`, { method: 'DELETE' }),
}

// ── Messages ──────────────────────────────────────────

export const messages = {
  list: (hallId: string, channelId: string, limit = 50, before?: string) => {
    const params = new URLSearchParams({ limit: String(limit) })
    if (before) params.set('before', before)
    return request<Message[]>(
      `/api/halls/${hallId}/channels/${channelId}/messages?${params}`,
    )
  },

  send: (hallId: string, channelId: string, content: string, replyTo?: string) =>
    request<Message>(`/api/halls/${hallId}/channels/${channelId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content, reply_to: replyTo }),
    }),

  edit: (id: string, content: string) =>
    request<void>(`/api/messages/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ content }),
    }),

  delete: (id: string) =>
    request<void>(`/api/messages/${id}`, { method: 'DELETE' }),

  pin: (id: string) =>
    request<void>(`/api/messages/${id}/pin`, { method: 'POST' }),

  unpin: (id: string) =>
    request<void>(`/api/messages/${id}/pin`, { method: 'DELETE' }),
}

// ── Members ───────────────────────────────────────────

export const members = {
  list: (hallId: string) =>
    request<Member[]>(`/api/halls/${hallId}/members`),

  updateRole: (hallId: string, userId: string, role: string) =>
    request<void>(`/api/halls/${hallId}/members/${userId}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role }),
    }),

  kick: (hallId: string, userId: string) =>
    request<void>(`/api/halls/${hallId}/members/${userId}`, { method: 'DELETE' }),
}

// ── Invites ───────────────────────────────────────────

export const invites = {
  create: (hallId: string, role?: string, expiryHours?: number, maxUses?: number) =>
    request<Invite>(`/api/halls/${hallId}/invites`, {
      method: 'POST',
      body: JSON.stringify({ role, expiry_hours: expiryHours, max_uses: maxUses }),
    }),

  list: (hallId: string) =>
    request<Invite[]>(`/api/halls/${hallId}/invites`),

  accept: (token: string) =>
    request<void>(`/api/invites/${token}/accept`, { method: 'POST' }),

  revoke: (id: string) =>
    request<void>(`/api/invites/${id}`, { method: 'DELETE' }),
}

// ── Profiles ──────────────────────────────────────────

export const profiles = {
  get: (userId: string) =>
    request<UserProfile>(`/api/profiles/${userId}`),

  update: (data: { display_name?: string; bio?: string; status?: number }) =>
    request<void>('/api/profiles/me', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
}

// ── DMs ───────────────────────────────────────────────

export const dms = {
  create: (targetUserId: string) =>
    request<DmChannel>('/api/dms', {
      method: 'POST',
      body: JSON.stringify({ target_user_id: targetUserId }),
    }),

  list: () =>
    request<DmChannel[]>('/api/dms'),

  listMessages: (channelId: string, limit = 50, before?: string) => {
    const params = new URLSearchParams({ limit: String(limit) })
    if (before) params.set('before', before)
    return request<Message[]>(`/api/dms/${channelId}/messages?${params}`)
  },

  send: (channelId: string, content: string) =>
    request<void>(`/api/dms/${channelId}/messages`, {
      method: 'POST',
      body: JSON.stringify({ content }),
    }),
}

// ── Reactions ─────────────────────────────────────────

export const reactions = {
  add: (messageId: string, emoji: string) =>
    request<void>(`/api/messages/${messageId}/reactions`, {
      method: 'POST',
      body: JSON.stringify({ emoji }),
    }),

  remove: (messageId: string, emoji: string) =>
    request<void>(`/api/messages/${messageId}/reactions/${encodeURIComponent(emoji)}`, {
      method: 'DELETE',
    }),

  list: (messageId: string) =>
    request<ReactionSummary[]>(`/api/messages/${messageId}/reactions`),
}

// ── Moderation ────────────────────────────────────────

export const moderation = {
  ban: (hallId: string, userId: string, reason?: string, durationHours?: number) =>
    request<Ban>(`/api/halls/${hallId}/bans`, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, reason, duration_hours: durationHours }),
    }),

  unban: (hallId: string, userId: string) =>
    request<void>(`/api/halls/${hallId}/bans/${userId}`, { method: 'DELETE' }),

  listBans: (hallId: string) =>
    request<Ban[]>(`/api/halls/${hallId}/bans`),

  auditLog: (hallId: string, limit = 50) =>
    request<AuditEntry[]>(`/api/halls/${hallId}/audit-log?limit=${limit}`),
}

// ── Notifications ─────────────────────────────────────

export const notifications = {
  markRead: (channelId: string, messageId: string) =>
    request<void>(`/api/channels/${channelId}/read`, {
      method: 'POST',
      body: JSON.stringify({ message_id: messageId }),
    }),

  listUnread: () =>
    request<UnreadChannel[]>('/api/notifications/unread'),
}

// ── Discovery ─────────────────────────────────────────

export const discovery = {
  search: (query?: string, category?: string, limit = 20) => {
    const params = new URLSearchParams({ limit: String(limit) })
    if (query) params.set('q', query)
    if (category) params.set('category', category)
    return request<DiscoveryHall[]>(`/api/discovery?${params}`)
  },
}

// ── Relay ─────────────────────────────────────────────

export const relay = {
  getToken: () =>
    request<RelayToken>('/api/relay/token', { method: 'POST' }),
}
