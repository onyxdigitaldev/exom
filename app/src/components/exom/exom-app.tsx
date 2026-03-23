import { useState } from "react"
import { TopNavigation } from "./top-navigation"
import { ChannelPanel } from "./channel-panel"
import { MessageFeed } from "./message-feed"
import { MemberPanel } from "./member-panel"
import { DMPanel } from "./dm-panel"
import { DiscoveryView } from "./discovery-view"
import { SettingsModal } from "./settings-modal"
import { InviteModal } from "./invite-modal"
import { AuthScreen } from "./auth-screen"

export function ExomApp() {
  const [isAuthenticated, setIsAuthenticated] = useState(true)
  const [activeHall, setActiveHall] = useState<string | null>("hall-1")
  const [activeChannel, setActiveChannel] = useState("ch-1")
  const [showMembers, setShowMembers] = useState(true)
  const [showDMs, setShowDMs] = useState(false)
  const [showDiscovery, setShowDiscovery] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [showInvite, setShowInvite] = useState(false)

  if (!isAuthenticated) {
    return <AuthScreen onLogin={() => setIsAuthenticated(true)} />
  }

  const handleHallSelect = (hallId: string | null) => {
    setActiveHall(hallId)
    setShowDMs(false)
    if (hallId) {
      setActiveChannel("ch-1")
    }
  }

  const handleShowDMs = () => {
    setShowDMs(true)
    setActiveHall(null)
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-background">
      {/* Top Navigation */}
      <TopNavigation
        activeHall={activeHall}
        onHallSelect={handleHallSelect}
        onShowDiscovery={() => setShowDiscovery(true)}
        onShowDMs={handleShowDMs}
        onShowSettings={() => setShowSettings(true)}
        showDMs={showDMs}
      />

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {showDMs ? (
          <DMPanel />
        ) : activeHall ? (
          <>
            {/* Channel Panel */}
            <div className="bg-card/50 border-r border-border/30">
              <ChannelPanel
                hallId={activeHall}
                activeChannel={activeChannel}
                onChannelSelect={setActiveChannel}
                onInvite={() => setShowInvite(true)}
                onHallSettings={() => {}}
                onCreateChannel={() => {}}
              />
            </div>

            {/* Message Feed */}
            <MessageFeed
              channelId={activeChannel}
              showMembers={showMembers}
              onToggleMembers={() => setShowMembers(!showMembers)}
            />

            {/* Members Panel */}
            {showMembers && <MemberPanel />}
          </>
        ) : (
          <DMPanel />
        )}
      </div>

      {/* Overlays */}
      {showDiscovery && (
        <DiscoveryView onClose={() => setShowDiscovery(false)} />
      )}

      {showSettings && (
        <SettingsModal onClose={() => setShowSettings(false)} />
      )}

      {showInvite && activeHall && (
        <InviteModal hallId={activeHall} onClose={() => setShowInvite(false)} />
      )}
    </div>
  )
}
