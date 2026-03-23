/// MediaManager — owns local audio, video, and screen share streams.
///
/// Single source of truth for what media we're capturing locally.
/// Handles device enumeration, stream acquisition, and cleanup.
/// Framework-agnostic — no React imports.

import type { AudioDevice, VideoDevice, LocalMediaState } from './types'

export class MediaManager {
  private audioStream: MediaStream | null = null
  private videoStream: MediaStream | null = null
  private screenStream: MediaStream | null = null
  private audioEnabled = false
  private videoEnabled = false
  private screenShareEnabled = false

  /** Preferred device IDs — set by the user in settings. */
  private preferredAudioInput: string | undefined
  private preferredAudioOutput: string | undefined
  private preferredVideoInput: string | undefined

  /** Callback for state changes — the hook subscribes to this. */
  private onStateChange: ((state: LocalMediaState) => void) | null = null

  setOnStateChange(cb: (state: LocalMediaState) => void): void {
    this.onStateChange = cb
  }

  /** Get current media state snapshot. */
  getState(): LocalMediaState {
    return {
      audioEnabled: this.audioEnabled,
      videoEnabled: this.videoEnabled,
      screenShareEnabled: this.screenShareEnabled,
      audioStream: this.audioStream,
      videoStream: this.videoStream,
      screenStream: this.screenStream,
    }
  }

  // ── Audio ─────────────────────────────────────

  /** Start capturing audio from the microphone. */
  async enableAudio(): Promise<MediaStream> {
    if (this.audioStream) return this.audioStream

    const constraints: MediaStreamConstraints = {
      audio: this.preferredAudioInput
        ? { deviceId: { exact: this.preferredAudioInput } }
        : { echoCancellation: true, noiseSuppression: true, autoGainControl: true },
    }

    this.audioStream = await navigator.mediaDevices.getUserMedia(constraints)
    this.audioEnabled = true
    this.emitState()
    return this.audioStream
  }

  /** Stop capturing audio. */
  disableAudio(): void {
    if (this.audioStream) {
      this.audioStream.getTracks().forEach((t) => t.stop())
      this.audioStream = null
    }
    this.audioEnabled = false
    this.emitState()
  }

  /** Mute audio without releasing the stream (keeps mic indicator active). */
  muteAudio(): void {
    this.audioStream?.getAudioTracks().forEach((t) => { t.enabled = false })
    this.audioEnabled = false
    this.emitState()
  }

  /** Unmute audio. */
  unmuteAudio(): void {
    this.audioStream?.getAudioTracks().forEach((t) => { t.enabled = true })
    this.audioEnabled = true
    this.emitState()
  }

  /** Toggle mute state. Returns new mute state. */
  toggleMute(): boolean {
    if (this.audioEnabled) {
      this.muteAudio()
      return true // muted
    } else {
      this.unmuteAudio()
      return false // unmuted
    }
  }

  // ── Video ─────────────────────────────────────

  /** Start capturing video from the camera. */
  async enableVideo(): Promise<MediaStream> {
    if (this.videoStream) return this.videoStream

    const constraints: MediaStreamConstraints = {
      video: this.preferredVideoInput
        ? { deviceId: { exact: this.preferredVideoInput }, width: 1280, height: 720 }
        : { width: 1280, height: 720, frameRate: 30 },
    }

    this.videoStream = await navigator.mediaDevices.getUserMedia(constraints)
    this.videoEnabled = true
    this.emitState()
    return this.videoStream
  }

  /** Stop capturing video. */
  disableVideo(): void {
    if (this.videoStream) {
      this.videoStream.getTracks().forEach((t) => t.stop())
      this.videoStream = null
    }
    this.videoEnabled = false
    this.emitState()
  }

  /** Toggle video. Returns true if video is now enabled. */
  async toggleVideo(): Promise<boolean> {
    if (this.videoEnabled) {
      this.disableVideo()
      return false
    } else {
      await this.enableVideo()
      return true
    }
  }

