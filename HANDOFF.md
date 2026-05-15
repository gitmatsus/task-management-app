# タスク管理アプリ 引継ぎドキュメント

## プロジェクト概要

- **ファイル**: `C:\AI\タスク管理アプリ\TList.html`（単一ファイル、約3040行）
- **リポジトリ**: `https://github.com/gitmatsus/task-management-app.git`（ブランチ: `master`）
- **公開URL**: https://gitmatsus.github.io/task-management-app/ （GitHub Pages、master 自動デプロイ）
- **構成**: バニラJS + localStorage、外部依存なし、単一HTMLファイルで完結
- **Web起動**: `npx serve` 等でローカルサーブ、またはファイルを直接ブラウザで開く
- **Electron起動**: `cd electron && npm install && npm start`
- **Tauri起動**: `cd tauri && npm install && npm run dev`（Rust要インストール）
- **Photino起動**: `cd photino && dotnet run`（.NET 9.0 SDK要）

---

## アーキテクチャ

### 状態管理
```javascript
// データ本体
let todos = [], trash = [], lists = [], selectedListId = '';
let listTrash = [], showTrash = false, showListTrash = false;

// 編集状態
let editId = null, editText = '', editDue = '', editStartDate = '';
let editFolderPath = '', editLinkType = 'folder', editDetail = '', skipFocusRestore = false;
let editingListId = null, editingListName = '', editingListType = 'todo';
let addingList = false, newListName = '';

// ドラッグ
let dragTodoId = null, dragTrashId = null, dragListId = null, dragOverEl = null;
let dragSubZoneTargetId = null;  // ドラッグ中のサブ化候補ターゲット

// モーダル・ポップアップ
let modalMode = null, modalLists = [], modalAllChecked = false, importPayload = null;
let movePopupTodoId = null, movePopupOpenedAt = 0;
let memoPopupId = null, memoPopupPos = null;
let ctxMenuTodoId = null, ctxMenuPos = null;   // タスク右クリックメニュー
let ctxMenuListId = null;                       // リスト右クリックメニュー
let ctxSubmenuOpen = false;                     // 「サブに登録」サブメニュー（モバイル用）

// UI状態
let fileDragOver = false;
let sidebarOpen = false;
let showListStats = localStorage.getItem('todo-app-show-stats') !== 'false';
let listExpanded = false;
const LIST_COLLAPSE_LIMIT = 5;
let collapsedIds = new Set();        // 折りたたみ中の親ID（セッション単位、非永続）
let subInputParentId = null;         // サブタスク連続追加モードの親ID
let taskFilter = 'all';              // all / week / actionable / overdue / today
let showCompletedSection = false;    // 完了セクションの開閉
```

### タスクデータ構造
```javascript
{
  id: string,             // crypto.randomUUID()
  text: string,           // タスク本文（改行含む可）
  completed: boolean,
  createdAt: number,      // Date.now()
  listId: string,
  dueDate?: string,       // 'YYYY-MM-DD' 終了日
  startDate?: string,     // 'YYYY-MM-DD' 開始日（メイン・サブ両方）
  completedAt?: number,   // 完了時刻 Date.now()（チェック時に自動付与）
  folderPath?: string,    // フォルダパス or URL
  linkType?: string,      // 'folder' | 'url'
  detail?: string,        // 詳細・メモ（複数行可）
  parentId?: string,      // 値があればサブタスク。親はメイン（parentId 無し）のみ。
                          // 2階層固定（サブのサブは禁止）
  _trashedWithParent?: string,  // ゴミ箱内部用: 親と一緒にゴミ箱化された際のペアID
                                 // エクスポート時は除外
}
```

### リストデータ構造
```javascript
{
  id: string,
  name: string,
  createdAt: number,
  type?: 'todo' | 'memo',  // memo はチェック概念なし
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
const hasTouch = 'ontouchstart' in window;
```

