# タスク管理アプリ 引継ぎドキュメント

## プロジェクト概要

- **ファイル**: `C:\AI\タスク管理アプリ\TList.html`（単一ファイル、約2160行）
- **リポジトリ**: `https://github.com/gitmatsus/task-management-app.git`（ブランチ: `master`）
- **構成**: バニラJS + localStorage、外部依存なし、単一HTMLファイルで完結
- **Web起動**: `npx serve` 等でローカルサーブ、またはファイルを直接ブラウザで開く
- **Electron起動**: `npm install` → `npm start`

---

## アーキテクチャ

### 状態管理
```javascript
// タスク・リスト
let todos = [], trash = [], lists = [], selectedListId = '';
let filter = 'all', showTrash = false;
let listTrash = [], showListTrash = false;

// 編集状態
let editId = null, editText = '', editDue = '', editFolderPath = '', editDetail = '', skipFocusRestore = false;
let editingListId = null, editingListName = '';
let addingList = false, newListName = '';

// ドラッグ
let dragTodoId = null, dragTrashId = null, dragListId = null, dragOverEl = null;

// モーダル・ポップアップ
let modalMode = null, modalLists = [], modalAllChecked = false, importPayload = null;
let movePopupTodoId = null, movePopupOpenedAt = 0;
let memoPopupId = null, memoPopupPos = null;

// UI状態
let fileDragOver = false;
let sidebarOpen = false;
let showListStats = localStorage.getItem('todo-app-show-stats') !== 'false';
let listExpanded = false;
const LIST_COLLAPSE_LIMIT = 5;
```

### タスクデータ構造
```javascript
{
  id: string,           // crypto.randomUUID()
  text: string,         // タスク本文（改行含む可）
  completed: boolean,
  createdAt: number,    // Date.now()
  listId: string,
  dueDate?: string,     // 'YYYY-MM-DD'
  folderPath?: string,  // フォルダパス（デスクトップ用）
  detail?: string,      // 詳細・メモ（複数行可）
}
```

### レンダリング方式
- `render()` → `document.getElementById('app').innerHTML = html()` で全体再描画
- イベントは `data-a` 属性による委譲（`document.addEventListener('click', ...)` 等）
- `const el = e.target.closest('[data-a]'); const a = el?.dataset.a;` パターン

### モバイル判定
```javascript
function isMobile() {
  return window.innerWidth <= 640 || ('ontouchstart' in window && window.innerWidth <= 1024);
}
const hasTouch = 'ontouchstart' in window; // ドラッグ可否・デスクトップ専用UI判定に使用
```

---

## 主要機能一覧

### 実装済み機能

| 機能 | 説明 |
|------|------|
| タスクCRUD | 追加・編集・削除・完了チェック |
| リスト管理 | 複数リスト、名前変更、削除（ゴミ箱付き）、ドラッグ並び替え |
| フィルター | すべて／未完了／完了 |
| 期限日 | 設定・バッジ表示（期限切れ/今日/未来） |
| ゴミ箱 | タスクゴミ箱・リストゴミ箱（それぞれ復元・完全削除） |
| ドラッグ＆ドロップ | タスク並び替え・リスト間移動・ゴミ箱へ（デスクトップ） |
| タッチ並び替え | ドラッグハンドル長押しで並び替え（モバイル） |
| リスト間移動 | タスク長押しでポップアップ選択（モバイル） |
| ダークモード | トグル、localStorage永続化 |
| インポート/エクスポート | JSON形式、リスト単位で選択可 |
| ファイルD&D | JSONファイルをドロップしてインポート |
| PWA対応 | Canvas生成アイコン、apple-touch-icon、manifest |
| セーフエリア | `env(safe-area-inset-*)` でiOS Dynamic Island対応 |
| 多行入力 | Shift+Enter で改行、Enter で追加/保存 |
| リストステータス | ドット表示（期限切れ/今日/未来/期限なし/完了）のトグル |
| リスト折りたたみ | 5件超は折りたたみ（選択中のリストは常に表示） |
| フォルダ登録 | タスクにフォルダパス紐付け、ボタンでクリップボードコピー（デスクトップのみ） |
| 詳細・メモ | タスクに詳細テキスト追加、メモボタンでポップアップ表示 |

---

## 最近追加した機能（前セッション）

### フォルダ登録機能（デスクトップ専用）
- 編集フォームに「フォルダ」入力欄（`id="ei-folder"`）
- `folderPath` をタスクに保存
- タスク行右端に📁ボタン表示（`!hasTouch && t.folderPath` の場合のみ）
- クリック → `navigator.clipboard.writeText()` + execCommand フォールバック
- `window.open('file:///...')` は全ブラウザNGのため不採用

