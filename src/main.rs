mod game;

use std::{sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use colored::Colorize;
use game::{GameState, MetaMove, PlayerMarker, PossibleMoves, DISPLAY_SIZE};
use rand::Rng;

/// Main function
/// 
/// Plays n games between two players and tracks the wins and draws
fn main() {
    println!("Display_Size: {}", DISPLAY_SIZE);

    let mut wins1 = 0;
    let mut wins2 = 0;
    let mut draws = 0;

    for _ in 0..10 {
        // let player1 = Box::new(RandomPlayer::new());
        // let player1 = Box::new(HumanPlayer::new());
        let player1 = Box::new(MonteCarloSync::new(500));
        let player2 = Box::new(MonteCarloAsync::new(Duration::from_millis(500)));
        let mut game = Game::new(player1, player2);
        let result = game.play();

        wins1 += result.max(0);
        wins2 -= result.min(0);
        draws += (result == 0) as i32;
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

enum MonteCarloAsyncMessage {
    AdvanceMove(MetaMove),
    Pause,
    Resume,
}

struct MonteCarloAsync {
    tree_head: Arc<Mutex<GameTreeKnot>>,
    _thread: JoinHandle<()>,
    sender: Sender<MonteCarloAsyncMessage>,
    think_time: Duration,
}

impl MonteCarloAsync {
    fn new(think_time: Duration) -> Self {
        if think_time.as_millis() == 0 {
            panic!("Think time must be greater than 0");
        }
        let (sender, receiver) = channel::<MonteCarloAsyncMessage>();
        let tree_head = Arc::new(Mutex::new(GameTreeKnot {
            children: vec![],
            move_: None,
            score: 0.,
            visit_count: 0.,
        }));
        
        MonteCarloAsync {
            tree_head: Arc::clone(&tree_head),
            sender,
            _thread: Self::spawn_thread(GameState::new(), tree_head, receiver),
            think_time,
        }
    }

    fn spawn_thread(game_state: GameState, head: Arc<Mutex<GameTreeKnot>>, receiver: Receiver<MonteCarloAsyncMessage>) -> JoinHandle<()> {

        thread::spawn(move || {
            let mut game_state = game_state;
            let mut tree_head = Some(head.lock().unwrap());
            let mut possible_moves = PossibleMoves::new();
            let mut next_move = MetaMove::new_empty();
            loop {
                if let Ok(message) = receiver.try_recv() {
                    match message {
                        MonteCarloAsyncMessage::AdvanceMove(move_) => {
                            game_state.set(move_).unwrap();
                            if tree_head.is_none() {
                                tree_head = Some(head.lock().unwrap());
                            }
                            tree_head.as_mut().unwrap().move_head(move_);
                        }
                        MonteCarloAsyncMessage::Pause => {
                            if let Some(tree_head) = tree_head.as_mut() {
                                tree_head.select_and_backtrack(&mut game_state, &mut possible_moves, &mut next_move);
                            }
                            tree_head = None;
                        }
                        MonteCarloAsyncMessage::Resume => {
                            if tree_head.is_some() {
                                continue;
                            }
                            tree_head = Some(head.lock().unwrap());
                        }
                    }
                } else if let Some(tree_head) = tree_head.as_mut(){
                    tree_head.select_and_backtrack(&mut game_state, &mut possible_moves, &mut next_move);
                } 
            }
        })
    }
}

impl Player for MonteCarloAsync {
    fn get_move(&mut self, board: GameState) -> MetaMove {
        if let Some(last_move) = board.last_move {
            let _ = self.sender.send(MonteCarloAsyncMessage::AdvanceMove(last_move));
        }

        thread::sleep(self.think_time);

        let _ = self.sender.send(MonteCarloAsyncMessage::Pause);
        if let Ok(tree_head) = self.tree_head.lock() {
            let best_move = tree_head.get_best_child_score().unwrap().move_.unwrap();
            let _ = self.sender.send(MonteCarloAsyncMessage::Resume);
            drop(tree_head);
            let _ = self.sender.send(MonteCarloAsyncMessage::AdvanceMove(best_move));
            return best_move;
        }
        MetaMove::new_empty()
    }
}

#[derive(Clone)]
struct MonteCarloSync {
    tree_head: GameTreeKnot,
    iterations: i32,
}

impl MonteCarloSync {
    fn new(iterations: i32) -> Self {
        MonteCarloSync {
            tree_head: GameTreeKnot {
                children: vec![],
                move_: None,
                score: 0.,
                visit_count: 0.,
            },
            iterations,
        }
    }

    fn move_head(&mut self, meta_board: &GameState) -> bool {
        if meta_board.last_move.is_some() && self.tree_head.move_.is_some() {
            let last_move = meta_board.last_move.unwrap();
            for child in self.tree_head.children.iter() {
                if child.move_ == Some(last_move) {
                    self.tree_head = child.to_owned();
                    return true;
                }
            }  
        }
        false
    }
}

impl Player for MonteCarloSync {
    fn get_move(&mut self, mut meta_board: GameState) -> MetaMove {
        let meta_board = &mut meta_board;
        
        if !self.move_head(&meta_board){
            // Reset head if move is not found
            self.tree_head = GameTreeKnot {
                children: vec![],
                move_: meta_board.last_move,
                score: 0.,
                visit_count: 0.,
            };
        }

        let possible_moves = &mut PossibleMoves::new();
        let next_move = &mut MetaMove::new_empty();

        for _ in 0..self.iterations {
            self.tree_head.select_and_backtrack(meta_board, possible_moves, next_move);
        }

        let best_move = self.tree_head.get_best_child_score();
        // if best_move.is_none() {
        //     return MetaMove::new_empty();
        // }
        self.tree_head = best_move.unwrap().to_owned();

        self.tree_head.move_.unwrap()
    }
}

impl GameTreeKnot {
    fn move_head(&mut self, meta_move: MetaMove) {
        if !self.children.is_empty() {
            for child in self.children.iter() {
                if child.move_ == Some(meta_move) {
                    *self = child.to_owned();
                    return;
                }
            }  
        }
        println!("Resetting tree head");
        *self = GameTreeKnot {
            children: vec![],
            move_: Some(meta_move),
            score: 0.,
            visit_count: 0.,
        };
    }
    
    /// Upper Confidence Bound for Trees (UCT) algorithm
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

    /// Returns the child with the best score
    /// 
    /// The score is calculated as the number of wins divided by the number of visits
    fn get_best_child_score(&self) -> Option<&GameTreeKnot> {
        if self.children.is_empty() {
            return None;
        }
        self.children.iter()
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

    /// Recursively selects a child node and backtracks the score
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

    /// Expands a leaf node and plays out a random game
    fn expand_and_playout(&mut self, mut meta_board: GameState, possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) -> f32 {
        meta_board.get_possible_moves(possible_moves, next_move);

        if possible_moves.is_empty() {
            let player_marker = meta_board.get_winner();
            return if player_marker == PlayerMarker::Draw {
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

    /// Plays out a random game until the end
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
        let score = if player_marker == PlayerMarker::Draw {
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



// ##############################
// # Game
// ##############################
struct Game {
    player1: Box<dyn Player>,
    player2: Box<dyn Player>,
    board: GameState,
    starting_player: i8,
}

impl Game {
    fn new(player1: Box<dyn Player>, player2: Box<dyn Player>) -> Self {
        Game {
            player1,
            player2,
            board: GameState::new(),
            starting_player: if rand::random() { 1 } else { -1 },
        }
    }

    /// Plays the game until a player wins or it's a draw
    /// 
    /// Returns the -1 if player 1 wins, 1 if player 2 wins, and 0 if it's a draw
    fn play(&mut self) -> i8 {
        let mut current_player_index = self.starting_player.clone();
        println!("Player {} starts!", if self.starting_player == 1 { 1 } else { 2 });

        loop {
            println!("{}", self.board);

            // let possible_moves = self.board.get_possible_moves();
            if !self.board.board.can_set(){
                println!("{}", "It's a draw!".yellow());
                return 0;
            }

            let current_player = if current_player_index == 1 {
                &mut self.player1
            } else {
                &mut self.player2
            };

            let chosen_move = current_player.get_move(self.board.clone());
            println!("Player {} chose {:?}", self.board.current_player.to_char(), chosen_move.absolute_index);

            if let Ok(player_marker) = self.board.set(chosen_move) {

                if player_marker == PlayerMarker::Draw {
                    println!("{}", "It's a draw!".yellow());
                    return 0;
                }
                
                if player_marker != PlayerMarker::Empty {
                    println!("Player {} wins!", player_marker.to_char());
                    println!("{}", self.board);
                    println!("Game over!");
                    return match player_marker{
                        PlayerMarker::X => self.starting_player,
                        PlayerMarker::O => self.starting_player * -1,
                        _ => 0,
                    };
                }
            } else {
                println!("Invalid move!");
                continue;
            }

            current_player_index *= -1;
        }
    }
}