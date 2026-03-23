/// Invite state store.
///
/// Manages invite creation, listing, acceptance, and revocation.

import { create } from 'zustand'
import { invites as invitesApi } from '@/lib/api'
import type { Invite } from '@/lib/types'

interface InviteState {
  invites: Invite[]
  loading: boolean

  loadInvites: (hallId: string) => Promise<void>
  createInvite: (hallId: string, role?: string, expiryHours?: number, maxUses?: number) => Promise<Invite>
  acceptInvite: (token: string) => Promise<void>
  revokeInvite: (id: string) => Promise<void>
  clear: () => void
}

export const useInviteStore = create<InviteState>((set) => ({
  invites: [],
  loading: false,

  loadInvites: async (hallId) => {
    set({ loading: true })
    try {
      const invites = await invitesApi.list(hallId)
      set({ invites, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  createInvite: async (hallId, role, expiryHours, maxUses) => {
    const invite = await invitesApi.create(hallId, role, expiryHours, maxUses)
    set((state) => ({ invites: [invite, ...state.invites] }))
    return invite
  },

  acceptInvite: async (token) => {
    await invitesApi.accept(token)
  },

  revokeInvite: async (id) => {
    await invitesApi.revoke(id)
    set((state) => ({
      invites: state.invites.filter((i) => i.id !== id),
    }))
  },

  clear: () => set({ invites: [] }),
}))
