# タスク管理アプリ 引継ぎドキュメント

## プロジェクト概要

- **ファイル**: `C:\AI\タスク管理アプリ\TList.html`（単一ファイル、約7500行）
- **現行バージョン**: **v1.8.0**（`APP_VERSION` と `tauri/package.json` / `Cargo.toml` / `tauri.conf.json` の4箇所を揃えて更新する）
- **最終更新**: 2026-10-28（v1.8.0）
- **リポジトリ**: `https://github.com/gitmatsus/task-management-app.git`（ブランチ: `master`）
- **公開URL**: https://gitmatsus.github.io/task-management-app/ （GitHub Pages、master 自動デプロイ）
- **構成**: バニラJS + localStorage、外部依存なし、単一HTMLファイルで完結
- **Web起動**: `npx serve` 等でローカルサーブ、またはファイルを直接ブラウザで開く
- **Tauri起動**: `cd tauri && npm install && npm run dev`（Rust要インストール）

> 旧 Electron / Photino 対応は撤廃済み（Tauri 一本化）。コミット履歴に Electron 関連が残るのは過去の経緯。

---

## バージョン別変更履歴

各版の大項目。詳細は各機能の節と末尾の「直近コミット履歴」を参照。

| 版 | 日付 | 主な変更 |
|---|---|---|
| v1.8.0 | 2026-10-28 | **SE 番号（工数管理）**: 今日の予定の実績時間に SE 番号（案件）を紐づけ（右クリックのドロップダウンから選択／新規登録。番号と案件名を記憶、同じタスクは前回の番号を自動設定）<br>**SE 集計**: 月単位で 行=SE 番号 × 列=日付 の実績時間表、表のコピー（Excel 貼り付け）と CSV 保存、SE 番号の管理（案件名変更・追加・削除）<br>設定に「SE番号（工数管理）」のオン/オフ |
| v1.7.0 | 2026-10-26〜10-27 | **タスク検索**（🔍 / Ctrl+K。全リストの本文・メモから候補表示、ダブルクリックでそのタスクへ移動）<br>**今日の予定「明日へ移動」**（右クリックで翌日の同じ枠へ移し、今日の予定から外す）<br>右クリックメニューが画面外にはみ出す問題を修正 |
| v1.6.0 | 2026-10-15〜10-21 | **時刻通知（Tauri 版のみ）**: 予定の時刻にポップアップ＋ウインドウ前面化・タスクバー点滅（最小化中も Rust 側で見張る）。何分前の指定（予定ごと）、再通知（5分後・取り消し可）、通知一覧。時刻は時・分プルダウンで選択<br>**実績時間**: 目安とは別に記録し、行・見出し・フッター・カレンダーに表示<br>**予定の内容編集**（タスクに紐づく予定は元タスク名を編集）<br>右クリックメニューの幅を調整 |
| v1.5.0 | 2026-09-28〜10-02 | **今日の予定（時間割）**: 午前/午後、目安時間、メモ、持ち越し（選択式・済だが継続中も対象）、カレンダーでの履歴、右クリック（⋯）メニューに操作を集約、四角チェック<br>**設定画面**: 今日の予定・Git 同期の表示/非表示、通知（バッジ）設定をヘッダーから移動<br>**iPhone 改善**: エクスポートを共有メニューから「ファイルに保存」、長押し「タスクの操作」メニュー、今日のToDo 切替バー、スマホ幅の表示崩れ修正<br>通常リストの右クリックから「今日の予定に追加」 |
| v1.4.1 | 2026-09-09 | インポートに「置き換え（全消去して取り込む）」を追加 |
| v1.4.0 | 2026-09-08 | **見出し（大項目）**: メインタスクをチェック不可の分類ラベルに。Pull の確認ダイアログと Pull 直前の自動バックアップ |
| v1.3.1 | 2026-08-26 | 今日のToDo に絞り込み（すべて／超過＋今日／進行中） |
| v1.3.0 | 2026-08-18 | 今日のToDo に「進行中」を追加 |
| （v1.2.x） | 2026-06〜07 | 「今日のToDo」集約ビュー、サイドバー追従、件数バッジの超過/今日同時表示、日付ソート改良 |
| v1.2.0 | 2026-05-21〜28 | **カレンダー（月ビュー）**、繰り返しタスクのリセットをイベント駆動に、iPhone PWA のバッジ・表示改善、UI 状態の永続化 |
| v1.1.0 | 2026-05-19 | **Git/GitHub 同期**（Pull/Push、3-way マージ、競合解決 UI） |
| v1.0.0 | 〜2026-05-18 | サブタスク・フィルター・繰り返し・ゴミ箱などの基本機能、Tauri 一本化、タスクバーバッジ |

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
let editRecurrence = null;                     // 編集中の繰り返し設定 or null
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
let collapsedIds = new Set();        // 折りたたみ中の親ID（todo-app-ui-state に永続化）
let expandedDoneSubs = new Set();    // 「▷ 完了 (N件)」トグル展開中の親ID（永続化）
let subInputParentId = null;         // サブタスク連続追加モードの親ID
let taskFilter = 'all';              // all / week / actionable / overdue / today（永続化）
let showCompletedSection = false;    // 完了セクションの開閉（永続化）

// 今日のToDo（仮想リスト）
const TODAY_LIST_ID = '__today__';   // selectedListId にこの値が入ると集約ビュー
let todayViewMode = 'date';          // 'date'=日付順 / 'list'=リスト毎（永続化）
let todayFilter = 'all';             // 'all' / 'urgent'(超過+今日) / 'progress'(進行中)（永続化）
let todayTab = 'todo';               // 'todo'=今日のToDo / 'plan'=今日の予定（永続化）

// 今日の予定（v1.5.0）
let dayPlan = [];                    // 予定項目のフラット配列（todo-app-day-plan / Git 同期対象）
let planDate = null;                 // 表示中の予定日（null=今日。非永続）
let planPickerOpen = false, planSearch = '';  // 「タスクから追加」パネル

// カレンダー
let calendarYear, calendarMonth;     // 表示中の年・月(1-12)
let calendarSelectedDate = null;     // 'YYYY-MM-DD'
let calendarShowDone = false;        // 完了済みも表示
let calendarFilterListId = null;     // null=全リスト
let calendarDateMode = 'due';        // 'due' / 'start' / 'both'
let calendarCellMenu = null;         // 日セル右クリックメニュー { date, x, y }

