#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

// ── Git 同期コマンド群 ───────────────────────────────────────
// 外部 git CLI を Command::new("git") で呼び出す。
// 認証は OS の credential manager に委ね、PAT 等はアプリ側で扱わない。
// セキュリティ: パストラバーサル防止 / `.git` 存在検証 / git ref の文字種検証 /
// シェル経由禁止（Vec<&str> 引数で渡す）。

#[derive(serde::Serialize)]
struct RepoInfo {
    branch: String,
    head: Option<String>,
    remote_url: Option<String>,
    file_exists: bool,
}

#[derive(serde::Serialize)]
struct StatusInfo {
    branch: String,
    clean: bool,
    ahead: u32,
    behind: u32,
    has_target_file_change: bool,
    other_dirty_files: Vec<String>,
    has_upstream: bool,
}

#[derive(serde::Serialize)]
struct CommitInfo {
    sha: String,
    pushed: bool,
    nothing_to_commit: bool,
}

fn validate_repo_path(repo: &str) -> Result<PathBuf, String> {
    if repo.trim().is_empty() {
        return Err("リポジトリパスが空です".into());
    }
    let p = PathBuf::from(repo);
    if !p.is_dir() {
        return Err(format!("リポジトリが見つかりません: {}", repo));
    }
    // .git は通常 dir、worktree の場合は file
    if !p.join(".git").exists() {
        return Err("指定パスは git リポジトリではありません（.git が見つかりません）".into());
    }
    Ok(p)
}

fn validate_file_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.starts_with('-')
        || Path::new(name).is_absolute()
    {
        return Err("ファイル名が不正です（リポジトリ直下のファイル名のみ可）".into());
    }
    Ok(())
}

fn validate_ref(git_ref: &str) -> Result<(), String> {
    if git_ref.is_empty() || git_ref.starts_with('-') {
        return Err("ref が不正です".into());
    }
    if !git_ref
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/_-.@{}".contains(c))
    {
        return Err("ref に使用できない文字が含まれています".into());
    }
    Ok(())
}

fn validate_remote(remote: &str) -> Result<(), String> {
    if remote.is_empty() || remote.starts_with('-') {
        return Err("リモート名が不正です".into());
    }
    if !remote
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))
    {
        return Err("リモート名に使用できない文字が含まれています".into());
    }
    Ok(())
}

fn validate_branch(branch: &str) -> Result<(), String> {
    if branch.is_empty() || branch.starts_with('-') {
        return Err("ブランチ名が不正です".into());
    }
    if !branch
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/_-.".contains(c))
    {
        return Err("ブランチ名に使用できない文字が含まれています".into());
    }
    Ok(())
}

fn run_git(repo: &Path, args: &[&str]) -> Result<(String, String, i32), String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never");
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().map_err(|e| {
        format!(
            "git の起動に失敗しました: {} (git CLI はインストールされていますか?)",
            e
        )
    })?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code = out.status.code().unwrap_or(-1);
    Ok((stdout, stderr, code))
}

fn run_git_global(args: &[&str]) -> Result<(String, String, i32), String> {
    let mut cmd = Command::new("git");
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never");
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("git の起動に失敗しました: {}", e))?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let code = out.status.code().unwrap_or(-1);
    Ok((stdout, stderr, code))
}

#[tauri::command]
fn git_check_available() -> Result<String, String> {
    let (out, err, code) = run_git_global(&["--version"])?;
    if code != 0 {
        return Err(format!("git CLI が見つかりません: {}", err));
    }
    Ok(out.trim().to_string())
}

#[tauri::command]
fn git_validate_repo(repo_path: String, file_name: Option<String>) -> Result<RepoInfo, String> {
    let repo = validate_repo_path(&repo_path)?;

    let (branch_out, branch_err, c1) = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = if c1 == 0 {
        branch_out.trim().to_string()
    } else if branch_err.contains("HEAD") || branch_err.contains("unknown revision") {
        // 初期コミット前
        String::new()
    } else {
        return Err(format!("ブランチ取得失敗: {}", branch_err));
    };

    let (head_out, _, c2) = run_git(&repo, &["rev-parse", "--short", "HEAD"])?;
    let head = if c2 == 0 {
        Some(head_out.trim().to_string())
    } else {
        None
    };

    let (remote_out, _, _) = run_git(&repo, &["config", "--get", "remote.origin.url"])?;
    let remote_url = if remote_out.trim().is_empty() {
        None
    } else {
        Some(remote_out.trim().to_string())
    };

    let file_exists = if let Some(name) = file_name.as_deref() {
        validate_file_name(name)?;
        repo.join(name).exists()
    } else {
        false
    };

    Ok(RepoInfo {
        branch,
        head,
        remote_url,
        file_exists,
    })
}

