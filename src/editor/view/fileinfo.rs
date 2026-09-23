use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct FileInfo {
    pub path: Option<PathBuf>,
}

impl FileInfo {
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub const fn has_path(&self) -> bool {
        self.path.is_some()
    }
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
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("[No Name]");

        write!(f, "{name}")
    }
}