// インポート / Git
let importMode = 'add';              // 'add'=既存に追記 / 'replace'=全消去して取り込む
let gitBusy = false;                 // pull/push 実行中フラグ
```

UI状態（`collapsedIds` / `expandedDoneSubs` / `showCompletedSection` / `listExpanded` / `taskFilter` / `todayViewMode` / `todayFilter`）は `todo-app-ui-state` に保存し、`load()` で復元・`render()` 末尾で保存する。`sidebarOpen` は意図的に対象外（モバイル起動時にサイドバーが開きっぱなしになるのを避けるため）。

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
  isHeading?: boolean,    // 見出し（分類ラベル）。メインのみ。チェック不可・件数集計から除外
                          // サブに降格すると自動解除（v1.4.0）
  recurrence?: {          // 繰り返し設定（あればこのタスクは繰り返しタスク）
    type: 'daily' | 'weekly' | 'monthly',
    weekday?: 0-6,        // type:'weekly' のとき、0=日〜6=土
    day?: 1-31 | 'last',  // type:'monthly' のとき、'last' は月末
  },
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
4. `recurrence` 型チェック: `typeof t.recurrence !== 'object'` ならクリア（旧データ互換）

---

## 主要機能一覧

### 実装済み機能

| 機能 | 説明 |
|------|------|
| タスクCRUD | 追加・編集・削除・完了チェック |
| 繰り返しタスク | 毎日/毎週X曜/毎月X日 or 月末日。完了後に周期境界を超えたら自動で未完了化＋次回 `dueDate` へ進める。`startDate` とは併用不可 |
| サブ完了トグル | 親メイン配下に完了サブが残るとき「▷ 完了 (N件)」で折り畳み展開。親IDごとに `expandedDoneSubs` で管理（セッション単位） |
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
| タスクの操作（モバイル） | タスク長押しで下から「タスクの操作」シートを開く（`htmlMovePopup()`。リストが1つでも開く）。iPhone は右クリックメニューを開けないため、PC の右クリックと同じ操作を揃えている: 今日の予定に追加／編集／見出しにする・解除／サブに登録▶（親候補を展開）／メインに昇格／別のリストへ移動▶（移動先を展開）／削除。展開中の項目は `movePopupSection` |
| モバイル切替バー | スマホ幅ではヘッダー直下に1行のバーを表示（`htmlMobileQuickBar()`、CSS で PC では非表示）。「☀ 今日のToDo（超過/今日の件数・📋予定 済/件数）」と「直前に開いた通常リスト」を1タップで行き来できる（`lastRegularListId`） |
| 全て折り畳み/展開 | リストヘッダーの▶/▽トグルボタン（hover ツールチップ） |
| 緊急度カラー | サイドバー件数バッジ 5段階: 超過=赤 > 今日=橙 > 今週中=緑 > 来週以降=水 > 既定 |
| タスクバーバッジ | Windows タスクバーアイコンに「超過＋今日」の合計件数を赤バッジで重ねて表示（Tauri 環境）。PWA インストール時は `setAppBadge` も呼ぶ。ウインドウタイトルにも `(N) TList` で反映 |
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
| Git/GitHub 同期（v1.1.0） | Tauri 版のみ。サイドバーの Pull / Push で `tasks.json` を Git リポジトリ経由で複数PC同期。3-way マージ＋競合解決UI。詳細は「Git 同期」節 |
| カレンダー（月ビュー） | ヘッダー「📅 カレンダー」から開く。全リストの期限/開始日タスクを集約表示。詳細は「カレンダー」節 |
| 今日のToDo（v1.3.x） | サイドバー最上部の仮想リスト。全リスト（メモ除く）の超過・今日・進行中を集約。詳細は「今日のToDo」節 |
| 見出し（v1.4.0） | メインを右クリック/長押し→「見出し」で分類ラベル化。チェック非表示、配下サブの進捗（完了数/全数）を表示、太字＋背景 |
| 今日の予定（v1.5.0） | 「今日のToDo」内のタブ。朝に午前/午後の時間割を組み、目安時間を付けてチェックしていく。チェック＝「今日の分は完了」でタスクの完了とは別。タスク以外の予定も可。Git 同期対象。詳細は「今日の予定」節 |
| 設定（v1.5.0） | サイドバー下部の「⚙ 設定」。「今日の予定」「Git 同期（Tauri 版のみ）」の表示/非表示を切り替える。非表示にしてもデータ・設定は残る。「🔔 通知（バッジ）」（Tauri 版以外）もここから開く |
| タスク検索 | ヘッダーの 🔍（または Ctrl+K）で検索モーダル（`modalMode='search'`、`htmlSearchModal()`）。入力のたびに全リストのタスクの本文・詳細メモから候補を出す（日本語入力は確定時に反映。入力欄のフォーカスを保つため結果欄だけ `renderSearchResults()` で差し替え）。一致部分を強調、1 行目で一致しなければ一致箇所を抜粋表示、サブタスクは「親 ›」付き。未完了 → 完了の順、「完了済みも含める」で切替、最大 100 件。候補のダブルクリック（モバイルはダブルタップ）または ↑↓＋Enter でそのタスクへ移動（`jumpFromCalendarToTodo` を流用）。クリックは選択のみ。検索語は次に開いたときも残る |
| iPhone のエクスポート | iPhone/iPad ではダウンロードすると JSON が画面に表示されるだけで保存されないため、`saveJsonFile()` が共有メニュー（`navigator.share` + File）を開き「"ファイル"に保存」で保存させる。共有非対応ならダウンロードにフォールバック、キャンセル時は何もしない。エクスポート画面に案内表示。ファイル名の日付はローカル日付（以前は UTC で朝 9 時前は前日になっていた） |
| インポート置き換え（v1.4.1） | インポート画面で「追加（既定）」/「置き換え（全リスト・タスク・ゴミ箱を消去して取り込む）」を選択。置き換えは警告バナー・危険色ボタン・確認ダイアログ付き |
| UI状態の永続化 | 折りたたみ・完了展開・フィルター・最後に開いたリスト等を再起動後も保持 |
| iPhone PWA バッジ | 「⚙ 設定」→「🔔 通知（バッジ）」から通知許可→ホーム画面アイコンにバッジ。未追加時はホーム画面追加手順を案内。Tauri 版では項目非表示。v1.5.0 でヘッダーの 🔔 ボタンから設定画面へ移動。iPhone で通知が未設定のときだけ、メニュー（☰）と「設定」ボタンに赤い点を表示（`needsNotifySetup()`。PC ブラウザでは出さない） |
| 日付ソート改良 | サブも日付順、完了は末尾（完了日の新しい順）、メインはグループ（自身+未完了サブ）の最古期日で判定。サブの日付フォールバックあり |
| サイドバー追従 | デスクトップでは `position:sticky` でスクロール追従（モバイルは縦積み） |
| 件数バッジ分割 | サイドバー件数を超過(赤)／今日(橙)に分けて同時表示 |

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
- メインが完了 **かつ未完了サブが残っていない** → そのサブも全て completed 側
- メインが完了 **だが未完了サブが残っている** → グループ全体を active 側に留める（a3eab13）

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

## 繰り返しタスク仕様

### データ構造
```javascript
recurrence?: {
  type: 'daily' | 'weekly' | 'monthly',
  weekday?: 0-6,          // type:'weekly' のとき。0=日〜6=土
  day?: 1-31 | 'last',    // type:'monthly' のとき。'last' は月末日
}
```

### 主要ヘルパー（TList.html 内）
| 関数 | 役割 |
|---|---|
| `addDaysYmd(ymd, n)` | YMD 文字列に日数を加算 |
| `monthlyDateFor(year, month, day)` | 月末対応の月日付算出（`day === 'last'` 含む） |
| `computeFirstDueDate(recurrence, baseYmd)` | 繰り返し設定から最初の発生日を算出 |
| `advanceRecurrenceDate(currentYmd, recurrence)` | 既存 `dueDate` から次回発生日を進める |
| `isoWeekKey(ymd)` | ISO週キー生成（毎週判定用） |
| `getPeriodKey(ymd, type)` | 日/週/月の周期キーを統一フォーマットで返す |
| `applyRecurringResets()` | 周期境界をまたいだ完了タスクを未完了に戻し `dueDate` を次回へ進める |

### リセット（v1.2.0 でイベント駆動に変更）
- チェック時はいったん完了状態にして「完了」セクションへ移動（達成感を残す）
- 以下のイベントで `applyRecurringResets()` を呼び、完了済みの繰り返しタスクを **即座に** 未完了化＋`dueDate` を次回発生日へ進める:
  - `load()`（アプリ起動）
  - `selectList()`（別リストへ切替）
  - `visibilitychange`（visible 復帰）
  - `applyExportPayloadInPlace()`（Git Pull / インポート / マージ反映）
- `render()` からの呼び出しは削除（チェック直後に即リセットされるのを防ぐため）
- 周期境界の判定には使わなくなったため、リセット時に `completedAt` はクリアする
- 旧仕様（周期境界をまたいだ render 時にリセット）では完了セクションに埋もれて次回予定が見えない問題があった

### 編集 UI の挙動
- 編集画面に「繰り返し」セレクト（なし／毎日／毎週／毎月）
- 毎週選択 → 曜日セレクト表示（`WEEKDAY_LABELS` 参照）
- 毎月選択 → 日付セレクト（1〜31 + 末日）
- `editRecurrence?.type` が真のとき、`startDate` 入力＋終了日入力は非表示
- `commitEdit()` で `editRecurrence` がセットされていれば `computeFirstDueDate()` で `dueDate` を自動設定し、`startDate` はクリア

### 表示バッジ
- `recurrence-badge` クラス（グレー配色）
- ラベル例: `🔁 毎日` / `🔁 毎週月` / `🔁 毎月末日`

### localStorage 互換性
- 旧データに `recurrence` がない場合は何もしない
- 値の型が object でない場合は load() で消去（マイグレーション §4）

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

完了済みタスクはどのフィルターにも該当しない（`matchFilter()` で除外）。見出し（`isHeading`）は残数の集計時に呼び出し側で除外する。
フィルター中は完了セクションも非表示。
フィルター中の表示結果からも完了済みは除外。
グループ単位でフィルター: メインまたはサブのいずれかが条件一致ならグループ全体を残す。
**サブの表示判定（a3eab13）**: 親が残るだけではサブは表示されない。サブ自身もフィルター条件を満たすことが必須。

---

## バッジ表示の詳細

### 日付バッジ
- **終了バッジ** (`dueBadge`): 📅 今日／明日／M/D。超過時「超過 (M/D)」赤色
- **開始バッジ** (`startBadge`): ▶ 今日／明日／M/D。**完了済みも含めて常時表示**（b034381 で変更）
  - 今日=黄、未来=水色、過去=グレー
- **同日統合バッジ** (`sameDayBadge`): 開始日 === 終了日の場合「▶📅 今日」のように両アイコン併記
- **繰り返しバッジ** (`recurrenceBadge`): 繰り返し設定があるとき「🔁 毎日 / 毎週月 / 毎月末日」など。グレー
- 期間表示クラス:
  - 開始日: `start-fu`（未来=水）、`start-td`（今日=黄）、`start-past`（過去=灰）
  - 終了日: `due-ov`（超過=赤）、`due-td`（今日=黄）、`due-fu`（今週中＝緑）、`due-far`（来週以降=水）

### サイドバー件数バッジ（list-count-pill）
通常リスト: **未完了件数**（s.active）を表示
メモリスト: 全件（s.total）を表示
緊急度カラー（5 段階、`listStats()` の `overdue / dueToday / dueWeek / weekPlus` を参照）:
- `urgent-ov`（超過あり）: 赤系。最優先
- `urgent-td`（今日が期限あり）: 橙系
- `urgent-fu`（今週中に期限あり）: 緑系
- `urgent-far`（来週以降に期限あり）: 水系
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
URL    → Tauri → window.open()
フォルダ → Tauri → クリップボードコピー（Web）
```

