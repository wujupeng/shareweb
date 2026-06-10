use std::path::PathBuf;

pub fn sanitize_path(path: &str, base_dir: &str) -> Result<String, String> {
    let relative = path.trim_start_matches('/');
    let full_path = if relative.is_empty() {
        PathBuf::from(base_dir)
    } else {
        let mut base = PathBuf::from(base_dir);
        base.push(relative);
        base
    };

    let mut components = Vec::new();
    for comp in full_path.components() {
        match comp {
            std::path::Component::ParentDir => {
                if components.pop().is_none() {
                    return Err("路径遍历攻击: 路径超出共享目录范围".to_string());
                }
            }
            std::path::Component::CurDir => {}
            std::path::Component::Normal(c) => {
                components.push(c.to_string_lossy().to_string());
            }
            std::path::Component::RootDir => {}
            _ => {}
        }
    }

    let mut result = PathBuf::from("/");
    for comp in &components {
        result.push(comp);
    }

    let result_str = result.to_string_lossy().to_string();
    let base_canonical = PathBuf::from(base_dir)
        .canonicalize()
        .map_err(|e| format!("基础目录规范化失败: {}", e))?;

    let base_str = base_canonical.to_string_lossy().to_string();

    if !result_str.starts_with(&base_str) && result_str != base_str.trim_end_matches('/') {
        if !result_str.starts_with(&format!("{}/", base_str)) && result_str != base_str {
            return Err("路径遍历攻击: 路径超出共享目录范围".to_string());
        }
    }

    Ok(result_str)
}

pub fn validate_filename(name: &str) -> Result<(), String> {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for ch in invalid_chars {
        if name.contains(ch) {
            return Err(format!("文件名包含非法字符: {}", ch));
        }
    }
    Ok(())
}
