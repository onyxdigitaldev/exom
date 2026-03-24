import { useState } from "react"
import { cn } from "@/lib/utils"
import { Home, Compass, Plus, Settings, Search, Bell, MessageCircle } from "lucide-react"
import { useAuthStore } from "@/stores/authStore"
import { useHallStore } from "@/stores/hallStore"

interface TopNavigationProps {
  activeHall: string | null
  onHallSelect: (hallId: string | null) => void
  onShowDiscovery: () => void
  onShowDMs: () => void
  onShowSettings: () => void
  showDMs: boolean
}

export function TopNavigation({
  activeHall,
  onHallSelect,
  onShowDiscovery,
  onShowDMs,
  onShowSettings,
  showDMs,
}: TopNavigationProps) {
  const [searchOpen, setSearchOpen] = useState(false)
  const { halls } = useHallStore()
  const { user } = useAuthStore()
  const displayName = user?.username ?? 'User'

  return (
      <header className="h-16 glass border-b border-border/50 flex items-center justify-between px-4 sticky top-0 z-50">
        {/* Left: Logo & Home */}
        <div className="flex items-center gap-3">
          <button
            onClick={() => onHallSelect(null)}
            className="flex items-center gap-2 px-3 py-2 rounded-xl hover:bg-secondary/50 transition-all"
          >
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <span className="text-primary-foreground font-bold text-sm">E</span>
            </div>
            <span className="font-semibold text-foreground hidden sm:inline">Exom</span>
          </button>

          <div className="h-6 w-px bg-border/50 hidden sm:block" />

          {/* Hall Navigation */}
          <nav className="flex items-center gap-1">
            <button
              title="Direct Messages"
              onClick={onShowDMs}
              className={cn(
                "w-10 h-10 rounded-xl flex items-center justify-center transition-all",
                showDMs
                  ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25"
                  : "hover:bg-secondary/70 text-muted-foreground hover:text-foreground"
              )}
            >
              <MessageCircle className="w-5 h-5" />
            </button>

            <div className="h-6 w-px bg-border/30 mx-1" />

            {halls.map((hall) => (
              <button
                key={hall.id}
                title={hall.name}
                onClick={() => onHallSelect(hall.id)}
                className={cn(
                  "relative w-10 h-10 rounded-xl flex items-center justify-center transition-all font-medium text-sm",
                  activeHall === hall.id
                    ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25"
                    : "bg-secondary/50 hover:bg-secondary text-foreground hover:scale-105"
                )}
              >
                {hall.name.charAt(0)}
              </button>
            ))}

            <button
              title="Add a Hall"
              onClick={onShowDiscovery}
              className="w-10 h-10 rounded-xl flex items-center justify-center bg-secondary/30 hover:bg-accent/20 text-muted-foreground hover:text-accent transition-all border-2 border-dashed border-border/50 hover:border-accent/50"
            >
              <Plus className="w-5 h-5" />
            </button>
          </nav>
        </div>

        {/* Center: Search */}
        <div className="hidden md:flex flex-1 max-w-md mx-8">
          <div className="relative w-full">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              placeholder="Search messages, channels, or members..."
              className="w-full h-10 pl-10 pr-4 rounded-xl bg-secondary/50 border border-border/50 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent transition-all"
            />
            <kbd className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground bg-background/50 px-2 py-0.5 rounded border border-border/50">
              ⌘K
            </kbd>
          </div>
        </div>

        {/* Right: User & Actions */}
        <div className="flex items-center gap-2">
          <button
            title="Notifications"
            className="w-10 h-10 rounded-xl flex items-center justify-center hover:bg-secondary/70 text-muted-foreground hover:text-foreground transition-all relative"
          >
            <Bell className="w-5 h-5" />
            <span className="absolute top-1.5 right-1.5 w-2 h-2 bg-destructive rounded-full" />
          </button>

          <button
            title="Discover Halls"
            onClick={onShowDiscovery}
            className="w-10 h-10 rounded-xl flex items-center justify-center hover:bg-secondary/70 text-muted-foreground hover:text-foreground transition-all"
          >
            <Compass className="w-5 h-5" />
          </button>

          <div className="h-6 w-px bg-border/50" />

          <button
            title={displayName}
            onClick={onShowSettings}
            className="flex items-center gap-2 px-2 py-1.5 rounded-xl hover:bg-secondary/70 transition-all"
          >
            <div className="relative">
              <div className="w-8 h-8 rounded-full bg-gradient-to-br from-primary to-accent flex items-center justify-center text-white font-medium text-sm">
                {displayName.charAt(0).toUpperCase()}
              </div>
              <span className="absolute -bottom-0.5 -right-0.5 w-3 h-3 bg-online rounded-full border-2 border-background" />
            </div>
          </button>
        </div>
      </header>
  )
}
