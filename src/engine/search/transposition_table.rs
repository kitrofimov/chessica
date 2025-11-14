use crate::engine::base::_move::Move;
use crate::constants::DEFAULT_HASH_MB_SIZE;

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub zobrist: u64,
    pub depth: u8,
    pub eval: i32,
    pub flag: NodeType,
    pub best_move: Option<Move>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

/// Caches previously computed positions
#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
    used: usize,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(DEFAULT_HASH_MB_SIZE)
    }
}

impl TranspositionTable {
    pub fn new(mb_size: usize) -> Self {
        let num_entries = (mb_size * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        Self {
            table: vec![None; num_entries],
            used: 0,
        }
    }

    fn get_idx(&self, zobrist: u64) -> usize {
        (zobrist % self.table.len() as u64) as usize
    }

    pub fn insert(&mut self, entry: TTEntry) {
        let idx = self.get_idx(entry.zobrist);
        if let Some(old) = &self.table[idx] {
            // If it is the same position, but the old entry is deeper, do not replace
            if old.zobrist == entry.zobrist && old.depth > entry.depth {
                return;
            }
        }
        if self.table[idx].is_none() {
            self.used += 1;
        }
        self.table[idx] = Some(entry);
    }

    pub fn probe(&self, zobrist: u64) -> Option<TTEntry> {
        let idx = self.get_idx(zobrist);
        if let Some(entry) = &self.table[idx] {
            // Assure no collisions
            if entry.zobrist == zobrist {
                return Some(*entry);
            }
        }
        None
    }

    pub fn hashfull(&self) -> u8 {
        ((self.used * 100) / self.table.len()) as u8
    }
}
