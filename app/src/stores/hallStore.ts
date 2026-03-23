/// Hall and channel state store.
///
/// Manages hall list, active hall/channel selection,
/// and channel listings.

import { create } from 'zustand'
import { halls as hallsApi, channels as channelsApi } from '@/lib/api'
import type { Hall, Channel } from '@/lib/types'

interface HallState {
  halls: Hall[]
  activeHallId: string | null
  activeChannelId: string | null
  channels: Channel[]
  loading: boolean

  loadHalls: () => Promise<void>
  selectHall: (hallId: string | null) => Promise<void>
  selectChannel: (channelId: string) => void
  createHall: (name: string, description?: string) => Promise<Hall>
  leaveHall: (hallId: string) => Promise<void>
  createChannel: (hallId: string, name: string, type: string, topic?: string) => Promise<Channel>
  deleteChannel: (channelId: string) => Promise<void>
}

export const useHallStore = create<HallState>((set, get) => ({
  halls: [],
  activeHallId: null,
  activeChannelId: null,
  channels: [],
  loading: false,

  loadHalls: async () => {
    set({ loading: true })
    try {
      const halls = await hallsApi.list()
      set({ halls, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  selectHall: async (hallId) => {
    set({ activeHallId: hallId, activeChannelId: null, channels: [] })
    if (!hallId) return

    try {
      const channels = await channelsApi.list(hallId)
      const firstText = channels.find((c) => c.channel_type === 'text')
      set({
        channels,
        activeChannelId: firstText?.id ?? channels[0]?.id ?? null,
      })
    } catch {
      // Hall may have been deleted
    }
  },

  selectChannel: (channelId) => {
    set({ activeChannelId: channelId })
  },

  createHall: async (name, description) => {
    const hall = await hallsApi.create(name, description)
    set((state) => ({ halls: [...state.halls, hall] }))
    return hall
  },

  leaveHall: async (hallId) => {
    await hallsApi.leave(hallId)
    set((state) => ({
      halls: state.halls.filter((h) => h.id !== hallId),
      activeHallId: state.activeHallId === hallId ? null : state.activeHallId,
    }))
  },

  createChannel: async (hallId, name, type, topic) => {
    const channel = await channelsApi.create(hallId, name, type, topic)
    set((state) => ({ channels: [...state.channels, channel] }))
    return channel
  },

  deleteChannel: async (channelId) => {
    await channelsApi.delete(channelId)
    set((state) => ({
      channels: state.channels.filter((c) => c.id !== channelId),
      activeChannelId:
        state.activeChannelId === channelId ? null : state.activeChannelId,
    }))
  },
}))
