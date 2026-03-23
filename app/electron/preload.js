const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('exom', {
  platform: process.platform,
  version: require('../package.json').version,

  /** Show a desktop notification. Clicking it focuses the window. */
  notify: (title, body) => ipcRenderer.invoke('show-notification', { title, body }),

  /** Set the dock/taskbar unread badge count. */
  setBadge: (count) => ipcRenderer.invoke('set-badge', count),
})
