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

// ── タスクバーバッジ ─────────────────────────────────────────
// 期限超過 + 今日が期限のタスク数を Windows タスクバーアイコンに
// オーバーレイ表示する。ITaskbarList3::SetOverlayIcon を使用。
// count == 0 のときは null HICON でオーバーレイをクリア。
// 非 Windows 環境では no-op。

#[cfg(target_os = "windows")]
#[tauri::command]
fn set_taskbar_badge(window: tauri::WebviewWindow, count: u32) -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{ITaskbarList3, TaskbarList};
    use windows::Win32::UI::WindowsAndMessaging::HICON;

    let hwnd_raw = window.hwnd().map_err(|e| e.to_string())?;
    let hwnd = HWND(hwnd_raw.0 as *mut std::ffi::c_void);

    unsafe {
        // COM 初期化（既に初期化済みなら無害なエラー）
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let taskbar: ITaskbarList3 = CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("CoCreateInstance failed: {}", e))?;
        taskbar
            .HrInit()
            .map_err(|e| format!("HrInit failed: {}", e))?;

        if count == 0 {
            taskbar
                .SetOverlayIcon(hwnd, HICON::default(), PCWSTR::null())
                .map_err(|e| format!("SetOverlayIcon(clear) failed: {}", e))?;
        } else {
            let hicon = create_count_overlay_icon(count)
                .map_err(|e| format!("create_count_overlay_icon: {}", e))?;
            let desc: Vec<u16> = "alert\0".encode_utf16().collect();
            taskbar
                .SetOverlayIcon(hwnd, hicon, PCWSTR::from_raw(desc.as_ptr()))
                .map_err(|e| format!("SetOverlayIcon failed: {}", e))?;
            // SetOverlayIcon は内部でアイコンを参照保持するので、ここでは破棄しない
            // （次回 SetOverlayIcon で置き換わるか、ウインドウ破棄時に OS が解放）
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn set_taskbar_badge(_count: u32) -> Result<(), String> {
    // Windows 以外では未対応（no-op）
    Ok(())
}

// ── HICON 動的生成 ────────────────────────────────────────────
// 32x32 の赤い背景に白い数字を描いたアイコンを返す。
// 表示テキスト: count <=9 で1桁、<=99 で2桁、それ以上は "99+"。
#[cfg(target_os = "windows")]
unsafe fn create_count_overlay_icon(
    count: u32,
) -> Result<windows::Win32::UI::WindowsAndMessaging::HICON, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi::{
        CreateBitmap, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW, CreateSolidBrush,
        DeleteDC, DeleteObject, DrawTextW, FillRect, GetDC, ReleaseDC, SelectObject, SetBkMode,
        SetTextColor, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY,
        DT_CENTER, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE, FW_BOLD, OUT_DEFAULT_PRECIS,
        TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

    const SIZE: i32 = 32;

    let text_str = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };
    // DrawTextW は &mut [u16] を取る。null 終端は不要（slice 長で渡る）
    let mut text_w: Vec<u16> = text_str.encode_utf16().collect();

    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    // カラー画像（32x32, スクリーン互換 24/32bpp）
    let hbm_color = CreateCompatibleBitmap(hdc_screen, SIZE, SIZE);

    // 1bpp マスク。全 0 = どこも不透明（背景の赤がそのまま出る）
    // バイト数 = ((SIZE + 31) / 32) * 4 * SIZE = 4 * 32 = 128 (32x32 の場合)
    let mask_stride = ((SIZE + 31) / 32) * 4;
    let mask_bytes = vec![0u8; (mask_stride * SIZE) as usize];
    let hbm_mask = CreateBitmap(
        SIZE,
        SIZE,
        1,
        1,
        Some(mask_bytes.as_ptr() as *const std::ffi::c_void),
    );

    let old_bmp = SelectObject(hdc_mem, hbm_color.into());

    // 背景を赤（BGR: 0x0000FF = 純赤）で塗りつぶし
    let red_brush = CreateSolidBrush(COLORREF(0x000000FFu32));
    let full_rect = RECT { left: 0, top: 0, right: SIZE, bottom: SIZE };
    FillRect(hdc_mem, &full_rect, red_brush);

    // フォント（数字桁数に応じてサイズ調整、太字）
    let font_h: i32 = match text_str.chars().count() {
        1 => -24, // 1桁: 大きめ
        2 => -18, // 2桁: 中
        _ => -14, // "99+": 小
    };
    let face_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    let hfont = CreateFontW(
        font_h,
        0,
        0,
        0,
        FW_BOLD.0 as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        DEFAULT_QUALITY,
        (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
        PCWSTR::from_raw(face_name.as_ptr()),
    );
    let old_font = SelectObject(hdc_mem, hfont.into());

    SetBkMode(hdc_mem, TRANSPARENT);
    SetTextColor(hdc_mem, COLORREF(0x00FFFFFFu32)); // 白

    let mut rect = RECT { left: 0, top: 0, right: SIZE, bottom: SIZE };
    DrawTextW(
        hdc_mem,
        &mut text_w,
        &mut rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    // GDI オブジェクト解放（SelectObject で戻してから DeleteObject）
    SelectObject(hdc_mem, old_font);
    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(hfont.into());
    let _ = DeleteObject(red_brush.into());
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    // ICONINFO に組み立てて HICON を生成
    let icon_info = ICONINFO {
        fIcon: true.into(),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };
    let hicon: HICON =
        CreateIconIndirect(&icon_info).map_err(|e| format!("CreateIconIndirect: {}", e))?;

    // CreateIconIndirect は内部でコピーするので元 bitmap は解放可
    let _ = DeleteObject(hbm_color.into());
    let _ = DeleteObject(hbm_mask.into());

    Ok(hicon)
}

fn main() {
    // WebView2のEdgeミニメニュー（テキスト選択時の翻訳ボタン等）を抑制
    #[cfg(target_os = "windows")]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-features=msEdgeMiniMenu,msEdgeAskMeAnything,TextSuggestionsForMiniMenu",
    );

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![open_folder, set_taskbar_badge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
