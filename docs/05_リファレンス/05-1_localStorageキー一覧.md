# 05-1 localStorageキー一覧

`TList.html` が利用する `localStorage` キーの全リスト。ブラウザ版と Tauri 版で localStorage は共有されません（[03-2 Tauri パッケージング](../03_開発とビルド/03-2_ネイティブパッケージング.md)）。

## キー一覧

| キー | 型 | 用途 | 設定箇所 |
|---|---|---|---|
| `todo-app-todos` | JSON配列 | タスク本体（サブも含む） | `save()` |
| `todo-app-trash` | JSON配列 | タスクゴミ箱 | `save()` |
| `todo-app-lists` | JSON配列 | リスト一覧 | `save()` |
| `todo-app-list-trash` | JSON配列 | リストゴミ箱（リスト + タスク） | `save()` |
| `todo-app-selected-list` | string | 現在選択中の `listId` | `save()` |
| `todo-app-show-stats` | `'true'` \| `'false'` | サイドバーのステータスドット表示 | `toggleListStats()` |
| `darkMode` | `'true'` \| `'false'` | ダークモード（プレフィックスなしのレガシー名） | `toggleDark()` / `<head>` インラインスクリプトで読込 |

## 廃止されたキー

| キー | 廃止理由 |
|---|---|
| `todo-app-add-to-top` | 先頭追加トグル廃止（`3dff973`）。新規タスクは常に先頭追加に固定 |
| `todo-app-auto-sort` | 自動整列廃止。完了セクションが分離するため不要 |

> 残っていても読み込まれません。手動削除は不要。

## 永続化されない状態（セッション内のみ）

以下はメモリ保持のみで、リロード／再起動で全リセットされます（仕様）。

| 状態 | 用途 |
|---|---|
| `collapsedIds` | サブを持つメインの折りたたみ |
| `expandedDoneSubs` | 「▷ 完了 (N件)」サブヘッダーの展開 |
| `showCompletedSection` | 完了セクション「▶ 完了 (N件)」の開閉 |
| `taskFilter` | フィルターチップ（all/week/actionable/overdue/today） |
| `showTrash` / `showListTrash` | ゴミ箱パネル開閉 |

永続化を追加する場合は、本ファイルとデータモデル章も更新してください。

## 読込タイミング

| キー | 読み込み箇所 |
|---|---|
| `darkMode` | `<head>` のインラインスクリプト（FOUC回避のため即時適用） |
| `todo-app-show-stats` | スクリプト先頭の状態変数初期化時 |
| `todo-app-todos` / `todo-app-trash` / `todo-app-lists` / `todo-app-list-trash` / `todo-app-selected-list` | `load()` 関数内 |

## 値の例

### `todo-app-todos`

```json
[
  {
    "id": "1c7e...",
    "text": "資料作成",
    "completed": false,
    "createdAt": 1714000000000,
    "listId": "9a02...",
    "dueDate": "2026-05-10",
    "startDate": "2026-05-09",
    "folderPath": "C:\\Work\\docs",
    "linkType": "folder",
    "detail": "テンプレート参照",
    "parentId": "0f12...",
    "recurrence": { "type": "weekly", "weekday": 1 }
  }
]
```

### `todo-app-lists`

```json
[
  { "id": "9a02...", "name": "マイタスク", "createdAt": 1700000000000 },
  { "id": "be41...", "name": "メモ",       "createdAt": 1710000000000, "type": "memo" }
]
```

## バックアップ・移行

`localStorage` は環境間で共有されないため、バックアップや環境間の移行は [02-5 インポートエクスポート](../02_機能ガイド/02-5_インポートエクスポート.md) のJSONを介して行います。

エクスポートJSONには `todo-app-todos` / `todo-app-lists` / `todo-app-trash` 相当のデータのみが含まれ、UI設定（`darkMode` / `todo-app-show-stats` 等）は含まれません。

## 次に読むべき章

- [05-2 既知の制約とハマりどころ](05-2_既知の制約とハマりどころ.md)
- [04-2 データモデルと永続化](../04_実装詳細/04-2_データモデルと永続化.md)
- [02-5 インポートエクスポート](../02_機能ガイド/02-5_インポートエクスポート.md)
