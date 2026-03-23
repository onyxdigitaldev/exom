/// Profile state store.
///
/// Manages user profile data — display name, bio, avatar, status.
/// Caches profiles by user ID to avoid redundant fetches.

import { create } from 'zustand'
import { profiles as profilesApi } from '@/lib/api'
import type { UserProfile } from '@/lib/types'

interface ProfileState {
  profiles: Record<string, UserProfile>
  loading: boolean

  loadProfile: (userId: string) => Promise<UserProfile | null>
  updateProfile: (data: { display_name?: string; bio?: string; status?: number }) => Promise<void>
  /** Cache a profile received from another source (e.g. WebSocket). */
  cacheProfile: (profile: UserProfile) => void
}

export const useProfileStore = create<ProfileState>((set, get) => ({
  profiles: {},
  loading: false,

  loadProfile: async (userId) => {
    const cached = get().profiles[userId]
    if (cached) return cached

    try {
      const profile = await profilesApi.get(userId)
      set((state) => ({
        profiles: { ...state.profiles, [userId]: profile },
      }))
      return profile
    } catch {
      return null
    }
  },

  updateProfile: async (data) => {
    await profilesApi.update(data)
  },

  cacheProfile: (profile) => {
    set((state) => ({
      profiles: { ...state.profiles, [profile.user_id]: profile },
    }))
  },
}))