  // ── Screen Share ──────────────────────────────

  /** Start screen sharing. */
  async enableScreenShare(): Promise<MediaStream> {
    if (this.screenStream) return this.screenStream

    this.screenStream = await navigator.mediaDevices.getDisplayMedia({
      video: { frameRate: 30 },
      audio: true,
    })

    // Handle the user clicking "Stop sharing" in the browser/OS prompt
    this.screenStream.getVideoTracks()[0]?.addEventListener('ended', () => {
      this.disableScreenShare()
    })

    this.screenShareEnabled = true
    this.emitState()
    return this.screenStream
  }

  /** Stop screen sharing. */
  disableScreenShare(): void {
    if (this.screenStream) {
      this.screenStream.getTracks().forEach((t) => t.stop())
      this.screenStream = null
    }
    this.screenShareEnabled = false
    this.emitState()
  }

  /** Toggle screen share. Returns true if sharing is now enabled. */
  async toggleScreenShare(): Promise<boolean> {
    if (this.screenShareEnabled) {
      this.disableScreenShare()
      return false
    } else {
      await this.enableScreenShare()
      return true
    }
  }

  // ── Device Enumeration ────────────────────────

  /** List available audio input devices (microphones). */
  async listAudioInputs(): Promise<AudioDevice[]> {
    const devices = await navigator.mediaDevices.enumerateDevices()
    return devices
      .filter((d) => d.kind === 'audioinput')
      .map((d) => ({ deviceId: d.deviceId, label: d.label || `Microphone ${d.deviceId.slice(0, 8)}`, kind: 'audioinput' as const }))
  }

  /** List available audio output devices (speakers/headphones). */
  async listAudioOutputs(): Promise<AudioDevice[]> {
    const devices = await navigator.mediaDevices.enumerateDevices()
    return devices
      .filter((d) => d.kind === 'audiooutput')
      .map((d) => ({ deviceId: d.deviceId, label: d.label || `Speaker ${d.deviceId.slice(0, 8)}`, kind: 'audiooutput' as const }))
  }

  /** List available video input devices (cameras). */
  async listVideoInputs(): Promise<VideoDevice[]> {
    const devices = await navigator.mediaDevices.enumerateDevices()
    return devices
      .filter((d) => d.kind === 'videoinput')
      .map((d) => ({ deviceId: d.deviceId, label: d.label || `Camera ${d.deviceId.slice(0, 8)}` }))
  }

  /** Set preferred audio input device. Takes effect on next enableAudio(). */
  setAudioInput(deviceId: string): void {
    this.preferredAudioInput = deviceId
  }

  /** Set preferred audio output device. */
  setAudioOutput(deviceId: string): void {
    this.preferredAudioOutput = deviceId
  }

  /** Get preferred audio output device ID. */
  getAudioOutput(): string | undefined {
    return this.preferredAudioOutput
  }

  /** Set preferred video input device. Takes effect on next enableVideo(). */
  setVideoInput(deviceId: string): void {
    this.preferredVideoInput = deviceId
  }

  // ── Lifecycle ─────────────────────────────────

  /** Release all media resources. Call on disconnect. */
  dispose(): void {
    this.disableAudio()
    this.disableVideo()
    this.disableScreenShare()
    this.onStateChange = null
  }

  /** Get all active local streams for adding to peer connections. */
  getActiveStreams(): MediaStream[] {
    const streams: MediaStream[] = []
    if (this.audioStream) streams.push(this.audioStream)
    if (this.videoStream) streams.push(this.videoStream)
    if (this.screenStream) streams.push(this.screenStream)
    return streams
  }

  /** Get all active local tracks for adding to peer connections. */
  getActiveTracks(): MediaStreamTrack[] {
    return this.getActiveStreams().flatMap((s) => s.getTracks())
  }

  private emitState(): void {
    this.onStateChange?.(this.getState())
  }
}
