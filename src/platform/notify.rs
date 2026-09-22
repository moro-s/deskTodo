#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::path::PathBuf;

pub(crate) fn send_notification(title: &str, body: &str) {
    #[cfg(target_os = "windows")]
    {
        send_windows_toast(title, body);
    }
    #[cfg(target_os = "linux")]
    {
        send_linux_notification(title, body);
    }
    #[cfg(target_os = "macos")]
    {
        send_macos_notification(title, body);
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = (title, body);
    }
}

#[cfg(target_os = "windows")]
const TOAST_SCRIPT: &str = r#"
$appId = 'DeskTodo.Reminder'
$key = "HKCU:\Software\Classes\AppUserModelId\$appId"
if (-not (Test-Path $key)) { New-Item -Path $key -Force | Out-Null }
New-ItemProperty -Path $key -Name 'DisplayName' -Value 'deskTodo' -PropertyType String -Force | Out-Null
if ($env:DESKTODO_ICON) {
    try {
        $iconUri = ([Uri]::new($env:DESKTODO_ICON)).AbsoluteUri
        New-ItemProperty -Path $key -Name 'IconUri' -Value $iconUri -PropertyType String -Force | Out-Null
    } catch { }
}
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType=WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom, ContentType=WindowsRuntime] | Out-Null
$template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)
$texts = $template.GetElementsByTagName('text')
[void]$texts.Item(0).AppendChild($template.CreateTextNode($env:DESKTODO_TITLE))
[void]$texts.Item(1).AppendChild($template.CreateTextNode($env:DESKTODO_BODY))
$toast = [Windows.UI.Notifications.ToastNotification]::new($template)
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($appId).Show($toast)
"#;

#[cfg(target_os = "windows")]
fn send_windows_toast(title: &str, body: &str) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = std::process::Command::new("powershell");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        TOAST_SCRIPT,
    ]);
    command.env("DESKTODO_TITLE", title);
    command.env("DESKTODO_BODY", body);
    if let Some(icon) = ensure_icon_file() {
        command.env("DESKTODO_ICON", icon);
    }
    match command.creation_flags(CREATE_NO_WINDOW).status() {
        Ok(status) if status.success() => {
            crate::log_info!("notify", "系统通知已发送：{title}");
        }
        Ok(status) => {
            crate::log_error!("notify", "通知命令异常退出：{status}");
        }
        Err(err) => {
            crate::log_error!("notify", "无法启动通知命令：{err}");
        }
    }
}

#[cfg(target_os = "linux")]
fn send_linux_notification(title: &str, body: &str) {
    let mut command = std::process::Command::new("notify-send");
    command.args(["-a", "deskTodo", "-t", "10000"]);
    if let Some(icon) = ensure_icon_file() {
        command.arg("-i").arg(icon);
    }
    match command.arg(title).arg(body).status() {
        Ok(status) if status.success() => {
            crate::log_info!("notify", "系统通知已发送：{title}");
        }
        Ok(status) => {
            crate::log_error!("notify", "通知命令异常退出：{status}");
        }
        Err(err) => {
            crate::log_error!("notify", "无法启动 notify-send：{err}");
        }
    }
}

#[cfg(target_os = "macos")]
fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
fn send_macos_notification(title: &str, body: &str) {
    let script = format!(
        "display notification \"{}\" with title \"{}\"",
        escape_applescript(body),
        escape_applescript(title)
    );
    match std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
    {
        Ok(status) if status.success() => {
            crate::log_info!("notify", "系统通知已发送：{title}");
        }
        Ok(status) => {
            crate::log_error!("notify", "通知命令异常退出：{status}");
        }
        Err(err) => {
            crate::log_error!("notify", "无法启动 osascript：{err}");
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn ensure_icon_file() -> Option<PathBuf> {
    let path = crate::core::storage::default_data_dir()?.join("notify_icon.png");
    if !path.exists() {
        let icon = crate::platform::icon::calendar_icon_data();
        let png = encode_png(&icon.rgba, icon.width, icon.height);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, png).ok()?;
    }
    Some(path)
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn encode_png(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let stride = width as usize * 4;
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);
    for row in rgba.chunks_exact(stride) {
        raw.push(0);
        raw.extend_from_slice(row);
    }

    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (index, block) in blocks.iter().enumerate() {
        let last = index + 1 == blocks.len();
        zlib.push(u8::from(last));
        let length = block.len() as u16;
        zlib.extend_from_slice(&length.to_le_bytes());
        zlib.extend_from_slice(&(!length).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::with_capacity(zlib.len() + 128);
    png.extend_from_slice(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    png.extend_from_slice(&png_chunk(b"IHDR", &ihdr));
    png.extend_from_slice(&png_chunk(b"IDAT", &zlib));
    png.extend_from_slice(&png_chunk(b"IEND", &[]));
    png
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn png_chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(data.len() + 12);
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(tag);
    chunk.extend_from_slice(data);
    let checksum = crc32(&chunk[4..]);
    chunk.extend_from_slice(&checksum.to_be_bytes());
    chunk
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a += byte as u32;
        if a >= 65_521 {
            a -= 65_521;
        }
        b += a;
        if b >= 65_521 {
            b -= 65_521;
        }
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    use super::encode_png;

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    #[test]
    fn png_encode_matches_expected_size() {
        let width = 2;
        let height = 2;
        let rgba: Vec<u8> = (0..16).collect();
        let png = encode_png(&rgba, width, height);
        assert_eq!(
            &png[..8],
            &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']
        );
        assert_eq!(&png[8..12], &13u32.to_be_bytes());
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(&png[16..20], &width.to_be_bytes());
        assert_eq!(&png[20..24], &height.to_be_bytes());
        // signature 8 + IHDR 25 + IDAT (12 + zlib 29) + IEND 12
        assert_eq!(png.len(), 86);
        assert_eq!(&png[37..41], b"IDAT");
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }
}