#[tauri::command]
fn git_status(repo_path: String, file_name: String) -> Result<StatusInfo, String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;

    let (branch_out, _, _) = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch_out.trim().to_string();

    let (porcelain, _, _) = run_git(&repo, &["status", "--porcelain=v1"])?;
    let mut other_dirty = Vec::new();
    let mut tasks_dirty = false;
    for line in porcelain.lines() {
        if line.len() < 4 {
            continue;
        }
        // " M path" / "?? path" / "MM path" など。先頭2文字がステータス、その後にスペース、パス
        let path = line[3..].trim_start_matches('"').trim_end_matches('"');
        if path == file_name {
            tasks_dirty = true;
        } else {
            other_dirty.push(path.to_string());
        }
    }
    let clean = porcelain.trim().is_empty();

    // ahead/behind は upstream が無いとエラーになる
    let (rev, _, code) = run_git(
        &repo,
        &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
    )?;
    let (has_upstream, behind, ahead) = if code == 0 {
        let parts: Vec<&str> = rev.split_whitespace().collect();
        let b: u32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let a: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (true, b, a)
    } else {
        (false, 0, 0)
    };

    Ok(StatusInfo {
        branch,
        clean,
        ahead,
        behind,
        has_target_file_change: tasks_dirty,
        other_dirty_files: other_dirty,
        has_upstream,
    })
}

#[tauri::command]
fn git_fetch(repo_path: String, remote: Option<String>) -> Result<(), String> {
    let repo = validate_repo_path(&repo_path)?;
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    validate_remote(&remote)?;
    let (_, err, code) = run_git(&repo, &["fetch", &remote])?;
    if code != 0 {
        return Err(format!("git fetch 失敗: {}", err));
    }
    Ok(())
}

#[tauri::command]
fn git_read_file(repo_path: String, file_name: String) -> Result<Option<String>, String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;
    let target = repo.join(&file_name);
    if !target.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&target).map_err(|e| format!("読み込み失敗: {}", e))?;
    Ok(Some(content))
}

#[tauri::command]
fn git_read_file_at_ref(
    repo_path: String,
    file_name: String,
    git_ref: String,
) -> Result<Option<String>, String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;
    validate_ref(&git_ref)?;
    let spec = format!("{}:{}", git_ref, file_name);
    // `--` で path 引数として明示
    let (out, err, code) = run_git(&repo, &["show", &spec])?;
    if code != 0 {
        // ファイルが存在しないケース
        if err.contains("does not exist")
            || err.contains("exists on disk, but not in")
            || err.contains("unknown revision")
            || err.contains("Path") && err.contains("does not exist")
        {
            return Ok(None);
        }
        return Err(format!("git show 失敗: {}", err));
    }
    Ok(Some(out))
}

#[tauri::command]
fn git_write_file(repo_path: String, file_name: String, content: String) -> Result<(), String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;
    let target = repo.join(&file_name);
    let tmp = repo.join(format!(".{}.tmp", file_name));
    std::fs::write(&tmp, content).map_err(|e| format!("一時ファイル書き込み失敗: {}", e))?;
    // Windows では rename で既存ファイルを上書きできるよう、先に削除を試みる
    if target.exists() {
        let _ = std::fs::remove_file(&target);
    }
    std::fs::rename(&tmp, &target).map_err(|e| format!("リネーム失敗: {}", e))?;
    Ok(())
}

