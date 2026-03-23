/// Voice state store.
///
/// Manages the local user's voice channel connection,
/// mute/deaf/video/stream state, and connected users per channel.

import { create } from 'zustand'

interface VoiceUser {
  user_id: string
  username: string
  self_mute: boolean
  self_deaf: boolean
  video: boolean
  streaming: boolean
}

interface VoiceState {
  /** Channel we're connected to, if any. */
  connectedChannel: { hall_id: string; channel_id: string; channel_name: string } | null
  /** Local user audio/video state. */
  selfMute: boolean
  selfDeaf: boolean
  video: boolean
  streaming: boolean
  /** Time we connected. */
  connectedAt: number | null
  /** Users in each voice channel: key is channel_id. */
  channelUsers: Record<string, VoiceUser[]>

  join: (hallId: string, channelId: string, channelName: string) => void
  leave: () => void
  toggleMute: () => void
  toggleDeaf: () => void
  toggleVideo: () => void
  toggleStream: () => void
  /** Called when relay reports a user joined voice. */
  addUser: (channelId: string, user: VoiceUser) => void
  /** Called when relay reports a user left voice. */
  removeUser: (channelId: string, userId: string) => void
  /** Called when relay reports a user's state changed. */
  updateUser: (channelId: string, userId: string, state: Partial<VoiceUser>) => void
}

export const useVoiceStore = create<VoiceState>((set) => ({
  connectedChannel: null,
  selfMute: false,
  selfDeaf: false,
  video: false,
  streaming: false,
  connectedAt: null,
  channelUsers: {},

  join: (hallId, channelId, channelName) => {
    set({
      connectedChannel: { hall_id: hallId, channel_id: channelId, channel_name: channelName },
      selfMute: false,
      selfDeaf: false,
      video: false,
      streaming: false,
      connectedAt: Date.now(),
    })
  },

  leave: () => {
    set({
      connectedChannel: null,
      selfMute: false,
      selfDeaf: false,
      video: false,
      streaming: false,
      connectedAt: null,
    })
  },

  toggleMute: () => set((s) => ({ selfMute: !s.selfMute })),
  toggleDeaf: () => set((s) => ({ selfDeaf: !s.selfDeaf })),
  toggleVideo: () => set((s) => ({ video: !s.video })),
  toggleStream: () => set((s) => ({ streaming: !s.streaming })),

  addUser: (channelId, user) => {
    set((s) => ({
      channelUsers: {
        ...s.channelUsers,
        [channelId]: [...(s.channelUsers[channelId] ?? []).filter((u) => u.user_id !== user.user_id), user],
      },
    }))
  },

  removeUser: (channelId, userId) => {
    set((s) => ({
      channelUsers: {
        ...s.channelUsers,
        [channelId]: (s.channelUsers[channelId] ?? []).filter((u) => u.user_id !== userId),
      },
    }))
  },

  updateUser: (channelId, userId, state) => {
    set((s) => ({
      channelUsers: {
        ...s.channelUsers,
        [channelId]: (s.channelUsers[channelId] ?? []).map((u) =>
          u.user_id === userId ? { ...u, ...state } : u,
        ),
      },
    }))
  },
}))
