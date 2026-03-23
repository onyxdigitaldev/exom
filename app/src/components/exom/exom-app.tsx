import { useEffect, useState } from "react"
import { TopNavigation } from "./top-navigation"
import { ChannelPanel } from "./channel-panel"
import { MessageFeed } from "./message-feed"
import { MemberPanel } from "./member-panel"
import { DMPanel } from "./dm-panel"
import { DiscoveryView } from "./discovery-view"
import { SettingsModal } from "./settings-modal"
import { InviteModal } from "./invite-modal"
import { AuthScreen } from "./auth-screen"
import { CommandPalette, type PaletteAction } from "./command-palette"

import { useAuthStore } from "@/stores/authStore"
import { useHallStore } from "@/stores/hallStore"
import { useMessageStore } from "@/stores/messageStore"
import { useMemberStore } from "@/stores/memberStore"
import { useUiStore } from "@/stores/uiStore"
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts"

export function ExomApp() {
  const [showCommandPalette, setShowCommandPalette] = useState(false)
  useKeyboardShortcuts({
    onCommandPalette: () => setShowCommandPalette((v) => !v),
  })

  const { isAuthenticated, user } = useAuthStore()
  const { activeHallId, activeChannelId, loadHalls, selectHall, selectChannel } = useHallStore()
  const { loadMessages } = useMessageStore()
  const { loadMembers } = useMemberStore()
  const ui = useUiStore()

  // Load halls after authentication
  useEffect(() => {
    if (isAuthenticated) {
      loadHalls()
    }
  }, [isAuthenticated, loadHalls])

  // Load channel data when hall/channel selection changes
  useEffect(() => {
    if (activeHallId && activeChannelId) {
      loadMessages(activeHallId, activeChannelId)
      loadMembers(activeHallId)
    }
  }, [activeHallId, activeChannelId, loadMessages, loadMembers])

  if (!isAuthenticated) {
    return <AuthScreen onLogin={() => {}} />
  }

  const handleHallSelect = (hallId: string | null) => {
    selectHall(hallId)
    ui.setShowDMs(false)
  }

  const handleShowDMs = () => {
    ui.setShowDMs(true)
    selectHall(null)
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-background">
      <TopNavigation
        activeHall={activeHallId}
        onHallSelect={handleHallSelect}
        onShowDiscovery={() => ui.setShowDiscovery(true)}
        onShowDMs={handleShowDMs}
        onShowSettings={() => ui.setShowSettings(true)}
        showDMs={ui.showDMs}
      />

      <div className="flex-1 flex overflow-hidden">
        {ui.showDMs ? (
          <DMPanel />
        ) : activeHallId ? (
          <>
            <div className="bg-card/50 border-r border-border/30">
              <ChannelPanel
                hallId={activeHallId}
                activeChannel={activeChannelId ?? ''}
                onChannelSelect={selectChannel}
                onInvite={() => ui.setShowInvite(true)}
                onHallSettings={() => ui.setShowHallSettings(true)}
                onCreateChannel={() => ui.setShowCreateChannel(true)}
              />
            </div>

            <MessageFeed
              channelId={activeChannelId ?? ''}
              showMembers={ui.showMembers}
              onToggleMembers={ui.toggleMembers}
            />

            {ui.showMembers && <MemberPanel />}
          </>
        ) : (
          <DMPanel />
        )}
      </div>

      {ui.showDiscovery && (
        <DiscoveryView onClose={() => ui.setShowDiscovery(false)} />
      )}

      {ui.showSettings && (
        <SettingsModal onClose={() => ui.setShowSettings(false)} />
      )}

      {ui.showInvite && activeHallId && (
        <InviteModal hallId={activeHallId} onClose={() => ui.setShowInvite(false)} />
      )}

      <CommandPalette
        open={showCommandPalette}
        onClose={() => setShowCommandPalette(false)}
        onNavigate={(action) => {
          switch (action.type) {
            case 'channel':
              selectHall(action.hallId)
              selectChannel(action.channelId)
              break
            case 'settings':
              ui.setShowSettings(true)
              break
            case 'discovery':
              ui.setShowDiscovery(true)
              break
          }
        }}
      />
    </div>
  )
}
