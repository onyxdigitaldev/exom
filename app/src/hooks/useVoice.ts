/// useVoice — React hook for voice channel participation.
///
/// Orchestrates the MediaManager, PeerManager, and WebSocket signaling
/// into a single interface the UI can consume.
///
/// Usage:
/// ```ts
/// const voice = useVoice()
/// await voice.join(hallId, channelId, channelName)
/// voice.toggleMute()
/// voice.leave()
/// ```

import { useCallback, useEffect, useRef, useState } from 'react'
import { MediaManager, PeerManager } from '@/lib/webrtc'
import type { PeerManagerEvent, LocalMediaState, PeerState } from '@/lib/webrtc'
import { useVoiceStore } from '@/stores/voiceStore'
import { useWebSocket, type RelayEvent } from '@/hooks/useWebSocket'
import { useAuthStore } from '@/stores/authStore'

/** Per-peer state exposed to the UI. */
export interface VoicePeer {
  userId: string
  state: PeerState
  audioStream: MediaStream | null
}

export function useVoice() {
  const { user } = useAuthStore()
  const voiceStore = useVoiceStore()
  const { send } = useWebSocket()

  const mediaRef = useRef<MediaManager | null>(null)
  const peerManagerRef = useRef<PeerManager | null>(null)
  const audioElementsRef = useRef<Map<string, HTMLAudioElement>>(new Map())

  const [localMedia, setLocalMedia] = useState<LocalMediaState>({
    audioEnabled: false,
    videoEnabled: false,
    screenShareEnabled: false,
    audioStream: null,
    videoStream: null,
    screenStream: null,
  })
  const [peers, setPeers] = useState<VoicePeer[]>([])
  const [isConnected, setIsConnected] = useState(false)

  /** Create or get the MediaManager singleton. */
  const getMedia = useCallback(() => {
    if (!mediaRef.current) {
      mediaRef.current = new MediaManager()
      mediaRef.current.setOnStateChange(setLocalMedia)
    }
    return mediaRef.current
  }, [])

  /** Play a remote audio stream through an HTMLAudioElement. */
  const playRemoteAudio = useCallback((userId: string, stream: MediaStream) => {
    let el = audioElementsRef.current.get(userId)
    if (!el) {
      el = new Audio()
      el.autoplay = true
      audioElementsRef.current.set(userId, el)
    }
    el.srcObject = stream
    el.play().catch(() => {
      // Autoplay may be blocked until user interaction — retry silently
    })

    // Apply preferred output device if set
    const media = mediaRef.current
    if (media?.getAudioOutput() && 'setSinkId' in el) {
      (el as any).setSinkId(media.getAudioOutput())
    }
  }, [])

  /** Stop playing a remote audio stream. */
  const stopRemoteAudio = useCallback((userId: string) => {
    const el = audioElementsRef.current.get(userId)
    if (el) {
      el.srcObject = null
      el.pause()
      audioElementsRef.current.delete(userId)
    }
  }, [])

  /** Handle PeerManager events — update UI state and play audio. */
  const handlePeerEvent = useCallback((event: PeerManagerEvent) => {
    switch (event.type) {
      case 'peer-added':
        setPeers((prev) => [
          ...prev.filter((p) => p.userId !== event.userId),
          { userId: event.userId, state: 'new', audioStream: null },
        ])
        break

      case 'peer-removed':
        setPeers((prev) => prev.filter((p) => p.userId !== event.userId))
        stopRemoteAudio(event.userId)
        break

      case 'peer-state-changed':
        setPeers((prev) =>
          prev.map((p) => (p.userId === event.userId ? { ...p, state: event.state } : p)),
        )
        break

      case 'remote-stream-added':
        // Play audio streams automatically
        if (event.stream.getAudioTracks().length > 0) {
          playRemoteAudio(event.userId, event.stream)
        }
        setPeers((prev) =>
          prev.map((p) =>
            p.userId === event.userId ? { ...p, audioStream: event.stream } : p,
          ),
        )
        break

      case 'remote-stream-removed':
        stopRemoteAudio(event.userId)
        break

      // Signaling events — forward to relay
      case 'offer-created':
        send({
          VoiceOffer: { target_user_id: event.userId, sdp: event.sdp },
        })
        break

      case 'answer-created':
        send({
          VoiceAnswer: { target_user_id: event.userId, sdp: event.sdp },
        })
        break

      case 'ice-candidate':
        send({
          VoiceIceCandidate: { target_user_id: event.userId, candidate: event.candidate },
        })
        break

      case 'error':
        console.error(`Voice peer error (${event.userId}):`, event.error)
        break
    }
  }, [send, playRemoteAudio, stopRemoteAudio])

  // ── Public API ────────────────────────────────

  /** Join a voice channel. Starts audio capture and signaling. */
  const join = useCallback(async (hallId: string, channelId: string, channelName: string) => {
    const media = getMedia()

    // Start capturing audio
    await media.enableAudio()

    // Create peer manager
    const pm = new PeerManager(media, handlePeerEvent)
    peerManagerRef.current = pm

    // Tell the relay we're joining
    send({ VoiceJoin: { hall_id: hallId, channel_id: channelId } })

    // Update store
    voiceStore.join(hallId, channelId, channelName)
    setIsConnected(true)
  }, [getMedia, handlePeerEvent, send, voiceStore])

  /** Leave the voice channel. Stops all media and closes connections. */
  const leave = useCallback(() => {
    // Close peer connections
    peerManagerRef.current?.dispose()
    peerManagerRef.current = null

    // Stop all media
    mediaRef.current?.dispose()
    mediaRef.current = null

    // Stop all audio playback
    for (const [userId] of audioElementsRef.current) {
      stopRemoteAudio(userId)
    }

    // Tell the relay
    send({ VoiceLeave: null })

    // Update store
    voiceStore.leave()
    setPeers([])
    setIsConnected(false)
    setLocalMedia({
      audioEnabled: false,
      videoEnabled: false,
      screenShareEnabled: false,
      audioStream: null,
      videoStream: null,
      screenStream: null,
    })
  }, [send, voiceStore, stopRemoteAudio])

  /** Toggle microphone mute. */
  const toggleMute = useCallback(() => {
    const media = getMedia()
    const muted = media.toggleMute()
    voiceStore.toggleMute()

    send({
      VoiceStateUpdate: {
        self_mute: muted,
        self_deaf: voiceStore.selfDeaf,
        video: voiceStore.video,
        streaming: voiceStore.streaming,
      },
    })
  }, [getMedia, voiceStore, send])

  /** Toggle deafen (mute all incoming audio). */
  const toggleDeaf = useCallback(() => {
    voiceStore.toggleDeaf()
    const deaf = !voiceStore.selfDeaf
    peerManagerRef.current?.setDeafened(deaf)

    // Deafening also mutes outgoing
    if (deaf) {
      getMedia().muteAudio()
      voiceStore.toggleMute()
    }

    send({
      VoiceStateUpdate: {
        self_mute: deaf || voiceStore.selfMute,
        self_deaf: deaf,
        video: voiceStore.video,
        streaming: voiceStore.streaming,
      },
    })
  }, [getMedia, voiceStore, send])

  /** Toggle camera. */
  const toggleVideo = useCallback(async () => {
    const media = getMedia()
    const enabled = await media.toggleVideo()
    voiceStore.toggleVideo()

    if (enabled && media.getState().videoStream) {
      const track = media.getState().videoStream!.getVideoTracks()[0]
      if (track) {
        peerManagerRef.current?.addTrackToAllPeers(track, media.getState().videoStream!)
      }
    } else {
      // Remove video track from all peers
      // (handled by track ending event in peer connection)
    }

    send({
      VoiceStateUpdate: {
        self_mute: voiceStore.selfMute,
        self_deaf: voiceStore.selfDeaf,
        video: enabled,
        streaming: voiceStore.streaming,
      },
    })
  }, [getMedia, voiceStore, send])

  /** Toggle screen share. */
  const toggleScreenShare = useCallback(async () => {
    const media = getMedia()
    try {
      const enabled = await media.toggleScreenShare()
      voiceStore.toggleStream()

      if (enabled && media.getState().screenStream) {
        const track = media.getState().screenStream!.getVideoTracks()[0]
        if (track) {
          peerManagerRef.current?.addTrackToAllPeers(track, media.getState().screenStream!)
        }
      }

      send({
        VoiceStateUpdate: {
          self_mute: voiceStore.selfMute,
          self_deaf: voiceStore.selfDeaf,
          video: voiceStore.video,
          streaming: enabled,
        },
      })
    } catch {
      // User cancelled the screen share picker
    }
  }, [getMedia, voiceStore, send])

  // ── Incoming signaling from relay ─────────────

  /** Handle voice-related relay events. Called by ExomApp's event dispatcher. */
  const handleVoiceEvent = useCallback((event: RelayEvent) => {
    const pm = peerManagerRef.current
    if (!pm) return

    switch (event.type) {
      case 'VoiceUserJoined': {
        const { user_id } = event as RelayEvent & { user_id: string }
        // When someone joins, we (as existing member) send them an offer
        if (user_id !== user?.user_id) {
          pm.createOffer(user_id)
        }
        break
      }
      case 'VoiceUserLeft': {
        const { user_id } = event as RelayEvent & { user_id: string }
        pm.removePeer(user_id)
        break
      }
      case 'VoiceOffer': {
        const { from_user_id, sdp } = event as RelayEvent & { from_user_id: string; sdp: string }
        pm.handleOffer(from_user_id, sdp)
        break
      }
      case 'VoiceAnswer': {
        const { from_user_id, sdp } = event as RelayEvent & { from_user_id: string; sdp: string }
        pm.handleAnswer(from_user_id, sdp)
        break
      }
      case 'VoiceIceCandidate': {
        const { from_user_id, candidate } = event as RelayEvent & { from_user_id: string; candidate: string }
        pm.handleIceCandidate(from_user_id, candidate)
        break
      }
    }
  }, [user?.user_id])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      peerManagerRef.current?.dispose()
      mediaRef.current?.dispose()
      for (const [userId] of audioElementsRef.current) {
        stopRemoteAudio(userId)
      }
    }
  }, [stopRemoteAudio])

  return {
    /** Whether we're connected to a voice channel. */
    isConnected,
    /** Current local media state (audio/video/screen). */
    localMedia,
    /** Connected peers with their state and streams. */
    peers,
    /** Connected channel info from the store. */
    connectedChannel: voiceStore.connectedChannel,

    // Actions
    join,
    leave,
    toggleMute,
    toggleDeaf,
    toggleVideo,
    toggleScreenShare,

    // Signaling handler (called by ExomApp)
    handleVoiceEvent,
  }
}
