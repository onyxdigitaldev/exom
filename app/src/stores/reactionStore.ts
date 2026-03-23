/// Reaction state store.
///
/// Caches reaction summaries per message. Handles add/remove
/// with optimistic updates for instant UI feedback.

import { create } from 'zustand'
import { reactions as reactionsApi } from '@/lib/api'
import type { ReactionSummary } from '@/lib/types'

interface ReactionState {
  /** Reaction summaries keyed by message_id. */
  reactions: Record<string, ReactionSummary[]>

  loadReactions: (messageId: string) => Promise<void>
  addReaction: (messageId: string, emoji: string) => Promise<void>
  removeReaction: (messageId: string, emoji: string) => Promise<void>
  /** Called when a reaction event arrives via WebSocket. */
  invalidate: (messageId: string) => void
}

export const useReactionStore = create<ReactionState>((set, get) => ({
  reactions: {},

  loadReactions: async (messageId) => {
    try {
      const summaries = await reactionsApi.list(messageId)
      set((s) => ({
        reactions: { ...s.reactions, [messageId]: summaries },
      }))
    } catch {
      // Silently fail — reactions are non-critical
    }
  },

  addReaction: async (messageId, emoji) => {
    // Optimistic update
    set((s) => {
      const current = s.reactions[messageId] ?? []
      const existing = current.find((r) => r.emoji === emoji)
      const updated = existing
        ? current.map((r) => r.emoji === emoji ? { ...r, count: r.count + 1, me: true } : r)
        : [...current, { emoji, is_custom: false, custom_emoji_id: null, count: 1, me: true }]
      return { reactions: { ...s.reactions, [messageId]: updated } }
    })

    try {
      await reactionsApi.add(messageId, emoji)
    } catch {
      // Revert on failure
      get().loadReactions(messageId)
    }
  },

  removeReaction: async (messageId, emoji) => {
    // Optimistic update
    set((s) => {
      const current = s.reactions[messageId] ?? []
      const updated = current
        .map((r) => r.emoji === emoji ? { ...r, count: r.count - 1, me: false } : r)
        .filter((r) => r.count > 0)
      return { reactions: { ...s.reactions, [messageId]: updated } }
    })

    try {
      await reactionsApi.remove(messageId, emoji)
    } catch {
      get().loadReactions(messageId)
    }
  },

  invalidate: (messageId) => {
    get().loadReactions(messageId)
  },
}))
