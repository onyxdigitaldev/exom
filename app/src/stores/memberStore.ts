/// Member state store.
///
/// Manages the member list for the active hall, handles
/// role changes, kicks, and presence updates from the relay.

import { create } from 'zustand'
import { members as membersApi } from '@/lib/api'
import type { Member } from '@/lib/types'

interface MemberState {
  members: Member[]
  loading: boolean

  loadMembers: (hallId: string) => Promise<void>
  updateRole: (hallId: string, userId: string, role: string) => Promise<void>
  kick: (hallId: string, userId: string) => Promise<void>
  /** Called when a member comes online via relay. */
  setOnline: (userId: string) => void
  /** Called when a member goes offline via relay. */
  setOffline: (userId: string) => void
  clear: () => void
}

export const useMemberStore = create<MemberState>((set) => ({
  members: [],
  loading: false,

  loadMembers: async (hallId) => {
    set({ loading: true })
    try {
      const members = await membersApi.list(hallId)
      set({ members, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  updateRole: async (hallId, userId, role) => {
    await membersApi.updateRole(hallId, userId, role)
    set((state) => ({
      members: state.members.map((m) =>
        m.user_id === userId ? { ...m, role } : m,
      ),
    }))
  },

  kick: async (hallId, userId) => {
    await membersApi.kick(hallId, userId)
    set((state) => ({
      members: state.members.filter((m) => m.user_id !== userId),
    }))
  },

  setOnline: (userId) => {
    set((state) => ({
      members: state.members.map((m) =>
        m.user_id === userId ? { ...m, is_online: true } : m,
      ),
    }))
  },

  setOffline: (userId) => {
    set((state) => ({
      members: state.members.map((m) =>
        m.user_id === userId ? { ...m, is_online: false } : m,
      ),
    }))
  },

  clear: () => set({ members: [] }),
}))
