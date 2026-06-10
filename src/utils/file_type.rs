use std::collections::HashMap;

pub fn get_mime_type(extension: &str) -> String {
    mime_guess::from_ext(extension)
        .first_or_octet_stream()
        .to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewType {
    Image,
    Text,
    Pdf,
    Video,
    Audio,
    None,
}

pub fn get_preview_type(extension: &str) -> PreviewType {
    let ext = extension.to_lowercase();
    let image_exts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico"];
    let text_exts = ["txt", "md", "json", "xml", "html", "css", "js", "ts", "py", "rs", "go", "java", "c", "cpp", "h", "sh", "yaml", "yml", "toml", "ini", "cfg", "log", "sql"];
    let pdf_exts = ["pdf"];
    let video_exts = ["mp4", "webm", "mkv", "avi", "mov"];
    let audio_exts = ["mp3", "wav", "ogg", "flac", "aac"];

    if image_exts.contains(&ext.as_str()) {
        PreviewType::Image
    } else if text_exts.contains(&ext.as_str()) {
        PreviewType::Text
    } else if pdf_exts.contains(&ext.as_str()) {
        PreviewType::Pdf
    } else if video_exts.contains(&ext.as_str()) {
        PreviewType::Video
    } else if audio_exts.contains(&ext.as_str()) {
        PreviewType::Audio
    } else {
        PreviewType::None
    }
}
