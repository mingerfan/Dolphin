use thiserror::Error;

#[derive(Debug, Error)]
pub enum RingBufferError {
    #[error("Buffer is full")]
    Full,
    #[error("Buffer is empty")]
    Empty,
}

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buf: Vec<T>,
    read: usize,
    write: usize,
    full: bool,
}

impl<T: Copy + Default> RingBuffer<T> {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            buf: vec![T::default(); size],
            read: 0,
            write: 0,
            full: false,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), RingBufferError> {
        if self.full {
            return Err(RingBufferError::Full);
        }
        self.buf[self.write] = item;
        self.write = (self.write + 1) % self.buf.len();
        if self.write == self.read {
            self.full = true;
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Result<T, RingBufferError> {
        if self.is_empty() {
            return Err(RingBufferError::Empty);
        }
        let item = self.buf[self.read];
        self.read = (self.read + 1) % self.buf.len();
        self.full = false;
        Ok(item)
    }

    pub fn push_overwrite(&mut self, item: T) {
        self.buf[self.write] = item;
        self.write = (self.write + 1) % self.buf.len();
        if self.full {
            self.read = self.write;
        } else if self.write == self.read {
            self.full = true;
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.full && self.read == self.write
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn len(&self) -> usize {
        if self.full {
            self.buf.len()
        } else {
            (self.write + self.buf.len() - self.read) % self.buf.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rb: RingBuffer<u8> = RingBuffer::new(10);
        assert_eq!(rb.capacity(), 10);
        assert!(rb.is_empty());
        assert!(!rb.is_full());
    }

    #[test]
    fn test_push_and_pop() {
        let mut rb = RingBuffer::new(3);
        assert!(rb.push(1).is_ok());
        assert!(rb.push(2).is_ok());
        assert!(rb.push(3).is_ok());
        assert!(rb.is_full());

        match rb.push(4) {
            Err(RingBufferError::Full) => (),
            _ => panic!("Expected Full error"),
        }

        assert_eq!(rb.pop().unwrap(), 1);
        assert_eq!(rb.pop().unwrap(), 2);
        assert_eq!(rb.pop().unwrap(), 3);
        assert!(rb.is_empty());

        match rb.pop() {
            Err(RingBufferError::Empty) => (),
            _ => panic!("Expected Empty error"),
        }
    }

    #[test]
    fn test_overwrite() {
        let mut rb = RingBuffer::new(2);
        assert!(rb.push(1).is_ok());
        assert!(rb.push(2).is_ok());
        assert!(rb.is_full());

        assert_eq!(rb.pop().unwrap(), 1);
        assert!(!rb.is_full());
        assert!(rb.push(3).is_ok());
        assert!(rb.is_full());

        assert_eq!(rb.pop().unwrap(), 2);
        assert_eq!(rb.pop().unwrap(), 3);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_push_overwrite() {
        let mut rb = RingBuffer::new(3);
        rb.push_overwrite(1);
        rb.push_overwrite(2);
        rb.push_overwrite(3);
        assert!(rb.is_full());
        rb.push_overwrite(4);
        assert!(rb.is_full());
        assert_eq!(rb.pop().unwrap(), 2);
        assert_eq!(rb.pop().unwrap(), 3);
        assert_eq!(rb.pop().unwrap(), 4);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_len() {
        let mut rb = RingBuffer::new(5);
        assert_eq!(rb.len(), 0);
        rb.push(1).unwrap();
        assert_eq!(rb.len(), 1);
        rb.push(2).unwrap();
        assert_eq!(rb.len(), 2);
        rb.pop().unwrap();
        assert_eq!(rb.len(), 1);
        rb.push(3).unwrap();
        rb.push(4).unwrap();
        rb.push(5).unwrap();
        rb.push(6).unwrap();
        assert_eq!(rb.len(), 5);
        assert!(rb.is_full());
        rb.pop().unwrap();
        assert_eq!(rb.len(), 4);
    }
}
