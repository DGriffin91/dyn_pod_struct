pub struct UpdateBitmask {
    pub bits: Vec<u8>,
    pub any: bool,
}

impl UpdateBitmask {
    #[inline]
    pub fn new(size: usize, default: bool) -> Self {
        let default_val = if default { u8::MAX } else { 0 };
        let bits = vec![default_val; (size + 7) >> 3]; // (size + 7) / 8
        UpdateBitmask { bits, any: default }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bits.fill(0);
        self.any = false;
    }

    #[inline]
    pub fn set_all(&mut self) {
        self.bits.fill(u8::MAX);
        self.any = true;
    }

    #[inline]
    pub fn any_set(&self) -> bool {
        self.any
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let bit_index = index % 8;
        let u8_index = index >> 3; // index / 8
        (self.bits[u8_index] & (1 << bit_index)) != 0
    }

    #[inline]
    pub fn set_one(&mut self, index: usize) {
        let bit_index = index % 8;
        let u8_index = index >> 3; // index / 8
        self.bits[u8_index] |= 1 << bit_index;
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
