import { useState } from "react"
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
import { mockUsers, mockDMs, type User } from "@/lib/mock-data"

type Tab = "online" | "all" | "pending" | "blocked"

interface DMPanelProps {
  onStartDM?: (user: User) => void
}

export function DMPanel({ onStartDM }: DMPanelProps) {
  const [activeTab, setActiveTab] = useState<Tab>("online")
  const [searchQuery, setSearchQuery] = useState("")

  const tabs: { id: Tab; label: string; count?: number }[] = [
    { id: "online", label: "Online", count: mockUsers.filter(u => u.status === 'online').length },
    { id: "all", label: "All", count: mockUsers.length },
    { id: "pending", label: "Pending", count: 2 },
    { id: "blocked", label: "Blocked" },
  ]

  const filteredUsers = mockUsers.filter(user => {
    if (searchQuery && !user.username.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }
    if (activeTab === "online") {
      return user.status === "online"
    }
    return true
  })

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
            {mockDMs.map((dm) => (
              <button
                key={dm.user.id}
                onClick={() => onStartDM?.(dm.user)}
                className="w-full flex items-center gap-3 px-2 py-2 rounded-lg hover:bg-secondary/50 transition-all group"
              >
                <div className="relative flex-shrink-0">
                  <div className="w-9 h-9 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-medium text-sm">
                    {dm.user.username.charAt(0).toUpperCase()}
                  </div>
                  <span className={cn(
                    "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-card",
                    dm.user.status === "online" && "bg-online",
                    dm.user.status === "idle" && "bg-idle",
                    dm.user.status === "dnd" && "bg-dnd",
                    dm.user.status === "offline" && "bg-offline"
                  )} />
                </div>
                <div className="flex-1 min-w-0 text-left">
                  <p className="text-sm font-medium text-foreground truncate">{dm.user.username}</p>
                  <p className="text-xs text-muted-foreground truncate">{dm.lastMessage}</p>
                </div>
                <X className="w-4 h-4 text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity" />
              </button>
            ))}
          </div>
        </div>

        {/* Friends Section */}
        <div className="flex-1 overflow-y-auto px-2">
          <h3 className="px-2 py-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            Friends — {filteredUsers.length}
          </h3>
          <div className="space-y-0.5">
            {filteredUsers.map(user => (
              <button
                key={user.id}
                onClick={() => onStartDM?.(user)}
                className={cn(
                  "w-full flex items-center gap-3 px-2 py-2 rounded-lg transition-all group",
                  user.status === 'offline' 
                    ? "opacity-60 hover:opacity-100" 
                    : "hover:bg-secondary/50"
                )}
              >
                <div className="relative flex-shrink-0">
                  <div className="w-9 h-9 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-medium text-sm">
                    {user.username.charAt(0).toUpperCase()}
                  </div>
                  <span className={cn(
                    "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-card",
                    user.status === "online" && "bg-online",
                    user.status === "idle" && "bg-idle",
                    user.status === "dnd" && "bg-dnd",
                    user.status === "offline" && "bg-offline"
                  )} />
                </div>
                <div className="flex-1 min-w-0 text-left">
                  <p className="text-sm font-medium text-foreground truncate">{user.username}</p>
                  {user.activity && (
                    <p className="text-xs text-muted-foreground truncate">{user.activity}</p>
                  )}
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
              <div className="text-sm text-muted-foreground mb-4">Pending — 2</div>
              {/* Pending requests */}
              <div className="flex items-center gap-4 p-4 rounded-xl bg-secondary/30 border border-border/30">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-semibold">
                  A
                </div>
                <div className="flex-1">
                  <p className="font-semibold text-foreground">AuroraStream</p>
                  <p className="text-sm text-muted-foreground">Incoming Friend Request</p>
                </div>
                <button className="w-10 h-10 rounded-xl bg-online/20 hover:bg-online/30 flex items-center justify-center transition-colors">
                  <Check className="w-5 h-5 text-online" />
                </button>
                <button className="w-10 h-10 rounded-xl bg-destructive/20 hover:bg-destructive/30 flex items-center justify-center transition-colors">
                  <X className="w-5 h-5 text-destructive" />
                </button>
              </div>
              <div className="flex items-center gap-4 p-4 rounded-xl bg-secondary/30 border border-border/30">
                <div className="w-12 h-12 rounded-full bg-gradient-to-br from-accent/80 to-primary/80 flex items-center justify-center text-white font-semibold">
                  R
                </div>
                <div className="flex-1">
                  <p className="font-semibold text-foreground">RiftWalker</p>
                  <p className="text-sm text-muted-foreground flex items-center gap-1">
                    <Clock className="w-3 h-3" />
                    Outgoing Friend Request
                  </p>
                </div>
                <button className="w-10 h-10 rounded-xl bg-destructive/20 hover:bg-destructive/30 flex items-center justify-center transition-colors">
                  <X className="w-5 h-5 text-destructive" />
                </button>
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredUsers.map(user => (
                <div
                  key={user.id}
                  className="flex items-center gap-4 p-4 rounded-xl bg-secondary/30 border border-border/30 hover:bg-secondary/50 transition-all cursor-pointer group"
                >
                  <div className="relative flex-shrink-0">
                    <div className="w-12 h-12 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-semibold">
                      {user.username.charAt(0).toUpperCase()}
                    </div>
                    <span className={cn(
                      "absolute -bottom-0.5 -right-0.5 w-4 h-4 rounded-full border-2 border-card",
                      user.status === "online" && "bg-online",
                      user.status === "idle" && "bg-idle",
                      user.status === "dnd" && "bg-dnd",
                      user.status === "offline" && "bg-offline"
                    )} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="font-semibold text-foreground truncate">{user.username}</p>
                    <p className="text-sm text-muted-foreground truncate">
                      {user.activity || (user.status === 'online' ? 'Online' : user.status === 'idle' ? 'Idle' : user.status === 'dnd' ? 'Do Not Disturb' : 'Offline')}
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
