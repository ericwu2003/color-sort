use std::hash;
use std::{fmt, mem::MaybeUninit};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Move {
    pub from: u8,
    pub to: u8,
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Move from {} to {}", self.from, self.to)?;
        Ok(())
    }
}

pub const MOVE_HISTORY_CAPACITY: usize = 40;

pub struct MoveHistory {
    pub moves: [MaybeUninit<Move>; MOVE_HISTORY_CAPACITY],
    pub size: u8,
}

impl hash::Hash for MoveHistory {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        for i in 0..self.size {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                self.moves[i as usize].assume_init().hash(state);
            }
        }
    }
}

impl PartialEq for MoveHistory {
    fn eq(&self, other: &Self) -> bool {
        if self.size != other.size {
            return false;
        }

        for i in 0..self.size {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                if self.moves[i as usize].assume_init() != other.moves[i as usize].assume_init() {
                    return false;
                }
            }
        }
        return true;
    }
}

impl Clone for MoveHistory {
    fn clone(&self) -> Self {
        let mut result = MoveHistory {
            moves: [MaybeUninit::uninit(); MOVE_HISTORY_CAPACITY],
            size: self.size,
        };
        for i in 0..self.size {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                result.moves[i as usize].write(self.moves[i as usize].assume_init_ref().clone());
            }
        }

        return result;
    }
}

impl Drop for MoveHistory {
    fn drop(&mut self) {
        for i in 0..self.size {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                self.moves[i as usize].assume_init_drop();
            }
        }
    }
}

impl MoveHistory {
    pub fn new() -> Self {
        return MoveHistory {
            moves: unsafe { MaybeUninit::uninit().assume_init() },
            size: 0,
        };
    }

    pub fn push(&mut self, m: Move) {
        // this function panics if we exceed the capacity of MOVE_HISTORY_CAPACITY
        self.moves[self.size as usize].write(m);
        self.size += 1;
    }

    pub fn get(&self, index: usize) -> Option<&Move> {
        if index >= self.size as usize {
            return None;
        } else {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                return Some(&self.moves[index].assume_init_ref());
            }
        }
    }

    pub fn len(&self) -> usize {
        return self.size as usize;
    }
}

impl fmt::Debug for MoveHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} moves:", self.size)?;

        for i in 0..self.size {
            unsafe {
                // safety: the MoveHistory type will ensure that the first size elements are initialized.
                writeln!(f, "  {:?}", self.moves[i as usize].assume_init())?;
            }
        }

        Ok(())
    }
}