### 判定方法
```javascript
// Tauri（ネイティブ環境）
if (window.__TAURI__?.core?.invoke) {
  window.__TAURI__.core.invoke('open_folder', { path });
}
// それ以外（ブラウザ）
// URL: window.open(url, '_blank')
// フォルダ: navigator.clipboard.writeText(path)
```

旧 Electron / Photino 対応は撤廃済み（Tauri 一本化）。

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
        └── main.rs           ← open_folder / alert_window / set_plan_alarms（＋通知の見張りスレッド）/ set_taskbar_badge / git_* コマンド + WebView2追加引数設定
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

### 起動の速さ（2026-10-29 調査）
- 起動時間の大半は WebView2 の立ち上げ（白画面〜1 秒）。データ量の影響は小さい（タスク 300 件・予定 360 件で読み込み 0.6ms、描画 16〜40ms。5000 件だと今日のToDo の描画が約 0.5 秒になるので、1000 件を超えたら描画の高速化を検討）
- `tauri.conf.json` のウインドウに `backgroundColor: "#030712"`（アプリのダーク背景色）を指定し、準備中の白い画面を出さない
- 先頭の PWA 用アイコン生成（512px キャンバス＋PNG 変換、約 40ms）は Tauri 版では行わず、ブラウザ版でも `load` 後に後回し

### 重要設定
- `tauri.conf.json` の `app.withGlobalTauri: true` で `window.__TAURI__` をグローバル注入
- `app.windows[0].dragDropEnabled: false` で HTML5 のドラッグドロップを有効化（OS既定のファイルドロップを無効化）
- HTML同期: `package.json` の `predev`/`prebuild` で `TList.html` → `dist/index.html` にコピー

---

---

## 重要な実装パターン・注意事項

