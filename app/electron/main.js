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

function startSidecar() {
  if (sidecarProcess) return

  const sidecarPath = isDev
    ? path.join(__dirname, '..', '..', 'target', 'debug', 'exom-sidecar')
    : path.join(process.resourcesPath, 'exom-sidecar')

  if (!fs.existsSync(sidecarPath)) {
    console.warn('Sidecar binary not found at', sidecarPath)
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

function createWindow() {
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
      preload: path.join(__dirname, 'preload.js'),
    },
  })

  if (state.maximized) {
    mainWindow.maximize()
  }

  // Show when ready to prevent flash
  mainWindow.once('ready-to-show', () => {
    mainWindow.show()
  })

  if (isDev) {
    mainWindow.loadURL('http://localhost:5173')
  } else {
    mainWindow.loadFile(path.join(__dirname, '..', 'dist', 'index.html'))
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

app.whenReady().then(() => {
  startSidecar()
  createTray()
  createWindow()

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
