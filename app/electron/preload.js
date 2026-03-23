const { contextBridge } = require('electron')

// Expose a minimal API to the renderer.
// The frontend communicates with the sidecar via HTTP and the relay via WebSocket,
// so we only need to expose platform info here.
contextBridge.exposeInMainWorld('exom', {
  platform: process.platform,
  version: require('../package.json').version,
})
