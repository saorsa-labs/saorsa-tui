//! Filesystem directory tree widget.
//!
//! Wraps [`Tree<PathBuf>`] with lazy directory loading, file/directory
//! icons, hidden file filtering, and sorted entries (directories first).

use std::path::{Path, PathBuf};

use crate::buffer::ScreenBuffer;
use crate::error::FaeCoreError;
use crate::event::Event;
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;

use super::tree::{Tree, TreeNode};
use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A directory tree widget backed by the filesystem.
///
/// Lazily loads directory contents on expand, with icons for files
/// and directories. Supports hiding dotfiles and sorting (directories
/// first, then files, alphabetically).
pub struct DirectoryTree {
    /// The underlying generic tree widget.
    tree: Tree<PathBuf>,
    /// Whether to show hidden (dot) files.
    show_hidden: bool,
}

impl DirectoryTree {
    /// Create a directory tree rooted at the given path.
    ///
    /// Returns an error if the path does not exist or is not a directory.
    pub fn new(root: PathBuf) -> Result<Self, FaeCoreError> {
        if !root.exists() {
            return Err(FaeCoreError::Widget(format!(
                "path does not exist: {}",
                root.display()
            )));
        }
        if !root.is_dir() {
            return Err(FaeCoreError::Widget(format!(
                "path is not a directory: {}",
                root.display()
            )));
        }

        let root_node = TreeNode::branch(root);
        let show_hidden = false;

        let tree = Tree::new(vec![root_node])
            .with_render_fn(|data: &PathBuf, _depth, _expanded, is_leaf| {
                let name = data
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| data.display().to_string());
                let icon = if is_leaf {
                    "\u{1f4c4}" // 📄
                } else {
                    "\u{1f4c1}" // 📁
                };
                vec![Segment::new(format!("{icon} {name}"))]
            })
            .with_lazy_load(move |path: &PathBuf| load_directory(path, false));

        Ok(Self { tree, show_hidden })
    }

    /// Set whether to show hidden (dot) files.
    #[must_use]
    pub fn with_show_hidden(mut self, enabled: bool) -> Self {
        self.show_hidden = enabled;
        // Rebuild lazy load with updated visibility setting
        let show = self.show_hidden;
        self.tree = self
            .tree
            .with_lazy_load(move |path: &PathBuf| load_directory(path, show));
        self
    }

    /// Set the style for unselected nodes.
    #[must_use]
    pub fn with_node_style(mut self, style: Style) -> Self {
        self.tree = self.tree.with_node_style(style);
        self
    }

    /// Set the style for the selected node.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.tree = self.tree.with_selected_style(style);
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.tree = self.tree.with_border(border);
        self
    }

    /// Get the selected path, if any.
    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.tree.selected_node().map(|node| &node.data)
    }

    /// Toggle expand/collapse at the selection.
    pub fn toggle_selected(&mut self) {
        self.tree.toggle_selected();
    }

    /// Expand the selected node.
    pub fn expand_selected(&mut self) {
        self.tree.expand_selected();
    }

    /// Collapse the selected node.
    pub fn collapse_selected(&mut self) {
        self.tree.collapse_selected();
    }

    /// Get the total number of visible nodes.
    pub fn visible_count(&self) -> usize {
        self.tree.visible_count()
    }

    /// Get whether hidden files are shown.
    pub fn show_hidden(&self) -> bool {
        self.show_hidden
    }
}

impl Widget for DirectoryTree {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        self.tree.render(area, buf);
    }
}

impl InteractiveWidget for DirectoryTree {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.tree.handle_event(event)
    }
}

