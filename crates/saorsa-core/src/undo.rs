//! Undo/redo stack for text editing operations.
//!
//! Provides an [`UndoStack`] that tracks [`EditOperation`] values and
//! supports bounded undo/redo history. Each operation is invertible so
//! that undo can be implemented by applying the inverse.

use crate::cursor::CursorPosition;

/// A single text editing operation that can be undone and redone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditOperation {
    /// Text was inserted at a position.
    Insert {
        /// Position where text was inserted.
        pos: CursorPosition,
        /// The text that was inserted.
        text: String,
    },
    /// Text was deleted from a position.
    Delete {
        /// Position where text was deleted.
        pos: CursorPosition,
        /// The text that was deleted.
        text: String,
    },
    /// Text was replaced at a position.
    Replace {
        /// Position where replacement started.
        pos: CursorPosition,
        /// The old text that was replaced.
        old_text: String,
        /// The new text that replaced it.
        new_text: String,
    },
}

impl EditOperation {
    /// Return the inverse of this operation.
    ///
    /// Applying the inverse undoes the original operation:
    /// - `Insert` → `Delete` (and vice versa)
    /// - `Replace` swaps `old_text` and `new_text`
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert { pos, text } => Self::Delete {
                pos: *pos,
                text: text.clone(),
            },
            Self::Delete { pos, text } => Self::Insert {
                pos: *pos,
                text: text.clone(),
            },
            Self::Replace {
                pos,
                old_text,
                new_text,
            } => Self::Replace {
                pos: *pos,
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
        }
    }
}

/// A bounded undo/redo stack for text editing.
///
/// Stores up to `max_history` operations. When the limit is reached,
/// the oldest operation is dropped. Pushing a new operation clears
/// the redo stack.
#[derive(Clone, Debug)]
pub struct UndoStack {
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
    max_history: usize,
}

impl UndoStack {
    /// Create a new undo stack with the given maximum history size.
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Push a new operation onto the undo stack.
    ///
    /// This clears the redo stack. If the undo stack exceeds
    /// `max_history`, the oldest operation is dropped.
    pub fn push(&mut self, op: EditOperation) {
        self.redo_stack.clear();
        self.undo_stack.push(op);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Pop the most recent operation and return its inverse for undoing.
    ///
    /// The operation is moved to the redo stack.
    pub fn undo(&mut self) -> Option<EditOperation> {
        let op = self.undo_stack.pop()?;
        let inverse = op.inverse();
        self.redo_stack.push(op);
        Some(inverse)
    }

    /// Pop the most recent redo operation and return it for reapplying.
    ///
    /// The operation is moved back to the undo stack.
    pub fn redo(&mut self) -> Option<EditOperation> {
        let op = self.redo_stack.pop()?;
        let result = op.clone();
        self.undo_stack.push(op);
        Some(result)
    }

    /// Returns `true` if there are operations to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are operations to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear both undo and redo stacks.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert_op(line: usize, col: usize, text: &str) -> EditOperation {
        EditOperation::Insert {
            pos: CursorPosition::new(line, col),
            text: text.to_string(),
        }
    }

    fn delete_op(line: usize, col: usize, text: &str) -> EditOperation {
        EditOperation::Delete {
            pos: CursorPosition::new(line, col),
            text: text.to_string(),
        }
    }

    #[test]
    fn push_and_undo() {
        let mut stack = UndoStack::new(100);
        stack.push(insert_op(0, 0, "hello"));
        assert!(stack.can_undo());
        let inv = stack.undo();
        match inv {
            Some(EditOperation::Delete { ref text, .. }) if text == "hello" => {}
            other => unreachable!("expected Delete('hello'), got {other:?}"),
        }
    }

    #[test]
    fn undo_then_redo() {
        let mut stack = UndoStack::new(100);
        stack.push(insert_op(0, 0, "a"));
        let _inv = stack.undo();
        assert!(stack.can_redo());
        let redo = stack.redo();
        match redo {
            Some(EditOperation::Insert { ref text, .. }) if text == "a" => {}
            other => unreachable!("expected Insert('a'), got {other:?}"),
        }
    }

    #[test]
    fn push_clears_redo() {
        let mut stack = UndoStack::new(100);
        stack.push(insert_op(0, 0, "a"));
        let _inv = stack.undo();
        assert!(stack.can_redo());
        stack.push(insert_op(0, 0, "b"));
        assert!(!stack.can_redo());
    }

    #[test]
    fn undo_multiple() {
        let mut stack = UndoStack::new(100);
        stack.push(insert_op(0, 0, "a"));
        stack.push(insert_op(0, 1, "b"));
        stack.push(insert_op(0, 2, "c"));

        // Undo c, b, a
        match stack.undo() {
            Some(EditOperation::Delete { ref text, .. }) if text == "c" => {}
            other => unreachable!("expected Delete('c'), got {other:?}"),
        }
        match stack.undo() {
            Some(EditOperation::Delete { ref text, .. }) if text == "b" => {}
            other => unreachable!("expected Delete('b'), got {other:?}"),
        }
        match stack.undo() {
            Some(EditOperation::Delete { ref text, .. }) if text == "a" => {}
            other => unreachable!("expected Delete('a'), got {other:?}"),
        }
        assert!(!stack.can_undo());
    }

    #[test]
    fn max_history_limit() {
        let mut stack = UndoStack::new(3);
        stack.push(insert_op(0, 0, "a"));
        stack.push(insert_op(0, 1, "b"));
        stack.push(insert_op(0, 2, "c"));
        stack.push(insert_op(0, 3, "d")); // "a" should be dropped

        // Only 3 items remain: b, c, d
        let _d = stack.undo();
        let _c = stack.undo();
        match stack.undo() {
            Some(EditOperation::Delete { ref text, .. }) if text == "b" => {}
            other => unreachable!("expected Delete('b'), got {other:?}"),
        }
        assert!(!stack.can_undo()); // "a" was dropped
    }

    #[test]
    fn inverse_insert() {
        let op = insert_op(1, 5, "hello");
        match op.inverse() {
            EditOperation::Delete { pos, ref text } => {
                assert!(pos == CursorPosition::new(1, 5));
                assert!(text == "hello");
            }
            other => unreachable!("expected Delete, got {other:?}"),
        }
    }

    #[test]
    fn inverse_delete() {
        let op = delete_op(2, 3, "world");
        match op.inverse() {
            EditOperation::Insert { pos, ref text } => {
                assert!(pos == CursorPosition::new(2, 3));
                assert!(text == "world");
            }
            other => unreachable!("expected Insert, got {other:?}"),
        }
    }

    #[test]
    fn inverse_replace() {
        let op = EditOperation::Replace {
            pos: CursorPosition::new(0, 0),
            old_text: "foo".to_string(),
            new_text: "bar".to_string(),
        };
        match op.inverse() {
            EditOperation::Replace {
                ref old_text,
                ref new_text,
                ..
            } => {
                assert!(old_text == "bar");
                assert!(new_text == "foo");
            }
            other => unreachable!("expected Replace, got {other:?}"),
        }
    }

    #[test]
    fn clear_resets_both_stacks() {
        let mut stack = UndoStack::new(100);
        stack.push(insert_op(0, 0, "a"));
        let _inv = stack.undo();
        assert!(stack.can_redo());
        stack.clear();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn empty_stack_undo_redo_returns_none() {
        let mut stack = UndoStack::new(100);
        assert!(stack.undo().is_none());
        assert!(stack.redo().is_none());
    }
}
