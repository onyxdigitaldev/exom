import { useState } from "react"
import { cn } from "@/lib/utils"
import { 
  X, 
  Copy, 
  Check,
  ChevronDown,
  Clock,
  Users
} from "lucide-react"
import { mockHalls, mockUsers } from "@/lib/mock-data"

interface InviteModalProps {
  hallId: string
  onClose: () => void
}

export function InviteModal({ hallId, onClose }: InviteModalProps) {
  const hall = mockHalls.find(h => h.id === hallId)
  const [copied, setCopied] = useState(false)
  const [showExpiry, setShowExpiry] = useState(false)
  const [expiry, setExpiry] = useState("7 days")
  const [maxUses, setMaxUses] = useState("No limit")

  const inviteLink = `exom.app/invite/Abc123Xyz`

  const handleCopy = () => {
    navigator.clipboard.writeText(inviteLink)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="w-full max-w-md bg-card rounded-2xl shadow-2xl border border-border/50 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-border/30">
          <div>
            <h2 className="text-lg font-bold text-foreground">Invite friends to {hall?.name}</h2>
            <p className="text-sm text-muted-foreground mt-0.5">Share this link with others to grant access</p>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg hover:bg-secondary/50 flex items-center justify-center transition-colors"
          >
            <X className="w-4 h-4 text-muted-foreground" />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-5">
          {/* Invite Link */}
          <div>
            <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
              Invite Link
            </label>
            <div className="flex items-center gap-2">
              <div className="flex-1 h-11 px-4 rounded-xl bg-secondary/50 border border-border/50 flex items-center">
                <span className="text-foreground text-sm truncate">{inviteLink}</span>
              </div>
              <button
                onClick={handleCopy}
                className={cn(
                  "h-11 px-5 rounded-xl font-medium text-sm flex items-center gap-2 transition-all",
                  copied
                    ? "bg-online text-white"
                    : "bg-primary text-primary-foreground hover:bg-primary/90"
                )}
              >
                {copied ? (
                  <>
                    <Check className="w-4 h-4" />
                    Copied!
                  </>
                ) : (
                  <>
                    <Copy className="w-4 h-4" />
                    Copy
                  </>
                )}
              </button>
            </div>
          </div>

          {/* Settings Toggle */}
          <button
            onClick={() => setShowExpiry(!showExpiry)}
            className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
          >
            <ChevronDown className={cn("w-4 h-4 transition-transform", showExpiry && "rotate-180")} />
            Edit invite link settings
          </button>

          {/* Expiry Settings */}
          {showExpiry && (
            <div className="space-y-4 p-4 rounded-xl bg-secondary/30 border border-border/30">
              <div>
                <label className="flex items-center gap-2 text-sm font-medium text-foreground mb-2">
                  <Clock className="w-4 h-4 text-muted-foreground" />
                  Expire After
                </label>
                <select 
                  value={expiry}
                  onChange={(e) => setExpiry(e.target.value)}
                  className="w-full h-10 px-3 rounded-lg bg-secondary/50 border border-border/50 text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                >
                  <option>30 minutes</option>
                  <option>1 hour</option>
                  <option>6 hours</option>
                  <option>12 hours</option>
                  <option>1 day</option>
                  <option>7 days</option>
                  <option>Never</option>
                </select>
              </div>
              <div>
                <label className="flex items-center gap-2 text-sm font-medium text-foreground mb-2">
                  <Users className="w-4 h-4 text-muted-foreground" />
                  Max Uses
                </label>
                <select 
                  value={maxUses}
                  onChange={(e) => setMaxUses(e.target.value)}
                  className="w-full h-10 px-3 rounded-lg bg-secondary/50 border border-border/50 text-foreground text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                >
                  <option>No limit</option>
                  <option>1 use</option>
                  <option>5 uses</option>
                  <option>10 uses</option>
                  <option>25 uses</option>
                  <option>50 uses</option>
                  <option>100 uses</option>
                </select>
              </div>
              <button className="w-full h-10 rounded-lg bg-primary/10 text-primary text-sm font-medium hover:bg-primary/20 transition-all">
                Generate New Link
              </button>
            </div>
          )}

          {/* Friends to Invite */}
          <div>
            <label className="block text-xs font-medium text-muted-foreground uppercase tracking-wide mb-2">
              Or send invite to a friend
            </label>
            <div className="space-y-1 max-h-40 overflow-y-auto">
              {mockUsers.slice(0, 5).map(user => (
                <div
                  key={user.id}
                  className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-secondary/50 transition-colors cursor-pointer"
                >
                  <div className="relative">
                    <div className="w-8 h-8 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white text-sm font-medium">
                      {user.username.charAt(0).toUpperCase()}
                    </div>
                    <span className={cn(
                      "absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-card",
                      user.status === "online" && "bg-online",
                      user.status === "idle" && "bg-idle",
                      user.status === "dnd" && "bg-dnd",
                      user.status === "offline" && "bg-offline"
                    )} />
                  </div>
                  <span className="flex-1 text-sm text-foreground">{user.username}</span>
                  <button className="px-3 py-1 rounded-md bg-secondary/50 text-xs text-foreground hover:bg-secondary transition-colors">
                    Invite
                  </button>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
