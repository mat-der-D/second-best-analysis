use crate::bit_utils::*;
use crate::board::*;
use fxhash::{FxHashMap, FxHashSet};

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

fn classify_ids(ids: &FxHashSet<u32>) -> (FxHashMap<BoardClass, FxHashSet<u32>>, FxHashSet<u32>) {
    let mut class_to_ids = FxHashMap::default();
    let mut residual = FxHashSet::default();
    for &id in ids {
        let board = Board::from(id);
        if let Some(class) = classify_board(board) {
            class_to_ids
                .entry(class)
                .or_insert_with(FxHashSet::default)
                .insert(id);
        } else {
            residual.insert(id);
        }
    }
    (class_to_ids, residual)
}

pub fn analyze_backward(
    show_progress: bool,
) -> (Vec<FxHashSet<u32>>, Vec<FxHashSet<u32>>, FxHashSet<u32>) {
    let mut ids = FxHashSet::default();
    for n in 0..=16 {
        ids.extend(find_board_ids(n));
    }
    let (class_to_ids, residual) = classify_ids(&ids);

    let mut wins = FxHashSet::default();
    let mut loses = FxHashSet::default();
    let mut both = FxHashSet::default();
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
        let mut loses_next = FxHashSet::default();
        let mut wins_next = FxHashSet::default();
        let mut res_next = FxHashSet::default();
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
