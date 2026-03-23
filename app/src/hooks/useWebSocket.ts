/// WebSocket hook for the Exom relay.
///
/// Connects to the relay, handles MessagePack encode/decode,
/// manages reconnection with exponential backoff, and dispatches
/// incoming ServerMessage events to subscribers.

import { useEffect, useRef, useCallback, useState } from 'react'
import { encode, decode } from '@msgpack/msgpack'
import { relay as relayApi } from '@/lib/api'

export type ConnectionState = 'disconnected' | 'connecting' | 'authenticating' | 'connected' | 'reconnecting'

/** Subset of ServerMessage variants the UI cares about. */
export interface RelayEvent {
  type: string
  [key: string]: unknown
}

interface UseWebSocketOptions {
  /** Relay WebSocket URL (default: ws://localhost:9400/ws) */
  url?: string
  /** Reconnect on disconnect (default: true) */
  autoReconnect?: boolean
  /** Handler for incoming relay events */
  onEvent?: (event: RelayEvent) => void
  /** Handler for connection state changes */
  onStateChange?: (state: ConnectionState) => void
}

const DEFAULT_URL = 'ws://localhost:9400/ws'
const HEARTBEAT_INTERVAL = 30_000
const INITIAL_RECONNECT_DELAY = 1_000
const MAX_RECONNECT_DELAY = 30_000

/**
 * Hook for managing the WebSocket connection to the Exom relay.
 *
 * Usage:
 * ```ts
 * const { send, state, connect, disconnect } = useWebSocket({
 *   onEvent: (event) => { ... },
 * })
 * ```
 */
export function useWebSocket(options: UseWebSocketOptions = {}) {
  const {
    url = DEFAULT_URL,
    autoReconnect = true,
    onEvent,
    onStateChange,
  } = options

  const [state, setState] = useState<ConnectionState>('disconnected')
  const wsRef = useRef<WebSocket | null>(null)
  const heartbeatRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const reconnectDelayRef = useRef(INITIAL_RECONNECT_DELAY)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const shouldReconnectRef = useRef(autoReconnect)

  const updateState = useCallback((newState: ConnectionState) => {
    setState(newState)
    onStateChange?.(newState)
  }, [onStateChange])

  /** Send a MessagePack-encoded client message to the relay. */
  const send = useCallback((message: Record<string, unknown>) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const encoded = encode(message)
      wsRef.current.send(encoded)
    }
  }, [])

  /** Start the heartbeat timer. */
  const startHeartbeat = useCallback(() => {
    stopHeartbeat()
    heartbeatRef.current = setInterval(() => {
      send({ Heartbeat: null })
    }, HEARTBEAT_INTERVAL)
  }, [send])

  /** Stop the heartbeat timer. */
  const stopHeartbeat = useCallback(() => {
    if (heartbeatRef.current) {
      clearInterval(heartbeatRef.current)
      heartbeatRef.current = null
    }
  }, [])

  /** Connect to the relay with authentication. */
  const connect = useCallback(async () => {
    if (wsRef.current) {
      wsRef.current.close()
    }

    updateState('connecting')
    shouldReconnectRef.current = autoReconnect

    try {
      // Get auth token from sidecar
      updateState('authenticating')
      const tokenData = await relayApi.getToken()

      const ws = new WebSocket(url)
      ws.binaryType = 'arraybuffer'
      wsRef.current = ws

      ws.onopen = () => {
        // Send authentication message
        const authMessage = {
          Authenticate: {
            user_id: tokenData.user_id,
            token: tokenData.token,
            timestamp: tokenData.timestamp,
            protocol_version: tokenData.protocol_version,
          },
        }
        const encoded = encode(authMessage)
        ws.send(encoded)
      }

      ws.onmessage = (event) => {
        try {
          const decoded = decode(new Uint8Array(event.data)) as Record<string, unknown>

          // Handle auth result
          if ('AuthResult' in decoded) {
            const result = decoded.AuthResult as { success: boolean; error?: string }
            if (result.success) {
              updateState('connected')
              reconnectDelayRef.current = INITIAL_RECONNECT_DELAY
              startHeartbeat()

              // Drain offline queue
              send({ QueueDrain: null })
            } else {
              console.error('Relay auth failed:', result.error)
              updateState('disconnected')
            }
            return
          }

          // Dispatch all other events
          const eventType = Object.keys(decoded)[0]
          if (eventType && onEvent) {
            onEvent({
              type: eventType,
              ...(decoded[eventType] as Record<string, unknown>),
            })
          }
        } catch (err) {
          console.error('Failed to decode relay message:', err)
        }
      }

      ws.onclose = () => {
        stopHeartbeat()
        if (shouldReconnectRef.current) {
          updateState('reconnecting')
          scheduleReconnect()
        } else {
          updateState('disconnected')
        }
      }

      ws.onerror = (err) => {
        console.error('WebSocket error:', err)
      }
    } catch (err) {
      console.error('Failed to connect to relay:', err)
      if (shouldReconnectRef.current) {
        updateState('reconnecting')
        scheduleReconnect()
      } else {
        updateState('disconnected')
      }
    }
  }, [url, autoReconnect, onEvent, updateState, send, startHeartbeat, stopHeartbeat])

  /** Schedule a reconnection with exponential backoff. */
  const scheduleReconnect = useCallback(() => {
    const delay = reconnectDelayRef.current
    reconnectDelayRef.current = Math.min(delay * 2, MAX_RECONNECT_DELAY)

    reconnectTimeoutRef.current = setTimeout(() => {
      if (shouldReconnectRef.current) {
        connect()
      }
    }, delay)
  }, [connect])

  /** Disconnect from the relay. */
  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false
    stopHeartbeat()

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }

    updateState('disconnected')
  }, [stopHeartbeat, updateState])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      shouldReconnectRef.current = false
      stopHeartbeat()
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [stopHeartbeat])

  return {
    /** Current connection state */
    state,
    /** Send a raw message to the relay */
    send,
    /** Connect to the relay (authenticates automatically) */
    connect,
    /** Disconnect from the relay */
    disconnect,
  }
}
