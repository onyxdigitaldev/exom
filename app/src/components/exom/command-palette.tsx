/// CommandPalette — Ctrl+K search for messages, channels, members, and actions.
///
/// Fuzzy search across all accessible content. Results grouped by type.
/// Keyboard navigable: arrow keys to select, Enter to act, Esc to close.

import { useState, useEffect, useRef, useCallback } from 'react'
import { cn } from '@/lib/utils'
import {
  Search,
  Hash,
  Volume2,
  User,
  MessageSquare,
  Settings,
  Users,
  Compass,
  X,
  ArrowRight,
} from 'lucide-react'
import { useHallStore } from '@/stores/hallStore'
import { useMemberStore } from '@/stores/memberStore'
import { useAuthStore } from '@/stores/authStore'
import { messages as messagesApi } from '@/lib/api'
import type { Message } from '@/lib/types'

interface CommandPaletteProps {
  open: boolean
  onClose: () => void
  onNavigate: (action: PaletteAction) => void
}

export type PaletteAction =
  | { type: 'channel'; hallId: string; channelId: string }
  | { type: 'member'; userId: string }
  | { type: 'settings' }
  | { type: 'discovery' }

interface SearchResult {
  id: string
  type: 'channel' | 'member' | 'action'
  icon: React.ReactNode
  title: string
  subtitle?: string
  action: PaletteAction | (() => void)
}

export function CommandPalette({ open, onClose, onNavigate }: CommandPaletteProps) {
  const [query, setQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const inputRef = useRef<HTMLInputElement>(null)
  const { halls, channels, activeHallId, selectChannel } = useHallStore()
  const { members } = useMemberStore()
  const { user } = useAuthStore()

  // Focus input when palette opens
  useEffect(() => {
    if (open) {
      setQuery('')
      setSelectedIndex(0)
      setTimeout(() => inputRef.current?.focus(), 50)
    }
  }, [open])

  // Build search results
  const results: SearchResult[] = []

  if (query.trim()) {
    const q = query.toLowerCase()

    // Search channels
    channels
      .filter((c) => c.name.toLowerCase().includes(q))
      .slice(0, 5)
      .forEach((c) => {
        results.push({
          id: `ch-${c.id}`,
          type: 'channel',
          icon: c.channel_type === 'voice' ? <Volume2 className="w-4 h-4" /> : <Hash className="w-4 h-4" />,
          title: c.name,
          subtitle: halls.find((h) => h.id === c.hall_id)?.name,
          action: { type: 'channel', hallId: c.hall_id, channelId: c.id },
        })
      })

    // Search members
    members
      .filter((m) => m.username.toLowerCase().includes(q))
      .slice(0, 5)
      .forEach((m) => {
        results.push({
          id: `mb-${m.user_id}`,
          type: 'member',
          icon: <User className="w-4 h-4" />,
          title: m.username,
          subtitle: m.role,
          action: { type: 'member', userId: m.user_id },
        })
      })
  } else {
    // Show quick actions when empty
    results.push(
      {
        id: 'action-settings',
        type: 'action',
        icon: <Settings className="w-4 h-4" />,
        title: 'User Settings',
        subtitle: 'Ctrl+,',
        action: { type: 'settings' },
      },
      {
        id: 'action-discover',
        type: 'action',
        icon: <Compass className="w-4 h-4" />,
        title: 'Discover Halls',
        action: { type: 'discovery' },
      },
    )
  }

  // Clamp selected index
  const clampedIndex = Math.min(selectedIndex, results.length - 1)

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault()
          setSelectedIndex((i) => Math.min(i + 1, results.length - 1))
          break
        case 'ArrowUp':
          e.preventDefault()
          setSelectedIndex((i) => Math.max(i - 1, 0))
          break
        case 'Enter':
          e.preventDefault()
          if (results[clampedIndex]) {
            const action = results[clampedIndex].action
            if (typeof action === 'function') {
              action()
            } else {
              onNavigate(action)
            }
            onClose()
          }
          break
        case 'Escape':
          onClose()
          break
      }
    },
    [results, clampedIndex, onClose, onNavigate],
  )

  if (!open) return null

  return (
    <div className="fixed inset-0 z-[100] flex items-start justify-center pt-[15vh]">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-background/60 backdrop-blur-sm" onClick={onClose} />

      {/* Palette */}
      <div className="relative w-full max-w-lg bg-card rounded-2xl shadow-2xl border border-border/50 overflow-hidden">
        {/* Search input */}
        <div className="flex items-center gap-3 px-4 h-14 border-b border-border/30">
          <Search className="w-5 h-5 text-muted-foreground flex-shrink-0" />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value)
              setSelectedIndex(0)
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search channels, members, or type a command..."
            className="flex-1 bg-transparent text-foreground placeholder:text-muted-foreground focus:outline-none"
          />
          <kbd className="text-xs text-muted-foreground bg-secondary/50 px-2 py-0.5 rounded border border-border/50">
            ESC
          </kbd>
        </div>

        {/* Results */}
        <div className="max-h-80 overflow-y-auto p-2">
          {results.length === 0 && query.trim() && (
            <div className="px-4 py-8 text-center text-muted-foreground text-sm">
              No results for "{query}"
            </div>
          )}

          {results.map((result, i) => (
            <button
              key={result.id}
              onClick={() => {
                const action = result.action
                if (typeof action === 'function') {
                  action()
                } else {
                  onNavigate(action)
                }
                onClose()
              }}
              className={cn(
                'w-full flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm transition-all',
                i === clampedIndex
                  ? 'bg-primary/15 text-primary'
                  : 'text-foreground hover:bg-secondary/50',
              )}
            >
              <div className="w-8 h-8 rounded-lg bg-secondary/50 flex items-center justify-center flex-shrink-0 text-muted-foreground">
                {result.icon}
              </div>
              <div className="flex-1 text-left min-w-0">
                <p className="font-medium truncate">{result.title}</p>
                {result.subtitle && (
                  <p className="text-xs text-muted-foreground truncate">{result.subtitle}</p>
                )}
              </div>
              {i === clampedIndex && (
                <ArrowRight className="w-4 h-4 text-primary flex-shrink-0" />
              )}
            </button>
          ))}
        </div>

        {/* Footer hints */}
        <div className="px-4 py-2 border-t border-border/30 flex items-center gap-4 text-[11px] text-muted-foreground">
          <span><kbd className="px-1 py-0.5 rounded bg-secondary/50 border border-border/50">↑↓</kbd> navigate</span>
          <span><kbd className="px-1 py-0.5 rounded bg-secondary/50 border border-border/50">↵</kbd> select</span>
          <span><kbd className="px-1 py-0.5 rounded bg-secondary/50 border border-border/50">esc</kbd> close</span>
        </div>
      </div>
    </div>
  )
}
