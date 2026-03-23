import { useState, useEffect, useCallback } from "react"
import { cn } from "@/lib/utils"
import {
  Users,
  UserPlus,
  Search,
  X,
  Check,
  Clock,
  MessageCircle
} from "lucide-react"
import { dms as dmsApi } from "@/lib/api"
import { useMemberStore } from "@/stores/memberStore"
import type { DmChannel, Member } from "@/lib/types"

type Tab = "online" | "all" | "pending" | "blocked"

interface DMPanelProps {
  onStartDM?: (member: Member) => void
}

export function DMPanel({ onStartDM }: DMPanelProps) {
  const [activeTab, setActiveTab] = useState<Tab>("online")
  const [searchQuery, setSearchQuery] = useState("")
  const [dmChannels, setDmChannels] = useState<DmChannel[]>([])

  const { members } = useMemberStore()

  const loadDMs = useCallback(async () => {
    try {
      const channels = await dmsApi.list()
      setDmChannels(channels)
    } catch {
      // Silently degrade — panel stays empty until connection returns
    }
  }, [])

  useEffect(() => {
    loadDMs()
  }, [loadDMs])

  const onlineMembers = members.filter(m => m.is_online)

  const filteredMembers = members.filter(member => {
    if (searchQuery && !member.username.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }
    if (activeTab === "online") {
      return member.is_online
    }
    return true
  })

  const tabs: { id: Tab; label: string; count?: number }[] = [
    { id: "online", label: "Online", count: onlineMembers.length },
    { id: "all", label: "All", count: members.length },
    { id: "pending", label: "Pending" },
    { id: "blocked", label: "Blocked" },
  ]

  return (
    <div className="flex-1 flex">
      {/* Left: Friends/DMs List */}
      <div className="w-64 flex flex-col border-r border-border/30">
        {/* Search */}
        <div className="p-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              placeholder="Search or start conversation"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full h-10 pl-9 pr-3 rounded-xl bg-secondary/50 border border-transparent text-sm placeholder:text-muted-foreground focus:outline-none focus:border-primary/50 transition-all"
            />
          </div>
        </div>

        {/* Quick Actions */}
        <div className="px-3 pb-3">
          <button className="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl bg-primary/10 hover:bg-primary/20 text-primary transition-all">
            <UserPlus className="w-5 h-5" />
            <span className="font-medium text-sm">Add Friend</span>
          </button>
        </div>

        {/* Recent DMs */}
        <div className="px-2 mb-4">
          <h3 className="px-2 py-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            Direct Messages
          </h3>
          <div className="space-y-0.5">
            {dmChannels.map((dm) => {
              const displayName = dm.name ?? dm.participants[0] ?? "Unknown"
              return (
                <button
                  key={dm.id}
                  className="w-full flex items-center gap-3 px-2 py-2 rounded-lg hover:bg-secondary/50 transition-all group"
                >
                  <div className="relative flex-shrink-0">
                    <div className="w-9 h-9 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-medium text-sm">
                      {displayName.charAt(0).toUpperCase()}
                    </div>
                  </div>
                  <div className="flex-1 min-w-0 text-left">
                    <p className="text-sm font-medium text-foreground truncate">{displayName}</p>
                    {dm.last_message_at && (
                      <p className="text-xs text-muted-foreground truncate">
                        {new Date(dm.last_message_at).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                  <X className="w-4 h-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
                </button>
              )
            })}
          </div>
        </div>

        {/* Friends Section */}
        <div className="flex-1 overflow-y-auto px-2">
          <h3 className="px-2 py-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            Friends — {filteredMembers.length}
          </h3>
          <div className="space-y-0.5">
            {filteredMembers.map(member => (
              <button
                key={member.user_id}
                onClick={() => onStartDM?.(member)}
                className={cn(
                  "w-full flex items-center gap-3 px-2 py-2 rounded-lg transition-all group",
                  !member.is_online
                    ? "opacity-60 hover:opacity-100"
                    : "hover:bg-secondary/50"
                )}
              >
                <div className="relative flex-shrink-0">
                  <div className="w-9 h-9 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-medium text-sm">
                    {member.username.charAt(0).toUpperCase()}
                  </div>
                  <span className={cn(
                    "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-card",
                    member.is_online ? "bg-online" : "bg-offline"
                  )} />
                </div>
                <div className="flex-1 min-w-0 text-left">
                  <p className="text-sm font-medium text-foreground truncate">{member.username}</p>
                </div>
                <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button className="w-8 h-8 rounded-lg hover:bg-secondary flex items-center justify-center">
                    <MessageCircle className="w-4 h-4 text-muted-foreground" />
                  </button>
                </div>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Right: Main Content Area */}
      <div className="flex-1 flex flex-col bg-card/30 rounded-2xl m-2 ml-0 border border-border/30">
        {/* Tabs */}
        <div className="h-14 px-4 flex items-center gap-6 border-b border-border/30">
          <div className="flex items-center gap-2">
            <Users className="w-5 h-5 text-muted-foreground" />
            <span className="font-semibold text-foreground">Friends</span>
          </div>
          <div className="h-6 w-px bg-border/50" />
          <div className="flex items-center gap-1">
            {tabs.map(tab => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={cn(
                  "px-3 py-1.5 rounded-lg text-sm font-medium transition-all",
                  activeTab === tab.id
                    ? "bg-primary/15 text-primary"
                    : "text-muted-foreground hover:text-foreground hover:bg-secondary/50"
                )}
              >
                {tab.label}
                {tab.count !== undefined && (
                  <span className="ml-1.5 text-xs opacity-70">({tab.count})</span>
                )}
              </button>
            ))}
          </div>
          <div className="flex-1" />
          <button className="px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-all">
            Add Friend
          </button>
        </div>

        {/* Friends Grid */}
        <div className="flex-1 overflow-y-auto p-6">
          {activeTab === "pending" ? (
            <div className="space-y-3">
              {/* TODO: Wire pending friend requests from a relationships API */}
              <div className="rounded-2xl bg-secondary/30 border border-border/30 p-8 text-center">
                <p className="text-muted-foreground">Pending requests will appear here</p>
              </div>
            </div>
          ) : activeTab === "blocked" ? (
            <div className="space-y-3">
              {/* TODO: Wire blocked users from a relationships API */}
              <div className="rounded-2xl bg-secondary/30 border border-border/30 p-8 text-center">
                <p className="text-muted-foreground">Blocked users will appear here</p>
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredMembers.map(member => (
                <div
                  key={member.user_id}
                  className="flex items-center gap-4 p-4 rounded-xl bg-secondary/30 border border-border/30 hover:bg-secondary/50 transition-all cursor-pointer group"
                >
                  <div className="relative flex-shrink-0">
                    <div className="w-12 h-12 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-semibold">
                      {member.username.charAt(0).toUpperCase()}
                    </div>
                    <span className={cn(
                      "absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full border-2 border-card",
                      member.is_online ? "bg-online" : "bg-offline"
                    )} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="font-semibold text-foreground truncate">{member.username}</p>
                    <p className="text-sm text-muted-foreground truncate">
                      {member.is_online ? 'Online' : 'Offline'}
                    </p>
                  </div>
                  <button className="w-10 h-10 rounded-xl bg-secondary/50 hover:bg-primary/20 flex items-center justify-center transition-colors opacity-0 group-hover:opacity-100">
                    <MessageCircle className="w-5 h-5 text-muted-foreground group-hover:text-primary" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