/// Load directory contents, sorted (directories first, then files, alphabetically).
///
/// Silently skips entries that cannot be read (permission denied, etc.).
fn load_directory(path: &Path, show_hidden: bool) -> Vec<TreeNode<PathBuf>> {
    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Filter hidden files
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        if entry_path.is_dir() {
            dirs.push(entry_path);
        } else {
            files.push(entry_path);
        }
    }

    // Sort alphabetically (case-insensitive)
    dirs.sort_by(|a, b| {
        let a_name = a.file_name().map(|n| n.to_string_lossy().to_lowercase());
        let b_name = b.file_name().map(|n| n.to_string_lossy().to_lowercase());
        a_name.cmp(&b_name)
    });
    files.sort_by(|a, b| {
        let a_name = a.file_name().map(|n| n.to_string_lossy().to_lowercase());
        let b_name = b.file_name().map(|n| n.to_string_lossy().to_lowercase());
        a_name.cmp(&b_name)
    });

    let mut nodes = Vec::with_capacity(dirs.len() + files.len());

    for dir in dirs {
        nodes.push(TreeNode::branch(dir));
    }
    for file in files {
        nodes.push(TreeNode::new(file));
    }

    nodes
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;
    use std::fs;

    /// Create a temporary directory structure for testing.
    fn create_test_dir() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // Create directories
        fs::create_dir_all(root.join("alpha")).unwrap();
        fs::create_dir_all(root.join("beta")).unwrap();
        fs::create_dir_all(root.join("alpha/nested")).unwrap();

        // Create files
        fs::write(root.join("file_a.txt"), "hello").unwrap();
        fs::write(root.join("file_b.txt"), "world").unwrap();
        fs::write(root.join("alpha/child.txt"), "child").unwrap();

        // Create hidden file/dir
        fs::write(root.join(".hidden_file"), "secret").unwrap();
        fs::create_dir_all(root.join(".hidden_dir")).unwrap();

        tmp
    }

    #[test]
    fn create_directory_tree() {
        let tmp = create_test_dir();
        let dt = DirectoryTree::new(tmp.path().to_path_buf());
        assert!(dt.is_ok());
    }

    #[test]
    fn error_on_nonexistent_path() {
        let result = DirectoryTree::new(PathBuf::from("/nonexistent/path/abc123"));
        assert!(result.is_err());
    }

    #[test]
    fn error_on_file_path() {
        let tmp = create_test_dir();
        let result = DirectoryTree::new(tmp.path().join("file_a.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn lazy_load_expand_directory() {
        let tmp = create_test_dir();
        let mut dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();

        // Initially only root is visible
        assert_eq!(dt.visible_count(), 1);

        // Expand root
        dt.expand_selected();
        // root + alpha + beta + file_a.txt + file_b.txt (hidden excluded)
        assert_eq!(dt.visible_count(), 5);
    }

    #[test]
    fn hidden_files_filtered_by_default() {
        let tmp = create_test_dir();
        let mut dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();

        dt.expand_selected();
        // Should NOT contain .hidden_file or .hidden_dir
        // root + alpha + beta + file_a.txt + file_b.txt = 5
        assert_eq!(dt.visible_count(), 5);
    }

    #[test]
    fn show_hidden_files() {
        let tmp = create_test_dir();
        let mut dt = DirectoryTree::new(tmp.path().to_path_buf())
            .unwrap()
            .with_show_hidden(true);

        dt.expand_selected();
        // root + .hidden_dir + alpha + beta + .hidden_file + file_a.txt + file_b.txt = 7
        assert_eq!(dt.visible_count(), 7);
    }

    #[test]
    fn selected_path_retrieval() {
        let tmp = create_test_dir();
        let dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();

        match dt.selected_path() {
            Some(p) => assert_eq!(p, tmp.path()),
            None => unreachable!("should have selected path"),
        }
    }

    #[test]
    fn navigate_and_expand_nested() {
        let tmp = create_test_dir();
        let mut dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();

        dt.expand_selected(); // root

        // Navigate to alpha (should be first child, a dir)
        let down = Event::Key(crate::event::KeyEvent {
            code: crate::event::KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        dt.handle_event(&down); // select alpha

        // Expand alpha
        dt.expand_selected();
        // root, alpha, nested, child.txt, beta, file_a.txt, file_b.txt = 7
        assert_eq!(dt.visible_count(), 7);
    }

    #[test]
    fn empty_directory_expands_to_no_children() {
        let tmp = tempfile::tempdir().unwrap();
        let empty = tmp.path().join("empty");
        fs::create_dir_all(&empty).unwrap();

        let mut dt = DirectoryTree::new(empty).unwrap();
        dt.expand_selected();
        // Just the root, no children
        assert_eq!(dt.visible_count(), 1);
    }

    #[test]
    fn render_directory_tree() {
        let tmp = create_test_dir();
        let dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();

        let mut buf = ScreenBuffer::new(Size::new(40, 10));
        dt.render(Rect::new(0, 0, 40, 10), &mut buf);

        // Should render root with folder icon
        // The render shows "📁 <dirname>" — the icon is multi-byte
        // Just verify something is rendered
        let cell = buf.get(0, 0);
        assert!(cell.is_some());
    }

    #[test]
    fn directories_sorted_before_files() {
        let tmp = create_test_dir();
        let mut dt = DirectoryTree::new(tmp.path().to_path_buf()).unwrap();
        dt.expand_selected();

        // Navigate down to check order: dirs first (alpha, beta), then files (file_a, file_b)
        let down = Event::Key(crate::event::KeyEvent {
            code: crate::event::KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });

        // First child should be alpha (directory)
        dt.handle_event(&down);
        match dt.selected_path() {
            Some(p) => {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string());
                assert_eq!(name.as_deref(), Some("alpha"));
            }
            None => unreachable!("should have selected path"),
        }

        // Second child should be beta (directory)
        dt.handle_event(&down);
        match dt.selected_path() {
            Some(p) => {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string());
                assert_eq!(name.as_deref(), Some("beta"));
            }
            None => unreachable!("should have selected path"),
        }

        // Third child should be file_a.txt (file)
        dt.handle_event(&down);
        match dt.selected_path() {
            Some(p) => {
                let name = p.file_name().map(|n| n.to_string_lossy().to_string());
                assert_eq!(name.as_deref(), Some("file_a.txt"));
            }
            None => unreachable!("should have selected path"),
        }
    }

    #[test]
    fn border_rendering() {
        let tmp = create_test_dir();
        let dt = DirectoryTree::new(tmp.path().to_path_buf())
            .unwrap()
            .with_border(BorderStyle::Single);

        let mut buf = ScreenBuffer::new(Size::new(40, 10));
        dt.render(Rect::new(0, 0, 40, 10), &mut buf);

        // Top-left corner
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
    }
}
