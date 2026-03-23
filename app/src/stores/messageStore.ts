/// Message state store.
///
/// Manages messages for the active channel, handles sending,
/// editing, deleting, pinning, and real-time message arrival.

import { create } from 'zustand'
import { messages as messagesApi } from '@/lib/api'
import type { Message } from '@/lib/types'

interface MessageState {
  messages: Message[]
  loading: boolean
  hasMore: boolean

  loadMessages: (hallId: string, channelId: string) => Promise<void>
  loadMore: (hallId: string, channelId: string) => Promise<void>
  sendMessage: (hallId: string, channelId: string, content: string, replyTo?: string) => Promise<void>
  editMessage: (messageId: string, content: string) => Promise<void>
  deleteMessage: (messageId: string) => Promise<void>
  pinMessage: (messageId: string) => Promise<void>
  unpinMessage: (messageId: string) => Promise<void>
  /** Called when a message arrives via WebSocket. */
  addMessage: (message: Message) => void
  /** Called when a message is edited via WebSocket. */
  updateMessage: (messageId: string, content: string, editedAt: string) => void
  /** Called when a message is deleted via WebSocket. */
  removeMessage: (messageId: string) => void
  clear: () => void
}

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: [],
  loading: false,
  hasMore: true,

  loadMessages: async (hallId, channelId) => {
    set({ loading: true, messages: [], hasMore: true })
    try {
      const messages = await messagesApi.list(hallId, channelId, 50)
      set({ messages, loading: false, hasMore: messages.length === 50 })
    } catch {
      set({ loading: false })
    }
  },

  loadMore: async (hallId, channelId) => {
    const { messages, hasMore } = get()
    if (!hasMore || messages.length === 0) return

    const oldest = messages[0]
    const older = await messagesApi.list(hallId, channelId, 50, oldest.timestamp)
    set({
      messages: [...older, ...messages],
      hasMore: older.length === 50,
    })
  },

  sendMessage: async (hallId, channelId, content, replyTo) => {
    const message = await messagesApi.send(hallId, channelId, content, replyTo)
    set((state) => ({ messages: [...state.messages, message] }))
  },

  editMessage: async (messageId, content) => {
    await messagesApi.edit(messageId, content)
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, content, is_edited: true } : m,
      ),
    }))
  },

  deleteMessage: async (messageId) => {
    await messagesApi.delete(messageId)
    set((state) => ({
      messages: state.messages.filter((m) => m.id !== messageId),
    }))
  },

  pinMessage: async (messageId) => {
    await messagesApi.pin(messageId)
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, is_pinned: true } : m,
      ),
    }))
  },

  unpinMessage: async (messageId) => {
    await messagesApi.unpin(messageId)
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, is_pinned: false } : m,
      ),
    }))
  },

  addMessage: (message) => {
    set((state) => {
      // Deduplicate (may have sent it ourselves)
      if (state.messages.some((m) => m.id === message.id)) return state
      return { messages: [...state.messages, message] }
    })
  },

  updateMessage: (messageId, content, _editedAt) => {
    set((state) => ({
      messages: state.messages.map((m) =>
        m.id === messageId ? { ...m, content, is_edited: true } : m,
      ),
    }))
  },

  removeMessage: (messageId) => {
    set((state) => ({
      messages: state.messages.filter((m) => m.id !== messageId),
    }))
  },

  clear: () => set({ messages: [], hasMore: true }),
}))
