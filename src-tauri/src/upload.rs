//! Uploads a local image to catbox.moe so Discord can load it from a link.

use std::fs;
use std::path::Path;
use std::time::Duration;

const ENDPOINT: &str = "https://catbox.moe/user/api.php";
const MAX_BYTES: u64 = 10 * 1024 * 1024;

fn mime_for(ext: &str) -> Option<&'static str> {
    match ext.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn multipart(boundary: &str, filename: &str, mime: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(data.len() + 512);
    let mut text_part = |name: &str, value: &str| {
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n").as_bytes(),
        );
    };
    text_part("reqtype", "fileupload");

    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"fileToUpload\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

/// Returns the public link and the file's display name.
pub fn upload(path: &Path) -> Result<(String, String), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
    let mime = mime_for(ext).ok_or("Only PNG, JPG, GIF and WebP images can be uploaded")?;

    let size = fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_BYTES {
        return Err("That image is over 10 MB. Pick a smaller one".into());
    }
    let data = fs::read(path).map_err(|e| e.to_string())?;

    // Keep the name plain: it ends up inside a header.
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect::<String>();

    let boundary = format!("----discordrpc{:x}", uuid::Uuid::new_v4().as_u128());
    let body = multipart(&boundary, &name, mime, &data);

    let response = ureq::post(ENDPOINT)
        .set("Content-Type", &format!("multipart/form-data; boundary={boundary}"))
        .timeout(Duration::from_secs(90))
        .send_bytes(&body)
        .map_err(|e| format!("Upload failed: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;

    let url = response.trim();
    if url.starts_with("https://") && !url.contains(char::is_whitespace) {
        Ok((url.to_owned(), name))
    } else {
        Err(format!("The host refused the upload: {}", url.chars().take(120).collect::<String>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipart_has_both_parts() {
        let body = multipart("B", "a.png", "image/png", b"DATA");
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("name=\"reqtype\"\r\n\r\nfileupload"));
        assert!(text.contains("filename=\"a.png\""));
        assert!(text.ends_with("--B--\r\n"));
    }

    #[test]
    fn only_images_are_accepted() {
        assert!(mime_for("PNG").is_some());
        assert!(mime_for("exe").is_none());
    }
}
