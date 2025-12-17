// Transposition Table implementation

use crate::types::*;

const TT_CLUSTER_SIZE: usize = 3;

#[repr(C, align(32))]
pub struct TTEntry {
    key16: u16,
    move16: u16,
    value16: i16,
    eval16: i16,
    depth8: i8,
    gen_bound8: u8,
}

impl TTEntry {
    const fn new() -> Self {
        TTEntry {
            key16: 0,
            move16: 0,
            value16: 0,
            eval16: 0,
            depth8: 0,
            gen_bound8: 0,
        }
    }

    fn save(&mut self, key: Key, value: Value, eval: Value, depth: Depth, m: Move, bound: Bound, generation: u8) {
        // Always replace if the new entry is from the current generation or has greater depth
        let replace = (generation & 0xFC) != (self.gen_bound8 & 0xFC)
            || depth as i8 + 3 > self.depth8
            || bound == Bound::Exact;

        if replace {
            self.key16 = (key >> 48) as u16;
            self.move16 = m.0;
            self.value16 = value as i16;
            self.eval16 = eval as i16;
            self.depth8 = depth as i8;
            self.gen_bound8 = generation | (bound as u8);
        }
    }

    fn key16(&self) -> u16 {
        self.key16
    }

    pub fn depth(&self) -> Depth {
        self.depth8 as Depth
    }

    pub fn get_move(&self) -> Move {
        Move(self.move16)
    }

    pub fn value(&self) -> Value {
        self.value16 as Value
    }

    fn eval(&self) -> Value {
        self.eval16 as Value
    }

    pub fn bound(&self) -> Bound {
        match self.gen_bound8 & 0x3 {
            0 => Bound::None,
            1 => Bound::Upper,
            2 => Bound::Lower,
            3 => Bound::Exact,
            _ => Bound::None,
        }
    }
}

#[repr(C, align(32))]
#[derive(Clone)]
struct TTCluster {
    entries: [TTEntry; TT_CLUSTER_SIZE],
    padding: [u8; 2], // Pad to 32 bytes
}

impl TTCluster {
    const fn new() -> Self {
        TTCluster {
            entries: [TTEntry::new(); TT_CLUSTER_SIZE],
            padding: [0; 2],
        }
    }
}

pub struct TranspositionTable {
    table: Vec<TTCluster>,
    generation: u8,
}

impl TranspositionTable {
    pub fn new(mb_size: usize) -> Self {
        let cluster_count = (mb_size * 1024 * 1024) / std::mem::size_of::<TTCluster>();
        let cluster_count = cluster_count.max(1);

        TranspositionTable {
            table: vec![TTCluster::new(); cluster_count],
            generation: 8,
        }
    }

    pub fn resize(&mut self, mb_size: usize) {
        let cluster_count = (mb_size * 1024 * 1024) / std::mem::size_of::<TTCluster>();
        let cluster_count = cluster_count.max(1);
        self.table = vec![TTCluster::new(); cluster_count];
        self.clear();
    }

    pub fn clear(&mut self) {
        for cluster in &mut self.table {
            *cluster = TTCluster::new();
        }
        self.generation = 8;
    }

    pub fn new_search(&mut self) {
        self.generation = self.generation.wrapping_add(4);
    }

    #[inline(always)]
    fn first_entry(&self, key: Key) -> &TTCluster {
        let index = ((key as u128 * self.table.len() as u128) >> 64) as usize;
        &self.table[index]
    }

    #[inline(always)]
    fn first_entry_mut(&mut self, key: Key) -> &mut TTCluster {
        let index = ((key as u128 * self.table.len() as u128) >> 64) as usize;
        &mut self.table[index]
    }

    pub fn probe(&self, key: Key) -> Option<TTEntry> {
        let key16 = (key >> 48) as u16;
        let cluster = self.first_entry(key);

        for entry in &cluster.entries {
            if entry.key16() == key16 {
                return Some(*entry);
            }
        }
        None
    }

    pub fn store(&mut self, key: Key, value: Value, eval: Value, depth: Depth, m: Move, bound: Bound) {
        let key16 = (key >> 48) as u16;
        let generation = self.generation;
        let cluster = self.first_entry_mut(key);

        // Find an entry to replace
        let mut replace_idx = 0;
        let mut found = false;

        // First, check if we already have this position
        for (i, entry) in cluster.entries.iter().enumerate() {
            if entry.key16() == key16 {
                replace_idx = i;
                found = true;
                break;
            }
        }

        // If not found, find the entry with lowest depth or oldest generation
        if !found {
            let mut min_score = i32::MAX;
            for (i, entry) in cluster.entries.iter().enumerate() {
                let score = if (entry.gen_bound8 & 0xFC) == (generation & 0xFC) {
                    entry.depth8 as i32
                } else {
                    entry.depth8 as i32 - 256
                };
                if score < min_score {
                    min_score = score;
                    replace_idx = i;
                }
            }
        }

        cluster.entries[replace_idx].save(key, value, eval, depth, m, bound, generation);
    }

    pub fn hashfull(&self) -> usize {
        let mut count = 0;
        let samples = 1000.min(self.table.len());

        for i in 0..samples {
            for entry in &self.table[i].entries {
                if (entry.gen_bound8 & 0xFC) == (self.generation & 0xFC) {
                    count += 1;
                }
            }
        }

        count * 1000 / (samples * TT_CLUSTER_SIZE)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Bound {
    None = 0,
    Upper = 1,
    Lower = 2,
    Exact = 3,
}

impl Copy for TTEntry {}
impl Clone for TTEntry {
    fn clone(&self) -> Self {
        *self
    }
}
