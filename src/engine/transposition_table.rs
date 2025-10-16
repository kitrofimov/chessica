use crate::engine::chess_move::Move;
use crate::constants::TRANSPOSITION_TABLE_MB_SIZE;

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

#[derive(Clone)]
pub struct TranspositionTable {
    table: Vec<Option<TTEntry>>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        let num_entries = (TRANSPOSITION_TABLE_MB_SIZE * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        Self {
            table: vec![None; num_entries],
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
}
