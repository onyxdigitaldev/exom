const { app, BrowserWindow, shell, Tray, Menu, nativeImage, Notification, ipcMain } = require('electron')
const path = require('path')
const fs = require('fs')
const { spawn } = require('child_process')

let mainWindow = null
let tray = null
let sidecarProcess = null
let isQuitting = false

const isDev = !app.isPackaged
const STATE_FILE = path.join(app.getPath('userData'), 'window-state.json')

// ── Window state persistence ────────────────────

function loadWindowState() {
  try {
    if (fs.existsSync(STATE_FILE)) {
      return JSON.parse(fs.readFileSync(STATE_FILE, 'utf8'))
    }
  } catch {}
  return { width: 1280, height: 800, x: undefined, y: undefined, maximized: false }
}

function saveWindowState() {
  if (!mainWindow) return
  const bounds = mainWindow.getBounds()
  const state = {
    width: bounds.width,
    height: bounds.height,
    x: bounds.x,
    y: bounds.y,
    maximized: mainWindow.isMaximized(),
  }
  try {
    fs.writeFileSync(STATE_FILE, JSON.stringify(state))
  } catch {}
}

// ── Sidecar lifecycle ───────────────────────────

async function startSidecar() {
  if (sidecarProcess) return

  // Check if sidecar is already running (e.g. started manually)
  try {
    const http = require('http')
    await new Promise((resolve, reject) => {
      const req = http.get('http://127.0.0.1:9401/health', (res) => {
        if (res.statusCode === 200) {
          console.log('[sidecar] Already running on port 9401')
          resolve()
        } else {
          reject()
        }
      })
      req.on('error', reject)
      req.setTimeout(1000, reject)
    })
    return // Already running, don't start another
  } catch {
    // Not running, start it
  }

  // Look for the release binary first, then debug
  const releasePath = path.join(__dirname, '..', '..', 'target', 'release', 'exom-sidecar')
  const debugPath = path.join(__dirname, '..', '..', 'target', 'debug', 'exom-sidecar')
  const prodPath = path.join(process.resourcesPath || '', 'exom-sidecar')

  const sidecarPath = isDev
    ? (fs.existsSync(releasePath) ? releasePath : debugPath)
    : prodPath

  if (!fs.existsSync(sidecarPath)) {
    console.warn('Sidecar binary not found at', sidecarPath)
    console.warn('Start it manually: cargo run -p exom-sidecar --release')
    return
  }

  sidecarProcess = spawn(sidecarPath, ['9401'], {
    stdio: 'pipe',
    env: { ...process.env },
  })

  sidecarProcess.stdout?.on('data', (data) => {
    console.log('[sidecar]', data.toString().trim())
  })

  sidecarProcess.stderr?.on('data', (data) => {
    console.error('[sidecar]', data.toString().trim())
  })

  sidecarProcess.on('exit', (code) => {
    console.log(`[sidecar] exited with code ${code}`)
    sidecarProcess = null
  })
}

function stopSidecar() {
  if (sidecarProcess) {
    sidecarProcess.kill('SIGTERM')
    sidecarProcess = null
  }
}

// ── System tray ─────────────────────────────────

function createTray() {
  const iconPath = path.join(__dirname, '..', 'public', 'icon-dark-32x32.png')
  let icon

  if (fs.existsSync(iconPath)) {
    icon = nativeImage.createFromPath(iconPath)
  } else {
    // Fallback: create a simple 16x16 icon
    icon = nativeImage.createEmpty()
  }

  tray = new Tray(icon)
  tray.setToolTip('Exom')

  const contextMenu = Menu.buildFromTemplate([
    { label: 'Show Exom', click: () => mainWindow?.show() },
    { type: 'separator' },
    { label: 'Quit', click: () => { isQuitting = true; app.quit() } },
  ])

  tray.setContextMenu(contextMenu)

  tray.on('click', () => {
    if (mainWindow) {
      mainWindow.isVisible() ? mainWindow.hide() : mainWindow.show()
    }
  })
}

// ── Main window ─────────────────────────────────

