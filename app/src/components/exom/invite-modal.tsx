import { useState, useEffect, useCallback } from "react"
import { cn } from "@/lib/utils"
import {
  X,
  Copy,
  Check,
  ChevronDown,
  Clock,
  Users
} from "lucide-react"
import { useHallStore } from "@/stores/hallStore"
import { useMemberStore } from "@/stores/memberStore"
import { invites as invitesApi } from "@/lib/api"

interface InviteModalProps {
  hallId: string
  onClose: () => void
}

/** Map expiry label to hours for the API. */
const expiryToHours: Record<string, number | undefined> = {
  "30 minutes": 0.5,
  "1 hour": 1,
  "6 hours": 6,
  "12 hours": 12,
  "1 day": 24,
  "7 days": 168,
  "Never": undefined,
}

/** Map max-uses label to the numeric value for the API. */
const maxUsesToNumber: Record<string, number | undefined> = {
  "No limit": undefined,
  "1 use": 1,
  "5 uses": 5,
  "10 uses": 10,
  "25 uses": 25,
  "50 uses": 50,
  "100 uses": 100,
}

export function InviteModal({ hallId, onClose }: InviteModalProps) {
  const { halls } = useHallStore()
  const { members } = useMemberStore()
  const hall = halls.find(h => h.id === hallId)

  const [copied, setCopied] = useState(false)
  const [showExpiry, setShowExpiry] = useState(false)
  const [expiry, setExpiry] = useState("7 days")
  const [maxUses, setMaxUses] = useState("No limit")
  const [inviteLink, setInviteLink] = useState("")
  const [generating, setGenerating] = useState(false)

  const generateInvite = useCallback(async (expiryLabel?: string, maxUsesLabel?: string) => {
    setGenerating(true)
    try {
      const hours = expiryToHours[expiryLabel ?? expiry]
      const uses = maxUsesToNumber[maxUsesLabel ?? maxUses]
      const invite = await invitesApi.create(hallId, undefined, hours, uses)
      setInviteLink(`exom.app/invite/${invite.token}`)
    } catch {
      setInviteLink("Failed to generate invite")
    } finally {
      setGenerating(false)
    }
  }, [hallId, expiry, maxUses])

  // Generate an invite on mount
  useEffect(() => {
    generateInvite()
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handleCopy = () => {
    navigator.clipboard.writeText(inviteLink)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleGenerateNew = () => {
    generateInvite(expiry, maxUses)
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
                <span className="text-foreground text-sm truncate">
                  {generating ? "Generating..." : inviteLink}
                </span>
              </div>
              <button
                onClick={handleCopy}
                disabled={generating || !inviteLink}
                className={cn(
                  "h-11 px-5 rounded-xl font-medium text-sm flex items-center gap-2 transition-all",
                  copied
                    ? "bg-online text-white"
                    : "bg-primary text-primary-foreground hover:bg-primary/90",
                  (generating || !inviteLink) && "opacity-50 cursor-not-allowed"
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
              <button
                onClick={handleGenerateNew}
                disabled={generating}
                className="w-full h-10 rounded-lg bg-primary/10 text-primary text-sm font-medium hover:bg-primary/20 transition-all disabled:opacity-50"
              >
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
              {members.slice(0, 5).map(member => (
                <div
                  key={member.user_id}
                  className="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-secondary/50 transition-colors cursor-pointer"
                >
                  <div className="relative">
                    <div className="w-8 h-8 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white text-sm font-medium">
                      {member.username.charAt(0).toUpperCase()}
                    </div>
                    <span className={cn(
                      "absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-card",
                      member.is_online ? "bg-online" : "bg-offline"
                    )} />
                  </div>
                  <span className="flex-1 text-sm text-foreground">{member.username}</span>
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
