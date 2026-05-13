#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    // Windows: cmd /C start でパス/URL/ファイルを既定アプリで開く
    // CREATE_NO_WINDOWフラグでコンソールウィンドウの一瞬表示を抑制
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    // WebView2のEdgeミニメニュー（テキスト選択時の翻訳ボタン等）を抑制
    #[cfg(target_os = "windows")]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-features=msEdgeMiniMenu,msEdgeAskMeAnything,TextSuggestionsForMiniMenu",
    );

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
