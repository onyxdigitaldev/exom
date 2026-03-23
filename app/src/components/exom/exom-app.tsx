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
import { useVoiceStore } from "@/stores/voiceStore"
import { usePresenceStore } from "@/stores/presenceStore"
import { useReactionStore } from "@/stores/reactionStore"
import { useWebSocket, type RelayEvent } from "@/hooks/useWebSocket"

export function ExomApp() {
  const { isAuthenticated, user } = useAuthStore()
  const { halls, activeHallId, activeChannelId, loadHalls, selectHall, selectChannel } = useHallStore()
  const { loadMessages, addMessage, updateMessage, removeMessage } = useMessageStore()
  const { loadMembers, setOnline, setOffline } = useMemberStore()
  const ui = useUiStore()
  const voice = useVoiceStore()
  const presence = usePresenceStore()
  const { invalidate: invalidateReactions } = useReactionStore()

  /** Dispatch incoming relay events to the appropriate stores. */
  const handleRelayEvent = useCallback((event: RelayEvent) => {
    switch (event.type) {
      case 'ChannelMessage': {
        const { sender_id, message } = event as RelayEvent & {
          sender_id: string
          message: { id: string; content: string; timestamp: string; reply_to?: string }
        }
        addMessage({
          id: message.id,
          sender_id,
          sender_username: '',
          sender_role: 'Agent',
          content: message.content,
          timestamp: message.timestamp,
          is_edited: false,
          reply_to: message.reply_to ?? null,
          thread_id: null,
          is_pinned: false,
          reaction_count: 0,
          thread_reply_count: 0,
        })
        break
      }
      case 'MessageEdited': {
        const { message_id, new_content, edited_at } = event as RelayEvent & {
          message_id: string; new_content: string; edited_at: string
        }
        updateMessage(message_id, new_content, edited_at)
        break
      }
      case 'MessageDeleted': {
        const { message_id } = event as RelayEvent & { message_id: string }
        removeMessage(message_id)
        break
      }
      case 'MemberOnline': {
        const { user_id } = event as RelayEvent & { user_id: string }
        setOnline(user_id)
        break
      }
      case 'MemberOffline': {
        const { user_id } = event as RelayEvent & { user_id: string }
        setOffline(user_id)
        break
      }
      case 'TypingStarted': {
        const { channel_id, user_id } = event as RelayEvent & { channel_id: string; user_id: string }
        presence.setTyping(channel_id, user_id)
        break
      }
      case 'PresenceUpdated': {
        const { user_id, status } = event as RelayEvent & { user_id: string; status: number }
        presence.setStatus(user_id, status)
        break
      }
      case 'ReactionAdded':
      case 'ReactionRemoved': {
        const { message_id } = event as RelayEvent & { message_id: string }
        invalidateReactions(message_id)
        break
      }
      case 'VoiceUserJoined': {
        const { channel_id, user_id } = event as RelayEvent & { channel_id: string; user_id: string }
        voice.addUser(channel_id, {
          user_id, username: '', self_mute: false, self_deaf: false, video: false, streaming: false,
        })
        break
      }
      case 'VoiceUserLeft': {
        const { channel_id, user_id } = event as RelayEvent & { channel_id: string; user_id: string }
        voice.removeUser(channel_id, user_id)
        break
      }
      case 'VoiceStateUpdated': {
        const { channel_id, user_id, self_mute, self_deaf, video, streaming } = event as RelayEvent & {
          channel_id: string; user_id: string; self_mute: boolean; self_deaf: boolean; video: boolean; streaming: boolean
        }
        voice.updateUser(channel_id, user_id, { self_mute, self_deaf, video, streaming })
        break
      }
    }
  }, [addMessage, updateMessage, removeMessage, setOnline, setOffline, presence, voice, invalidateReactions, activeHallId, activeChannelId, loadMessages])

  // Re-subscribe to active hall on reconnect
  const handleWsStateChange = useCallback((state: string) => {
    if (state === 'connected' && activeHallId) {
      send({ HallJoin: { hall_id: activeHallId } })
    }
  }, [activeHallId, send])

  const { connect, send, state: wsState } = useWebSocket({
    onEvent: handleRelayEvent,
    onStateChange: handleWsStateChange,
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
