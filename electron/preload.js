const { contextBridge, ipcRenderer } = require('electron');

// TList.html から安全に呼び出せるAPIを公開
contextBridge.exposeInMainWorld('electronAPI', {
  openFolder: (folderPath) => ipcRenderer.invoke('open-folder', folderPath),
});