### 詳細・メモ機能
- 編集フォームに `<textarea id="ei-detail">` を追加（タスク本文の直下）
- `detail` をタスクに保存
- タスク行に📄ボタン表示（`t.detail` があれば常に表示）
- クリック → ボタン近傍にポップアップ表示（`getBoundingClientRect()` で位置計算）
- 再クリックまたは外クリックで閉じる
- `memoPopupId`, `memoPopupPos` で状態管理

**編集フォームの表示順（上→下）：**
1. タスク本文 textarea
2. 詳細・メモ textarea
3. フォルダパス（デスクトップのみ）
4. 日付設定
5. キャンセル / 保存

**タスク行の表示順（左→右）：**
ドラッグハンドル | チェックボックス | タスク本文 | 📄メモ | 📁フォルダ | 📅日付 | 🗑️削除

---

## 重要な実装パターン・注意事項

### SVGアイコン
`const I = { ... }` オブジェクトに全アイコンを定義。`I.folder`, `I.memo` など。

### ポップアップ内のSVGサイズ
ポップアップのラベル内にSVGを入れると大きく表示されることがある。CSSで必ずサイズを制限すること：
```css
.memo-popup-label svg { width:11px; height:11px; flex-shrink:0; }
```

### デスクトップ専用UIの出し分け
```javascript
const hasTouch = 'ontouchstart' in window;
// ドラッグ属性
draggable="${!editing && !hasTouch}"
// デスクトップ専用ボタン
${!hasTouch && t.folderPath ? `...` : ''}
```

### movePopupのタッチ誤作動防止
```javascript
let movePopupOpenedAt = 0;
function showMovePopup(todoId) { movePopupTodoId = todoId; movePopupOpenedAt = Date.now(); render(); }
function closeMovePopup() { if (Date.now() - movePopupOpenedAt < 400) return; ... }
```

### モバイル入力サイズ
モバイルでは `font-size:16px` 未満だとiOSがズームするため注意：
```css
@media (max-width:640px) { .edit-text { font-size:16px; } }
```

---

## Electron対応

### ファイル構成
```
C:\AI\タスク管理アプリ\
├── TList.html        ← メインアプリ（Web/Electron共用）
├── HANDOFF.md
└── electron\
    ├── main.js       ← Electronメインプロセス
    ├── preload.js    ← contextBridge経由でAPIを安全に公開
    └── package.json  ← Electron依存定義
```

### 起動方法
```bash
# electron フォルダに移動してから実行
cd electron
npm install   # 初回のみ（electron/node_modules/ が生成される）
npm start     # Electronアプリとして起動
```

### Electron/Web の自動判定
`folder-open` クリックハンドラで `window.electronAPI?.openFolder` の有無を判定：
```javascript
if (window.electronAPI?.openFolder) {
  // Electron：shell.openPath() でフォルダを直接開く
} else {
  // Web：クリップボードコピー
}
```

### preload.js の役割
`contextIsolation: true`（セキュア設定）のため、レンダラーから Node.js APIを直接呼べない。
`preload.js` が `contextBridge.exposeInMainWorld('electronAPI', {...})` で安全に橋渡しする。

### shell.openPath の戻り値
```javascript
// 成功時: '' (空文字)
// 失敗時: エラーメッセージ文字列
const errMsg = await shell.openPath(folderPath);
return { success: errMsg === '', error: errMsg };
```

---

## 未実装・検討事項

- **データ同期**（Firebase Firestore方式Bを検討したが未実装）
- **フォルダピッカーUI**（`webkitdirectory` や `showDirectoryPicker()` はフルパスを返さないため意味なし）
- **Electronビルド**（`electron-builder` 等でexe化は未設定）

---

## 直近コミット履歴

```
（最新）Electron対応：フォルダを直接開く機能（main.js / preload.js / package.json）
10602ad app.html を TList.html にリネーム、HANDOFF.md を追加
e18ed88 メモポップアップの表示改善
81182b0 タスクに詳細・メモ機能を追加
4474fb0 フォルダ機能の整理：コピーのみに戻し表示順を変更
1af6c11 タスクにフォルダ登録機能を追加（デスクトップ版限定）
965d36d Add multi-line task text support with Shift+Enter
d0cffcd Add sidebar list stats toggle and collapsible list view
33bc271 Add mobile task move between lists via long-press popup
```
