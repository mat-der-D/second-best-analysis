#![allow(unused)]
mod analysis;
mod bit_utils;
mod board;

use std::collections::HashSet;

use analysis::{analyze_backward, Evaluator};
use bit_utils::Canonicalizable;
use board::{Board, Color};

fn display_analysis_result() {
    let (wins_vec, loses_vec, _) = analyze_backward(true);

    let first_position_id: u32 = Board::new().into();
    for (n, w) in wins_vec.into_iter().enumerate() {
        if w.contains(&first_position_id) {
            println!("The first player wins in {n} steps");
        }
    }
    for (n, l) in loses_vec.into_iter().enumerate() {
        if l.contains(&first_position_id) {
            println!("The second player wins in {n} steps");
        }
    }
}

fn first_few_steps(num_step: usize) {
    let (wins_vec, loses_vec, both) = analyze_backward(true);
    let evaluator = Evaluator::from_analyzed(wins_vec, loses_vec, both);

    let mut board = Board::new();
    let mut player = Color::Black;
    for n in 0..num_step {
        println!("{board}");
        let values_map = evaluator.evaluate(board, player);
        for (a, val) in values_map.iter() {
            println!("- {a} -> {val}");
        }

        let mut values = values_map.into_iter().collect::<Vec<_>>();
        values.sort_by(|(a1, val1), (a2, val2)| {
            if *val1 == *val2 {
                a1.partial_cmp(a2).unwrap()
            } else {
                val1.cmp(val2).reverse()
            }
        });
        let sorted_actions = values.into_iter().map(|(a, _)| a).collect::<Vec<_>>();
        let next_action = sorted_actions[1];
        println!("NEXT ACTION >> {next_action}");
        board.perform(next_action);
        player = !player;
        println!("------------------------------");
    }
}

fn count_best_playing() {
    let (wins_vec, loses_vec, both) = analyze_backward(true);
    let evaluator = Evaluator::from_analyzed(wins_vec, loses_vec, both);

    let mut bests_vec = vec![HashSet::from([u32::from(Board::new()).canonicalize()])];
    for n in 0..42 {
        let mut next_bests = HashSet::new();
        for id0 in &bests_vec[n] {
            let b0 = Board::from(*id0);
            let player = b0.canonical_player();
            let values_map = evaluator.evaluate(b0, player);
            let mut values = values_map.values().collect::<Vec<_>>();
            values.sort_by(|x, y| x.cmp(y).reverse());
            let second_best_value = *values[1];
            let second_best_actions = values_map
                .iter()
                .filter(|(_, val)| **val == second_best_value)
                .map(|(a, val)| *a);
            for a in second_best_actions {
                let mut b1 = b0.perform_copied(a);
                if b0.num_stones() == 16 {
                    b1.swap_color();
                }
                let id1 = u32::from(b1).canonicalize();
                next_bests.insert(id1);
            }
        }
        bests_vec.push(next_bests);
    }

    for (n_step, bests) in bests_vec.into_iter().enumerate() {
        println!("| {n_step} | {} |", bests.len());
    }
}

fn main() {
    // count_best_playing();
    first_few_steps(9);
}