async function createWindow() {
  const state = loadWindowState()

  mainWindow = new BrowserWindow({
    width: state.width,
    height: state.height,
    x: state.x,
    y: state.y,
    minWidth: 1024,
    minHeight: 768,
    title: 'Exom',
    backgroundColor: '#0f0f14',
    titleBarStyle: 'hiddenInset',
    show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      webSecurity: true,
      allowRunningInsecureContent: false,
      preload: path.join(__dirname, 'preload.js'),
    },
  })

  // Content Security Policy — restrict what the renderer can load
  mainWindow.webContents.session.webRequest.onHeadersReceived((details, callback) => {
    callback({
      responseHeaders: {
        ...details.responseHeaders,
        'Content-Security-Policy': [
          [
            "default-src 'self'",
            "script-src 'self' 'unsafe-inline'",   // Vite HMR needs inline in dev
            "style-src 'self' 'unsafe-inline'",     // Tailwind needs inline styles
            "img-src 'self' data: blob: http://localhost:*",
            "font-src 'self' data:",
            "connect-src 'self' http://localhost:* ws://localhost:*",
            "media-src 'self' blob:",
            "worker-src 'self' blob:",
          ].join('; '),
        ],
      },
    })
  })

  // Block navigation to external URLs (prevent phishing via redirect)
  mainWindow.webContents.on('will-navigate', (event, url) => {
    const parsed = new URL(url)
    if (parsed.origin !== 'http://localhost:5173' && !url.startsWith('file://')) {
      event.preventDefault()
    }
  })

  if (state.maximized) {
    mainWindow.maximize()
  }

  // Show when ready to prevent flash
  mainWindow.once('ready-to-show', () => {
    mainWindow.show()
  })

  // Load the frontend — try Vite dev server first, fall back to built files
  const distPath = path.join(__dirname, '..', 'dist', 'index.html')

  if (isDev) {
    // Check if Vite dev server is running
    const http = require('http')
    const viteRunning = await new Promise((resolve) => {
      const req = http.get('http://localhost:5173', () => resolve(true))
      req.on('error', () => resolve(false))
      req.setTimeout(1000, () => { req.destroy(); resolve(false) })
    })

    if (viteRunning) {
      mainWindow.loadURL('http://localhost:5173')
    } else if (fs.existsSync(distPath)) {
      console.log('[electron] Vite not running, loading built files from dist/')
      mainWindow.loadFile(distPath)
    } else {
      console.error('[electron] No frontend available. Run "npm run build" or "npm run dev" first.')
      mainWindow.loadURL('data:text/html,<body style="background:#0f0f14;color:#e4e4e7;font-family:system-ui;display:flex;align-items:center;justify-content:center;height:100vh;margin:0"><div style="text-align:center"><h1>Exom</h1><p>Frontend not built yet.</p><pre style="color:#a1a1aa">cd app %26%26 npm run build</pre></div></body>')
    }
  } else {
    mainWindow.loadFile(distPath)
  }

  // Open external links in system browser
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url)
    return { action: 'deny' }
  })

  // Save window state on move/resize
  mainWindow.on('resize', saveWindowState)
  mainWindow.on('move', saveWindowState)
  mainWindow.on('maximize', saveWindowState)
  mainWindow.on('unmaximize', saveWindowState)

  // Minimize to tray instead of closing (unless quitting)
  mainWindow.on('close', (e) => {
    if (!isQuitting) {
      e.preventDefault()
      mainWindow.hide()
    }
  })

  mainWindow.on('closed', () => {
    mainWindow = null
  })
}

// ── IPC handlers ────────────────────────────────

ipcMain.handle('show-notification', (_, { title, body }) => {
  if (Notification.isSupported()) {
    const notification = new Notification({ title, body })
    notification.on('click', () => {
      mainWindow?.show()
      mainWindow?.focus()
    })
    notification.show()
  }
})

ipcMain.handle('set-badge', (_, count) => {
  if (app.setBadgeCount) {
    app.setBadgeCount(count)
  }
  if (tray) {
    tray.setToolTip(count > 0 ? `Exom (${count} unread)` : 'Exom')
  }
})

// ── App lifecycle ───────────────────────────────

app.whenReady().then(async () => {
  await startSidecar()
  createTray()
  await createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    } else {
      mainWindow?.show()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    isQuitting = true
    app.quit()
  }
})

app.on('before-quit', () => {
  isQuitting = true
  saveWindowState()
  stopSidecar()
})
