mod game;

use std::{sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use colored::Colorize;
use game::{GameState, MetaMove, PlayerMarker, PossibleMoves, DISPLAY_SIZE};
use rand::Rng;

fn main() {
    println!("Display_Size: {}", DISPLAY_SIZE);

    let mut wins1 = 0;
    let mut wins2 = 0;
    let mut draws = 0;

    for _ in 0..10 {
        let player1 = Box::new(RandomPlayer::new());
        // let player1 = Box::new(HumanPlayer::new());
        // let player1 = Box::new(MonteCarlo::new(100, false));
        let player2 = Box::new(MonteCarloPlayer::new());
        let mut game = Game::new(player1, player2);
        match game.play(){
            PlayerMarker::X => wins1 += 1,
            PlayerMarker::O => wins2 += 1,
            PlayerMarker::Empty => draws += 1,
        }
    }

    println!(
        "Player 1: {} | Player 2 {} | Draws {}",
        wins1.to_string().as_str().red(),
        wins2.to_string().as_str().green(),
        draws.to_string().as_str().yellow()
    )
}

// ##############################
// # Player
// ##############################

trait Player {
    fn get_move(&mut self, board: GameState) -> MetaMove;
}

#[derive(Clone)]
struct RandomPlayer;

impl RandomPlayer {
    fn new() -> Self {
        RandomPlayer {}
    }
}

impl Player for RandomPlayer {
    fn get_move(&mut self, board: GameState) -> MetaMove {
        let mut rng = rand::thread_rng();

        let possible_moves = &mut PossibleMoves::new();
        let next_move = &mut MetaMove::new_empty();

        board.get_possible_moves(possible_moves, next_move);

        possible_moves[rng.gen_range(0..possible_moves.len())]
    }
}

struct HumanPlayer;

impl HumanPlayer {
    fn new() -> Self {
        HumanPlayer {}
    }
}

impl Player for HumanPlayer {
    fn get_move(&mut self, board: GameState) -> MetaMove {
        let mut input = String::new();
        let possible_moves = &mut PossibleMoves::new();
        let next_move = &mut MetaMove::new_empty();

        loop {
            possible_moves.clear();
            board.get_possible_moves(possible_moves, next_move);

            for (i, m) in possible_moves.into_iter().enumerate() {
                println!("{}: {:?}", i, m.absolute_index);
            }

            println!("Enter your move: ");
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();
            if let Ok(index) = input.parse::<usize>() {
                if index < possible_moves.len() {
                    return possible_moves[index];
                }
            }
            println!("Invalid move!");
        }
    }
}


#[derive(Clone, Debug, PartialEq, PartialOrd)]
struct GameTreeKnot {
    children: Vec<GameTreeKnot>,
    move_: Option<MetaMove>,
    score: f32,
    visit_count: f32,
}

impl GameTreeKnot {
    fn uct(&self, child: &GameTreeKnot) -> f64 {
        if child.visit_count == 0. {
            return std::f64::MAX; // Return the maximum floating-point number possible
        }
        let exploration = 1.1;
        let exploitation = child.score as f64 / child.visit_count as f64;
        let parent_visits = self.visit_count as f64;
        let child_visits = child.visit_count as f64;
        exploitation + exploration * (parent_visits.ln() / child_visits).sqrt()
    }

    fn pv(&mut self, pv: &mut Vec<MetaMove>) {
        let best_child = self.get_best_child_score();
        if let Some(best_child) = best_child {
            if let Some(best_move) = best_child.move_ {
                pv.push(best_move);
            }
            best_child.pv(pv);
        }
    }


    fn get_best_child_score(&mut self) -> Option<&mut GameTreeKnot> {
        if self.children.is_empty() {
            return None;
        }
        self.children.iter_mut()
            .filter(|node| node.visit_count > 0.) // Filter out nodes with zero visits
            .max_by(|a, b| {
                let a_rate = if a.visit_count > 0. {
                    a.score as f64 / a.visit_count as f64
                } else {
                    0.0
                };
                let b_rate = if b.visit_count > 0. {
                    b.score as f64 / b.visit_count as f64
                } else {
                    0.0
                };
                a_rate.partial_cmp(&b_rate).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn select_and_backtrack(
        &mut self, 
        meta_board: &mut GameState, 
        possible_moves: &mut PossibleMoves, 
        next_move: &mut MetaMove
    ) -> f32 
        {
        self.visit_count += 1.;

        if self.children.is_empty() {
            let score = self.expand_and_playout(meta_board.clone(), possible_moves, next_move);
            self.score += score;
            return score;
        }

        let mut best_child = 0;
        let mut best_score = self.uct(&self.children[0]);
        for (i, child) in self.children.iter().enumerate().skip(1) {
            let score = self.uct(child);
            if score > best_score {
                best_score = score;
                best_child = i;
            }
        }

        let best_node = &mut self.children[best_child];

        let move_ = best_node.move_.unwrap();

        meta_board.set(move_).unwrap();
        let result = 1. - best_node.select_and_backtrack(meta_board, possible_moves, next_move);
        self.score += result;

        meta_board.unset(self.move_);
        result
    }

    fn expand_and_playout(&mut self, mut meta_board: GameState, possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) -> f32 {
        meta_board.get_possible_moves(possible_moves, next_move);

        if possible_moves.is_empty() {
            let player_marker = meta_board.get_winner();
            return if player_marker == PlayerMarker::Empty {
                0.5
            } else {
                if player_marker == meta_board.current_player {
                    0.
                } else {
                    1.
                }
            };
        }

        for move_ in possible_moves.into_iter() {
            self.children.push(GameTreeKnot {
                children: vec![],
                move_: Some(*move_),
                score: 0.,
                visit_count: 0.,
            });
        }

        let rand_index = rand::thread_rng().gen_range(0..possible_moves.len());
        1. - self.children[rand_index].playout(&mut meta_board, possible_moves, next_move)
    }

    fn playout(&mut self, meta_board: &mut GameState, possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) -> f32 {
        let mut rng = rand::thread_rng();
        let current_player = meta_board.current_player;

        meta_board.set(self.move_.unwrap()).unwrap();

        loop {
            meta_board.get_possible_moves(possible_moves, next_move);
            if possible_moves.is_empty() {
                break;
            }
            let index = rng.gen_range(0..possible_moves.len());
            meta_board.set(possible_moves[index]).unwrap();
        }
        
        let player_marker =  meta_board.get_winner();
        let score = if player_marker == PlayerMarker::Empty {
            0.5
        } else {
            if player_marker == current_player {
                1.
            } else {
                0.
            }
        };

        self.visit_count += 1.;
        self.score += score;
        score
    }
}

#[derive(Clone, Debug)]
struct MonteCarlo {
    meta_board : GameState,
    tree_head: GameTreeKnot,
    debug: bool,
}

impl MonteCarlo {
    fn new(debug: bool) -> Self {
        MonteCarlo {
            tree_head: GameTreeKnot {
                children: vec![],
                move_: None,
                score: 0.,
                visit_count: 0.,
            },
            meta_board : GameState::new(),
            debug,
        }
    }

    fn next_move(&mut self, next_move: MetaMove) {
        if self.tree_head.move_.is_some() {
            let mut check = false;
            for child in self.tree_head.children.iter() {
                if child.move_ == Some(next_move) {
                    self.tree_head = child.to_owned();
                    check = true;
                    break;
                }
            }

            if !check {
                // panic!("No child found for last move");
                println!("{:?}", self.tree_head.children.iter().map(|x| x.move_));
                println!("No child found for last move");
            }
        }
    }

    // fn iterate(&mut self, stop_flag : Arc<AtomicBool>) {
    //     let possible_moves = &mut PossibleMoves::new();
    //     let next_move = &mut MetaMove::new_empty();

    //     while !stop_flag.load(Ordering::SeqCst){
    //         self.tree_head.select_and_backtrack(&mut self.meta_board, possible_moves, next_move);
    //     }
    // }
}

struct MonteCarloPlayer {
    mcts: Arc<Mutex<MonteCarlo>>,
    flag: Arc<AtomicBool>,
    thread : Option<JoinHandle<()>>
}

impl MonteCarloPlayer {
    fn new() -> Self {
        MonteCarloPlayer {
            mcts : Arc::new(Mutex::new(MonteCarlo::new(false))),
            flag : Arc::new(AtomicBool::new(false)),
            thread : None
        }
    }
}

impl Drop for MonteCarloPlayer {
    fn drop(&mut self) {
        self.flag.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}

impl Player for MonteCarloPlayer {
    fn get_move(&mut self, game_state: GameState) -> MetaMove {
        if let Some(last_move) = game_state.last_move {
            self.mcts
                .lock()
                .unwrap()
                .next_move(last_move);
        }

        if self.thread.is_none() {
            let mct_ref = Arc::clone(&self.mcts);
            let should_stop = Arc::clone(&self.flag);
            self.thread = Some(
                thread::spawn(move || {
                    let possible_moves = &mut PossibleMoves::new();
                    let next_move = &mut MetaMove::new_empty();

                    while !should_stop.load(Ordering::SeqCst){
                        mct_ref
                            .lock()
                            .unwrap()
                            .tree_head
                            .select_and_backtrack(&mut mct_ref.lock().unwrap().meta_board, possible_moves, next_move);

                        thread::sleep(Duration::from_millis(1));
                    }
                })
            )
        }

        thread::sleep(Duration::from_secs(2));

        let best_move = {
            let mut mct_guard = self.mcts.lock().unwrap();
            mct_guard
                .tree_head
                .get_best_child_score()
                .unwrap()
                .move_
                .unwrap()
        };
    
        // Update the MCTS with the best move
        self.mcts.lock().unwrap().next_move(best_move);

        best_move
    }

    // fn get_move(&self, mut meta_board: GameState) -> MetaMove {
    //     let meta_board = &mut meta_board;
    //     let mcts = &mut self.mcts;

    //     if meta_board.last_move.is_some() && mcts.tree_head.move_.is_some() {
    //         let last_move = meta_board.last_move.unwrap();
    //         let mut check = false;
    //         for child in mcts.tree_head.children.iter() {
    //             if child.move_ == Some(last_move) {
    //                 mcts.tree_head = child.to_owned();
    //                 check = true;
    //                 break;
    //             }
    //         }

    //         if !check {
    //             // panic!("No child found for last move");
    //             println!("{:?}", mcts.tree_head.children.iter().map(|x| x.move_));
    //             println!("No child found for last move");
    //         }
    //     }

    //     let possible_moves = &mut PossibleMoves::new();
    //     let next_move = &mut MetaMove::new_empty();

    //     for _ in 0..mcts.iterations {
    //         mcts.tree_head.select_and_backtrack(meta_board, possible_moves, next_move);
    //     }

    //     if mcts.debug {

    //         for child in mcts.tree_head.children.iter() {
    //             println!(
    //                 "{:?}: {} {}",
    //                 child.move_.unwrap().absolute_index,
    //                 child.score,
    //                 child.visit_count
    //             );
    //         }

    //         println!("Score: {}", mcts.tree_head.score);
    //         println!("Visits: {}", mcts.tree_head.visit_count);
    //         println!(
    //             "Score: {}",
    //             1. - (mcts.tree_head.score / mcts.tree_head.visit_count)
    //         );
    //         let mut pv = vec![];
    //         mcts.tree_head.pv(&mut pv);
    //         for move_ in pv {
    //             println!("{:?}", move_.absolute_index);
    //         }
    //     }

    //     let best_move = mcts.tree_head.get_best_child_score().unwrap();
    //     mcts.tree_head = best_move.to_owned();

    //     mcts.tree_head.move_.unwrap()
    // }
}


// ##############################
// # Game
// ##############################
struct Game {
    player1: Box<dyn Player>,
    player2: Box<dyn Player>,
    board: GameState,
    current_player: i32,
}

impl Game {
    fn new(player1: Box<dyn Player>, player2: Box<dyn Player>) -> Self {
        Game {
            player1,
            player2,
            board: GameState::new(),
            current_player: 1,
        }
    }

    fn play(&mut self) -> PlayerMarker {

        loop {
            println!("{}", self.board);

            // let possible_moves = self.board.get_possible_moves();
            if !self.board.board.can_set(){
                println!("{}", "It's a draw!".yellow());
                return PlayerMarker::Empty;
            }

            let current_player = if self.current_player == 1 {
                &mut self.player1
            } else {
                &mut self.player2
            };

            let chosen_move = current_player.get_move(self.board.clone());
            println!("Player {} chose {:?}", self.board.current_player.to_char(), chosen_move.absolute_index);

            if let Ok(player_marker) = self.board.set(chosen_move) {
                if player_marker != PlayerMarker::Empty {
                    println!("Player {} wins!", player_marker.to_char());
                    println!("{}", self.board);
                    println!("Game over!");
                    return player_marker
                }
            } else {
                println!("Invalid move!");
                continue;
            }

            self.current_player *= -1;
        }
    }
}