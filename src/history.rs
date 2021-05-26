#[derive(Clone)]
pub struct History<T> {
    stack: Vec<T>,
    pointer: usize,
}

impl<T: std::fmt::Debug> std::fmt::Debug for History<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
            .field("stack", &self.stack)
            .field("pointer", &self.pointer)
            .finish()
    }
}

impl<T: Default> Default for History<T> {
    fn default() -> Self {
        Self {
            stack: vec![T::default()],
            pointer: 0,
        }
    }
}

impl<T: Clone> History<T> {
    pub fn new(initial_state: T) -> Self {
        Self {
            stack: vec![initial_state],
            pointer: 0,
        }
    }

    pub fn reset(&mut self, new_state: T) {
        self.stack = vec![new_state];
        self.pointer = 0;
    }

    pub fn position(&self) -> usize {
        self.pointer
    }

    pub fn peek_prev(&self) -> Option<(&T, usize)> {
        if self.pointer == 0 {
            return None;
        }

        let prev_pointer = self.pointer - 1;

        let history = self.stack.get(prev_pointer).unwrap_or_else(|| {
            panic!(
                "Unable to fetch previous history for history_pointer: {}. History items: {}",
                self.pointer,
                self.stack.len()
            )
        });

        Some((history, prev_pointer))
    }

    pub fn peek_next(&self) -> Option<(&T, usize)> {
        if self.pointer >= self.stack.len() - 1 {
            return None;
        }

        let next_pointer = self.pointer + 1;

        let history = self.stack.get(next_pointer).unwrap_or_else(|| {
            panic!(
                "Unable to fetch next history for history_pointer: {}. History items: {}",
                self.pointer,
                self.stack.len()
            )
        });

        Some((history, next_pointer))
    }

    pub fn peek_current(&self) -> (&T, usize) {
        let history = self.stack.get(self.pointer).unwrap_or_else(|| {
            panic!(
                "Unable to fetch current history for history_pointer: {}. History items: {}",
                self.pointer,
                self.stack.len()
            )
        });

        (history, self.pointer)
    }

    pub fn push(&mut self, item: T) -> usize {
        if self.pointer < self.stack.len() - 1 {
            self.stack.drain((self.pointer + 1)..self.stack.len());
        }

        self.stack.push(item);

        self.pointer += 1;

        self.pointer
    }

    pub fn undo(&mut self) -> Option<(&T, usize)> {
        if self.pointer == 0 {
            return None;
        }

        let item = self.stack.get(self.pointer - 1).unwrap_or_else(|| {
            panic!(
                "Unable to undo history for history_pointer: {}. History items: {}",
                self.pointer,
                self.stack.len()
            )
        });
        self.pointer -= 1;
        Some((item, self.pointer))
    }

    pub fn redo(&mut self) -> Option<(&T, usize)> {
        if self.pointer >= self.stack.len() - 1 {
            return None;
        }

        let item = self.stack.get(self.pointer + 1).unwrap_or_else(|| {
            panic!(
                "Unable to redo history for history_pointer: {}. History items: {}",
                self.pointer,
                self.stack.len()
            )
        });
        self.pointer += 1;
        Some((item, self.pointer))
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod test {
    use super::History;

    #[test]
    fn test() {
        let mut history = History::new(0);
        assert_eq!((&0, 0), history.peek_current());
        assert_eq!(None, history.peek_prev());
        assert_eq!(None, history.peek_next());
        assert_eq!(1, history.len());

        assert_eq!(None, history.undo());

        assert_eq!(1, history.push(5));
        assert_eq!((&5, 1), history.peek_current());
        assert_eq!(Some((&0, 0)), history.peek_prev());
        assert_eq!(None, history.peek_next());
        assert_eq!(2, history.len());

        assert_eq!(2, history.push(10));
        assert_eq!((&10, 2), history.peek_current());
        assert_eq!(Some((&5, 1)), history.peek_prev());
        assert_eq!(None, history.peek_next());
        assert_eq!(3, history.len());

        assert_eq!(Some((&5, 1)), history.undo());
        assert_eq!((&5, 1), history.peek_current());
        assert_eq!(Some((&0, 0)), history.peek_prev());
        assert_eq!(Some((&10, 2)), history.peek_next());
        assert_eq!(3, history.len());

        // Overwrites the previous third entry
        assert_eq!(2, history.push(7));
        assert_eq!((&7, 2), history.peek_current());
        assert_eq!(Some((&5, 1)), history.peek_prev());
        assert_eq!(None, history.peek_next());
        assert_eq!(3, history.len());

        assert_eq!(Some((&5, 1)), history.undo());
        assert_eq!((&5, 1), history.peek_current());
        assert_eq!(Some((&0, 0)), history.peek_prev());
        assert_eq!(Some((&7, 2)), history.peek_next());
        assert_eq!(3, history.len());

        assert_eq!(Some((&7, 2)), history.redo());
        assert_eq!((&7, 2), history.peek_current());
        assert_eq!(Some((&5, 1)), history.peek_prev());
        assert_eq!(None, history.peek_next());
        assert_eq!(3, history.len());

        assert_eq!(None, history.redo());
    }
}