### 日付ヘルパー（ローカル時刻ベース）
```javascript
const ymd = (d) => `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;
const today    = () => ymd(new Date());
const tomorrow = () => ymd(new Date(Date.now() + 86400000));
const endOfWeekYmd = () => ymd(new Date(Date.now() + 6 * 86400000));
```

### マイグレーション（load() 内）
旧データを読み込んだ時の自動修復:
1. 孤児サブの修復: 親が存在しない／親が別リストにある／親自身がサブの場合は `parentId` をクリア
2. `todos` の id 重複を除去（過去の復元バグで重複した場合に最初の1件のみ残す）
3. ゴミ箱に残った「親不在の paired サブ」を除去（過去の `_trashedWithParent` バグの残骸）

---

## 主要機能一覧

### 実装済み機能

| 機能 | 説明 |
|------|------|
| タスクCRUD | 追加・編集・削除・完了チェック |
| 2階層サブタスク | parentId による親子関係。メイン⇔サブ変換（ドラッグor右クリック）、メイン昇格、グループ単位の削除/復元 |
| リスト管理 | 複数リスト、名前変更、削除（ゴミ箱付き）、ドラッグ並び替え |
| タスクフィルター | 4種チップ（今週中/着手可能/超過/今日）+ すべて |
| 完了セクション | 完了タスクを下部の折りたたみセクション「▶ 完了 (N件)」に集約 |
| 期間管理 | 開始日（▶アイコン）と終了日（📅アイコン）、同日は統合バッジ「▶📅 今日」、超過は「超過 (M/D)」 |
| 完了日時 | チェックONで `completedAt` 自動付与、タスク下に「✓ MM/DD HH:mm 完了」表示 |
| ゴミ箱 | タスクゴミ箱（親子関係維持表示）・リストゴミ箱 |
| 右クリックメニュー | 編集/削除/サブに登録▶（候補メイン一覧）/メインに昇格 |
| リスト右クリック/長押し | 名前変更/リスト削除（誤操作防止） |
| ＋ボタン | 各メイン右側、押すと「`<親名>` のサブタスクを入力...」モードで連続追加 |
| ドラッグ＆ドロップ | タスク並び替え・リスト間移動・サブ化（行下60%）・メイン化（上40%）・ゴミ箱へ |
| タッチ並び替え | ドラッグハンドル長押しで並び替え（モバイル） |
| リスト間移動 | タスク長押しでポップアップ選択（モバイル）・サブの場合は「⬆️ メインに昇格」も |
| 全て折り畳み/展開 | リストヘッダーの▶/▽トグルボタン（hover ツールチップ） |
| 緊急度カラー | サイドバー件数バッジ: 超過あり=赤、今日あり=橙 |
| ダークモード | トグル、localStorage永続化 |
| インポート/エクスポート | JSON形式、リスト単位で選択可、サブの parentId をIDマップでリマップ |
| ファイルD&D | JSONファイルをドロップしてインポート |
| PWA対応 | Canvas生成アイコン、apple-touch-icon、manifest |
| セーフエリア | `env(safe-area-inset-*)` でiOS Dynamic Island対応 |
| 多行入力 | Shift+Enter で改行、Enter で追加/保存 |
| リストステータス | ドット表示（期限切れ/今日/未来/期限なし/完了）のトグル |
| リスト折りたたみ | 5件超は折りたたみ（選択中のリストは常に表示） |
| フォルダ/URL登録 | タスクにフォルダパスまたはURLを紐付け、ボタンで開く |
| URLボタン（モバイル） | linkType='url' なら touch 環境でも表示 |
| 詳細・メモ | タスクに詳細テキスト追加、メモボタンでポップアップ表示 |
| 右クリック標準メニュー抑制 | 入力欄を除き標準コンテキストメニューを抑制 |

### 撤廃された機能
- ❌ 3タブフィルター（すべて/未完了/完了）→ 完了セクション + フィルターチップに置き換え
- ❌ 自動整列（チェックON時に末尾移動）→ 完了セクションが分離されるため不要
- ❌ 先頭追加トグル → 常に先頭追加がデフォルト
- ❌ 未完了タスクのゴミ箱ボタン → 右クリックメニューに移行（誤操作防止）
- ❌ リストのゴミ箱ボタン → 右クリック/長押しメニューに移行
- ❌ ゴミ箱→フィルターボタンのドラッグ復元 → 復元ボタンのみ

---

## サブタスク仕様

### 2階層の制約
- メイン: `parentId` 無し
- サブ: `parentId` に親メインのIDを保持
- サブのサブは作れない（サブ化対象がサブ持ちメインの場合は拒否）

### 振り分けルール（splitGroupsByCompletion）
- メインの完了状態でグループ単位に active / completed に振り分け
- メインが未完了 → そのサブも全て active 側
- メインが完了 → そのサブも全て completed 側

### 配置順序
- 配列内: 親メインの直下に各サブ。新規サブは「親の最後のサブの直後」に挿入
- 描画順: `arrangeForRender()` が「メイン → そのサブ群」の順に整列、folding 反映

### ドラッグドロップでのサブ化判定
- ターゲット行の **下60%** にドロップ = サブ化
- 上40% = 並び替え or サブ→メイン化
- サブ持ちメインのサブ化は拒否

### リスト間移動時
- メイン移動: サブも一緒に追従、親子関係保持
- サブ単体移動: `parentId` をクリアしてメイン化（親は元リストに残るため）

### ゴミ箱内
- 親をゴミ箱へ送る: そのサブも `_trashedWithParent` マーカー付与してゴミ箱へ
- 親復元: `_trashedWithParent === parent.id` または `parentId === parent.id` の両方を関連サブとして復元
- 親完全削除: 同基準で関連サブも完全削除
- サブ単体復元: 元の親が todos に存在しなければメイン化して復元
- 表示はメインの下にサブをインデントしてグルーピング

---

## タスクフィルターチップ

入力欄の下に5つのチップ:

| キー | ラベル | 条件 |
|---|---|---|
| `all` | すべて | デフォルト |
| `week` | 📅 今週中 | `dueDate <= today+6日`（超過も含む） |
| `actionable` | ▶️ 着手可能 | `startDate <= today` または `startDate` なし |
| `overdue` | 🔥 超過 | `dueDate < today` |
| `today` | ⚡ 今日 | `dueDate === today` |

完了済みタスクはどのフィルターにも該当しない（`matchFilter()` で除外）。
フィルター中は完了セクションも非表示。
フィルター中の表示結果からも完了済みは除外。
グループ単位でフィルター: メインまたはサブのいずれかが条件一致ならグループ全体を残す。

---

## バッジ表示の詳細

### 日付バッジ
- **終了バッジ** (`dueBadge`): 📅 今日／明日／M/D。超過時「超過 (M/D)」赤色
- **開始バッジ** (`startBadge`): ▶ 今日／明日／M/D。今日=黄、未来=青。完了済みは非表示
- **同日統合バッジ** (`sameDayBadge`): 開始日 === 終了日の場合「▶📅 今日」のように両アイコン併記
- 期間表示クラス: `start-fu`（未来=青）、`start-td`（今日=黄）、`due-ov`（超過=赤）、`due-td`（今日=橙）、`due-fu`（未来=緑）

### サイドバー件数バッジ（list-count-pill）
通常リスト: **未完了件数**（s.active）を表示
メモリスト: 全件（s.total）を表示
緊急度カラー:
- `urgent-ov`（超過あり）: 赤系。最優先
- `urgent-td`（今日が期限あり）: 橙系
- それ以外: 既定色

### 完了タスクの完了日時
```
✓ MM/DD HH:mm 完了
```
タスクテキストの直下に `completed-time` クラスで小さく表示。

---

## 右クリックメニュー / 長押しメニュー

### タスク右クリック（デスクトップ）/ コンテキスト
| 項目 | 動作 |
|---|---|
| ✏️ 編集 | 編集モード開始 |
| ✏️ サブに登録 ▶ | hover/タップで親候補メニューを展開、選択でサブ化 |
| ⬆️ メインに昇格 | （サブの場合のみ表示）`parentId` をクリア |
| 🗑️ 削除 | ゴミ箱へ移動 |

候補親 = 同リスト内のメイン（自身・現在の親・完了済みを除外）。
自身がサブを持つ場合は「サブに登録」項目は非表示（2階層制限）。
モバイル: タップでサブメニュー展開（`ctxSubmenuOpen` フラグ）。

### リスト右クリック / モバイル長押し（名前部分）
| 項目 | 動作 |
|---|---|
| ✏️ 名前を変更 | rename モード |
| 🗑️ リストを削除 | ゴミ箱へ移動。最後の1リストは非表示 |

モバイルではリストの **ドラッグハンドル長押し** = 並び替え、**名前/カウント長押し** = メニュー表示と分岐。

### 右クリック標準メニュー抑制
入力欄（`textarea, input`）以外でブラウザ標準コンテキストメニューを抑制。

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
  ? path.join(process.resourcesPath, 'TList.html')
  : path.join(__dirname, '..', 'TList.html');
```

