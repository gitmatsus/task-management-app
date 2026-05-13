# タスク管理アプリ 引継ぎドキュメント

## プロジェクト概要

- **ファイル**: `C:\AI\タスク管理アプリ\TList.html`（単一ファイル、約2360行）
- **リポジトリ**: `https://github.com/gitmatsus/task-management-app.git`（ブランチ: `master`）
- **構成**: バニラJS + localStorage、外部依存なし、単一HTMLファイルで完結
- **Web起動**: `npx serve` 等でローカルサーブ、またはファイルを直接ブラウザで開く
- **Electron起動**: `cd electron && npm install && npm start`
- **Tauri起動**: `cd tauri && npm install && npm run dev`（Rust要インストール）
- **Photino起動**: `cd photino && dotnet run`（.NET 9.0 SDK要）

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
  folderPath?: string,  // フォルダパス or URL
  linkType?: string,    // 'folder' | 'url'
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

### 日付ヘルパー（ローカル時刻ベース）
```javascript
const ymd = (d) => `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;
const today    = () => ymd(new Date());
const tomorrow = () => ymd(new Date(Date.now() + 86400000));
```
※ 以前は `toISOString().slice(0,10)` でUTC基準だったため、JST深夜0時〜9時で期限切れ判定がずれるバグがあった。修正済み。

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
| フォルダ/URL登録 | タスクにフォルダパスまたはURLを紐付け、ボタンで開く |
| 詳細・メモ | タスクに詳細テキスト追加、メモボタンでポップアップ表示 |

---

## メモポップアップの配置ロジック

`htmlMemoPopup()` 関数で位置計算：
- **下側優先表示**: ボタン下に120px以上のスペースがあれば下、なければ上
- **動的max-height**: 選択方向の利用可能スペースに応じて設定（最低100px保証）
- **ボタン中央寄せ**: ボタン位置基準で水平中央に配置、画面端でクランプ
- **幅**: `min(500px, 100vw - 32px)` でコンテナ幅に合わせる

---

## ネイティブ環境のフォルダ/URL開く処理

`folder-open` ハンドラ内で環境を自動判定し分岐：

```
URL → Tauri → Photino → window.open()
フォルダ → Tauri → Photino → Electron → クリップボードコピー（Web）
```

### 判定方法
```javascript
// Tauri
if (window.__TAURI__?.core?.invoke) { ... }

// Photino
if (window.external?.sendMessage) { ... }

// Electron
if (window.electronAPI?.openFolder) { ... }
```

### Photinoメッセージ受信
Init セクション前に `window.external.receiveMessage()` でC#側からの応答を受信し、トースト表示。

---

## Electron対応

### ファイル構成
```
electron\
├── main.js       ← Electronメインプロセス（shell.openPath）
├── preload.js    ← contextBridge経由でAPIを安全に公開
└── package.json  ← electron v35, electron-builder v25
```

### 起動・ビルド
```bash
cd electron
npm install   # 初回のみ
npm start     # 開発起動
npm run build # dist/ にインストーラ生成（150MB+）
```

### パス解決（開発 vs パッケージ）
```javascript
const htmlPath = app.isPackaged
  ? path.join(process.resourcesPath, 'TList.html')  // インストール後
  : path.join(__dirname, '..', 'TList.html');         // 開発時
```

---

## Tauri対応（v2）

### ファイル構成
```
tauri\
├── package.json              ← @tauri-apps/cli v2
├── dist\
│   └── index.html            ← TList.html のコピー（predev/prebuild で自動同期）
├── app-icon.png              ← アイコン元画像（暫定「T」ロゴ）
└── src-tauri\
    ├── Cargo.toml            ← tauri v2, serde
    ├── build.rs
    ├── tauri.conf.json       ← withGlobalTauri:true, 1000x860, NSIS
    ├── capabilities\
    │   └── default.json      ← core:default
    ├── icons\                ← tauri icon で自動生成済み
    └── src\
        └── main.rs           ← open_folder コマンド（CREATE_NO_WINDOW付き）
