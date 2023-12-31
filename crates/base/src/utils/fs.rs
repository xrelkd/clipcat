use std::path::{Path, PathBuf};

pub fn read_dir_recursively<P>(dir_path: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let dir_path = dir_path.as_ref().to_path_buf();
    let mut files = Vec::new();
    let mut stack = vec![dir_path.clone()];
    while let Some(current_entry) = stack.pop() {
        if current_entry.is_file() {
            files.push(current_entry);
        } else if current_entry.is_dir() {
            if let Ok(dir_entry) = current_entry.read_dir() {
                for entry in dir_entry.flatten() {
                    stack.push([&dir_path, &entry.path()].iter().collect());
                }
            }
        }
    }
    files.sort_unstable();
    files
}

pub async fn read_dir_recursively_async<P>(dir_path: P) -> Vec<PathBuf>
where
    P: AsRef<Path> + Send,
{
    let dir_path = dir_path.as_ref().to_path_buf();
    tokio::task::spawn_blocking(move || read_dir_recursively(dir_path)).await.unwrap_or_default()
}
