pub struct Queue<T: Copy + Default, const N: usize> {
    buf: [T; N],
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

impl<T: Copy + Default, const N: usize> Default for Queue<T, N> {
    fn default() -> Self {
        Self {
            buf: [T::default(); N],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }
}

impl<T: Copy + Default, const N: usize> Queue<T, N> {
    pub const fn new(initial_val: T) -> Self {
        Self {
            buf: [initial_val; N],
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub fn is_full(&self) -> bool {
        self.count == N
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let val = self.buf[self.read_pos];
        self.read_pos = (self.read_pos + 1) % N;
        self.count -= 1;
        Some(val)
    }

    pub fn push(&mut self, val: T) -> Result<(), T> {
        if self.is_full() {
            return Err(val);
        }
        self.buf[self.write_pos] = val;
        self.write_pos = (self.write_pos + 1) % N;
        self.count += 1;
        Ok(())
    }
}