### SVGアイコン
`const I = { ... }` オブジェクトに全アイコンを定義。`I.folder`, `I.memo`, `I.play`, `I.cal` など。
撤廃: `I.arrowUp`, `I.sort`

### 曜日ラベル定数
`const WEEKDAY_LABELS = ['日','月','火','水','木','金','土']` を全体で共通参照（437bfb5 で集約）。
繰り返し設定 UI の曜日セレクトや、毎週繰り返しバッジ表示に使用。個別配列定義は重複排除済み。

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
- **Web (ブラウザ)**: 各ブラウザの localStorage 領域（ドメイン単位）
- **Tauri**: `%APPDATA%\com.tlist.app\EBWebView\` 配下（WebView2）
- ブラウザ版と Tauri 版でデータは**共有されない**。移行はJSONエクスポート/インポートで対応。

### localStorage キー一覧
| キー | 用途 |
|---|---|
| `todo-app-todos` | タスク配列 |
| `todo-app-trash` | ゴミ箱内タスク |
| `todo-app-lists` | リスト配列 |
| `todo-app-list-trash` | リストゴミ箱 |
| `todo-app-selected-list` | 選択中リストID（`__today__` も可。リスト切替だけでも保存） |
| `todo-app-dark` | ダークモードフラグ |
| `todo-app-show-stats` | リストステータス表示フラグ |
| `todo-app-ui-state` | UI表示状態（折りたたみ・フィルター・今日のToDoの並び/絞り込み/タブ等） |
| `todo-app-day-plan` | 今日の予定（`dayPlan` 配列。180日より古い分は load 時に削除） |
| `todo-app-settings` | 設定（`showDayPlan` / `showGitSync` / `planNotify`）。端末ごとの設定で Git 同期しない |
| `todo-app-plan-notified` | 時刻通知の通知済みキー一覧（端末ごと。Git 同期しない） |
| `todo-app-se-list` | SE 番号（案件）の一覧（Git 同期対象） |
| `todo-app-plan-snooze` | 時刻通知の再通知予約（予定ID → 時刻 ms。端末ごと。Git 同期しない） |
| `todo-app-git-config` | Git 同期設定（enabled / repoPath / fileName / remote / branch / commitTemplate） |
| `todo-app-git-baseline` | 3-way マージの基準（最後に同期した内容） |
| `todo-app-git-stash` | 競合時に退避したローカル内容（マージ確定で削除） |
| `todo-app-git-last-sync` | 最終同期日時（ISO文字列） |

### 撤廃された localStorage キー
- `todo-app-auto-sort` — 自動整列廃止に伴い不要（残っても無視される）
- `todo-app-add-to-top` — 先頭追加トグル廃止に伴い不要

---

## タスクバーバッジ

期限超過 + 今日が期限のタスク数を、Windows タスクバーアイコンに赤バッジで重ねて表示します。

### バッジ数の算出
`computeBadgeCount()`（`TList.html`）が `lists` を走査し、メモリスト以外について `listStats(l.id).overdue + .dueToday` を合算します。

### 配信経路
`render()` の末尾で `updateBadge(computeBadgeCount())` を呼ぶ。前回値と一致すれば早期 return（無駄な native 呼び出しを避ける）。

| 環境 | 経路 | 表示 |
|---|---|---|
| Tauri (Windows) | `invoke('set_taskbar_badge', { count })` → Rust 側で `ITaskbarList3::SetOverlayIcon` | タスクバーアイコンの右下に赤い数字バッジ |
| PWA (Chrome/Edge にインストール時) | `navigator.setAppBadge(n)` / `clearAppBadge()` | OS のバッジ機構（環境依存） |
| ブラウザ全般 | `document.title = '(N) TList'` | タイトルバー／タブ／タスクバー hover に件数表示 |

### Rust 側 (`src-tauri/src/main.rs`)
- `set_taskbar_badge(window, count)` コマンド
- `count == 0` のときは `SetOverlayIcon(hwnd, null, "")` でオーバーレイクリア
- 0 より大: `create_count_overlay_icon(count)` で 32×32 の HICON を動的生成（赤背景に白い数字、`99+` で打ち切り）
- 依存: `windows` クレート 0.61（Tauri 2 と同バージョン）の `Win32_Foundation` / `Win32_System_Com` / `Win32_UI_Shell` / `Win32_UI_WindowsAndMessaging` / `Win32_Graphics_Gdi`

### 制約
- バッジ数は更新時の差分検知（`_lastBadgeCount`）で重複呼び出しを抑制
- 非 Windows 環境では `set_taskbar_badge` は no-op（ビルドエラーを避けるため `#[cfg]` 分岐）

---

## Git 同期（v1.1.0〜、Tauri 版のみ）

ローカル Git リポジトリを介して GitHub 上の `tasks.json` にデータを保存し、複数PCで同期する。Firebase 方式は採用せず、こちらに置き換えた。

### 構成
- **設定モーダル**: リポジトリパス・ファイル名（既定 `tasks.json`）・リモート（既定 `origin`）・ブランチ・コミットメッセージテンプレート。接続テスト、ベースラインリセット（次回 Pull を初回扱いにする）ボタンあり
- **サイドバー**: Pull / Push ボタン。リモートが進んでいる件数（behind）をバッジ表示
- **Rust 側**（`main.rs`）: git CLI を子プロセスで起動するコマンド群（依存クレート追加なし）
  - `git_check_available` / `git_validate_repo` / `git_status` / `git_fetch`
  - `git_read_file` / `git_read_file_at_ref` / `git_write_file`
  - `git_pull_ff_only` / `git_commit_and_push` / `git_commit_only` / `git_abort_state`
  - セキュリティ: パストラバーサル防止、ref/remote/branch の文字種検証、`GIT_TERMINAL_PROMPT=0`。認証は OS の credential manager に任せ、PAT は扱わない

### Pull（`gitPull()`）
1. 手動 Pull は確認ダイアログを表示（Push フローから内部で呼ぶ Pull では出さない）
2. ローカルに未同期の変更があるときだけ、直前に全データを `tasks-backup-<日時>.json` としてダウンロード退避（`backupBeforePullIfNeeded()`）。復元は通常のインポートで行う
3. `git_fetch` → `git_status`。上流ブランチが未設定なら中止
4. `behind > 0` なら `FETCH_HEAD`、そうでなければ `HEAD` の `tasks.json` を読む。behind=0 でもローカルとリモートの内容が違えば処理を続ける
5. リモートにファイルが無い場合は**中止**する（無言でローカル上書きしない）。ローカルが空なら「Push で初回作成してください」と案内する
6. `git pull --ff-only` でローカルブランチを実際に進めてから、`mergeThreeWay(base, ours, theirs)` を実行
7. 競合がなければ自動マージし、件数をトーストに表示。競合があれば ours を `git-stash` に退避してマージ UI を開く（タスク・リスト単位で ours/theirs/両方残すを選び、一括選択も可）

