/// Authentication state store.
///
/// Manages login/register flow, session persistence,
/// and current user identity.

import { create } from 'zustand'
import { auth as authApi } from '@/lib/api'
import type { User } from '@/lib/types'

interface AuthState {
  isAuthenticated: boolean
  user: User | null
  sessionId: string | null
  loading: boolean
  error: string | null

  login: (username: string, password: string) => Promise<void>
  register: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
  checkSession: () => Promise<void>
  clearError: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  isAuthenticated: false,
  user: null,
  sessionId: null,
  loading: false,
  error: null,

  login: async (username, password) => {
    set({ loading: true, error: null })
    try {
      const res = await authApi.login(username, password)
      set({
        isAuthenticated: true,
        user: { user_id: res.user_id, username: res.username },
        sessionId: res.session_id,
        loading: false,
      })
    } catch (err) {
      set({ loading: false, error: (err as Error).message })
    }
  },

  register: async (username, password) => {
    set({ loading: true, error: null })
    try {
      const res = await authApi.register(username, password)
      set({
        isAuthenticated: true,
        user: { user_id: res.user_id, username: res.username },
        sessionId: res.session_id,
        loading: false,
      })
    } catch (err) {
      set({ loading: false, error: (err as Error).message })
    }
  },

  logout: async () => {
    try {
      await authApi.logout()
    } finally {
      set({
        isAuthenticated: false,
        user: null,
        sessionId: null,
      })
    }
  },

  checkSession: async () => {
    try {
      const user = await authApi.me()
      set({ isAuthenticated: true, user })
    } catch {
      set({ isAuthenticated: false, user: null, sessionId: null })
    }
  },

  clearError: () => set({ error: null }),
}))
