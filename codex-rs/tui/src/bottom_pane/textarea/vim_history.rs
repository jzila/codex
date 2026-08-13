use std::collections::VecDeque;

use super::TextElement;

const VIM_HISTORY_LIMIT: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VimEditSnapshot {
    pub(super) text: String,
    pub(super) cursor_pos: usize,
    pub(super) elements: Vec<TextElement>,
    pub(super) next_element_id: u64,
}

impl VimEditSnapshot {
    pub(super) fn has_same_content(&self, other: &Self) -> bool {
        self.text == other.text
            && self.elements == other.elements
            && self.next_element_id == other.next_element_id
    }
}

#[derive(Debug)]
pub(crate) struct VimHistory<T> {
    undo: VecDeque<T>,
    redo: VecDeque<T>,
    insert_start: Option<T>,
}

impl<T> Default for VimHistory<T> {
    fn default() -> Self {
        Self {
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            insert_start: None,
        }
    }
}

impl<T: Clone> VimHistory<T> {
    pub(crate) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.insert_start = None;
    }

    pub(crate) fn begin_insert(&mut self, snapshot: T) -> bool {
        if self.insert_start.is_some() {
            return false;
        }
        self.insert_start = Some(snapshot);
        true
    }

    pub(crate) fn insert_start(&self) -> Option<&T> {
        self.insert_start.as_ref()
    }

    pub(crate) fn finish_insert(&mut self, changed: bool) -> bool {
        let Some(start) = self.insert_start.take() else {
            return false;
        };
        if !changed {
            return false;
        }
        self.record_edit(start);
        true
    }

    pub(crate) fn record_edit(&mut self, before: T) {
        self.push_undo(before);
        self.redo.clear();
    }

    pub(crate) fn undo(&mut self, mut current: T, count: usize) -> (Option<T>, usize) {
        let mut restored = None;
        let mut steps = 0;
        for _ in 0..count {
            let Some(previous) = self.undo.pop_back() else {
                break;
            };
            push_bounded(&mut self.redo, current);
            current = previous.clone();
            restored = Some(previous);
            steps += 1;
        }
        (restored, steps)
    }

    pub(crate) fn redo(&mut self, mut current: T, count: usize) -> (Option<T>, usize) {
        let mut restored = None;
        let mut steps = 0;
        for _ in 0..count {
            let Some(next) = self.redo.pop_back() else {
                break;
            };
            self.push_undo(current);
            current = next.clone();
            restored = Some(next);
            steps += 1;
        }
        (restored, steps)
    }

    fn push_undo(&mut self, snapshot: T) {
        push_bounded(&mut self.undo, snapshot);
    }
}

fn push_bounded<T>(history: &mut VecDeque<T>, snapshot: T) {
    if history.len() == VIM_HISTORY_LIMIT {
        history.pop_front();
    }
    history.push_back(snapshot);
}
