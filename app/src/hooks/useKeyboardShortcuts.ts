/// Global keyboard shortcut handler.
///
/// Registers shortcuts at the document level and dispatches actions.
/// Shortcuts are suppressed when typing in inputs (except Ctrl combos).

import { useEffect } from 'react'
import { useUiStore } from '@/stores/uiStore'
import { useVoiceStore } from '@/stores/voiceStore'

/** Whether the active element is a text input. */
function isInputFocused(): boolean {
  const el = document.activeElement
  if (!el) return false
  const tag = el.tagName.toLowerCase()
  return tag === 'input' || tag === 'textarea' || (el as HTMLElement).isContentEditable
}

interface ShortcutOptions {
  onCommandPalette?: () => void
}

export function useKeyboardShortcuts(options: ShortcutOptions = {}) {
  const ui = useUiStore()

  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      const ctrl = e.ctrlKey || e.metaKey
      const shift = e.shiftKey

      // Escape: close topmost overlay (always active)
      if (e.key === 'Escape') {
        if (ui.showSettings) { ui.setShowSettings(false); return }
        if (ui.showHallSettings) { ui.setShowHallSettings(false); return }
        if (ui.showDiscovery) { ui.setShowDiscovery(false); return }
        if (ui.showInvite) { ui.setShowInvite(false); return }
        if (ui.showCreateHall) { ui.setShowCreateHall(false); return }
        if (ui.showCreateChannel) { ui.setShowCreateChannel(false); return }
        if (ui.replyingTo) { ui.setReplyingTo(null); return }
        return
      }

      // Skip non-ctrl shortcuts when in an input
      if (isInputFocused() && !ctrl) return

      // Ctrl+K: command palette
      if (ctrl && e.key === 'k') {
        e.preventDefault()
        options.onCommandPalette?.()
        return
      }

      // Ctrl+Shift+M: toggle mute
      if (ctrl && shift && e.key === 'M') {
        e.preventDefault()
        useVoiceStore.getState().toggleMute()
        return
      }

      // Ctrl+Shift+D: toggle deafen
      if (ctrl && shift && e.key === 'D') {
        e.preventDefault()
        useVoiceStore.getState().toggleDeaf()
        return
      }

      // Ctrl+,: settings
      if (ctrl && e.key === ',') {
        e.preventDefault()
        ui.setShowSettings(true)
        return
      }

      // Ctrl+Shift+I: invite dialog
      if (ctrl && shift && e.key === 'I') {
        e.preventDefault()
        ui.setShowInvite(true)
        return
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [ui, options])
}
