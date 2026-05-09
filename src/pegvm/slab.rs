//! Minimal slab allocator: stable `u32` indices, LIFO free-list,
//! `Vec`-backed contiguous storage. The VM uses this for the branched
//! capture arena (`vm::Branch`) where we need stable IDs that survive
//! across pushes plus explicit reclamation on `fail()`.
//!
//! Not exported. Inline rather than depending on the `slab` crate so
//! the project keeps its zero-deps property (see `README.md`).
//!
//! Capacity is `u32`-bounded; `len` is also `u32` for symmetry. The
//! VM's expected slab population (per-Choice branches) is in the
//! 100k-1M range for the largest realistic inputs, so 32 bits is
//! plenty and keeps `BranchId` cache-friendly.

#[derive(Debug)]
enum Slot<T> {
    Occupied(T),
    Vacant { next_free: u32 },
}

const NO_FREE: u32 = u32::MAX;

#[derive(Debug)]
pub(crate) struct Slab<T> {
    slots: Vec<Slot<T>>,
    free_head: u32,
    len: u32,
}

impl<T> Slab<T> {
    pub fn new() -> Self {
        Slab {
            slots: Vec::new(),
            free_head: NO_FREE,
            len: 0,
        }
    }

    /// Live entry count. Diagnostic, not load-bearing — the VM keeps
    /// branches alive via the slab itself plus parent-pointer reachability.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn insert(&mut self, value: T) -> u32 {
        if self.free_head != NO_FREE {
            let idx = self.free_head;
            let slot = &mut self.slots[idx as usize];
            match *slot {
                Slot::Vacant { next_free } => self.free_head = next_free,
                Slot::Occupied(_) => unreachable!("free_head pointed at occupied slot"),
            }
            *slot = Slot::Occupied(value);
            self.len += 1;
            idx
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(Slot::Occupied(value));
            self.len += 1;
            idx
        }
    }

    pub fn remove(&mut self, idx: u32) -> T {
        let slot = &mut self.slots[idx as usize];
        match std::mem::replace(
            slot,
            Slot::Vacant {
                next_free: self.free_head,
            },
        ) {
            Slot::Occupied(value) => {
                self.free_head = idx;
                self.len -= 1;
                value
            }
            Slot::Vacant { next_free } => {
                // Restore the slot before panicking; double-remove is a
                // bug but we don't want to corrupt the free list on top
                // of the bug.
                *slot = Slot::Vacant { next_free };
                panic!("Slab::remove: slot {} is already vacant", idx);
            }
        }
    }

    pub fn get(&self, idx: u32) -> &T {
        match &self.slots[idx as usize] {
            Slot::Occupied(v) => v,
            Slot::Vacant { .. } => panic!("Slab::get: slot {} is vacant", idx),
        }
    }

    pub fn get_mut(&mut self, idx: u32) -> &mut T {
        match &mut self.slots[idx as usize] {
            Slot::Occupied(v) => v,
            Slot::Vacant { .. } => panic!("Slab::get_mut: slot {} is vacant", idx),
        }
    }
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_returns_sequential_indices() {
        let mut s: Slab<u32> = Slab::new();
        assert_eq!(s.insert(10), 0);
        assert_eq!(s.insert(20), 1);
        assert_eq!(s.insert(30), 2);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn remove_recycles_indices_lifo() {
        let mut s: Slab<u32> = Slab::new();
        let a = s.insert(10);
        let b = s.insert(20);
        let c = s.insert(30);
        s.remove(a);
        s.remove(b);
        s.remove(c);
        // Most recently freed slot comes back first.
        assert_eq!(s.insert(99), c);
        assert_eq!(s.insert(99), b);
        assert_eq!(s.insert(99), a);
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn remove_returns_owned_value() {
        let mut s: Slab<String> = Slab::new();
        let a = s.insert("hello".to_string());
        let v = s.remove(a);
        assert_eq!(v, "hello");
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn get_and_get_mut_round_trip() {
        let mut s: Slab<u32> = Slab::new();
        let a = s.insert(10);
        assert_eq!(s.get(a), &10);
        *s.get_mut(a) = 42;
        assert_eq!(s.get(a), &42);
    }

    #[test]
    fn other_slots_keep_values_after_remove() {
        let mut s: Slab<u32> = Slab::new();
        let a = s.insert(10);
        let b = s.insert(20);
        let c = s.insert(30);
        s.remove(b);
        assert_eq!(s.get(a), &10);
        assert_eq!(s.get(c), &30);
        assert_eq!(s.len(), 2);
        // Reused slot doesn't disturb neighbours.
        let d = s.insert(40);
        assert_eq!(d, b);
        assert_eq!(s.get(a), &10);
        assert_eq!(s.get(c), &30);
        assert_eq!(s.get(d), &40);
    }

    #[test]
    #[should_panic(expected = "vacant")]
    fn double_remove_panics() {
        let mut s: Slab<u32> = Slab::new();
        let a = s.insert(0);
        s.remove(a);
        s.remove(a);
    }

    #[test]
    #[should_panic(expected = "vacant")]
    fn get_after_remove_panics() {
        let mut s: Slab<u32> = Slab::new();
        let a = s.insert(0);
        s.remove(a);
        let _ = s.get(a);
    }
}
