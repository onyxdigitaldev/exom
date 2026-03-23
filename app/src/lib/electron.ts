/// Electron bridge — typed access to the preload API.
///
/// The preload script exposes `window.exom` via contextBridge.
/// This module provides typed wrappers and graceful fallbacks
/// when running outside Electron (e.g. in a browser for dev).

interface ExomBridge {
  platform: string
  version: string
  notify: (title: string, body: string) => Promise<void>
  setBadge: (count: number) => Promise<void>
}

declare global {
  interface Window {
    exom?: ExomBridge
  }
}

/** Whether we're running inside Electron. */
export const isElectron = typeof window !== 'undefined' && !!window.exom

/** Show a desktop notification. Falls back to browser Notification API. */
export async function showNotification(title: string, body: string): Promise<void> {
  if (window.exom) {
    await window.exom.notify(title, body)
  } else if ('Notification' in window && Notification.permission === 'granted') {
    new Notification(title, { body })
  } else if ('Notification' in window && Notification.permission !== 'denied') {
    const permission = await Notification.requestPermission()
    if (permission === 'granted') {
      new Notification(title, { body })
    }
  }
}

/** Update the dock/taskbar badge count. */
export async function setBadgeCount(count: number): Promise<void> {
  if (window.exom) {
    await window.exom.setBadge(count)
  }
}

/** Get the platform string (linux, darwin, win32). */
export function getPlatform(): string {
  return window.exom?.platform ?? navigator.platform.toLowerCase()
}
