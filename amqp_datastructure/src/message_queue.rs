// using std::sync::Mutex because it's only temporary anyways
use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter},
    sync::Mutex,
};

/// The data structure behind the message queue.
///
/// Needs to support:
/// * concurrent access
/// * priority
///
/// Currently supports
/// * mutex lol
// todo: see above
pub struct MessageQueue<T> {
    deque: Mutex<VecDeque<T>>,
}

impl<T> MessageQueue<T> {
    pub fn new() -> Self {
        Self {
            deque: Mutex::default(),
        }
    }

    pub fn append(&self, message: T) {
        let mut lock = self.deque.lock().unwrap();
        lock.push_back(message);
    }

    pub fn try_get(&self) -> Option<T> {
        let mut lock = self.deque.lock().unwrap();
        lock.pop_front()
    }

    pub fn len(&self) -> usize {
        self.deque.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for MessageQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for MessageQueue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageQueue").finish_non_exhaustive()
    }
}