### Push（`gitPush()`）
- まず Pull を実行し、続けて `tasks.json` の書き出し→コミット→push まで行う（Pull だけで終わらない）
- 書き出す JSON からは `exportedAt` を除く（差分が毎回出ないようにするため）

### 注意
- 別PCで初回 Pull するときは baseline が無い。初回扱いのロジックで localStorage に反映する（`062a5a1` で修正済み）
- Pull/Push 後は `gitBusy` を必ず解除する（`finally` で解除。`b55d759`）

---

## カレンダー（月ビュー）

- ヘッダー「📅 カレンダー」ボタンからモーダルで開く。6週×7日、土日と国民の祝日（2025〜2030）を色分け
- 表示対象: 終了日 / 開始日 / 両方（`calendarDateMode`）、リスト絞り込み、完了済みの表示切替
- 起動時は今日を選択。月送りで選択を解除する。選択中セルを再クリックしても解除しない
- サイドパネルにメモ・フォルダ/URL アイコン、開始/終了日チップを表示
- タスクをダブルクリックすると該当リストへジャンプし、移動先を一時ハイライトする。折りたたまれていれば自動で展開
- 日セルを右クリックするとタスク一覧ポップアップを開き、クリックで移動
- カレンダー内の操作は専用の部分 render にして、ちらつきを抑えている
- iPhone PWA では × ボタンが押せない問題を修正済み（`81345f0` / `1dbf1c3`）

---

## 今日のToDo（集約ビュー）

- サイドバー最上部の仮想リスト（`selectedListId === TODAY_LIST_ID`）。描画は `htmlTodayView()`
- 集約対象: メモリスト以外の全リストで、超過・今日が期限・進行中（開始日 ≤ 今日 ≤ 終了日）の未完了タスク。見出しは除外
- 並び: `todayViewMode` = 日付順（フラット）/ リスト毎
- 絞り込みチップ: すべて / 超過＋今日 / 進行中（通常リストのフィルターチップと同じ様式）
- リスト名バッジは固定幅（タスク文字の開始位置を揃える）。日付は通常の due-badge 配色
- 行をダブルクリック/ダブルタップすると実タスクへジャンプ

---

## 今日の予定（v1.5.0）

「今日のToDo」画面上部のタブ（☀️ 今日のToDo ／ 📋 今日の予定）で切り替える。選択中のタブはオレンジの塗りつぶし＋白文字＋太字、非選択は淡いグレー文字で、ひと目で区別できるようにしている（スマホ幅ではアイコンを省略）。スマホ幅では、ヘッダーがはみ出して「日付順/リスト毎」ボタンが見えなくならないよう、太陽アイコンを省き並び替えボタンはアイコンのみにしている。朝に時間割を作り、その順にチェックしていく使い方を想定。

### データ構造（`dayPlan` の各要素）
```javascript
{
  id: string,
  date: 'YYYY-MM-DD',     // 予定日
  slot: 'am' | 'pm',      // 午前 / 午後
  order: number,          // 枠内の並び順
  todoId?: string,        // タスクに紐づく場合。無ければ「タスク以外の予定」
  text: string,           // 追加時点の本文（1行目）。紐づくタスクがあれば表示はタスク側の現在の本文
  estimate?: number,      // 目安時間（時間単位: 0.25〜8）
  done: boolean,          // 今日の分は完了（タスクの completed とは独立）
  doneAt?: number,
  createdAt: number,
  carriedFrom?: string,   // 持ち越し元の日付
  carried?: boolean,      // この項目は持ち越し処理済み（持ち越した／持ち越さないを選んだ）
  memo?: string,          // 項目ごとのメモ（複数行可。持ち越し時はコピーしない）
  actual?: number,        // 実績時間（時間単位。持ち越し時はコピーしない）
  seNo?: string,          // 実績を紐づける SE 番号（案件）。持ち越し・明日へ移動で引き継ぐ
}
```
タスク本体（`todos`）には何も書き込まないので、既存の完了判定・件数集計・バッジには影響しない。

### 操作
- **追加**: 入力欄でタスク以外の予定を追加（午前/午後を選択、Enter で連続追加）。「タスクから追加」で今日のToDo（超過/今日/進行中）が候補に出る。検索欄に語を入れると全タスク（未完了・見出し/メモリスト除く）から検索できる
- 今日のToDo タブの各行の「＋」でも今日の予定に追加できる（12時前は午前、以降は午後。追加済みの行は 📋 表示）
- **今日に属さないタスク**（期限が先・期限なし）も追加できる: ①候補パネルの検索欄（全リストの未完了タスクが対象。パネルに案内文あり）②通常リストのタスク右クリックメニュー、モバイルは長押しポップアップの「📋 今日の予定に追加」（`ctx-add-to-plan`。見出し・完了済み・メモリストは対象外、追加済みなら「追加済み」表示、設定でオフなら非表示）
- **チェック**: 今日の分が終わった印。通常 ToDo（丸）と区別するため **四角のチェック枠**。後から読み返せるよう取り消し線は使わず、緑のチェック＋淡い緑の背景＋チェック時刻（✓ 11:39）で表示
- **バッジの並び**: 本文の後ろに「出どころ（リスト名 or 予定）→ 状態（タスク完了/削除済み）→ 持ち越し元 → 目安時間 → チェック時刻」の順。出どころは同じ見た目（`.plan-src`、グレー）
- **色の意味**: 緑＝今日の予定の「済」（チェック枠・チェック時刻・行背景）／オレンジ＝持ち越しパネルの「持ち越す」選択チェック（`.carry-cb`。済の緑と混同しないため）／藍色＝タスク自体の完了（「タスク完了」タグ。通常 ToDo の完了チェック・サイドバーの完了件数と同じ色）
- **行に常時出すのは** チェック・本文・↑↓（枠内の並び替え）・⋯ だけ。誤操作防止のため、次の操作は **右クリック／⋯ メニュー**（`planMenuId`、位置は `ctxMenuPos` を共用、描画は `htmlCtxMenu()`）に集約:
  - 目安時間（なし / 15分〜8h のチップから選択。設定済みなら行に「⏱ 1h」と表示）
  - 実績時間（目安と同じチップ、または「分」で自由入力して「設定」/ Enter。0・空欄で解除）。行に「実績 1.5h」チップ（目安以内=緑／目安超過=橙／目安なし=中立）。午前・午後の見出し、フッター、カレンダーの予定履歴に実績の合計も表示。時間表示 `formatHours()` は 30 分刻みなら「1.5h」、それ以外は「1h20分」
  - 内容を編集（タスク以外の予定）／タスク名を編集（タスクに紐づく予定。元タスクの 1 行目を書き換える）。行がその場で入力欄になり Enter 保存 / Esc 取消（`startPlanTextEdit()` / `commitPlanText()`、空なら保存しない）。タスク以外の予定は本文のダブルクリック（モバイルはダブルタップ）でも編集
  - メモを書く／編集（行の下に表示。メモ部分クリックでも編集。Ctrl+Enter 保存 / Esc 取消）
  - 午後へ移動／午前へ移動
  - 明日へ移動（今日以外を表示中は「翌日へ移動」）: 予定を翌日の同じ枠の最後へ移し、元の日からは消す（`movePlanToNextDay()`）。目安・メモ・通知時刻は引き継ぎ、チェックは未済に戻し、`carriedFrom` に元の日付を入れる（翌日側に「↪ 10/27」）。翌日に同じタスクがすでにあれば二重にせず元を消すだけ。再通知の予約は取り消す
  - 元のタスクを開く・タスクも完了にする（タスクに紐づく場合のみ。完了で予定側も済になる）
  - 予定から外す（赤。タスクは残る）
