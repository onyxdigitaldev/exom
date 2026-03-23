import { useEffect, useCallback } from "react"
import { TopNavigation } from "./top-navigation"
import { ChannelPanel } from "./channel-panel"
import { MessageFeed } from "./message-feed"
import { MemberPanel } from "./member-panel"
import { DMPanel } from "./dm-panel"
import { DiscoveryView } from "./discovery-view"
import { SettingsModal } from "./settings-modal"
import { InviteModal } from "./invite-modal"
import { AuthScreen } from "./auth-screen"

import { useAuthStore } from "@/stores/authStore"
import { useHallStore } from "@/stores/hallStore"
import { useMessageStore } from "@/stores/messageStore"
import { useMemberStore } from "@/stores/memberStore"
import { useUiStore } from "@/stores/uiStore"
import { useWebSocket, type RelayEvent } from "@/hooks/useWebSocket"

export function ExomApp() {
  const { isAuthenticated, user } = useAuthStore()
  const { halls, activeHallId, activeChannelId, loadHalls, selectHall, selectChannel } = useHallStore()
  const { loadMessages, addMessage, updateMessage, removeMessage } = useMessageStore()
  const { loadMembers, setOnline, setOffline } = useMemberStore()
  const ui = useUiStore()

  // Handle incoming relay events and dispatch to stores
  const handleRelayEvent = useCallback((event: RelayEvent) => {
    switch (event.type) {
      case 'ChannelMessage': {
        const msg = event as RelayEvent & {
          sender_id: string
          message: { id: string; content: string; timestamp: string }
        }
        // Only add if it's for the active channel
        addMessage({
          id: msg.message.id,
          sender_id: msg.sender_id,
          sender_username: '', // Will be resolved on next load
          sender_role: 'Agent',
          content: msg.message.content,
          timestamp: msg.message.timestamp,
          is_edited: false,
          reply_to: null,
          thread_id: null,
          is_pinned: false,
          reaction_count: 0,
          thread_reply_count: 0,
        })
        break
      }
      case 'MessageEdited': {
        const edit = event as RelayEvent & { message_id: string; new_content: string; edited_at: string }
        updateMessage(edit.message_id, edit.new_content, edit.edited_at)
        break
      }
      case 'MessageDeleted': {
        const del = event as RelayEvent & { message_id: string }
        removeMessage(del.message_id)
        break
      }
      case 'MemberOnline': {
        const online = event as RelayEvent & { user_id: string }
        setOnline(online.user_id)
        break
      }
      case 'MemberOffline': {
        const offline = event as RelayEvent & { user_id: string }
        setOffline(offline.user_id)
        break
      }
      case 'TypingStarted': {
        const typing = event as RelayEvent & { user_id: string }
        // Typing indicator handled via uiStore
        // Would need username lookup — simplified for now
        break
      }
    }
  }, [addMessage, updateMessage, removeMessage, setOnline, setOffline])

  const { connect, send, state: wsState } = useWebSocket({
    onEvent: handleRelayEvent,
  })

  // Load halls on auth
  useEffect(() => {
    if (isAuthenticated) {
      loadHalls()
      connect()
    }
  }, [isAuthenticated, loadHalls, connect])

  // Load channel messages and members when hall/channel changes
  useEffect(() => {
    if (activeHallId && activeChannelId) {
      loadMessages(activeHallId, activeChannelId)
      loadMembers(activeHallId)

      // Subscribe to hall events on relay
      send({ HallJoin: { hall_id: activeHallId } })
    }
  }, [activeHallId, activeChannelId, loadMessages, loadMembers, send])

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
    </div>
  )
}
