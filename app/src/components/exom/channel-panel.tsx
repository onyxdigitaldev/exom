import { useState } from "react"
import { cn } from "@/lib/utils"
import { 
  Hash, 
  Volume2, 
  Megaphone, 
  ChevronDown, 
  ChevronRight,
  Plus,
  Settings,
  UserPlus,
  Lock,
  Users
} from "lucide-react"
import { mockChannels, mockHalls, type Channel } from "@/lib/mock-data"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

interface ChannelPanelProps {
  hallId: string
  activeChannel: string
  onChannelSelect: (channelId: string) => void
  onInvite: () => void
  onHallSettings: () => void
  onCreateChannel: () => void
}

export function ChannelPanel({
  hallId,
  activeChannel,
  onChannelSelect,
  onInvite,
  onHallSettings,
  onCreateChannel,
}: ChannelPanelProps) {
  const hall = mockHalls.find(h => h.id === hallId)
  const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())

  const toggleCategory = (category: string) => {
    const next = new Set(collapsedCategories)
    if (next.has(category)) {
      next.delete(category)
    } else {
      next.add(category)
    }
    setCollapsedCategories(next)
  }

  const categories = Array.from(new Set(mockChannels.map(c => c.category)))

  const getChannelIcon = (type: Channel["type"]) => {
    switch (type) {
      case "voice":
        return Volume2
      case "announcement":
        return Megaphone
      default:
        return Hash
    }
  }

  return (
    <div className="w-64 flex flex-col h-full">
      {/* Hall Header */}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="h-14 px-4 flex items-center justify-between hover:bg-secondary/50 transition-all border-b border-border/30 group">
            <div className="flex items-center gap-3">
              <div 
                className="w-9 h-9 rounded-xl flex items-center justify-center font-semibold text-sm text-white shadow-lg"
                style={{ backgroundColor: hall?.color || '#5865F2' }}
              >
                {hall?.name.charAt(0)}
              </div>
              <div className="text-left">
                <h2 className="font-semibold text-foreground text-sm truncate max-w-[120px]">
                  {hall?.name}
                </h2>
                <p className="text-xs text-muted-foreground">{hall?.memberCount?.toLocaleString()} members</p>
              </div>
            </div>
            <ChevronDown className="w-4 h-4 text-muted-foreground group-hover:text-foreground transition-colors" />
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-56">
          <DropdownMenuItem onClick={onInvite}>
            <UserPlus className="w-4 h-4 mr-2" />
            Invite People
          </DropdownMenuItem>
          <DropdownMenuItem onClick={onHallSettings}>
            <Settings className="w-4 h-4 mr-2" />
            Hall Settings
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={onCreateChannel}>
            <Plus className="w-4 h-4 mr-2" />
            Create Channel
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Quick Actions */}
      <div className="p-3 flex gap-2">
        <button 
          onClick={onInvite}
          className="flex-1 h-9 rounded-lg bg-primary/10 hover:bg-primary/20 text-primary text-xs font-medium flex items-center justify-center gap-1.5 transition-all"
        >
          <UserPlus className="w-3.5 h-3.5" />
          Invite
        </button>
        <button className="flex-1 h-9 rounded-lg bg-secondary/50 hover:bg-secondary text-foreground text-xs font-medium flex items-center justify-center gap-1.5 transition-all">
          <Users className="w-3.5 h-3.5" />
          Members
        </button>
      </div>

      {/* Channel List */}
      <div className="flex-1 overflow-y-auto px-2 pb-4">
        {categories.map((category) => {
          const categoryChannels = mockChannels.filter(c => c.category === category)
          const isCollapsed = collapsedCategories.has(category)

          return (
            <div key={category} className="mb-4">
              <button
                onClick={() => toggleCategory(category)}
                className="flex items-center gap-1 w-full px-2 py-1.5 text-xs font-semibold text-muted-foreground hover:text-foreground uppercase tracking-wide transition-colors group"
              >
                {isCollapsed ? (
                  <ChevronRight className="w-3 h-3" />
                ) : (
                  <ChevronDown className="w-3 h-3" />
                )}
                <span className="flex-1 text-left">{category}</span>
                <Plus className="w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity" />
              </button>

              {!isCollapsed && (
                <div className="mt-1 space-y-0.5">
                  {categoryChannels.map((channel) => {
                    const Icon = getChannelIcon(channel.type)
                    const isActive = channel.id === activeChannel

                    return (
                      <button
                        key={channel.id}
                        onClick={() => onChannelSelect(channel.id)}
                        className={cn(
                          "w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm transition-all group",
                          isActive
                            ? "bg-primary/15 text-primary"
                            : "text-muted-foreground hover:text-foreground hover:bg-secondary/50"
                        )}
                      >
                        <Icon className={cn(
                          "w-4 h-4 flex-shrink-0",
                          channel.type === "voice" && "text-accent"
                        )} />
                        <span className="flex-1 text-left truncate">
                          {channel.name}
                        </span>
                        {channel.mentionCount && channel.mentionCount > 0 && (
                          <span className="px-1.5 py-0.5 text-xs font-medium bg-destructive text-destructive-foreground rounded-full">
                            {channel.mentionCount}
                          </span>
                        )}
                        {channel.unread && !channel.mentionCount && (
                          <span className="w-2 h-2 bg-primary rounded-full" />
                        )}

                        {/* Voice channel connected users */}
                        {channel.type === "voice" && channel.connectedUsers && channel.connectedUsers.length > 0 && (
                          <span className="text-xs text-muted-foreground">
                            {channel.connectedUsers.length}
                          </span>
                        )}
                      </button>
                    )
                  })}
                  
                  {/* Show connected users for voice channels */}
                  {categoryChannels
                    .filter(c => c.type === "voice" && c.connectedUsers?.length)
                    .map(channel => (
                      <div key={`${channel.id}-users`} className="pl-8 space-y-1 mb-2">
                        {channel.connectedUsers?.map(user => (
                          <div 
                            key={user.id}
                            className="flex items-center gap-2 py-1 px-2 rounded text-xs text-muted-foreground"
                          >
                            <div className="relative">
                              <div className="w-5 h-5 rounded-full bg-secondary flex items-center justify-center text-[10px] font-medium">
                                {user.username.charAt(0)}
                              </div>
                              <span className={cn(
                                "absolute -bottom-0.5 -right-0.5 w-2 h-2 rounded-full border border-card",
                                user.status === "online" && "bg-online",
                                user.status === "idle" && "bg-idle",
                                user.status === "dnd" && "bg-dnd",
                                user.status === "offline" && "bg-offline"
                              )} />
                            </div>
                            <span className="truncate">{user.username}</span>
                          </div>
                        ))}
                      </div>
                    ))
                  }
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
