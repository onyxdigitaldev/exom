/// VoicePanel — voice connected bar and controls.
///
/// Shown in the channel sidebar when connected to a voice channel.
/// Displays: channel name, connection status, elapsed time, disconnect button.

import { useState, useEffect } from 'react'
import { cn } from '@/lib/utils'
import {
  PhoneOff,
  Mic,
  MicOff,
  Headphones,
  HeadphoneOff,
  Video,
  VideoOff,
  Monitor,
  MonitorOff,
  Signal,
} from 'lucide-react'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

interface VoicePanelProps {
  channelName: string
  connectedAt: number
  selfMute: boolean
  selfDeaf: boolean
  video: boolean
  streaming: boolean
  peerCount: number
  onToggleMute: () => void
  onToggleDeaf: () => void
  onToggleVideo: () => void
  onToggleScreenShare: () => void
  onDisconnect: () => void
}

function formatElapsed(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
}

export function VoicePanel({
  channelName,
  connectedAt,
  selfMute,
  selfDeaf,
  video,
  streaming,
  peerCount,
  onToggleMute,
  onToggleDeaf,
  onToggleVideo,
  onToggleScreenShare,
  onDisconnect,
}: VoicePanelProps) {
  const [elapsed, setElapsed] = useState('00:00')

  // Update elapsed timer every second
  useEffect(() => {
    const interval = setInterval(() => {
      setElapsed(formatElapsed(Date.now() - connectedAt))
    }, 1000)
    return () => clearInterval(interval)
  }, [connectedAt])

  return (
    <TooltipProvider delayDuration={100}>
      <div className="mx-2 mb-2 rounded-xl bg-online/10 border border-online/20 overflow-hidden">
        {/* Status bar */}
        <div className="px-3 py-2 flex items-center justify-between">
          <div className="flex items-center gap-2 min-w-0">
            <Signal className="w-4 h-4 text-online flex-shrink-0" />
            <div className="min-w-0">
              <p className="text-xs font-semibold text-online truncate">Voice Connected</p>
              <p className="text-[11px] text-muted-foreground truncate">{channelName}</p>
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            <span className="text-xs text-muted-foreground tabular-nums">{elapsed}</span>
            {peerCount > 0 && (
              <span className="text-[10px] text-muted-foreground bg-secondary/50 px-1.5 py-0.5 rounded">
                {peerCount + 1}
              </span>
            )}
          </div>
        </div>

        {/* Controls */}
        <div className="px-2 pb-2 flex items-center gap-1">
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onToggleMute}
                className={cn(
                  'flex-1 h-8 rounded-lg flex items-center justify-center transition-all',
                  selfMute
                    ? 'bg-destructive/20 text-destructive'
                    : 'hover:bg-secondary/50 text-foreground',
                )}
              >
                {selfMute ? <MicOff className="w-4 h-4" /> : <Mic className="w-4 h-4" />}
              </button>
            </TooltipTrigger>
            <TooltipContent><p>{selfMute ? 'Unmute' : 'Mute'}</p></TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onToggleDeaf}
                className={cn(
                  'flex-1 h-8 rounded-lg flex items-center justify-center transition-all',
                  selfDeaf
                    ? 'bg-destructive/20 text-destructive'
                    : 'hover:bg-secondary/50 text-foreground',
                )}
              >
                {selfDeaf ? <HeadphoneOff className="w-4 h-4" /> : <Headphones className="w-4 h-4" />}
              </button>
            </TooltipTrigger>
            <TooltipContent><p>{selfDeaf ? 'Undeafen' : 'Deafen'}</p></TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onToggleVideo}
                className={cn(
                  'flex-1 h-8 rounded-lg flex items-center justify-center transition-all',
                  video
                    ? 'bg-primary/20 text-primary'
                    : 'hover:bg-secondary/50 text-muted-foreground',
                )}
              >
                {video ? <Video className="w-4 h-4" /> : <VideoOff className="w-4 h-4" />}
              </button>
            </TooltipTrigger>
            <TooltipContent><p>{video ? 'Stop Camera' : 'Start Camera'}</p></TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onToggleScreenShare}
                className={cn(
                  'flex-1 h-8 rounded-lg flex items-center justify-center transition-all',
                  streaming
                    ? 'bg-primary/20 text-primary'
                    : 'hover:bg-secondary/50 text-muted-foreground',
                )}
              >
                {streaming ? <Monitor className="w-4 h-4" /> : <MonitorOff className="w-4 h-4" />}
              </button>
            </TooltipTrigger>
            <TooltipContent><p>{streaming ? 'Stop Sharing' : 'Share Screen'}</p></TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={onDisconnect}
                className="flex-1 h-8 rounded-lg flex items-center justify-center bg-destructive/10 hover:bg-destructive/20 text-destructive transition-all"
              >
                <PhoneOff className="w-4 h-4" />
              </button>
            </TooltipTrigger>
            <TooltipContent><p>Disconnect</p></TooltipContent>
          </Tooltip>
        </div>
      </div>
    </TooltipProvider>
  )
}
