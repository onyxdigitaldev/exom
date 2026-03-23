/// PeerManager — orchestrates WebRTC peer connections in a mesh topology.
///
/// For each user in a voice channel, we maintain one RTCPeerConnection.
/// When a new user joins, existing peers create offers. The new user
/// responds with answers. ICE candidates are exchanged via the relay.
///
/// The manager is framework-agnostic. It communicates with the outside
/// world exclusively through the event callback.
///
/// Mesh topology supports up to 8 concurrent peers before bandwidth
/// becomes a concern. Beyond that, an SFU would be needed.

import type { Peer, PeerState, PeerManagerEvent, IceServer } from './types'
import { DEFAULT_ICE_SERVERS } from './types'
import { MediaManager } from './MediaManager'

const MAX_MESH_PEERS = 8

export class PeerManager {
  private peers: Map<string, Peer> = new Map()
  private media: MediaManager
  private iceServers: IceServer[]
  private onEvent: (event: PeerManagerEvent) => void
  private disposed = false

  constructor(
    media: MediaManager,
    onEvent: (event: PeerManagerEvent) => void,
    iceServers: IceServer[] = DEFAULT_ICE_SERVERS,
  ) {
    this.media = media
    this.onEvent = onEvent
    this.iceServers = iceServers
  }

  // ── Peer Lifecycle ────────────────────────────

  /**
   * Create a peer connection and send an SDP offer.
   * Called when we are already in the channel and a new user joins,
   * or when we join and need to connect to existing users.
   */
  async createOffer(userId: string): Promise<void> {
    if (this.disposed || this.peers.has(userId)) return
    if (this.peers.size >= MAX_MESH_PEERS) {
      this.emit({ type: 'error', userId, error: `Mesh limit reached (${MAX_MESH_PEERS} peers)` })
      return
    }

    const pc = this.createPeerConnection(userId)
    this.addLocalTracks(pc)

    const offer = await pc.createOffer()
    await pc.setLocalDescription(offer)

    this.emit({ type: 'offer-created', userId, sdp: offer.sdp! })
  }

  /**
   * Handle an incoming SDP offer from a remote peer.
   * Creates the peer connection, sets the remote description,
   * and generates an answer.
   */
  async handleOffer(userId: string, sdp: string): Promise<void> {
    if (this.disposed) return

    // If we already have a connection to this peer, close and recreate.
    // This handles the case where both sides try to connect simultaneously.
    if (this.peers.has(userId)) {
      this.removePeer(userId)
    }

    const pc = this.createPeerConnection(userId)
    this.addLocalTracks(pc)

    await pc.setRemoteDescription(new RTCSessionDescription({ type: 'offer', sdp }))

    const answer = await pc.createAnswer()
    await pc.setLocalDescription(answer)

    this.emit({ type: 'answer-created', userId, sdp: answer.sdp! })
  }

  /**
   * Handle an incoming SDP answer from a remote peer.
   * Completes the signaling handshake.
   */
  async handleAnswer(userId: string, sdp: string): Promise<void> {
    const peer = this.peers.get(userId)
    if (!peer || this.disposed) return

    // Only set remote description if we're in the right state
    if (peer.connection.signalingState === 'have-local-offer') {
      await peer.connection.setRemoteDescription(
        new RTCSessionDescription({ type: 'answer', sdp }),
      )
    }
  }

  /**
   * Handle an incoming ICE candidate from a remote peer.
   */
  async handleIceCandidate(userId: string, candidateJson: string): Promise<void> {
    const peer = this.peers.get(userId)
    if (!peer || this.disposed) return

    try {
      const candidate = JSON.parse(candidateJson)
      await peer.connection.addIceCandidate(new RTCIceCandidate(candidate))
    } catch (err) {
      // ICE candidate errors are non-fatal — some candidates are redundant
      console.warn(`ICE candidate error for ${userId}:`, err)
    }
  }

  /** Remove a peer connection (user left the channel). */
  removePeer(userId: string): void {
    const peer = this.peers.get(userId)
    if (!peer) return

    peer.connection.close()
    this.peers.delete(userId)
    this.emit({ type: 'peer-removed', userId })
  }

  /** Get all currently connected peer user IDs. */
  getConnectedPeerIds(): string[] {
    return Array.from(this.peers.entries())
      .filter(([_, p]) => p.state === 'connected')
      .map(([id]) => id)
  }

  /** Get the remote streams for a specific peer. */
  getRemoteStreams(userId: string): MediaStream[] {
    return this.peers.get(userId)?.remoteStreams ?? []
  }

