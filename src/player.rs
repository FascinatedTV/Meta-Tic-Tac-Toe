use game::{GameState, MetaMove, PlayerMarker};
use rand::Rng;

trait Player {
    fn get_move(&self, board: GameState) -> MetaMove;
}

#[derive(Clone)]
struct RandomPlayer;

impl RandomPlayer {
    fn new() -> Self {
        RandomPlayer {}
    }
}

impl Player for RandomPlayer {
    fn get_move(&self, board: GameState) -> MetaMove {
        let mut rng = rand::thread_rng();
        let moves = board.get_possible_moves();
        moves[rng.gen_range(0..moves.len())]
    }
}
