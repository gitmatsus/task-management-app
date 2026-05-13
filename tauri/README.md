# TList Tauri版

軽量化のためのTauri v2ベースのデスクトップアプリ。

## 初回セットアップ

### 1. 前提ツール（未インストールの場合のみ）

- **Rust**: https://www.rust-lang.org/tools/install から `rustup-init.exe` を実行
- **VS C++ Build Tools**: Visual Studio Installer で「C++によるデスクトップ開発」ワークロードを追加
- **Node.js**: 既にインストール済みならOK
- **WebView2 Runtime**: Windows 11は標準搭載、Windows 10は https://developer.microsoft.com/microsoft-edge/webview2/ から

インストール後、新しいPowerShell/コマンドプロンプトを開いて以下を確認：

```
rustc --version
cargo --version
```

### 2. 依存パッケージのインストール

```
cd C:\AI\タスク管理アプリ\tauri
npm install
```

### 3. アイコン生成（初回のみ）

Tauriのビルドにはアイコンが必要。ベース画像（1024x1024以上の `.png`）を用意して以下を実行：

```
npm run tauri icon path\to\icon.png
```

※ ベース画像がない場合、暫定で任意の正方形PNG画像を使えばOK。`src-tauri\icons\` 配下に必要なサイズのアイコンが自動生成される。

## 開発実行

```
npm run dev
```

初回は依存crateのダウンロード&コンパイルで5〜10分かかる。2回目以降は数秒。

## 配布用ビルド

```
npm run build
```

生成物は `src-tauri\target\release\bundle\nsis\` 配下の `.exe`（NSISインストーラ）。
期待サイズ: 10MB前後。

## ファイル構成

- `dist/index.html` — `TList.html` のコピー（`npm run dev` / `npm run build` 時に自動同期）
- `src-tauri/src/main.rs` — Rustバックエンド（`open_folder` コマンドのみ）
- `src-tauri/tauri.conf.json` — アプリ設定
- `src-tauri/Cargo.toml` — Rust依存定義

## 既知の制約

- **データはElectron版と別管理**: localStorageの保存先が異なるため、Electron版のデータは自動引き継ぎされない。必要なら既存の JSON エクスポート機能で移行する。
