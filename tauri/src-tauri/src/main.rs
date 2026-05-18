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
        // COM 初期化（既に別モードで初期化済みなら S_FALSE が返るが問題なし）
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
            // SetOverlayIcon は内部でアイコンの参照を保持するため、ここでは破棄しない
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
// 32x32 BGRA DIB を直接作り、赤背景に白い数字を描いたアイコンを返す。
// ポイント: GDI の DrawText はアルファチャンネルを 0 にしてしまうため、
// CreateCompatibleBitmap だとアイコンが完全透明になって見えない問題が起きる。
// CreateDIBSection で生のピクセルにアクセスし、描画後にアルファ=0xFF を強制。
#[cfg(target_os = "windows")]
unsafe fn create_count_overlay_icon(
    count: u32,
) -> Result<windows::Win32::UI::WindowsAndMessaging::HICON, String> {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{COLORREF, RECT};
    use windows::Win32::Graphics::Gdi::{
        CreateBitmap, CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject,
        DrawTextW, GetDC, ReleaseDC, SelectObject, SetBkMode, SetTextColor, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH,
        DEFAULT_QUALITY, DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE, DT_VCENTER, FF_DONTCARE,
        FW_BOLD, OUT_DEFAULT_PRECIS, RGBQUAD, TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{CreateIconIndirect, HICON, ICONINFO};

    const SIZE: i32 = 32;

    let text_str = if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    };
    let mut text_w: Vec<u16> = text_str.encode_utf16().collect();

    let hdc_screen = GetDC(None);
    let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

    // 32bpp top-down BGRA DIB section（biHeight が負 = 上から下へのスキャンライン）
    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SIZE,
            biHeight: -SIZE,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default()],
    };
    let mut bits_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
    let hbm_color = CreateDIBSection(
        Some(hdc_screen),
        &bmi,
        DIB_RGB_COLORS,
        &mut bits_ptr,
        None,
        0,
    )
    .map_err(|e| format!("CreateDIBSection: {}", e))?;

    // ピクセル配列に直接アクセス（4 byte/pixel, BGRA メモリ順）
    // 32bit little-endian の u32 表現: 0xAARRGGBB
    // 赤不透明 = A=0xFF, R=0xFF, G=0x00, B=0x00 = 0xFFFF0000
    let pixel_count = (SIZE * SIZE) as usize;
    let pixels: &mut [u32] = std::slice::from_raw_parts_mut(bits_ptr as *mut u32, pixel_count);

    // 円内の判定（中心 (SIZE/2, SIZE/2), 半径 SIZE/2）
    let r = SIZE as f32 / 2.0;
    let r_sq = r * r;
    let is_inside_circle = |x: i32, y: i32| -> bool {
        let dx = x as f32 - r + 0.5;
        let dy = y as f32 - r + 0.5;
        dx * dx + dy * dy <= r_sq
    };

    // 初期化: 円の内側 = 赤不透明、外側 = 完全透明
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = (y * SIZE + x) as usize;
            pixels[idx] = if is_inside_circle(x, y) {
                0xFFFF0000u32
            } else {
                0u32
            };
        }
    }

    // 1bpp マスク、全 0 = どこも不透明（カラーアルファを尊重させるため）
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

    // フォント（数字桁数に応じてサイズ調整、太字）
    let font_h: i32 = match text_str.chars().count() {
        1 => -24,
        2 => -18,
        _ => -14,
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

    let mut rect = RECT {
        left: 0,
        top: 0,
        right: SIZE,
        bottom: SIZE,
    };
    DrawTextW(
        hdc_mem,
        &mut text_w,
        &mut rect,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );

    // GDI 後始末
    SelectObject(hdc_mem, old_font);
    SelectObject(hdc_mem, old_bmp);
    let _ = DeleteObject(hfont.into());
    let _ = DeleteDC(hdc_mem);
    ReleaseDC(None, hdc_screen);

    // GDI 描画後の後処理:
    // - 円の内側: アルファ 0xFF を強制（描画されたテキスト含めて不透明化）
    // - 円の外側: 完全透明にクリア（GDI が円外にテキストを描いた場合の保険）
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = (y * SIZE + x) as usize;
            if is_inside_circle(x, y) {
                pixels[idx] |= 0xFF000000u32;
            } else {
                pixels[idx] = 0u32;
            }
        }
    }

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