---

## Tauri対応（v2）

### ファイル構成
```
tauri\
├── package.json              ← @tauri-apps/cli v2
├── dist\
│   └── index.html            ← TList.html のコピー（predev/prebuild で自動同期）
├── app-icon.png              ← アイコン元画像
└── src-tauri\
    ├── Cargo.toml            ← tauri v2, serde
    ├── build.rs
    ├── tauri.conf.json       ← withGlobalTauri:true, dragDropEnabled:false, NSIS
    ├── capabilities\
    │   └── default.json      ← core:default
    ├── icons\
    └── src\
        └── main.rs           ← open_folder コマンド + WebView2追加引数設定
```

### 起動・ビルド
```bash
cd tauri
npm install     # 初回のみ
npm run dev     # 開発起動
npm run build   # 配布用ビルド → src-tauri/target/release/bundle/nsis/
```

### 前提条件
- **Rust** (`rustup-init.exe` でインストール) ← インストール済み
- **VS C++ Build Tools** ← VS 2022あり
- **Node.js** v24 ← インストール済み
- **WebView2 Runtime** ← Windows 11標準搭載

### Rust側（main.rs）の主要部分
```rust
// Edgeミニメニュー（翻訳ボタン等）の抑制
#[cfg(target_os = "windows")]
std::env::set_var(
    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    "--disable-features=msEdgeMiniMenu,msEdgeAskMeAnything,TextSuggestionsForMiniMenu",
);

// open_folder コマンド: cmd /C start でURL/フォルダ/ファイルを既定アプリで開く
// CREATE_NO_WINDOWフラグでコンソール非表示
```

