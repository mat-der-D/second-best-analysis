mod analysis;
mod bit_utils;
mod board;

fn main() {
    let (wins_vec, loses_vec, _) = analysis::analyze_backward(true);

    let first_position_id: u32 = board::Board::new().into();
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