- スマホは右クリックが無いので ⋯ ボタンで同じメニューを開く
- 紐づく行はダブルクリック（モバイルはダブルタップ）で元タスクへジャンプ
- 日付送り（◀ ▶ / 今日へ）で過去や翌日の予定も見られる。過去分は日報のように残る。日付表示（「10/29（木）今日 ▾」）をクリックすると標準のカレンダー（`<input type="date">` の `showPicker()`）が開き、日付を直接選んで移動できる
- **持ち越し**: 今日を表示中、前日以前に未処理（`carried` でない）の候補があればパネルを出す（`planCarryCandidates()`）
  - 候補の種類: 「未済」＝チェックが付かなかった予定（全過去日。タスクが完了/削除済みのものは除く）／「済・継続中」＝チェックは付けたがタスク自体は未完了（**直近の予定日のみ**。繰り返しタスク・タスク以外の予定は除く）
  - 候補を一覧表示し、チェックで持ち越す項目を選ぶ（既定はすべて選択。外した ID は `planCarryUnchecked`）。「選んだ N 件を持ち越す」で今日にコピー（`carriedFrom` 付き、未済の状態で。今日すでに予定に入っているタスクはコピーしない）。選ばなかった候補と「持ち越さない」は処理済み（`carried=true`）にするだけ
- 各枠の見出しとフッターに目安時間の合計・済件数・残り時間を表示
- **カレンダーでの履歴**: 予定がある日のセルに「📋 済/件数」を表示（全部済なら緑）。日付を選ぶと詳細パネル上部に「この日の予定」（午前/午後・済・目安・メモ）を表示し、「予定を開く」でその日の予定タブへ移動。今日以降で予定がない日は「予定を作る」を表示（`htmlCalendarDayPlan()` / `calendar-open-plan`）

### SE 番号（工数管理）
社内ルールで「毎日どの SE 番号（案件）に何時間使ったか」を管理するための機能。設定「🏷 SE番号（工数管理）」（`appSettings.showSe`、既定オン）でオン/オフ（`seEnabled()`）。
- **データ**: `seList = [{ id: 番号, no, name, lastUsed }]`（`todo-app-se-list`。Git 同期・エクスポート対象。マージ比較は `no` / `name` のみ。`lastUsed` は端末ごとの値で、現在は並びに使っていない）。予定側は `p.seNo`（1 予定 = 1 SE 番号。複数の SE に分けたいときは予定を分ける）
- **入力**: 予定の右クリック（⋯）メニューの実績時間の下「🏷 SE番号」。ドロップダウン（「番号：案件名」。並びは SE 集計・表のコピー・CSV・管理一覧と共通の `seCompareNo()` で、固定番号 99999999 → 番号順。使うたびに場所が変わらないよう最近使った順にはしない）から選ぶか、「＋ 新しいSE番号を登録…」で番号と案件名を入力して登録（`seRegister()`。既存番号なら案件名を更新）。登録欄は「キャンセル」または Esc で閉じる。実績時間と SE 番号はセットで集計に使うため、**片方だけ入力した時点ではメニューを閉じず**もう片方の入力を促し、両方そろったとき（または値を外したとき）に閉じる（`afterPlanActualOrSe()`）。マウス操作の PC ではウインドウが狭くてもメニュー内の入力欄を 13px にそろえる（タッチ端末は iPhone のズーム防止で 16px）
- **引き継ぎ**: タスクに紐づく予定を追加すると、そのタスクで最後に使った SE 番号を自動で設定（`seDefaultForTodo()`）。持ち越し・明日へ移動でも引き継ぐ
- **表示**: 行に「🏷 SE-1001」チップ（ホバーで案件名）。実績があるのに未設定なら橙の破線「SE未設定」
- **SE 集計**（今日の予定タブの「📊 SE集計」→ `modalMode='se-report'`、`htmlSeReportModal()`）: 転記先のリストに合わせて 月単位で **行=SE 番号（番号順、未設定は最後）× 列=日付（横方向）**＋SE ごとの月合計列・日計行の表。SE 番号の列は左端に固定して横スクロール。土日祝・今日の列は色分け。時間は転記先に合わせて「時:分」（5.5 時間 → 5:30。`hHm()`。画面・コピー・CSV 共通）。特別な番号 `99999999` は `SE_PINNED` で常に一覧の一番上（ドロップダウン・集計の行・管理一覧）。日計が 8 時間（`SE_DAY_LIMIT`）を超えた日は日計セルを黄色で表示（ちょうど 8:00 は対象外）。ポップアップと表の枠は表の内容に合わせた幅（`width:fit-content`。最小 600px、最大は画面幅の 96%／1440px）で、列が少ない月は小さく、全画面なら月の全日でも横スクロールなし。長い注記は幅を広げないよう親の幅で折り返す。セルのホバーで予定の内訳。「実績のある日だけ」（既定オン）で画面は入力のある日の列だけ表示。「📋 表をコピー」（タブ区切りで Excel に貼り付け）と「💾 CSV」（BOM 付き UTF-8、`SE実績_YYYY-MM.csv`）は同じ向きで「SE番号, 案件名, 10/1 … 10/31, 合計」＋日計行を、常に月の全日で出力（`seMonthData()` / `seMonthTable()`）。番号と案件名の区切りは「：」
- **SE 番号の管理**: 集計画面下の「SE番号の管理」で案件名の変更・追加・一覧からの削除（削除しても予定側の記録は残り、集計には番号で出る）
- インポート: 置き換えは取り込み内容で置き換え、追加は未登録の番号だけ加える

