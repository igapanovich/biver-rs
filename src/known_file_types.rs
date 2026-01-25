const IMAGE_FILE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "psd"];

pub fn is_image(file_extension: &str) -> bool {
    IMAGE_FILE_EXTENSIONS.contains(&file_extension.to_lowercase().as_str())
}
