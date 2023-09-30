mod game_state;
mod move_type;
mod tube;

use game_state::GameState;

fn main() {
    let gs = GameState::new_from_file();
    println!("{:?}", gs);

    gs.search_for_solution();
}
