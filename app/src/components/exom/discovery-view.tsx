import { useState } from "react"
import { cn } from "@/lib/utils"
import { 
  Search, 
  Compass, 
  TrendingUp, 
  Gamepad2, 
  Music, 
  Palette, 
  GraduationCap,
  Users,
  Plus,
  X,
  Upload
} from "lucide-react"
import { discoveryHalls } from "@/lib/mock-data"

interface DiscoveryViewProps {
  onClose: () => void
  onJoinHall?: (hallId: string) => void
}

const categories = [
  { id: "all", label: "All", icon: Compass },
  { id: "gaming", label: "Gaming", icon: Gamepad2 },
  { id: "music", label: "Music", icon: Music },
  { id: "art", label: "Art & Design", icon: Palette },
  { id: "education", label: "Education", icon: GraduationCap },
  { id: "tech", label: "Technology", icon: TrendingUp },
]

export function DiscoveryView({ onClose, onJoinHall }: DiscoveryViewProps) {
  const [searchQuery, setSearchQuery] = useState("")
  const [activeCategory, setActiveCategory] = useState("all")
  const [showCreateModal, setShowCreateModal] = useState(false)

  const filteredHalls = discoveryHalls.filter(hall => {
    if (searchQuery && !hall.name.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false
    }
    return true
  })

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="w-full max-w-5xl h-[85vh] bg-card rounded-3xl shadow-2xl border border-border/50 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-border/30">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-primary/20 flex items-center justify-center">
              <Compass className="w-5 h-5 text-primary" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-foreground">Discover Halls</h2>
              <p className="text-sm text-muted-foreground">Find communities that match your interests</p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 rounded-xl bg-primary text-primary-foreground font-medium text-sm hover:bg-primary/90 transition-all flex items-center gap-2"
            >
              <Plus className="w-4 h-4" />
              Create Hall
            </button>
            <button
              onClick={onClose}
              className="w-10 h-10 rounded-xl hover:bg-secondary/50 flex items-center justify-center transition-colors"
            >
              <X className="w-5 h-5 text-muted-foreground" />
            </button>
          </div>
        </div>

        {/* Search & Categories */}
        <div className="p-6 pb-4 space-y-4 border-b border-border/30">
          <div className="relative max-w-md">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-muted-foreground" />
            <input
              type="text"
              placeholder="Search for Halls..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full h-12 pl-12 pr-4 rounded-2xl bg-secondary/50 border border-border/50 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-transparent transition-all"
            />
          </div>

          <div className="flex items-center gap-2 flex-wrap">
            {categories.map(cat => {
              const Icon = cat.icon
              return (
                <button
                  key={cat.id}
                  onClick={() => setActiveCategory(cat.id)}
                  className={cn(
                    "flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all",
                    activeCategory === cat.id
                      ? "bg-primary text-primary-foreground shadow-lg shadow-primary/25"
                      : "bg-secondary/50 text-foreground hover:bg-secondary"
                  )}
                >
                  <Icon className="w-4 h-4" />
                  {cat.label}
                </button>
              )
            })}
          </div>
        </div>

        {/* Hall Grid */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredHalls.map(hall => (
              <div
                key={hall.id}
                className="group relative rounded-2xl border border-border/50 bg-secondary/20 hover:bg-secondary/40 transition-all overflow-hidden"
              >
                {/* Banner */}
                <div 
                  className="h-24 w-full"
                  style={{ 
                    background: `linear-gradient(135deg, ${hall.color}40, ${hall.color}80)` 
                  }}
                />
                
                {/* Content */}
                <div className="p-4 -mt-8 relative">
                  <div 
                    className="w-14 h-14 rounded-2xl flex items-center justify-center text-white font-bold text-xl shadow-lg border-4 border-card"
                    style={{ backgroundColor: hall.color }}
                  >
                    {hall.name.charAt(0)}
                  </div>
                  
                  <div className="mt-3">
                    <h3 className="font-semibold text-foreground text-lg">{hall.name}</h3>
                    <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{hall.description}</p>
                  </div>

                  <div className="flex items-center justify-between mt-4">
                    <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                      <Users className="w-4 h-4" />
                      <span>{hall.memberCount?.toLocaleString()} members</span>
                    </div>
                    <button
                      onClick={() => onJoinHall?.(hall.id)}
                      className="px-4 py-2 rounded-xl bg-primary/10 text-primary font-medium text-sm hover:bg-primary hover:text-primary-foreground transition-all"
                    >
                      Join
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Create Hall Modal */}
        {showCreateModal && (
          <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center p-6">
            <div className="w-full max-w-lg bg-card rounded-2xl shadow-2xl border border-border/50 overflow-hidden">
              <div className="p-6 border-b border-border/30">
                <h3 className="text-xl font-bold text-foreground">Create Your Hall</h3>
                <p className="text-sm text-muted-foreground mt-1">Give your new community a personality</p>
              </div>
              
              <div className="p-6 space-y-6">
                {/* Icon Upload */}
                <div className="flex items-center gap-4">
                  <button className="w-20 h-20 rounded-2xl bg-secondary/50 border-2 border-dashed border-border hover:border-primary/50 flex flex-col items-center justify-center gap-1 transition-all">
                    <Upload className="w-6 h-6 text-muted-foreground" />
                    <span className="text-[10px] text-muted-foreground">Upload</span>
                  </button>
                  <div className="flex-1">
                    <p className="text-sm font-medium text-foreground">Hall Icon</p>
                    <p className="text-xs text-muted-foreground mt-0.5">Recommended: 512x512 PNG or JPG</p>
                  </div>
                </div>

                {/* Hall Name */}
                <div>
                  <label className="block text-sm font-medium text-foreground mb-2">Hall Name</label>
                  <input
                    type="text"
                    placeholder="My Awesome Hall"
                    className="w-full h-11 px-4 rounded-xl bg-secondary/50 border border-border/50 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all"
                  />
                </div>

                {/* Template */}
                <div>
                  <label className="block text-sm font-medium text-foreground mb-3">Template</label>
                  <div className="grid grid-cols-2 gap-3">
                    <button className="p-4 rounded-xl bg-primary/10 border-2 border-primary text-left">
                      <p className="font-medium text-foreground">Create My Own</p>
                      <p className="text-xs text-muted-foreground mt-1">Start from scratch</p>
                    </button>
                    <button className="p-4 rounded-xl bg-secondary/50 border border-border/50 hover:border-primary/50 text-left transition-all">
                      <p className="font-medium text-foreground">Gaming</p>
                      <p className="text-xs text-muted-foreground mt-1">Voice channels & roles</p>
                    </button>
                  </div>
                </div>
              </div>

              <div className="p-6 pt-0 flex items-center justify-end gap-3">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="px-4 py-2.5 rounded-xl text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                >
                  Cancel
                </button>
                <button className="px-6 py-2.5 rounded-xl bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-all">
                  Create Hall
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