### 重要設定
- `tauri.conf.json` の `app.withGlobalTauri: true` で `window.__TAURI__` をグローバル注入
- `app.windows[0].dragDropEnabled: false` で HTML5 のドラッグドロップを有効化（OS既定のファイルドロップを無効化）
- HTML同期: `package.json` の `predev`/`prebuild` で `TList.html` → `dist/index.html` にコピー

---

## Photino対応（.NET）

### ファイル構成
```
photino\
├── TList.csproj    ← Photino.NET 4.x, net9.0, PublishSingleFile, SelfContained
├── Program.cs      ← PhotinoWindow + メッセージハンドラ + WebView2引数設定
└── icon.ico        ← アプリアイコン
```

### 起動・ビルド
```bash
cd photino
dotnet run         # 開発起動
dotnet publish     # 配布用ビルド
```

### 前提条件
- **.NET 9.0 SDK** ← インストール済み

### Program.cs の主要部分
```csharp
// WebView2のEdgeミニメニューを抑制（PhotinoWindow 構築前）
Environment.SetEnvironmentVariable(
    "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
    "--disable-features=msEdgeMiniMenu,msEdgeAskMeAnything,TextSuggestionsForMiniMenu");

// アイコンは2箇所で設定:
// 1. csproj の <ApplicationIcon>icon.ico</ApplicationIcon> → exeリソース埋め込み
// 2. .SetIconFile(iconPath) → 実行時のタイトルバー・タスクバーアイコン
var window = new PhotinoWindow()
    .SetTitle("TList")
    .SetIconFile(iconPath)
    .SetSize(new Size(1000, 860))
    ...
```

### csproj の重要設定
```xml
<ItemGroup>
  <!-- icoファイルを出力先にもコピー（SetIconFile用） -->
  <None Update="icon.ico">
    <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
  </None>
</ItemGroup>
```

### 名前空間の注意
- ✅ `using Photino.NET;`（正しい）
- ❌ `using PhotinoNET;`（古い情報で間違い）

---

## 3方式のサイズ比較（見込み）

| 方式 | 配布サイズ | 状態 |
|------|-----------|------|
| Electron | 150〜200MB | 既存・動作確認済み |
| Tauri | 3〜10MB | dev起動確認済み、配布ビルド未実施 |
| Photino | 15〜30MB | dev起動確認済み |

---

## 重要な実装パターン・注意事項

### SVGアイコン
`const I = { ... }` オブジェクトに全アイコンを定義。`I.folder`, `I.memo`, `I.play`, `I.cal` など。
撤廃: `I.arrowUp`, `I.sort`

### chevronRight アイコン
`htmlMain()` 内で定義し、以下で使い回し:
- `.sub-toggle`（各メインの折り畳みトグル）
- `.completed-section-caret`（完了セクションヘッダー）
- `.collapse-all-btn`（全て折り畳み/展開ボタン）

CSSの `.expanded` クラスで90°回転して ▽ 表示。

### ツールチップ（.tip）
- `.tip-wrap` で囲み、内部に `.tip` 要素
- `.tip-wrap:hover .tip { opacity:1 }` で表示
- `.sort-btns .tip` だけは `top:100%` で下方向（カードの `overflow:hidden` 対策）