### 時刻通知（v1.6.0・Tauri 版のみ・既定オフ）
- 設定画面「⏰ 予定の時刻通知」（`appSettings.planNotify`）をオンにすると有効。Tauri 版以外・「今日の予定」オフのときは UI も動作も一切出さない（`planNotifyEnabled()`）
- **登録**: 予定の右クリック（⋯）メニュー「⏰ 通知時刻」に時刻を入れて「設定」。その下のチップで **何分前に知らせるか（ちょうど/5/10/15分前、既定ちょうど）を予定ごとに選ぶ**。`p.notifyAt`（'HH:MM'）・`p.notifyLead`（分。0 は保存しない）に保存し Git 同期される。行に「⏰ 13:00」（何分前なら「・10分前」）チップ（橙=これから／灰=通知済み・予定済／赤=通知せず時刻を過ぎた）。持ち越し時はコピーしない
  - 時刻の選択は「時」「分（5分刻み）」のプルダウン（`htmlTimeSelects()` / `readTimeSelects()`）。`<input type="time">` はマウスホイールで回りすぎて合わせにくいため置き換えた。既存の値が 5 分刻みでなければその分も選択肢に加える
  - 未設定時の初期値（`defaultPlanNotifyTime()`）: 午後の予定は 12:00 前（または明日以降の予定）なら 12:00、12:00 を過ぎていれば現在時刻。午前の予定は現在時刻
  - 当初は全体設定（5分前固定）だったが、分かりにくいため v1.6.0 内で予定ごとの選択に変更した（`appSettings.planNotifyLead` は廃止。残っていても無視）
- **通知一覧**: 今日の予定タブの日付行右の「⏰ 通知一覧 (これからの件数)」。今日以降の通知を日付・時刻順に表示し、時刻の変更（time 入力）と解除ができる
- **最小化中の見張り（Rust 側）**: WebView2 は最小化中に JS のタイマーがほぼ止まり、JS だけでは通知できない。そのため JS は `render()` のたびに `syncPlanAlarms()` で「これから通知する時刻（Unix ms）の一覧」を Rust の `set_plan_alarms` に渡す（前回と同じ内容なら送らない。スヌーズ分も含む）。Rust の見張りスレッド（`start_plan_alarm_watcher`、5 秒間隔）が時刻になったものを取り除き、ウインドウを前面化（`bring_window_to_front`）して `plan-alarm` イベントを送る。JS は `window.__TAURI__.event.listen('plan-alarm')` で受けて `checkPlanNotifications()` を実行し、ポップアップと音を出す
- **動作**: `checkPlanNotifications()` を 20 秒ごと＋起動時＋復帰時（visibilitychange）＋ `plan-alarm` 受信時に実行。今日・未済・通知時刻あり・未通知で、「時刻 − 何分前」を過ぎたものを通知する
  - アプリ内ポップアップ `htmlPlanAlert()`（z-index 400、時刻順）。ボタン: 予定を開く／5分後にもう一度／OK
  - **再通知（5分後にもう一度）**: `planSnooze`（予定ID → 再通知時刻 ms）を `todo-app-plan-snooze` に保存し、再起動しても残る。待っている間は状態「再通知待ち」として、行のチップに「・再通知 13:05」、通知一覧に「再通知 13:05」（橙）と表示し、件数にも数える。通知一覧の行と予定の右クリック（⋯）メニューの「再通知をやめる」で予約だけ取り消せる（通知時刻の登録は残る）。予定が済・削除・今日でなくなった予約は自動で捨てる
  - 検知が 2 分以上遅れ、かつ予定の時刻そのものも過ぎていたもの（アプリを閉じていた間など）は「時刻を過ぎています」として表示し、前面化・音は行わない（予定の時刻がまだ先なら通常の通知にする）
  - 通常の通知では Rust の `alert_window` コマンドでウインドウを最小化解除・前面化（一時的に最前面固定 → 1.5 秒後に解除）・タスクバー点滅（`request_user_attention(Critical)`）し、短い通知音を鳴らす
- **通知済みの記録**は端末ごと（`todo-app-plan-notified`、キー = `予定ID@日付 時刻`。今日より前は自動で捨てる）。時刻を変えると再通知、「何分前」の変更では再通知しない。設定をオンにした瞬間は、すでに過ぎた今日の分を通知済み扱いにしてまとめて出ないようにしている
- 注意: `alert_window` / `set_plan_alarms` は Rust 側の機能なので、**v1.6.0 以降のインストーラでないと前面化・点滅・最小化中の通知は効かない**（旧ビルドでは invoke 失敗を無視し、表示中のポップアップのみ）

### 設定（表示/非表示）
- `appSettings = { showDayPlan, showGitSync }`（既定はどちらも true）。`todo-app-settings` に保存し、Git 同期しない（端末ごとの好み）
- `showDayPlan=false`: 今日のToDo のタブ・「＋」ボタン・カレンダーの予定表示を隠し、従来の「今日のToDo」だけの画面にする。`dayPlan` のデータと同期は維持
- `showGitSync=false`: サイドバーの Git 同期エリアを隠し、`refreshGitStatus()` も止める。Git 設定は保持。設定画面の Git 項目は Tauri 版でのみ表示

### 同期・エクスポート
- `buildExportPayload()` は全リストを対象にしたときだけ `dayPlan` を含める（一部リストのエクスポートには含めない）
- インポートの「置き換え」では `dayPlan` も消去し、ファイル内の `dayPlan` を復元する（`todoId` は新IDへ付け替え、見つからなければタスク以外の予定として残す）。「追加」では予定は取り込まない
- Git 同期: `SYNC_CATEGORIES` に `dayPlan` を追加し、3-way マージ・競合 UI（種別「予定」）の対象にした
- 旧バージョンが書いた tasks.json（`dayPlan` キーなし）は「base から変更なし」とみなしてローカルの予定を保持する（`mergeThreeWay` / `applyExportPayloadInPlace`）。ただし旧バージョンの PC から Push すると `dayPlan` が落ちた内容になるため、**同期する全 PC を v1.5.0 以上に揃えること**
- 併せて `MEANINGFUL_FIELDS_TODO` に `isHeading` を追加（見出しの切替だけの変更が同期で検知されない問題の修正）

---

## 見出し（isHeading）

- メインの右クリック/長押しメニューで見出しの ON/OFF を切り替える
- 見出し行はチェックボックスと繰り返しバッジを表示しない。配下サブの進捗「完了数/全数」を表示
- サイドバー件数・フッター残数・今日のToDo・バッジ数（`listStats` 系）から除外する
- サブに降格した時点でフラグを自動解除

---

## 配布ビルドの状況

`tauri/src-tauri/target/release/bundle/nsis/` にビルド済みインストーラがある:
`TList_1.0.0` / `1.2.0` / `1.3.0` / `1.3.1`（setup2）/ `1.4.0` / `1.5.0` / `1.6.0` / `1.7.0` / `1.8.0`。
`tauri/build.bat`（ビルド）と `tauri/debug.bat`（`npm run dev`。未コミット）を用意している。

