pub struct UpdateBitmask {
    pub bits: Vec<u16>,
    pub any: bool,
}

impl UpdateBitmask {
    #[inline]
    pub fn new(size: usize, default: bool) -> Self {
        let default_val = if default { u16::MAX } else { 0 };
        let bits = vec![default_val; (size + 15) >> 4]; // (size + 7) / 8
        UpdateBitmask { bits, any: default }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bits.fill(0);
        self.any = false;
    }

    #[inline]
    pub fn set_all(&mut self) {
        self.bits.fill(u16::MAX);
        self.any = true;
    }

    #[inline]
    pub fn any_set(&self) -> bool {
        self.any
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let bit_index = index % 16;
        let u16_index = index >> 4; // index / 16
        (self.bits[u16_index] & (0b1000000000000000 >> bit_index)) != 0
    }

    #[inline]
    pub fn set_one(&mut self, index: usize) {
        let bit_index = index % 16;
        let u16_index = index >> 4; // index / 16
        self.bits[u16_index] |= 0b1000000000000000 >> bit_index;
        self.any = true;
    }

    #[inline]
    pub fn set(&mut self, range: std::ops::Range<usize>) {
        for i in range {
            self.set_one(i)
        }
    }
}

// Slow, would need to recompute any
//#[inline]
//pub fn unset(&mut self, index: usize) {
//    let bit_index = index % 64;
//    let u64_index = index / 64;
//    self.bits[u64_index] &= !(1 << bit_index);
//    // Recompute any
//}
