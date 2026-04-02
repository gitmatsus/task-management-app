const { app, BrowserWindow, ipcMain, shell } = require('electron');
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

  win.loadFile('TList.html');
}

// フォルダを開くIPCハンドラ
ipcMain.handle('open-folder', async (_event, folderPath) => {
  const errMsg = await shell.openPath(folderPath);
  // openPath は成功時に '' を返す
  return { success: errMsg === '', error: errMsg };
});

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
