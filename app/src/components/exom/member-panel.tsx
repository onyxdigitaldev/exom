import { cn } from "@/lib/utils"
import { roleConfig, type RoleKey } from "@/lib/constants"
import { Search } from "lucide-react"
import { useMemberStore } from "@/stores/memberStore"
import type { Member } from "@/lib/types"

interface MemberPanelProps {
  onMemberClick?: (member: Member) => void
}

export function MemberPanel({ onMemberClick }: MemberPanelProps) {
  const { members } = useMemberStore()

  // Group members by role
  const membersByRole: Record<RoleKey, Member[]> = {
    builder: [],
    prefect: [],
    moderator: [],
    agent: [],
    fellow: [],
  }

  members.forEach(member => {
    const key = member.role.toLowerCase() as RoleKey
    if (membersByRole[key]) {
      membersByRole[key].push(member)
    } else {
      membersByRole.agent.push(member)
    }
  })

  const roleOrder: RoleKey[] = ['builder', 'prefect', 'moderator', 'agent', 'fellow']

  return (
    <div className="w-60 flex flex-col h-full bg-card/30 rounded-2xl m-2 mr-0 ml-0 border border-border/30 overflow-hidden">
      {/* Search */}
      <div className="p-3 border-b border-border/30">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Search members"
            className="w-full h-9 pl-9 pr-3 rounded-lg bg-secondary/50 border border-transparent text-sm placeholder:text-muted-foreground focus:outline-none focus:border-primary/50 transition-all"
          />
        </div>
      </div>

      {/* Member List */}
      <div className="flex-1 overflow-y-auto p-2">
        {roleOrder.map(role => {
          const users = membersByRole[role]
          if (users.length === 0) return null

          const config = roleConfig[role]
          const onlineCount = users.filter(u => u.is_online).length

          return (
            <div key={role} className="mb-4">
              <h3 className="px-2 py-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                {config.label} — {onlineCount}
              </h3>
              <div className="space-y-0.5">
                {users.map(member => (
                  <button
                    key={member.user_id}
                    onClick={() => onMemberClick?.(member)}
                    className={cn(
                      "w-full flex items-center gap-3 px-2 py-2 rounded-lg transition-all group",
                      !member.is_online
                        ? "opacity-50 hover:opacity-100"
                        : "hover:bg-secondary/50"
                    )}
                  >
                    <div className="relative flex-shrink-0">
                      <div
                        className="w-8 h-8 rounded-full flex items-center justify-center text-white font-medium text-sm"
                        style={{
                          background: `linear-gradient(135deg, ${config.color}80, ${config.color})`
                        }}
                      >
                        {member.username.charAt(0).toUpperCase()}
                      </div>
                      <span className={cn(
                        "absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-card",
                        member.is_online ? "bg-online" : "bg-offline"
                      )} />
                    </div>

                    <div className="flex-1 min-w-0 text-left">
                      <p
                        className="text-sm font-medium truncate"
                        style={{ color: config.color }}
                      >
                        {member.username}
                      </p>
                      {member.is_host && (
                        <p className="text-xs text-primary/70 truncate">Host</p>
                      )}
                    </div>
                  </button>
                ))}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
