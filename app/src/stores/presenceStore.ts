/// Presence state store.
///
/// Tracks user online/idle/dnd/offline status and typing indicators.
/// Updated by relay events, consumed by member list and chat UI.

import { create } from 'zustand'

interface PresenceState {
  /** User status by user_id: 0=offline, 1=online, 2=idle, 3=dnd, 4=invisible */
  statuses: Record<string, number>
  /** Currently typing users by channel_id. Each entry auto-expires after 10s. */
  typing: Record<string, Set<string>>

  setStatus: (userId: string, status: number) => void
  setTyping: (channelId: string, username: string) => void
  clearTyping: (channelId: string, username: string) => void
  getTypingUsers: (channelId: string) => string[]
}

export const usePresenceStore = create<PresenceState>((set, get) => ({
  statuses: {},
  typing: {},

  setStatus: (userId, status) => {
    set((s) => ({
      statuses: { ...s.statuses, [userId]: status },
    }))
  },

  setTyping: (channelId, username) => {
    set((s) => {
      const current = new Set(s.typing[channelId])
      current.add(username)
      return { typing: { ...s.typing, [channelId]: current } }
    })

    // Auto-clear after 10 seconds
    setTimeout(() => {
      get().clearTyping(channelId, username)
    }, 10_000)
  },

  clearTyping: (channelId, username) => {
    set((s) => {
      const current = new Set(s.typing[channelId])
      current.delete(username)
      return { typing: { ...s.typing, [channelId]: current } }
    })
  },

  getTypingUsers: (channelId) => {
    return Array.from(get().typing[channelId] ?? [])
  },
}))
