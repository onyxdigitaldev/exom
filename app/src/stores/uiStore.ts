/// UI state store.
///
/// Manages panel visibility, active modals, and ephemeral UI state
/// that doesn't belong to any domain-specific store.

import { create } from 'zustand'

interface UiState {
  showMembers: boolean
  showDMs: boolean
  showDiscovery: boolean
  showSettings: boolean
  showHallSettings: boolean
  showInvite: boolean
  showCreateHall: boolean
  showCreateChannel: boolean
  replyingTo: { id: string; author: string; content: string } | null
  typingUsers: string[]

  toggleMembers: () => void
  setShowDMs: (show: boolean) => void
  setShowDiscovery: (show: boolean) => void
  setShowSettings: (show: boolean) => void
  setShowHallSettings: (show: boolean) => void
  setShowInvite: (show: boolean) => void
  setShowCreateHall: (show: boolean) => void
  setShowCreateChannel: (show: boolean) => void
  setReplyingTo: (msg: UiState['replyingTo']) => void
  addTypingUser: (username: string) => void
  removeTypingUser: (username: string) => void
}

export const useUiStore = create<UiState>((set) => ({
  showMembers: true,
  showDMs: false,
  showDiscovery: false,
  showSettings: false,
  showHallSettings: false,
  showInvite: false,
  showCreateHall: false,
  showCreateChannel: false,
  replyingTo: null,
  typingUsers: [],

  toggleMembers: () => set((s) => ({ showMembers: !s.showMembers })),
  setShowDMs: (show) => set({ showDMs: show }),
  setShowDiscovery: (show) => set({ showDiscovery: show }),
  setShowSettings: (show) => set({ showSettings: show }),
  setShowHallSettings: (show) => set({ showHallSettings: show }),
  setShowInvite: (show) => set({ showInvite: show }),
  setShowCreateHall: (show) => set({ showCreateHall: show }),
  setShowCreateChannel: (show) => set({ showCreateChannel: show }),
  setReplyingTo: (msg) => set({ replyingTo: msg }),
  addTypingUser: (username) =>
    set((s) => ({
      typingUsers: s.typingUsers.includes(username)
        ? s.typingUsers
        : [...s.typingUsers, username],
    })),
  removeTypingUser: (username) =>
    set((s) => ({
      typingUsers: s.typingUsers.filter((u) => u !== username),
    })),
}))
