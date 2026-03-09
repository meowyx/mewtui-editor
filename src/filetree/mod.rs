use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct FileTree {
    pub root: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        let entries = read_dir_sorted(&root, 0);
        FileTree {
            root,
            entries,
            selected: 0,
        }
    }

    pub fn refresh(&mut self) {
        let expanded_paths: Vec<PathBuf> = self
            .entries
            .iter()
            .filter(|e| e.is_dir && e.expanded)
            .map(|e| e.path.clone())
            .collect();

        self.entries = build_tree(&self.root, 0, &expanded_paths);

        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.selected) {
            if entry.is_dir {
                entry.expanded = !entry.expanded;
                self.refresh();
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }
}

fn read_dir_sorted(path: &Path, depth: usize) -> Vec<FileEntry> {
    let Ok(read_dir) = fs::read_dir(path) else {
        return Vec::new();
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in read_dir.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden files
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let is_dir = path.is_dir();

        let file_entry = FileEntry {
            name,
            path,
            is_dir,
            depth,
            expanded: false,
        };

        if is_dir {
            dirs.push(file_entry);
        } else {
            files.push(file_entry);
        }
    }

    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    dirs.extend(files);
    dirs
}

fn build_tree(path: &Path, depth: usize, expanded: &[PathBuf]) -> Vec<FileEntry> {
    let mut result = Vec::new();
    let children = read_dir_sorted(path, depth);

    for mut entry in children {
        if entry.is_dir && expanded.contains(&entry.path) {
            entry.expanded = true;
            result.push(entry.clone());
            let subtree = build_tree(&entry.path, depth + 1, expanded);
            result.extend(subtree);
        } else {
            result.push(entry);
        }
    }

    result
}
