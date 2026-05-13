# 05-1 localStorageキー一覧

`TList.html` が利用する `localStorage` キーの全リスト。3方式（Electron／Tauri／Photino）間でこれらは共有されません（[03-2](../03_開発とビルド/03-2_ネイティブパッケージング.md)）。

## キー一覧

| キー | 型 | 用途 | 設定箇所 |
|---|---|---|---|
| `todo-app-todos` | JSON配列 | タスク本体 | `save()` |
| `todo-app-trash` | JSON配列 | タスクゴミ箱 | `save()` |
| `todo-app-lists` | JSON配列 | リスト一覧 | `save()` |
| `todo-app-list-trash` | JSON配列 | リストゴミ箱（リスト + タスク） | `save()` |
| `todo-app-selected-list` | string | 現在選択中の `listId` | `save()` |
| `todo-app-show-stats` | `'true'` \| `'false'` | サイドバーのステータスドット表示 | `toggleListStats()` |
| `todo-app-add-to-top` | `'true'` \| `'false'` | 新規タスクを先頭に追加するか | `toggle-add-top` ハンドラ |
| `todo-app-auto-sort` | `'true'` \| `'false'` | 期限日順の自動整列 | `toggle-auto-sort` ハンドラ |
| `darkMode` | `'true'` \| `'false'` | ダークモード | `toggleDark()` / `<head>` インラインスクリプトで読込 |

## 読込タイミング

| キー | 読み込み箇所 |
|---|---|
| `darkMode` | `<head>` のインラインスクリプト（FOUC回避のため即時適用、[TList.html:63](../../TList.html)） |
| `todo-app-show-stats` / `todo-app-add-to-top` / `todo-app-auto-sort` | スクリプト先頭の状態変数初期化時 |
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
    "folderPath": "C:\\Work\\docs",
    "linkType": "folder",
    "detail": "テンプレート参照"
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

エクスポートJSONには `todo-app-todos` / `todo-app-lists` / `todo-app-trash` 相当のデータのみが含まれ、UI設定（`todo-app-add-to-top` 等）や `darkMode` は含まれません。

## 次に読むべき章

- [05-2 既知の制約とハマりどころ](05-2_既知の制約とハマりどころ.md)
- [04-2 データモデルと永続化](../04_実装詳細/04-2_データモデルと永続化.md)
- [02-5 インポートエクスポート](../02_機能ガイド/02-5_インポートエクスポート.md)
