/// WebRTC type definitions.
///
/// These types define the contracts between the WebRTC layer,
/// the signaling layer (WebSocket), and the UI layer (React).

/** ICE server configuration for STUN/TURN. */
export interface IceServer {
  urls: string | string[]
  username?: string
  credential?: string
}

/** Default ICE configuration — public STUN servers. */
export const DEFAULT_ICE_SERVERS: IceServer[] = [
  { urls: 'stun:stun.l.google.com:19302' },
  { urls: 'stun:stun1.l.google.com:19302' },
]

/** State of a single peer connection. */
export type PeerState = 'new' | 'connecting' | 'connected' | 'disconnected' | 'failed' | 'closed'

/** A connected peer with their media streams. */
export interface Peer {
  userId: string
  state: PeerState
  connection: RTCPeerConnection
  /** Remote audio/video streams from this peer. */
  remoteStreams: MediaStream[]
}

/** Audio device info for device selection UI. */
export interface AudioDevice {
  deviceId: string
  label: string
  kind: 'audioinput' | 'audiooutput'
}

/** Video device info for device selection UI. */
export interface VideoDevice {
  deviceId: string
  label: string
}

/** Local media state — what we're capturing and sending. */
export interface LocalMediaState {
  audioEnabled: boolean
  videoEnabled: boolean
  screenShareEnabled: boolean
  audioStream: MediaStream | null
  videoStream: MediaStream | null
  screenStream: MediaStream | null
}

/** Events emitted by the PeerManager to the UI layer. */
export type PeerManagerEvent =
  | { type: 'peer-added'; userId: string }
  | { type: 'peer-removed'; userId: string }
  | { type: 'peer-state-changed'; userId: string; state: PeerState }
  | { type: 'remote-stream-added'; userId: string; stream: MediaStream }
  | { type: 'remote-stream-removed'; userId: string; stream: MediaStream }
  | { type: 'ice-candidate'; userId: string; candidate: string }
  | { type: 'offer-created'; userId: string; sdp: string }
  | { type: 'answer-created'; userId: string; sdp: string }
  | { type: 'error'; userId: string; error: string }
