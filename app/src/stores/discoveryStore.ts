/// Discovery state store.
///
/// Manages hall search and browsing for the discovery view.

import { create } from 'zustand'
import { discovery as discoveryApi } from '@/lib/api'
import type { DiscoveryHall } from '@/lib/types'

interface DiscoveryState {
  results: DiscoveryHall[]
  loading: boolean
  query: string
  category: string | null

  search: (query?: string, category?: string) => Promise<void>
  setQuery: (query: string) => void
  setCategory: (category: string | null) => void
  clear: () => void
}

export const useDiscoveryStore = create<DiscoveryState>((set) => ({
  results: [],
  loading: false,
  query: '',
  category: null,

  search: async (query, category) => {
    set({ loading: true })
    try {
      const results = await discoveryApi.search(query, category, 24)
      set({ results, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  setQuery: (query) => set({ query }),
  setCategory: (category) => set({ category }),
  clear: () => set({ results: [], query: '', category: null }),
}))
