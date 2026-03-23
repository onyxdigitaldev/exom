import { useState, useRef, useEffect } from "react"
import { cn } from "@/lib/utils"
import { 
  Hash, 
  Pin, 
  Bell, 
  Users, 
  Search,
  Smile,
  Plus,
  Gift,
  ImageIcon,
  MoreHorizontal,
  Reply,
  Pencil,
  Trash2,
  MessageSquare,
  ArrowUp
} from "lucide-react"
import { mockChannels, mockMessages, roleConfig, type Message, type User } from "@/lib/mock-data"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

interface MessageFeedProps {
  channelId: string
  showMembers: boolean
  onToggleMembers: () => void
}

function formatTime(date: Date): string {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function formatDate(date: Date): string {
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  if (date.toDateString() === today.toDateString()) {
    return "Today"
  } else if (date.toDateString() === yesterday.toDateString()) {
    return "Yesterday"
  }
  return date.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' })
}

function getUserRoleColor(user: User): string {
  return roleConfig[user.role]?.color || '#9CA3AF'
}

function MessageComponent({ 
  message, 
  isGrouped 
}: { 
  message: Message
  isGrouped: boolean 
}) {
  const [showActions, setShowActions] = useState(false)
  const roleColor = getUserRoleColor(message.author)

  return (
    <div 
      className={cn(
        "group relative px-4 py-1 hover:bg-secondary/30 transition-colors",
        !isGrouped && "mt-4 pt-2"
      )}
      onMouseEnter={() => setShowActions(true)}
      onMouseLeave={() => setShowActions(false)}
    >
      {/* Action bar on hover */}
      {showActions && (
        <div className="absolute -top-4 right-4 flex items-center gap-0.5 bg-card border border-border rounded-lg shadow-lg p-1 z-10">
          <TooltipProvider delayDuration={100}>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="p-1.5 rounded hover:bg-secondary transition-colors">
                  <Smile className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Add Reaction</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="p-1.5 rounded hover:bg-secondary transition-colors">
                  <Reply className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Reply</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="p-1.5 rounded hover:bg-secondary transition-colors">
                  <MessageSquare className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Create Thread</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="p-1.5 rounded hover:bg-secondary transition-colors">
                  <MoreHorizontal className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>More</p></TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      )}

      <div className="flex gap-4">
        {/* Avatar or timestamp gutter */}
        <div className="w-10 flex-shrink-0">
          {!isGrouped ? (
            <div className="w-10 h-10 rounded-full bg-gradient-to-br from-primary/80 to-accent/80 flex items-center justify-center text-white font-medium text-sm shadow-md">
              {message.author.username.charAt(0).toUpperCase()}
            </div>
          ) : (
            <span className="text-[10px] text-muted-foreground opacity-0 group-hover:opacity-100 transition-opacity leading-[40px]">
              {formatTime(message.timestamp)}
            </span>
          )}
        </div>

        <div className="flex-1 min-w-0">
          {/* Header (username, time) */}
          {!isGrouped && (
            <div className="flex items-baseline gap-2 mb-1">
              <span 
                className="font-semibold text-sm hover:underline cursor-pointer"
                style={{ color: roleColor }}
              >
                {message.author.username}
              </span>
              <span className="text-xs text-muted-foreground">
                {formatTime(message.timestamp)}
              </span>
              {message.edited && (
                <span className="text-[10px] text-muted-foreground">(edited)</span>
              )}
            </div>
          )}

          {/* Reply reference */}
          {message.replyTo && (
            <div className="flex items-center gap-2 mb-1 text-xs text-muted-foreground">
              <div className="w-8 h-4 border-l-2 border-t-2 border-muted-foreground/30 rounded-tl-md ml-1" />
              <span className="font-medium" style={{ color: getUserRoleColor(message.replyTo.author) }}>
                {message.replyTo.author.username}
              </span>
              <span className="truncate max-w-[300px] opacity-70">{message.replyTo.content}</span>
            </div>
          )}

          {/* Content */}
          {message.content && (
            <p className="text-foreground leading-relaxed break-words">{message.content}</p>
          )}

          {/* Image attachment */}
          {message.image && (
            <div className="mt-2 max-w-md">
              <div className="rounded-xl overflow-hidden border border-border/50 bg-gradient-to-br from-primary/20 via-accent/20 to-primary/10 h-48 flex items-center justify-center">
                <ImageIcon className="w-12 h-12 text-muted-foreground/50" />
              </div>
            </div>
          )}

          {/* File attachment */}
          {message.file && (
            <div className="mt-2 inline-flex items-center gap-3 px-4 py-3 rounded-xl bg-secondary/50 border border-border/50">
              <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                <ImageIcon className="w-5 h-5 text-primary" />
              </div>
              <div>
                <p className="text-sm font-medium text-foreground">{message.file.name}</p>
                <p className="text-xs text-muted-foreground">{message.file.size}</p>
              </div>
            </div>
          )}

          {/* Reactions */}
          {message.reactions && message.reactions.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-2">
              {message.reactions.map((reaction, i) => (
                <button
                  key={i}
                  className={cn(
                    "inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-sm transition-all",
                    reaction.reacted
                      ? "bg-primary/20 border border-primary/40 text-primary"
                      : "bg-secondary/50 border border-transparent hover:border-border text-foreground"
                  )}
                >
                  <span>{reaction.emoji}</span>
                  <span className="text-xs font-medium">{reaction.count}</span>
                </button>
              ))}
              <button className="w-7 h-7 rounded-lg bg-secondary/30 hover:bg-secondary/50 flex items-center justify-center transition-colors">
                <Smile className="w-4 h-4 text-muted-foreground" />
              </button>
            </div>
          )}

          {/* Thread link */}
          {message.threadCount && message.threadCount > 0 && (
            <button className="mt-2 flex items-center gap-2 text-sm text-primary hover:underline">
              <MessageSquare className="w-4 h-4" />
              <span>{message.threadCount} replies</span>
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export function MessageFeed({ channelId, showMembers, onToggleMembers }: MessageFeedProps) {
  const channel = mockChannels.find(c => c.id === channelId)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const [inputValue, setInputValue] = useState("")

  // Group messages by date and consecutive author
  const groupedMessages: { date: string; messages: { message: Message; isGrouped: boolean }[] }[] = []
  
  let currentDate = ""
  let lastAuthorId = ""
  let lastTimestamp: Date | null = null

  mockMessages.forEach((msg) => {
    const msgDate = formatDate(msg.timestamp)
    
    if (msgDate !== currentDate) {
      currentDate = msgDate
      groupedMessages.push({ date: msgDate, messages: [] })
      lastAuthorId = ""
      lastTimestamp = null
    }

    const timeDiff = lastTimestamp 
      ? (msg.timestamp.getTime() - lastTimestamp.getTime()) / 1000 / 60 
      : Infinity

    const isGrouped = msg.author.id === lastAuthorId && timeDiff < 5 && !msg.replyTo

    groupedMessages[groupedMessages.length - 1].messages.push({
      message: msg,
      isGrouped,
    })

    lastAuthorId = msg.author.id
    lastTimestamp = msg.timestamp
  })

  return (
    <div className="flex-1 flex flex-col min-w-0 bg-card/30 rounded-2xl m-2 ml-0 overflow-hidden border border-border/30">
      {/* Channel Header */}
      <div className="h-14 px-4 flex items-center justify-between border-b border-border/30 bg-card/50">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-secondary/50 flex items-center justify-center">
            <Hash className="w-4 h-4 text-muted-foreground" />
          </div>
          <div>
            <h2 className="font-semibold text-foreground">{channel?.name}</h2>
            {channel?.topic && (
              <p className="text-xs text-muted-foreground truncate max-w-[300px]">{channel.topic}</p>
            )}
          </div>
        </div>

        <div className="flex items-center gap-1">
          <TooltipProvider delayDuration={100}>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="w-8 h-8 rounded-lg hover:bg-secondary/50 flex items-center justify-center transition-colors">
                  <MessageSquare className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Threads</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="w-8 h-8 rounded-lg hover:bg-secondary/50 flex items-center justify-center transition-colors">
                  <Pin className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Pinned Messages</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="w-8 h-8 rounded-lg hover:bg-secondary/50 flex items-center justify-center transition-colors">
                  <Bell className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Notification Settings</p></TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger asChild>
                <button 
                  onClick={onToggleMembers}
                  className={cn(
                    "w-8 h-8 rounded-lg flex items-center justify-center transition-colors",
                    showMembers ? "bg-primary/20 text-primary" : "hover:bg-secondary/50 text-muted-foreground"
                  )}
                >
                  <Users className="w-4 h-4" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Member List</p></TooltipContent>
            </Tooltip>
            <div className="w-px h-5 bg-border/50 mx-1" />
            <Tooltip>
              <TooltipTrigger asChild>
                <button className="w-8 h-8 rounded-lg hover:bg-secondary/50 flex items-center justify-center transition-colors">
                  <Search className="w-4 h-4 text-muted-foreground" />
                </button>
              </TooltipTrigger>
              <TooltipContent><p>Search</p></TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto">
        {groupedMessages.map((group) => (
          <div key={group.date}>
            {/* Date divider */}
            <div className="flex items-center gap-4 px-4 py-4">
              <div className="flex-1 h-px bg-border/50" />
              <span className="text-xs font-medium text-muted-foreground">{group.date}</span>
              <div className="flex-1 h-px bg-border/50" />
            </div>

            {/* Messages for this date */}
            {group.messages.map(({ message, isGrouped }) => (
              <MessageComponent key={message.id} message={message} isGrouped={isGrouped} />
            ))}
          </div>
        ))}
        <div ref={messagesEndRef} className="h-4" />
      </div>

      {/* Message Input */}
      <div className="p-4 pt-2">
        <div className="relative">
          <div className="flex items-end gap-2 bg-secondary/50 rounded-2xl border border-border/50 p-2 focus-within:border-primary/50 transition-colors">
            <button className="w-10 h-10 rounded-xl hover:bg-secondary flex items-center justify-center transition-colors flex-shrink-0">
              <Plus className="w-5 h-5 text-muted-foreground" />
            </button>
            
            <textarea
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder={`Message #${channel?.name}`}
              className="flex-1 bg-transparent resize-none text-foreground placeholder:text-muted-foreground focus:outline-none py-2.5 px-1 max-h-32 min-h-[40px]"
              rows={1}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  // Send message logic
                  setInputValue("")
                }
              }}
            />

            <div className="flex items-center gap-1 flex-shrink-0">
              <button className="w-10 h-10 rounded-xl hover:bg-secondary flex items-center justify-center transition-colors">
                <Gift className="w-5 h-5 text-muted-foreground" />
              </button>
              <button className="w-10 h-10 rounded-xl hover:bg-secondary flex items-center justify-center transition-colors">
                <ImageIcon className="w-5 h-5 text-muted-foreground" />
              </button>
              <button className="w-10 h-10 rounded-xl hover:bg-secondary flex items-center justify-center transition-colors">
                <Smile className="w-5 h-5 text-muted-foreground" />
              </button>
              {inputValue.trim() && (
                <button className="w-10 h-10 rounded-xl bg-primary hover:bg-primary/90 flex items-center justify-center transition-colors">
                  <ArrowUp className="w-5 h-5 text-primary-foreground" />
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