#[tauri::command]
fn git_pull_ff_only(
    repo_path: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<(), String> {
    let repo = validate_repo_path(&repo_path)?;
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    validate_remote(&remote)?;
    let mut args: Vec<String> = vec!["pull".into(), "--ff-only".into(), remote];
    if let Some(b) = branch {
        validate_branch(&b)?;
        args.push(b);
    }
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (_, err, code) = run_git(&repo, &args_ref)?;
    if code != 0 {
        return Err(format!("git pull --ff-only 失敗: {}", err));
    }
    Ok(())
}

#[tauri::command]
fn git_commit_and_push(
    repo_path: String,
    file_name: String,
    message: String,
    remote: Option<String>,
    branch: Option<String>,
) -> Result<CommitInfo, String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;
    let remote = remote.unwrap_or_else(|| "origin".to_string());
    validate_remote(&remote)?;

    // add
    let (_, e_add, c_add) = run_git(&repo, &["add", "--", &file_name])?;
    if c_add != 0 {
        return Err(format!("git add 失敗: {}", e_add));
    }

    // diff --cached --quiet: 差分があれば exit=1、無ければ exit=0
    let (_, _, c_diff) = run_git(&repo, &["diff", "--cached", "--quiet"])?;
    let nothing_to_commit = c_diff == 0;

    if !nothing_to_commit {
        let (_, e_commit, c_commit) = run_git(&repo, &["commit", "-m", &message])?;
        if c_commit != 0 {
            return Err(format!("git commit 失敗: {}", e_commit));
        }
    }

    // push（コミットがなくても、ローカルが先行している場合は push する必要があるので常に実行）
    let current_branch = match branch {
        Some(b) => {
            validate_branch(&b)?;
            b
        }
        None => {
            let (out, err, code) = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"])?;
            if code != 0 {
                return Err(format!("ブランチ取得失敗: {}", err));
            }
            out.trim().to_string()
        }
    };
    let (_, e_push, c_push) = run_git(&repo, &["push", &remote, &current_branch])?;
    if c_push != 0 {
        return Err(format!("git push 失敗: {}", e_push));
    }

    let (sha_out, _, _) = run_git(&repo, &["rev-parse", "--short", "HEAD"])?;
    Ok(CommitInfo {
        sha: sha_out.trim().to_string(),
        pushed: true,
        nothing_to_commit,
    })
}

#[tauri::command]
fn git_commit_only(
    repo_path: String,
    file_name: String,
    message: String,
) -> Result<CommitInfo, String> {
    let repo = validate_repo_path(&repo_path)?;
    validate_file_name(&file_name)?;

    let (_, e_add, c_add) = run_git(&repo, &["add", "--", &file_name])?;
    if c_add != 0 {
        return Err(format!("git add 失敗: {}", e_add));
    }

    let (_, _, c_diff) = run_git(&repo, &["diff", "--cached", "--quiet"])?;
    let nothing_to_commit = c_diff == 0;

    if !nothing_to_commit {
        let (_, e_commit, c_commit) = run_git(&repo, &["commit", "-m", &message])?;
        if c_commit != 0 {
            return Err(format!("git commit 失敗: {}", e_commit));
        }
    }

    let (sha_out, _, _) = run_git(&repo, &["rev-parse", "--short", "HEAD"])?;
    Ok(CommitInfo {
        sha: sha_out.trim().to_string(),
        pushed: false,
        nothing_to_commit,
    })
}

#[tauri::command]
fn git_abort_state(repo_path: String) -> Result<(), String> {
    let repo = validate_repo_path(&repo_path)?;
    if repo.join(".git/MERGE_HEAD").exists() {
        let (_, _, _) = run_git(&repo, &["merge", "--abort"])?;
    }
    if repo.join(".git/CHERRY_PICK_HEAD").exists() {
        let (_, _, _) = run_git(&repo, &["cherry-pick", "--abort"])?;
    }
    if repo.join(".git/rebase-merge").exists() || repo.join(".git/rebase-apply").exists() {
        let (_, _, _) = run_git(&repo, &["rebase", "--abort"])?;
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
        .invoke_handler(tauri::generate_handler![
            open_folder,
            set_taskbar_badge,
            git_check_available,
            git_validate_repo,
            git_status,
            git_fetch,
            git_read_file,
            git_read_file_at_ref,
            git_write_file,
            git_pull_ff_only,
            git_commit_and_push,
            git_commit_only,
            git_abort_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
