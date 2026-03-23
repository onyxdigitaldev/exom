/// DM state store.
///
/// Manages DM channel list, active DM selection,
/// and message history for direct conversations.

import { create } from 'zustand'
import { dms as dmsApi } from '@/lib/api'
import type { DmChannel, Message } from '@/lib/types'

interface DmState {
  channels: DmChannel[]
  activeChannelId: string | null
  messages: Message[]
  loading: boolean

  loadChannels: () => Promise<void>
  selectChannel: (channelId: string | null) => void
  createDm: (targetUserId: string) => Promise<DmChannel>
  loadMessages: (channelId: string) => Promise<void>
  sendMessage: (channelId: string, content: string) => Promise<void>
  /** Called when a DM arrives via WebSocket. */
  addMessage: (message: Message) => void
  clear: () => void
}

export const useDmStore = create<DmState>((set, get) => ({
  channels: [],
  activeChannelId: null,
  messages: [],
  loading: false,

  loadChannels: async () => {
    set({ loading: true })
    try {
      const channels = await dmsApi.list()
      set({ channels, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  selectChannel: (channelId) => {
    set({ activeChannelId: channelId, messages: [] })
    if (channelId) {
      get().loadMessages(channelId)
    }
  },

  createDm: async (targetUserId) => {
    const channel = await dmsApi.create(targetUserId)
    set((state) => ({
      channels: [channel, ...state.channels.filter((c) => c.id !== channel.id)],
    }))
    return channel
  },

  loadMessages: async (channelId) => {
    set({ loading: true })
    try {
      const messages = await dmsApi.listMessages(channelId, 50)
      set({ messages, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  sendMessage: async (channelId, content) => {
    await dmsApi.send(channelId, content)
    // Reload to get the full message with metadata
    get().loadMessages(channelId)
  },

  addMessage: (message) => {
    set((state) => {
      if (state.messages.some((m) => m.id === message.id)) return state
      return { messages: [...state.messages, message] }
    })
  },

  clear: () => set({ channels: [], activeChannelId: null, messages: [] }),
}))
