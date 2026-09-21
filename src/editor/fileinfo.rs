use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Default)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
}

impl<T: AsRef<Path>> From<T> for FileInfo {
    fn from(value: T) -> Self {
        Self {
            path: Some(value.as_ref().to_path_buf()),
        }
    }
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("[No Name]");

        write!(f, "{name}")
    }
}
