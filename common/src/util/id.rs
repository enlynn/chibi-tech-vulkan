use std::collections::VecDeque;

pub type IdType         = u64;
pub type IndexType      = u64;
pub type GenerationType = u16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Id(pub IdType);

pub const GEN_BITS: u64 = 16;
pub const IDX_BITS: u64 = (std::mem::size_of::<IdType>() as u64 * 8) - GEN_BITS;

const IDX_MASK: IndexType      = ((1u64 << IDX_BITS) - 1) as IndexType;
const GEN_MASK: GenerationType = ((1u64 << GEN_BITS) - 1) as GenerationType;

const MIN_FREE_INDICES: usize = 10;

pub const INVALID_ID: Id = Id(std::u64::MAX);

pub enum IdError {
    OutOfSpace,
}

pub struct IdSystem {
    gens:         Vec<GenerationType>,
    free_indices: VecDeque<IndexType>,
}

impl Id {
    #[inline(always)]
    pub fn new(index: IndexType, gen: GenerationType) -> Self {
        return Self(index as IdType | ((gen as IdType) << IDX_BITS) as IdType);
    }

    #[inline(always)]
    pub fn get_index(&self) -> IndexType {
        return self.0 & IDX_MASK;
    }

    #[inline(always)]
    pub fn get_generation(&self) -> GenerationType {
        return (self.0 >> IDX_BITS) as GenerationType & GEN_MASK;
    }

    #[inline(always)]
    pub fn set_index(&mut self, idx: IndexType) -> &mut Self {
        self.0 = idx as IdType | ((self.get_generation() as IdType) << IDX_BITS);
        self
    }

    #[inline(always)]
    pub fn set_generation(&mut self, gen: GenerationType) -> &mut Self {
        self.0 = self.get_index() as IdType | ((gen as IdType) << IDX_BITS) as IdType;
        self
    }

    #[inline(always)]
    pub fn inc_generation(&mut self) -> &mut Self {
        self.set_generation(self.get_generation() + 1)
    }
}

impl IdSystem {
    pub fn new(max_ids: usize) -> Self {
        Self {
            gens:         Vec::with_capacity(max_ids),
            free_indices: VecDeque::new(),
        }
    }

    #[inline(always)]
    fn is_live_generation(gen: GenerationType) -> bool {
        // Odd-Numbered Generations are considered Live handles
        return (gen & 1) == 1;
    }

    #[inline(always)]
    fn is_free_generation(gen: GenerationType) -> bool {
        // Even-Numbered Generations are considered Live handles
        return (gen & 1) == 0;
    }

    #[inline(always)]
    fn mark_generation_as_alive(gen: GenerationType) -> GenerationType {
        if IdSystem::is_free_generation(gen) {
            return gen + 1;
        }

        assert!(false, "Tried to mark an alive generation as alive");
        return gen;
    }

    #[inline(always)]
    fn mark_generation_as_free(gen: GenerationType) -> GenerationType {
        if IdSystem::is_live_generation(gen) {
            return gen + 1;
        }

        assert!(false, "Tried to mark a freed generation as free");
        return gen;
    }

    pub fn is_id_valid(&self, id: Id) -> bool {
        let index = id.get_index();
        let gen   = id.get_generation();
        return (index as usize) < self.gens.len() && IdSystem::is_live_generation(gen) && self.gens[index as usize] == gen;
    }

    pub fn alloc_id(&mut self) -> Result<Id, IdError> {
        let new_len = self.gens.len() + 1;
        let result = if self.free_indices.len() > MIN_FREE_INDICES || new_len >= self.gens.capacity() {
            let index = match self.free_indices.pop_front() {
                Some(i) => i,
                None    => return Err(IdError::OutOfSpace),
            };

            assert!((index as usize) < self.gens.len());

            self.gens[index as usize] = IdSystem::mark_generation_as_alive(self.gens[index as usize]);
            let gen = self.gens[index as usize];

            Id::new(index, gen)
        } else {
            let index = self.gens.len() as IndexType;
            let gen   = IdSystem::mark_generation_as_alive(0);
            match self.gens.push_within_capacity(gen) {
                Ok(_)  => {},
                Err(_) => return Err(IdError::OutOfSpace),
            }

            Id::new(index, gen)
        };

        assert!(self.is_id_valid(result));

        return Ok(result);
    }

    pub fn free_id(&mut self, id: Id) {
        if self.is_id_valid(id) {
            let index = id.get_index();
            let gen   = id.get_generation();

            self.gens[index as usize] = IdSystem::mark_generation_as_free(gen);
            self.free_indices.push_back(index);
        }
    }
}

#[cfg(test)]
mod id_tests {
    use super::*;

    #[test]
    fn id_compose() {
        println!("Hellow world");

        let mut index = 0;
        let mut gen   = 0;
        let mut id = Id::new(index, gen);

        assert_eq!(id.get_index(),      index);
        assert_eq!(id.get_generation(), gen);
        assert_eq!(IdSystem::is_free_generation(gen), true);
        assert_eq!(IdSystem::is_live_generation(gen), false);

        index = 1;
        gen   = 0;
        id    = Id::new(index, gen);

        assert_eq!(id.get_index(),      index);
        assert_eq!(id.get_generation(), gen);
        assert_eq!(IdSystem::is_free_generation(gen), true);
        assert_eq!(IdSystem::is_live_generation(gen), false);

        index = 1;
        gen   = 1;
        id    = Id::new(index, gen);

        assert_eq!(id.get_index(),      index);
        assert_eq!(id.get_generation(), gen);
        assert_eq!(IdSystem::is_free_generation(gen), false);
        assert_eq!(IdSystem::is_live_generation(gen), true);

        index = 10;
        gen   = 2;
        id    = Id::new(index, gen);

        assert_eq!(id.get_index(),      index);
        assert_eq!(id.get_generation(), gen);
        assert_eq!(IdSystem::is_free_generation(gen), true);
        assert_eq!(IdSystem::is_live_generation(gen), false);
    }
}
