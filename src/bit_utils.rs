use std::collections::HashSet;

pub struct HotBitIter {
    bits: u32,
}

impl From<u32> for HotBitIter {
    fn from(value: u32) -> Self {
        HotBitIter { bits: value }
    }
}

impl Iterator for HotBitIter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits != 0 {
            let unit = 1 << self.bits.trailing_zeros();
            self.bits &= !unit;
            Some(unit)
        } else {
            None
        }
    }
}

pub trait Canonicalizable: Copy + Ord {
    fn reverse(&self) -> Self;
    fn rotate(&self) -> Self;
    fn canonicalize(&self) -> Self {
        let mut out = *self;
        let mut tmp = *self;

        for _ in 0..7 {
            tmp = tmp.rotate();
            out = out.min(tmp);
        }
        tmp = tmp.reverse();
        out = out.min(tmp);
        for _ in 0..7 {
            tmp = tmp.rotate();
            out = out.min(tmp);
        }
        out
    }
}

impl Canonicalizable for u16 {
    fn reverse(&self) -> Self {
        let mut x = *self;
        x = ((x & 0xcccc) >> 2) | ((x & 0x3333) << 2);
        x = ((x & 0xf0f0) >> 4) | ((x & 0x0f0f) << 4);
        x = ((x & 0xff00) >> 8) | ((x & 0x00ff) << 8);
        x
    }

    fn rotate(&self) -> Self {
        self.rotate_left(2)
    }
}

impl Canonicalizable for u32 {
    fn reverse(&self) -> Self {
        let mut x = *self;
        x = ((x & 0xf0f0f0f0) >> 4) | ((x & 0x0f0f0f0f) << 4);
        x = ((x & 0xff00ff00) >> 8) | ((x & 0x00ff00ff) << 8);
        x = ((x & 0xffff0000) >> 16) | ((x & 0x0000ffff) << 16);
        x
    }

    fn rotate(&self) -> Self {
        self.rotate_left(4)
    }
}

fn find_shapes(num_stones: u16) -> HashSet<u16> {
    fn count_stones(mut shape: u16) -> u16 {
        let mut count = 0;
        for _ in 0..8 {
            count += shape & 0b11;
            shape >>= 2;
        }
        count
    }

    let capacity = [
        1, 1, 5, 10, 28, 52, 105, 167, 265, 352, 454, 506, 543, 506, 454, 352, 265,
    ][num_stones as usize];
    let mut shapes = HashSet::with_capacity(capacity);
    for raw_shape in u16::MIN..=u16::MAX {
        if count_stones(raw_shape) != num_stones {
            continue;
        }
        shapes.insert(raw_shape.canonicalize());
    }
    shapes
}

fn make_repunit(num_digit: usize) -> u16 {
    let mut repunit = 0;
    for _ in 0..num_digit {
        repunit <<= 1;
        repunit |= 1;
    }
    repunit
}

fn create_board_id(mut shape: u16, mut pattern: u16) -> u32 {
    let mut id = 0;
    for n in 0..8 {
        let num_stones = shape & 0b11;

        let mask = make_repunit(num_stones as usize);
        let id_part = ((pattern & mask) | (mask + 1)) as u32;
        id |= id_part << (4 * n);

        shape >>= 2;
        pattern >>= num_stones;
    }
    id
}

pub fn find_board_ids(num_stones: u16) -> HashSet<u32> {
    let capacity = [
        1, 1, 6, 27, 139, 478, 1826, 5487, 16933, 42192, 106332, 223700, 468444, 829912, 1444680,
        2144640, 3078229,
    ][num_stones as usize];

    let mut ids = HashSet::with_capacity(capacity);
    let max_pattern = make_repunit(num_stones as usize);
    for shape in find_shapes(num_stones) {
        for pattern in 0..=max_pattern {
            if pattern.count_ones() != (num_stones as u32 + 1) / 2 {
                continue;
            }
            ids.insert(create_board_id(shape, pattern).canonicalize());
        }
    }
    ids
}