### ポップアップ内のSVGサイズ
ポップアップのラベル内にSVGを入れると大きく表示されることがある。CSSで必ずサイズを制限：
```css
.memo-popup-label svg { width:11px; height:11px; flex-shrink:0; }
```

### デスクトップ専用UIの出し分け
```javascript
const hasTouch = 'ontouchstart' in window;
draggable="${!editing && !hasTouch}"
// URLボタンは linkType='url' なら touch でも表示
${t.folderPath && !editing && (t.linkType === 'url' || !hasTouch) ? ... : ''}
```

### + ボタン（サブタスク追加）のタッチ表示
```css
.item-add-sub button { opacity:0; }
.todo-item:hover .item-add-sub button { opacity:.6; }
@media (hover: none) {
  .item-add-sub button { opacity:.7; }  /* タッチ環境では常時表示 */
}
```

### サブタスクのインデント
```css
.todo-item.is-sub { padding-left:32px; }
/* 親と最初のサブ、サブ間の境界線を消す */
.todo-item:has(+ .todo-item.is-sub) { border-bottom:none; }
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

### localStorage キー一覧
| キー | 用途 |
|---|---|
| `todo-app-todos` | タスク配列 |
| `todo-app-trash` | ゴミ箱内タスク |
| `todo-app-lists` | リスト配列 |
| `todo-app-list-trash` | リストゴミ箱 |
| `todo-app-selected-list` | 選択中リストID |
| `todo-app-dark` | ダークモードフラグ |
| `todo-app-show-stats` | リストステータス表示フラグ |

### 撤廃された localStorage キー
- `todo-app-auto-sort` — 自動整列廃止に伴い不要（残っても無視される）
- `todo-app-add-to-top` — 先頭追加トグル廃止に伴い不要

---

## 未実装・検討事項

- **データ同期**（Firebase Firestore方式Bを検討したが未実装）
- **Tauri配布ビルド**: `npm run build` で NSIS インストーラ生成（未実施）
- **Photino配布ビルド**: `dotnet publish` で単一exe生成（未実施）
- **3方式の最終選定**: サイズ・動作を比較して採用方式を決定
- **Outlook連携**: `outlook:` プロトコルがWindows未登録のため断念

---

## 開発者向け資料

`docs/` 配下に MDViewer 流の章立てで詳細資料を整備（README + 14章）:
- `01_概要/` (アプリ概要、全体アーキテクチャ)
- `02_機能ガイド/` (タスク/リスト管理、期限と表示制御、D&D、メモとリンク、インポートエクスポート)
- `03_開発とビルド/` (ローカル開発、ネイティブパッケージング)
- `04_実装詳細/` (フロントエンド構造、データモデル、ネイティブ環境連携)
- `05_リファレンス/` (localStorageキー、既知の制約とハマりどころ)

ただし最新の機能追加（サブタスク以降のフィルターチップ・右クリックメニュー等）はこの引継ぎ資料を一次情報源とし、`docs/` は段階的に追従させていく方針。

---

## 直近コミット履歴

```
e4fbcc2 ui: サイドバーの件数バッジを緊急度で色分け
42ec6f3 fix: フィルター適用時に完了済みタスクを除外
d1448e0 ui: サイドバーのリスト件数表示を「未完了件数」に変更
56e2382 ui: ソートボタンのツールチップを下方向に表示
3dff973 refactor: 全て折り畳みアイコンを ▶/▽ に統一 / 先頭追加トグル廃止
21ce223 feat: 全て折り畳む / 全て展開する トグルボタン
33e13a6 feat: 右クリックメニューに「サブに登録」サブメニュー追加
8ea0f65 fix: ゴミ箱で親復元時に先に削除済みのサブも一緒に復元
b9355f8 fix: ゴミ箱で親子関係を表示 / サブ単体復元時にメイン化
2346da4 fix: 親復元時にサブがゴミ箱に残り、復元で重複する不具合
292f859 fix: メモリストでもサブ階層と折りたたみを反映
bb6cc74 feat: リスト削除を右クリック/長押しメニューに変更
ea69a74 fix: メモタスクにゴミ箱ボタンを表示
831c546 fix: サブタスク単体を別リストに移動した時の孤児化を修正
9e54034 fix: メモリストでもサブタスクを完全サポート / ＋ボタンをタッチでも表示
2b506ff feat: サブタスク・フィルター・ネイティブ統合の大幅拡張 (PR #1 merge)
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
