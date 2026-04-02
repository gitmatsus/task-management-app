const { app, BrowserWindow, ipcMain, shell, Menu } = require('electron');
Menu.setApplicationMenu(null);
const path = require('path');

function createWindow() {
  const win = new BrowserWindow({
    width: 1280,
    height: 860,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  // パッケージ版: resources/TList.html
  // 開発版: ../TList.html（electron フォルダの一つ上）
  const htmlPath = app.isPackaged
    ? path.join(process.resourcesPath, 'TList.html')
    : path.join(__dirname, '..', 'TList.html');
  win.loadFile(htmlPath);

  // ファイルのドラッグ＆ドロップによるページ遷移を防止
  win.webContents.on('will-navigate', (e) => e.preventDefault());
}

// フォルダを開くIPCハンドラ
ipcMain.handle('open-folder', async (_event, folderPath) => {
  const errMsg = await shell.openPath(folderPath);
  // openPath は成功時に '' を返す
  return { success: errMsg === '', error: errMsg };
});

// 二重起動防止：既に起動中なら既存ウィンドウをフォーカスして終了
const gotLock = app.requestSingleInstanceLock();
if (!gotLock) {
  app.quit();
} else {
  app.on('second-instance', () => {
    const win = BrowserWindow.getAllWindows()[0];
    if (win) {
      if (win.isMinimized()) win.restore();
      win.focus();
    }
  });

  app.whenReady().then(() => {
    createWindow();
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
  });
}

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
