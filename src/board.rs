use crate::bit_utils::HotBitIter;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum Color {
    Black,
    White,
}

impl std::ops::Not for Color {
    type Output = Self;
    fn not(self) -> Self::Output {
        use Color::*;
        match self {
            Black => White,
            White => Black,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Put { player: Color, target: u32 },
    Move { from: u32, to: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    b: u32,
    w: u32,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_array: [[String; 3]; 8] = Default::default();
        for n in 0..8 {
            let mut cursor = 1 << (4 * n);
            for i in 0..3 {
                if self.b & cursor != 0 {
                    str_array[n][i] = "B".to_string();
                } else if self.w & cursor != 0 {
                    str_array[n][i] = "W".to_string();
                }
                cursor <<= 1;
            }
        }
        write!(
            f,
            "{}",
            str_array
                .into_iter()
                .enumerate()
                .map(|(n, strs)| format!("{}: {}", n, strs.join("")))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl From<u32> for Board {
    fn from(mut id: u32) -> Self {
        let mut black = 0;
        let mut white = 0;
        for n in 0..8 {
            let part = id & 0xf;
            if part == 0 {
                panic!("invalid id");
            }
            let mask = (1 << (31 - part.leading_zeros())) - 1;
            let black_part = part & mask;
            let white_part = black_part ^ mask;

            black |= black_part << (4 * n);
            white |= white_part << (4 * n);

            id >>= 4;
        }
        Self { b: black, w: white }
    }
}

impl From<Board> for u32 {
    fn from(board: Board) -> Self {
        let mut id = 0;
        let mut bw = board.b | board.w;
        let mut b = board.b;
        for n in 0..8 {
            let bw_part = bw & 0xf;
            let b_part = b & 0xf;
            let id_part = (bw_part + 1) | b_part;
            id |= id_part << (4 * n);
            bw >>= 4;
            b >>= 4;
        }
        id
    }
}

impl Board {
    pub fn new() -> Self {
        Self { b: 0, w: 0 }
    }

    fn bit(&self, player: Color) -> &u32 {
        match player {
            Color::Black => &self.b,
            Color::White => &self.w,
        }
    }

    fn bit_mut(&mut self, player: Color) -> &mut u32 {
        match player {
            Color::Black => &mut self.b,
            Color::White => &mut self.w,
        }
    }

    pub fn num_stones(&self) -> u32 {
        (self.b | self.w).count_ones()
    }

    pub fn canonical_player(&self) -> Color {
        match self.num_stones() % 2 {
            0 => Color::Black,
            1 => Color::White,
            _ => unreachable!(),
        }
    }

    pub fn legal_actions(&self, player: Color) -> Vec<Action> {
        let mut actions = Vec::new();
        let bw = self.b | self.w;
        let to_able = 0x1111_1111 & ((!bw) >> 2);

        if self.num_stones() == 16 {
            // Move
            let from_able = 0x1111_1111 & self.bit(player);
            for from in HotBitIter::from(from_able) {
                let to_candidates = 0x1001_0010u32.rotate_left(from.trailing_zeros());
                for to in HotBitIter::from(to_candidates & to_able) {
                    actions.push(Action::Move { from, to });
                }
            }
        } else {
            // Put
            if player == self.canonical_player() {
                for target in HotBitIter::from(to_able) {
                    actions.push(Action::Put { player, target });
                }
            }
        }
        actions
    }

    fn put_stone(&mut self, player: Color, target: u32) {
        let zeros = target.trailing_zeros();
        let mask = 0b0111 << zeros;
        let outer_mask = !(0b1111 << zeros);

        let bit = self.bit_mut(player);
        *bit = (*bit & outer_mask) | ((*bit & mask) << 1) | target;

        let bit_other = self.bit_mut(!player);
        *bit_other = (*bit_other & outer_mask) | ((*bit_other & mask) << 1);
    }

    fn remove_stone(&mut self, target: u32) -> Option<Color> {
        let zeros = target.trailing_zeros();
        let mask = 0b1110 << zeros;
        let outer_mask = !(0b1111 << zeros);

        let mut stone_color = None;
        for color in Color::iter() {
            let bit = self.bit_mut(color);
            if *bit & target == target {
                stone_color = Some(color);
            }
            *bit = (*bit & outer_mask) | ((*bit & mask) >> 1);
        }
        stone_color
    }

    fn move_stone(&mut self, from: u32, to: u32) {
        let stone_color = self.remove_stone(from).unwrap();
        self.put_stone(stone_color, to);
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Put { player, target } => self.put_stone(player, target),
            Action::Move { from, to } => self.move_stone(from, to),
        }
    }

    pub fn perform_copied(&self, action: Action) -> Self {
        let mut board_tmp = *self;
        board_tmp.perform(action);
        board_tmp
    }

    pub fn swap_color(&mut self) {
        std::mem::swap(&mut self.b, &mut self.w);
    }

    pub fn lines_up(&self, player: Color) -> bool {
        let bit = self.bit(player);

        let mut mask_h = 0x1111;
        let mut mask_v = 0b0111;
        for _ in 0..8 {
            if bit & mask_h == mask_h || bit & mask_v == mask_v {
                return true;
            }
            mask_h = mask_h.rotate_left(4);
            mask_v <<= 4;
        }
        false
    }
}