  /** Get all remote audio streams (for audio output). */
  getAllRemoteAudioStreams(): { userId: string; stream: MediaStream }[] {
    const result: { userId: string; stream: MediaStream }[] = []
    for (const [userId, peer] of this.peers) {
      for (const stream of peer.remoteStreams) {
        if (stream.getAudioTracks().length > 0) {
          result.push({ userId, stream })
        }
      }
    }
    return result
  }

  /** Get peer count. */
  get peerCount(): number {
    return this.peers.size
  }

  // ── Track Management ──────────────────────────

  /**
   * Add or replace a local track on all peer connections.
   * Called when the user enables video, screen share, etc.
   */
  addTrackToAllPeers(track: MediaStreamTrack, stream: MediaStream): void {
    for (const [_, peer] of this.peers) {
      const sender = peer.connection.getSenders().find((s) => s.track?.kind === track.kind)
      if (sender) {
        sender.replaceTrack(track)
      } else {
        peer.connection.addTrack(track, stream)
      }
    }
  }

  /**
   * Remove a local track from all peer connections.
   * Called when the user disables video, screen share, etc.
   */
  removeTrackFromAllPeers(track: MediaStreamTrack): void {
    for (const [_, peer] of this.peers) {
      const sender = peer.connection.getSenders().find((s) => s.track === track)
      if (sender) {
        peer.connection.removeTrack(sender)
      }
    }
  }

  // ── Deafen ────────────────────────────────────

  /** Deafen: mute all incoming audio without closing connections. */
  setDeafened(deaf: boolean): void {
    for (const [_, peer] of this.peers) {
      for (const stream of peer.remoteStreams) {
        stream.getAudioTracks().forEach((t) => { t.enabled = !deaf })
      }
    }
  }

  // ── Cleanup ───────────────────────────────────

  /** Close all peer connections and release resources. */
  dispose(): void {
    this.disposed = true
    for (const [userId] of this.peers) {
      this.removePeer(userId)
    }
    this.peers.clear()
  }

  // ── Private ───────────────────────────────────

  /** Create and configure an RTCPeerConnection for a specific peer. */
  private createPeerConnection(userId: string): RTCPeerConnection {
    const config: RTCConfiguration = {
      iceServers: this.iceServers,
      iceCandidatePoolSize: 10,
    }

    const pc = new RTCPeerConnection(config)

    const peer: Peer = {
      userId,
      state: 'new',
      connection: pc,
      remoteStreams: [],
    }

    this.peers.set(userId, peer)
    this.emit({ type: 'peer-added', userId })

    // ── ICE candidate events ──

    pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.emit({
          type: 'ice-candidate',
          userId,
          candidate: JSON.stringify(event.candidate.toJSON()),
        })
      }
    }

    // ── Connection state tracking ──

    pc.onconnectionstatechange = () => {
      const state = this.mapConnectionState(pc.connectionState)
      peer.state = state
      this.emit({ type: 'peer-state-changed', userId, state })

      if (state === 'failed') {
        console.error(`Peer connection to ${userId} failed`)
        // Attempt to reconnect by creating a new offer
        this.removePeer(userId)
        setTimeout(() => {
          if (!this.disposed && !this.peers.has(userId)) {
            this.createOffer(userId)
          }
        }, 2000)
      }
    }

    // ── Remote stream handling ──

    pc.ontrack = (event) => {
      for (const stream of event.streams) {
        if (!peer.remoteStreams.some((s) => s.id === stream.id)) {
          peer.remoteStreams.push(stream)
          this.emit({ type: 'remote-stream-added', userId, stream })

          // Handle stream removal
          stream.onremovetrack = () => {
            if (stream.getTracks().length === 0) {
              peer.remoteStreams = peer.remoteStreams.filter((s) => s.id !== stream.id)
              this.emit({ type: 'remote-stream-removed', userId, stream })
            }
          }
        }
      }
    }

    return pc
  }

  /** Add all current local tracks to a peer connection. */
  private addLocalTracks(pc: RTCPeerConnection): void {
    for (const track of this.media.getActiveTracks()) {
      const streams = this.media.getActiveStreams().filter((s) =>
        s.getTracks().includes(track),
      )
      if (streams.length > 0) {
        pc.addTrack(track, streams[0])
      }
    }
  }

  /** Map browser connection state to our simplified state enum. */
  private mapConnectionState(state: RTCPeerConnectionState): PeerState {
    switch (state) {
      case 'new': return 'new'
      case 'connecting': return 'connecting'
      case 'connected': return 'connected'
      case 'disconnected': return 'disconnected'
      case 'failed': return 'failed'
      case 'closed': return 'closed'
      default: return 'new'
    }
  }

  private emit(event: PeerManagerEvent): void {
    if (!this.disposed) {
      this.onEvent(event)
    }
  }
}