---

## 未実装・検討事項

- **配布ビルド**: `cd tauri && npm run build` で NSIS インストーラを作る（`tauri/src-tauri/target/release/bundle/nsis/`）。v1.5.0・v1.6.0 はビルド済み。v1.6.0 は Rust 側に `alert_window` を追加しているので、時刻通知の前面化・点滅を使うには v1.6.0 以降のインストーラが必須。画面（TList.html）だけの変更でも Tauri 版に反映するにはビルドし直しが必要（dist/index.html が exe に埋め込まれるため）
- **今日の予定の今後の改善候補**: ドラッグでの並び替え、週単位の振り返り表示（目安と実績の比較など）。~~実績時間の記録~~・~~時刻指定（通知時刻）~~は v1.6.0 で実現済み
- **ブラウザ版（GitHub Pages / PWA）での同期**: Git 同期は Tauri 専用。ブラウザ/iPhone 版は JSON エクスポート/インポートで手動移行
- **`docs/` の追従**: v1.1.0 以降（Git 同期・カレンダー・今日のToDo・見出し等）は `docs/` に未反映。この HANDOFF.md を一次情報源とする
- **祝日データ**: 2025〜2030 年分をハードコードしている。2031年以降は追加が必要
- **Outlook連携**: `outlook:` プロトコルが Windows に未登録のため断念
- ~~データ同期（Firebase Firestore 方式B）~~ → Git/GitHub 同期（v1.1.0）で実現済み
- ~~Tauri 配布ビルド~~ → v1.6.0 まで実施済み

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
a841f41 feat: タスク検索と今日の予定の「明日へ移動」を追加し v1.7.0 へ
542de2c fix: 右クリックメニューが画面外にはみ出して選べない問題を修正
48e51ef feat: 今日の予定に実績時間を追加し、右クリックメニューの幅を調整
3e48d54 feat: 今日の予定の内容を後から編集できるように
56e0ed5 fix: 通知時刻の選択を時・分プルダウンにし、午後の予定は 12:00 を初期値に
ccd44c9 feat: 今日の予定に時刻通知を追加し v1.6.0 へ（Tauri 版のみ）
c6f829e feat: スマホの長押し「タスクの操作」メニューと今日のToDo切替バーを追加
de6d907 fix: iPhone でエクスポートしたファイルを保存できるよう共有メニューを開く
06d9218 feat: 今日の予定の持ち越しを選択式にし、タブの選択状態を見やすく
b9c49b9 feat: 「今日の予定」（午前/午後の時間割）と設定画面を追加し v1.5.0 へ
23af2ce feat: インポートに「置き換え（全消去して取り込む）」を追加し v1.4.1 へ
f563203 feat: メインタスクの見出し（分類ラベル）化を追加し v1.4.0 へ
55d40f7 feat: Pull に確認ダイアログと直前自動バックアップを追加
1e61154 fix: 今日のToDoの絞り込みチップを通常リストと同じ様式に統一
5662317 feat: 今日のToDoに絞り込み（すべて/超過＋今日/進行中）追加し v1.3.1 へ
2970931 feat: 今日のToDoに「進行中」を追加し v1.3.0 へ
79f13e2 feat: 「今日のToDo」集約ビュー追加とサイドバー追従・日付ソート改良
171d16b feat: サイドバーのリスト件数で超過+今日を同時表示
c2310b6 feat: 日付ソートを 2 点改良（サブ日付フォールバック / リスト日付ソート）
da758e8 chore: Tauri 版バージョンを 1.2.0 に揃える
68ae353 feat: 繰り返しタスクのリセットをイベント駆動に変更 (v1.2.0)
1dbf1c3 fix: iPhone PWA でカレンダー × ボタンが押せない問題を再修正
81345f0 fix: iPhone でカレンダーが操作しにくい 2 件を修正
3ca7c54 fix: カレンダーからのジャンプ時に折りたたみを自動展開
bd44749 feat: タスク一覧にカレンダー（月ビュー）機能を追加
4ac9cff fix: リスト切り替えだけでも次回起動時に最後のリストが復元されるよう保存
1e87da6 feat: UI 表示状態（展開・折りたたみ・フィルター）をアプリ再起動後も保持
c844842 fix: 長押し後に画面切り替え先で選択ハイライトが残る現象を抑制
0607ac4 fix: iPhone 長押し時のテキスト選択ハイライトを抑制
76831b6 fix: 日付入力ラベルも 16px に揃えて iOS Safari の自動ズームを抑える
0d55c01 fix: iPhone Safari の編集モード自動ズームを防止
1ccd173 fix: モバイルでバッジ・アイコンを更にコンパクトにして 1 行に収まりやすく
d86be71 chore: Tauri 版では 🔔 通知設定ボタンを非表示に
947ce8d fix: モバイル表示でタスク 1 件が縦長になる問題を緩和
0deaf6f feat: iPhone (PWA) でホーム画面アイコンにバッジ通知を出せるよう改造
0f90638 fix: Push 押下時に Pull だけして終了せず、続けて Push まで実行する
4a44c9e fix: 3-way マージ前にローカルブランチを ff-only で前進させる
2bffd55 fix: Pull で git pull --ff-only を実行してローカルブランチを実際に前進させる
b55d759 fix: Pull/Push 後にボタンの disabled が解除されない問題を修正
3fda5e0 fix: リモート tasks.json が取得できないときに無音でローカル上書きしないようにする
bea0f93 chore: 自動マージ完了トーストにマージ結果の件数を表示
1583fc0 fix: behind=0 でもローカル内容がリモートと食い違うときは Pull を実行
191432c feat: 設定モーダルにベースラインリセットボタンを追加
062a5a1 fix: 別 PC での初回 Pull で localStorage に反映されないバグを修正
204222f fix: Git 同期で書き出す tasks.json から exportedAt を除外
5eabf88 fix: 接続テストが入力途中の値を反映しないバグを修正
c84343a feat: Git/GitHub 連携によるタスクデータ同期機能を追加 (v1.1.0)
8dcce88 feat: タスクバーバッジ機能を追加（Tauri / PWA / Title） (#3)
27a8178 refactor: Tauri に一本化（Electron / Photino 撤廃）+ ドキュメント最新化 (#2)
437bfb5 refactor: 重複 CSS と曜日配列を集約
b87a2bc feat: 繰り返し機能（毎日/毎週X曜/毎月X日 or 末日）を追加
c65408c ui: サブグループ内に完了トグル「▷ 完了 (N件)」を追加
a3eab13 fix: 親完了時のサブ扱いとサブのフィルター判定を修正
b034381 ui: 期限/開始日バッジを統一配色化＋開始日を常時表示
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