```

### 起動・ビルド
```bash
cd tauri
npm install     # 初回のみ
npm run dev     # 開発起動（初回5〜10分、2回目以降数秒）
npm run build   # 配布用ビルド → src-tauri/target/release/bundle/nsis/
```

### 前提条件
- **Rust** (`rustup-init.exe` でインストール) ← インストール済み
- **VS C++ Build Tools** (Visual Studio Installerで「C++によるデスクトップ開発」) ← VS 2022あり
- **Node.js** v24 ← インストール済み
- **WebView2 Runtime** ← Windows 11標準搭載

### Rust側コマンド
```rust
#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    Command::new("cmd")
        .args(["/C", "start", "", &path])
        .creation_flags(CREATE_NO_WINDOW)  // コンソール非表示
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

### 重要設定
- `tauri.conf.json` の `app.withGlobalTauri: true` で `window.__TAURI__` をグローバル注入
- HTML同期: `package.json` の `predev`/`prebuild` で `TList.html` → `dist/index.html` にコピー

---

## Photino対応（.NET）

### ファイル構成
```
photino\
├── TList.csproj    ← Photino.NET 4.x, net9.0, PublishSingleFile, SelfContained
├── Program.cs      ← PhotinoWindow + メッセージハンドラ（open-folder, open-url）
└── icon.ico        ← Tauri版と共通アイコン
```

### 起動・ビルド
```bash
cd photino
dotnet run         # 開発起動
dotnet publish     # 配布用ビルド（未実施）
```

### 前提条件
- **.NET 9.0 SDK** ← インストール済み（.NET 10も入っている）

### C#側メッセージハンドラ
```csharp
// JS → C#: window.external.sendMessage('open-folder:C:\\path')
// C# → JS: window.SendWebMessage('folder-result:success')
window.RegisterWebMessageReceivedHandler((sender, message) => {
    if (message.StartsWith("open-folder:")) {
        Process.Start(new ProcessStartInfo { FileName = path, UseShellExecute = true });
    }
});
```

### HTMLコピー
- csproj の `CopyHtml` ターゲットでビルド後に `TList.html` を出力ディレクトリへコピー
- 日本語パスの問題で `Content Include` ではなく `Exec Command="copy"` を使用

### 名前空間の注意
- ✅ `using Photino.NET;`（正しい）
- ❌ `using PhotinoNET;`（古い情報で間違い）

---

## 3方式のサイズ比較（見込み）

| 方式 | 配布サイズ | 状態 |
|------|-----------|------|
| Electron | 150〜200MB | 既存・動作確認済み |
| Tauri | 3〜10MB | dev起動確認済み、配布ビルド未実施 |
| Photino | 15〜30MB | ビルド成功、起動確認中 |

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

### localStorage の保存先
- **Electron**: Electron独自のパス
- **Tauri**: `%APPDATA%\com.tlist.app\EBWebView\` 配下（WebView2）
- **Photino**: WebView2のデフォルトパス
- 3方式間でデータは**共有されない**。移行はJSONエクスポート/インポートで対応。

---

## 未実装・検討事項

- **データ同期**（Firebase Firestore方式Bを検討したが未実装）
- **Tauri配布ビルド**: `npm run build` で NSIS インストーラ生成（未実施）
- **Photino配布ビルド**: `dotnet publish` で単一exe生成（未実施）
- **3方式の最終選定**: サイズ・動作を比較して採用方式を決定
- **アイコン差し替え**: 暫定「T」ロゴを正式アイコンに置き換え
- **Outlook連携**: `outlook:` プロトコルがWindows未登録のため断念

---

## 直近コミット履歴

```
9ab45b4 深夜0時跨ぎで期限切れ判定されない不具合を修正
3e6cc03 メモポップアップの画面外はみ出し修正・表示位置改善
bbdc065 メモリストへの移動時チェック解除・インポート時のリストtype保持
dd9e80a コードブラッシュアップ：重複コード解消・デッドコード削除・バグ修正
11f8468 メモリスト・URL切替・自動整列・先頭追加・ドラッグ改善など複数機能追加
27aeb6c タスク整列・日付ソート機能追加、Electron改善、ドラッグオーバーレイ修正
1dbddfa Electronメニューバーを非表示にする
ec731e0 Add app.html redirect for backward compatibility
d8475ed Add index.html redirect to TList.html
6cac7e0 electron-builder によるインストーラービルド対応
```
