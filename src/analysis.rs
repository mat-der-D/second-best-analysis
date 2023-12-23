use crate::bit_utils::*;
use crate::board::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BoardClass {
    WinsDefinitely,
    LosesDefinitely,
    BothLineUp,
    LosesPassively,
}

fn classify_board(board: Board) -> Option<BoardClass> {
    let player = board.canonical_player();

    use BoardClass::*;
    match (board.lines_up(player), board.lines_up(!player)) {
        (true, true) => return Some(BothLineUp),
        (true, false) => return Some(WinsDefinitely),
        (false, true) => return Some(LosesDefinitely),
        (false, false) => (),
    }

    if board.legal_actions(player).len() <= 1 {
        Some(LosesPassively)
    } else {
        None
    }
}

fn classify_ids(ids: &HashSet<u32>) -> (HashMap<BoardClass, HashSet<u32>>, HashSet<u32>) {
    let mut class_to_ids = HashMap::new();
    let mut residual = HashSet::new();
    for &id in ids {
        let board = Board::from(id);
        if let Some(class) = classify_board(board) {
            class_to_ids
                .entry(class)
                .or_insert_with(HashSet::new)
                .insert(id);
        } else {
            residual.insert(id);
        }
    }
    (class_to_ids, residual)
}

pub fn analyze_backward(
    show_progress: bool,
) -> (Vec<HashSet<u32>>, Vec<HashSet<u32>>, HashSet<u32>) {
    let mut ids = HashSet::new();
    for n in 0..=16 {
        ids.extend(find_board_ids(n));
    }
    let (class_to_ids, residual) = classify_ids(&ids);

    let mut wins = HashSet::new();
    let mut loses = HashSet::new();
    let mut both = HashSet::new();
    for (class, ids) in class_to_ids {
        use BoardClass::*;
        match class {
            WinsDefinitely => wins.extend(ids),
            LosesDefinitely | LosesPassively => loses.extend(ids),
            BothLineUp => both.extend(ids),
        }
    }
    let mut residual = residual;

    let mut wins_vec = vec![wins.clone()];
    let mut loses_vec = vec![loses.clone()];
    for n in 1..=140 {
        if show_progress {
            println!("Progress: {n} / 140 ({:.1}%)", (100f32 * n as f32) / 140f32);
        }
        let mut loses_next = HashSet::new();
        let mut wins_next = HashSet::new();
        let mut res_next = HashSet::new();
        for id0 in residual {
            let b0 = Board::from(id0);
            let num_stones = b0.num_stones();
            let player = b0.canonical_player();
            let actions = b0.legal_actions(player);
            let num_actions = actions.len();

            let mut num_win = 0;
            let mut num_lose = 0;
            for a in actions {
                let mut b1 = b0.perform_copied(a);
                if num_stones == 16 {
                    b1.swap_color();
                }
                let id1 = u32::from(b1).canonicalize();
                if wins.contains(&id1) {
                    num_lose += 1;
                } else if loses.contains(&id1) || both.contains(&id1) {
                    num_win += 1;
                }
                if num_win >= 2 {
                    break;
                }
            }

            if num_win >= 2 {
                wins_next.insert(id0);
            } else if num_lose >= num_actions - 1 {
                loses_next.insert(id0);
            } else {
                res_next.insert(id0);
            }
        }

        wins_vec.push(wins_next.clone());
        loses_vec.push(loses_next.clone());
        wins.extend(wins_next);
        loses.extend(loses_next);

        residual = res_next;
    }

    (wins_vec, loses_vec, both)
}

#[derive(Debug, Clone)]
pub struct Evaluator {
    wins_vec: Vec<HashSet<u32>>,
    loses_vec: Vec<HashSet<u32>>,
    both: HashSet<u32>,
}

impl Evaluator {
    pub fn new() -> Self {
        let (wins_vec, loses_vec, both) = analyze_backward(false);
        Self {
            wins_vec,
            loses_vec,
            both,
        }
    }

    pub fn from_analyzed(
        wins_vec: Vec<HashSet<u32>>,
        loses_vec: Vec<HashSet<u32>>,
        both: HashSet<u32>,
    ) -> Self {
        Self {
            wins_vec,
            loses_vec,
            both,
        }
    }

    pub fn evaluate(&self, board: Board, next_player: Color) -> HashMap<Action, i32> {
        let mut values = HashMap::new();
        let max_value = self.wins_vec.len() as i32;
        let min_value = -max_value;

        for a in board.legal_actions(next_player) {
            let mut b1 = board.perform_copied(a);
            if board.num_stones() == 16 && matches!(next_player, Color::Black) {
                b1.swap_color();
            }
            let id1 = u32::from(b1).canonicalize();

            if self.both.contains(&id1) {
                values.insert(a, max_value);
                continue;
            }

            let mut evaluated = false;
            for (n, wins) in self.wins_vec.iter().enumerate() {
                if wins.contains(&id1) {
                    values.insert(a, min_value + n as i32);
                    evaluated = true;
                    break;
                }
            }

            if evaluated {
                continue;
            }

            for (n, loses) in self.loses_vec.iter().enumerate() {
                if loses.contains(&id1) {
                    values.insert(a, max_value - n as i32);
                    evaluated = true;
                    break;
                }
            }

            if evaluated {
                continue;
            }

            values.insert(a, 0);
        }

        values
    }
}
