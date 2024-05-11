use colored::Colorize;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{fmt::Display, iter};

const BOARD_SIZE: usize = 3;
const BOARD_SIZE_SQUARED: usize = usize::pow(BOARD_SIZE, 2);
const META_BOARD_DEPTH: usize = 1; // 0 = 3x3, 1 = 9x9, 2 = 27x27

const META_BOARD_SIZE: usize = usize::pow(BOARD_SIZE_SQUARED, META_BOARD_DEPTH as u32);
const META_BOARD_SIDE: usize = usize::pow(BOARD_SIZE, META_BOARD_DEPTH as u32);

const WINNING_POSITIONS: [u16; 8] = [
    0b111_000_000,
    0b000_111_000,
    0b000_000_111, // Zeilen
    0b100_100_100,
    0b010_010_010,
    0b001_001_001, // Spalten
    0b100_010_001,
    0b001_010_100, // Diagonalen
];

#[derive(Clone, Copy, PartialEq)]
enum PlayerMarker {
    X,
    O,
}

impl PlayerMarker {
    fn other(&self) -> PlayerMarker {
        match self {
            PlayerMarker::X => PlayerMarker::O,
            PlayerMarker::O => PlayerMarker::X,
        }
    }
}

impl Display for PlayerMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PlayerMarker::X => "X",
                PlayerMarker::O => "O",
            }
        )
    }
}

impl Distribution<PlayerMarker> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PlayerMarker {
        if rng.gen() {
            PlayerMarker::X
        } else {
            PlayerMarker::O
        }
    }
}

// ######################################
// # BitBoard
// ######################################

#[derive(Clone, Copy, PartialEq)]
struct BitBoard {
    x: u16,
    o: u16,
}

impl BitBoard {
    fn new() -> Self {
        BitBoard { o: 0, x: 0 }
    }

    fn set(&mut self, player: PlayerMarker, position: usize) {
        let mask = 1 << position;
        match player {
            PlayerMarker::X => self.x |= mask,
            PlayerMarker::O => self.o |= mask,
        }
    }

    fn unset(&mut self, player: PlayerMarker, position: usize) {
        let mask = 1 << position;
        match player {
            PlayerMarker::X => self.x &= !mask,
            PlayerMarker::O => self.o &= !mask,
        }
    }

    fn is_full(&self) -> bool {
        (self.x | self.o) == 0b111_111_111
    }

    fn can_set(&self) -> bool {
        !self.is_full() && self.get_winner().is_none()
    }

    fn get_empty_positions(&self) -> Vec<usize> {
        if !self.can_set() {
            return vec![];
        }
        iter::successors(Some(0), move |&i| Some(i + 1))
            .take(BOARD_SIZE_SQUARED)
            .filter(move |&i| (self.x | self.o) & (1 << i) == 0)
            .collect()
    }

    fn get_winner(&self) -> Option<PlayerMarker> {
        let x = self.x;
        let o = self.o;
        for &winning_position in WINNING_POSITIONS.iter() {
            if x & winning_position == winning_position {
                return Some(PlayerMarker::X);
            }
            if o & winning_position == winning_position {
                return Some(PlayerMarker::O);
            }
        }
        None
    }

    fn get_row(&self, row: usize) -> [char; BOARD_SIZE] {
        let mut result = ['_'; BOARD_SIZE];
        for i in 0..BOARD_SIZE {
            let mask = 1 << (row * BOARD_SIZE + i);
            if self.x & mask != 0 {
                result[i] = 'X';
            } else if self.o & mask != 0 {
                result[i] = 'O';
            }
        }
        result
    }
}

// ######################################
// # MetaMove
// ######################################
#[derive(Clone, Copy, Debug)]
struct MetaMove {
    absolute_index: [usize; META_BOARD_DEPTH],
    meta_index: usize,
    board_index: usize,
}

impl PartialEq for MetaMove {
    fn eq(&self, other: &Self) -> bool {
        self.meta_index == other.meta_index && self.board_index == other.board_index
    }
}

impl MetaMove {
    fn shift_left(&self) -> MetaMove {
        let mut absolute_index = self.absolute_index;
        if absolute_index.len() == 0 {
            return MetaMove {
                absolute_index,
                meta_index: 0,
                board_index: 0,
            };
        }

        for i in 0..absolute_index.len() - 1 {
            absolute_index[i] = absolute_index[i + 1];
        }
        absolute_index[absolute_index.len() - 1] = self.board_index;
        MetaMove {
            absolute_index,
            meta_index: Self::absolute_index_to_meta(&absolute_index),
            board_index: 0,
        }
    }

    fn absolute_index_to_meta(absolute_index: &[usize]) -> usize {
        absolute_index
            .iter()
            .fold(0, |acc, &index| acc * BOARD_SIZE * BOARD_SIZE + index)
    }

    fn meta_to_absolute_index(meta_index: usize) -> [usize; META_BOARD_DEPTH] {
        let mut absolute_index = [0; META_BOARD_DEPTH];
        let mut meta_index = meta_index;
        for i in (0..META_BOARD_DEPTH).rev() {
            absolute_index[i] = meta_index % BOARD_SIZE_SQUARED;
            meta_index /= BOARD_SIZE_SQUARED;
        }
        absolute_index
    }
}

impl From<(usize, usize)> for MetaMove {
    fn from((meta_index, board_index): (usize, usize)) -> Self {
        MetaMove {
            absolute_index: MetaMove::meta_to_absolute_index(meta_index),
            meta_index,
            board_index,
        }
    }
}

impl From<([usize; META_BOARD_DEPTH], usize)> for MetaMove {
    fn from(value: ([usize; META_BOARD_DEPTH], usize)) -> Self {
        MetaMove {
            absolute_index: value.0,
            meta_index: MetaMove::absolute_index_to_meta(&value.0),
            board_index: value.1,
        }
    }
}

impl Display for MetaMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str("Move (");

        for i in 0..META_BOARD_DEPTH {
            result.push_str(&format!("{}", self.absolute_index[i]));
            result.push_str(", ");
        }
        result.push_str(&format!("{})", self.board_index));

        write!(f, "{}", result)
    }
}

// ######################################
// # MetaBoard
// ######################################

#[derive(Clone)]
struct MetaBoard {
    last_move: Option<MetaMove>,
    current_player: PlayerMarker,
    boards: [BitBoard; META_BOARD_SIZE],
}

impl MetaBoard {
    fn new() -> Self {
        MetaBoard {
            boards: [BitBoard::new(); META_BOARD_SIZE],
            last_move: None,
            current_player: PlayerMarker::X,
        }
    }

    fn set(&mut self, meta_move: MetaMove) {
        self.boards[meta_move.meta_index].set(self.current_player, meta_move.board_index);
        self.last_move = Some(meta_move);
        self.current_player = self.current_player.other();
    }

    fn unset(&mut self, previous_move: Option<MetaMove>) {
        let last_move = self.last_move.unwrap();
        self.current_player = self.current_player.other();

        self.boards[last_move.meta_index].unset(self.current_player, last_move.board_index);

        self.last_move = previous_move;
    }

    fn get_empty_positions(&self) -> Vec<MetaMove> {
        self.boards
            .iter()
            .enumerate()
            .flat_map(|(meta_index, board)| {
                board
                    .get_empty_positions()
                    .into_iter()
                    .map(move |board_index| From::from((meta_index, board_index)))
            })
            .collect()
    }

    fn get_winner(&self, index: &[usize]) -> Option<PlayerMarker> {
        if index.len() == META_BOARD_DEPTH {
            return self.boards[MetaMove::absolute_index_to_meta(index)].get_winner();
        }

        let mut board_x: u16 = 0;
        let mut board_o: u16 = 0;

        for i in 0..BOARD_SIZE_SQUARED {
            let new_index = [index, &[i]].concat();
            let result = self.get_winner(new_index.as_slice());
            if result.is_none() {
                continue;
            }

            match result.unwrap() {
                PlayerMarker::X => board_x |= 1 << i,
                PlayerMarker::O => board_o |= 1 << i,
            }
        }

        let board = BitBoard {
            x: board_x,
            o: board_o,
        };
        board.get_winner()
    }

    fn is_valid_move(&self, meta_move: MetaMove) -> bool {
        if meta_move.meta_index >= META_BOARD_SIZE
            || meta_move.board_index >= BOARD_SIZE_SQUARED
            || !self.boards[meta_move.meta_index].can_set()
        {
            return false;
        }

        for i in 0..META_BOARD_DEPTH {
            if self.get_winner(&meta_move.absolute_index[..i]).is_some() {
                return false;
            }
        }

        true
    }

    fn get_possible_moves2(&self) -> Vec<MetaMove> {
        if self.last_move.is_none() {return self.get_empty_positions();}
        let last_move = self.last_move.unwrap();
        let next_move = last_move.shift_left();

        fn accumulate_moves(meta_board: &MetaBoard, index: &[usize], current_index: &[usize]) -> Vec<MetaMove> {
            let meta_index = MetaMove::absolute_index_to_meta(current_index);
            if current_index.len() == META_BOARD_DEPTH {
                return meta_board.boards[meta_index]
                            .get_empty_positions()
                            .iter()
                            .map(|m| From::from((meta_index, *m)))
                            .collect();
            }

            


            vec![]
        }

        accumulate_moves(&self, &next_move.absolute_index, &[])
    }

    fn get_possible_moves(&self) -> Vec<MetaMove> {
        match self.last_move {
            Some(last_move) => {
                if self.get_winner(&[]).is_some() {
                    return vec![];
                }

                let next_move = last_move.shift_left();
                let next_meta_index = next_move.meta_index;

                if self.boards[next_meta_index].can_set() {
                    self.boards[next_meta_index]
                        .get_empty_positions()
                        .into_iter()
                        .map(move |board_index| From::from((next_meta_index, board_index)))
                        .collect()
                } else {
                    let mut i = 0;
                    loop {
                        if self.get_winner(&last_move.absolute_index[..i]).is_some()
                            || i == META_BOARD_DEPTH
                        {
                            break;
                        }
                        i += 1;
                    }

                    if i == 0 {
                        return vec![];
                    }

                    for i in (0..i).rev() {
                        let start =
                            MetaMove::absolute_index_to_meta(&last_move.absolute_index[..i]);
                        let end = start + BOARD_SIZE * BOARD_SIZE;
                        let mut vet: Vec<MetaMove> = vec![];
                        for meta_index in start..end {
                            if self.boards[meta_index].can_set() {
                                for board_index in self.boards[meta_index].get_empty_positions() {
                                    vet.push(From::from((meta_index, board_index)));
                                }
                            }
                        }
                        if !vet.is_empty() {
                            return vet;
                        }
                    }
                    self.get_empty_positions()
                }
            }
            None => self.get_empty_positions(),
        }
    }

    fn to_string(&self, index: Vec<usize>) -> Vec<String> {
        // let side = usize::pow(BOARD_SIZE, META_BOARD_DEPTH as u32 - index.len() as u32);
        let mut result = vec![];

        if index.len() == META_BOARD_DEPTH {
            // result.push("-".repeat(BOARD_SIZE).to_string());
            for row in 0..BOARD_SIZE {
                result.push(
                    self.boards[MetaMove::absolute_index_to_meta(&index)]
                        .get_row(row)
                        .iter()
                        .collect(),
                );
            }
            result.push("-".repeat(BOARD_SIZE).to_string());
            return result;
        }

        for row in 0..BOARD_SIZE {
            let res = self.to_string([index.clone(), vec![row * BOARD_SIZE + 0]].concat());
            for i in 0..res.len() {
                result.push(res[i].clone());
            }

            for col in 1..BOARD_SIZE {
                let res = self.to_string([index.clone(), vec![row * BOARD_SIZE + col]].concat());

                for i in 0..res.len() {
                    result[row * res.len() + i]
                        .push_str("|".repeat(META_BOARD_DEPTH - index.len()).as_str());
                    result[row * res.len() + i].push_str(res[i].as_str());
                }
            }
        }
        result
    }
}

impl Display for MetaBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();

        let result = self.to_string(vec![]);

        for row in result {
            string.push_str(&row);
            // string.push('|');
            string.push('\n');
        }

        write!(f, "{}", string)
    }
}

// ######################################
// Player
// ######################################

trait Player {
    fn get_move(&mut self, meta_board: MetaBoard) -> MetaMove;
}

#[derive(Clone)]
struct RandomPlayer {}

impl RandomPlayer {
    fn new() -> Self {
        RandomPlayer {}
    }
}

impl Player for RandomPlayer {
    fn get_move(&mut self, meta_board: MetaBoard) -> MetaMove {
        let possible_moves = meta_board.get_possible_moves();
        let mut rng = rand::thread_rng();
        possible_moves[rng.gen_range(0..possible_moves.len())]
    }
}

struct HumanPlayer {}

impl Player for HumanPlayer {
    fn get_move(&mut self, meta_board: MetaBoard) -> MetaMove {
        println!("Possible Moves:");
        let possible_moves = meta_board.get_possible_moves();
        for (i, move_) in possible_moves.iter().enumerate() {
            println!("{}: {}", i, move_);
        }
        loop {
            println!("Please enter your move:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let index: usize = match input.trim().parse() {
                Ok(index) => index,
                Err(_) => {
                    println!("Invalid input. Please enter a number.");
                    continue;
                }
            };
            if index < possible_moves.len() {
                return possible_moves[index];
            } else {
                println!(
                    "Invalid input. Please enter a number between 0 and {}.",
                    possible_moves.len() - 1
                );
            }
        }
    }
}

#[derive(Clone, Debug)]
struct GameTreeKnot {
    children: Vec<GameTreeKnot>,
    move_: Option<MetaMove>,
    score: f32,
    visit_count: f32,
}

#[derive(Clone)]
struct MonteCarlo {
    tree_head: GameTreeKnot,
}

impl MonteCarlo {
    fn new() -> Self {
        MonteCarlo {
            tree_head: GameTreeKnot {
                children: vec![],
                move_: None,
                score: 0.,
                visit_count: 0.,
            },
        }
    }
}

impl GameTreeKnot {
    fn get_best_child(&mut self) -> Option<&mut GameTreeKnot> {
        if self.children.is_empty() {
            return None;
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
        Some(&mut self.children[best_child])
    }

    fn select_and_backtrack(&mut self, meta_board: &mut MetaBoard) -> f32 {
        self.visit_count += 1.;

        if self.children.is_empty() {
            let score = self.expand_and_playout(meta_board.clone());
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

        meta_board.set(move_);
        let result = 1. - best_node.select_and_backtrack(meta_board);
        self.score += result;

        meta_board.unset(self.move_);
        result
    }

    //TODO: Inspect UCT
    fn uct(&self, child: &GameTreeKnot) -> f64 {
        if child.visit_count == 0. {
            return std::f64::MAX; // Return the maximum floating-point number possible
        }
        let exploration = 1.4;
        let exploitation = child.score as f64 / child.visit_count as f64;
        let parent_visits = self.visit_count as f64;
        let child_visits = child.visit_count as f64;
        exploitation + exploration * (parent_visits.ln() / child_visits).sqrt()
    }

    fn pv(&mut self, pv: &mut Vec<MetaMove>) {
        let best_child = self.get_best_child();
        if let Some(best_child) = best_child {
            if let Some(best_move) = best_child.move_ {
                pv.push(best_move);
            }
            best_child.pv(pv);
        }
    }

    fn expand_and_playout(&mut self, mut meta_board: MetaBoard) -> f32 {
        let possible_moves = meta_board.get_possible_moves();

        if possible_moves.is_empty() {
            return match meta_board.get_winner(&[]) {
                Some(winning_player) => {
                    if winning_player == meta_board.current_player {
                        0.
                    } else {
                        1.
                    }
                }
                None => 0.5,
            };
        }

        for move_ in &possible_moves {
            self.children.push(GameTreeKnot {
                children: vec![],
                move_: Some(*move_),
                score: 0.,
                visit_count: 0.,
            });
        }

        let rand_index = rand::thread_rng().gen_range(0..possible_moves.len());
        self.children[rand_index].playout(&mut meta_board)
    }

    // fn check_default_move_range(&self, meta_board: &MetaBoard, move_ : MetaMove) -> Option<MetaMove> {
    //     let mut rng = rand::thread_rng();
    //     let mut possible_moves:  Vec<MetaMove> = vec![];
    //     for i in 0..BOARD_SIZE_SQUARED {
    //         let next_move = MetaMove::from((move_.meta_index, i));
    //         if meta_board.is_valid_move(next_move) {
    //             possible_moves.push(next_move);
    //         }
    //     }

    //     if possible_moves.is_empty() {
    //         return None;
    //     }

    //     let index = rng.gen_range(0..possible_moves.len());
    //     Some(possible_moves[index])
    // }

    // TODO: Implement playout
    fn playout(&mut self, meta_board: &mut MetaBoard) -> f32 {
        // print!("p");
        let mut rng = rand::thread_rng();
        let current_player = meta_board.current_player;
        meta_board.set(self.move_.unwrap());

        loop {
            let possible_moves = meta_board.get_possible_moves();
            if possible_moves.is_empty() {
                break;
            }
            let index = rng.gen_range(0..possible_moves.len());
            meta_board.set(possible_moves[index]);
        }

        // let winner = if let Some(value) = meta_board.get_winner(&[]) {value.to_string()} else {"Draw".to_string()};
        // println!("End of playout Winner: {} \n{}", winner, meta_board);

        let score = match meta_board.get_winner(&[]) {
            Some(value) => {
                if value == current_player {
                    1.
                } else {
                    0.
                }
            }
            None => 0.5,
        };
        // println!("End of playout {}: {} \n{}", self.player_marker, score, meta_board);

        self.visit_count += 1.;
        self.score += score;
        score
    }
}

impl Player for MonteCarlo {
    fn get_move(&mut self, mut meta_board: MetaBoard) -> MetaMove {
        let meta_board = &mut meta_board;
        if meta_board.last_move.is_some() && self.tree_head.move_.is_some() {
            let last_move = meta_board.last_move.unwrap();
            let mut check = false;
            for child in self.tree_head.children.iter() {
                if child.move_ == Some(last_move) {
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

        for _ in 0..20000 {
            self.tree_head.select_and_backtrack(meta_board);
        }

        for child in self.tree_head.children.iter() {
            println!(
                "{}: {} {}",
                child.move_.unwrap(),
                child.score,
                child.visit_count
            );
        }

        println!("Score: {}", self.tree_head.score);
        println!("Visits: {}", self.tree_head.visit_count);
        println!(
            "Score: {}",
            1. - (self.tree_head.score / self.tree_head.visit_count)
        );
        let mut pv = vec![];
        self.tree_head.pv(&mut pv);
        for move_ in pv {
            println!("{:?}{}", move_.absolute_index, move_.board_index);
        }

        let best_index = self
            .tree_head
            .children
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| {
                (x.score / x.visit_count)
                    .partial_cmp(&(y.score / y.visit_count))
                    .unwrap()
            })
            .map(|(index, _)| index)
            .unwrap();

        let best_move = self.tree_head.children[best_index].move_.unwrap();
        self.tree_head = self.tree_head.children.remove(best_index);

        best_move
    }
}

// ######################################
// Game
// ######################################

struct Game {
    meta_board: MetaBoard,
    player_one: Box<dyn Player>,
    player_two: Box<dyn Player>,
    current_player: usize,
}

impl Game {
    fn new(player_one: impl Player + 'static, player_two: impl Player + 'static) -> Self {
        Game {
            meta_board: MetaBoard::new(),
            player_one: Box::new(player_one),
            player_two: Box::new(player_two),
            // current_player: rand::random()
            current_player: 1,
        }
    }

    fn start(&mut self) -> Option<PlayerMarker> {
        println!("Starting Game");
        println!("{}", self.meta_board);
        loop {
            self.current_player = (self.current_player + 1) % 2;
            let current_player = if self.current_player % 2 == 0 {
                &mut self.player_one
            } else {
                &mut self.player_two
            };

            let move_ = current_player.get_move(self.meta_board.clone());

            if !self.meta_board.is_valid_move(move_) {
                println!(
                    "Player {} made an invalid move: {}",
                    self.meta_board.current_player,
                    move_.to_string().as_str().red()
                );
                break;
            }

            self.meta_board.set(move_);

            println!("Player {} played {}", self.current_player + 1, move_);
            println!("{}", self.meta_board);

            if self.meta_board.get_possible_moves().len() <= 0 {
                break;
            }
        }

        let winner = self.meta_board.get_winner(&[]);
        match winner {
            Some(PlayerMarker::X) => println!("{}", "Player X won!".green()),
            Some(PlayerMarker::O) => println!("{}", "Player O won!".blue()),
            None => println!("{}", "It's a draw!".yellow()),
        }

        println!("{}", self.meta_board);
        winner
    }
}

fn play_game(
    player_one: impl Player + 'static,
    player_two: impl Player + 'static,
) -> Option<PlayerMarker> {
    let now = std::time::Instant::now();
    let mut game = Game::new(player_one, player_two);
    let winner = game.start();
    println!("Time: {} micro s", now.elapsed().as_micros());
    winner
}

// ############################################################################
// ############################################################################
// # Main Function
// ############################################################################
// ############################################################################

fn main() {
    // let player_one = RandomPlayer { value: Value::X };
    let player_one = MonteCarlo::new();
    let player_two = RandomPlayer::new();

    let mut win1 = 0;
    let mut win2 = 0;
    let mut draw = 0;
    for i in 0..50 {
        println!("Game {}", i + 1);
        let winner = play_game(player_one.clone(), player_two.clone());
        match winner {
            Some(PlayerMarker::X) => win1 += 1,
            Some(PlayerMarker::O) => win2 += 1,
            None => draw += 1,
        }
        // if winner == Some(PlayerMarker::O) {
        //     break;
        // }
    }
    println!(
        "Player 1: {} Player 2: {} Draws {}",
        win1.to_string().as_str().green(),
        win2.to_string().as_str().blue(),
        draw.to_string().as_str().yellow()
    );
}

// ############################################################################
// ############################################################################
// # Tests
// ############################################################################
// ############################################################################

mod tests;

// ######################################
// # BitBoard
// ######################################

#[test]
fn test_bitboard_positions() {
    let mut bitboard = BitBoard::new();

    for i in 0..9 {
        assert_eq!(
            bitboard.get_empty_positions(),
            (i..9).collect::<Vec<usize>>()
        );
        bitboard.set(PlayerMarker::X, i);
    }
    assert!(bitboard.get_empty_positions().is_empty());
}

#[test]
fn test_bitboard_full() {
    let mut bitboard = BitBoard::new();

    for i in 0..9 {
        assert!(!bitboard.is_full());
        bitboard.set(PlayerMarker::X, i);
    }
    assert!(bitboard.is_full());
}

// ######################################
// # MetaMove
// ######################################

#[test]
fn test_meta_move_absolute_index_to_meta() {
    // let absolute_index = [2, 3];
    // let meta_index = MetaMove::absolute_index_to_meta(&absolute_index);
    // assert_eq!(meta_index, 2 * 3 * 3 + 3);

    // for i in 0..BOARD_SIZE_SQUARED {
    //     for j in 0..BOARD_SIZE_SQUARED {
    //         let absolute_index = [i, j];
    //         let meta_index = MetaMove::absolute_index_to_meta(&absolute_index);
    //         println!("{} {} {}", i, j, meta_index);
    //         assert_eq!(meta_index, i * BOARD_SIZE_SQUARED + j);
    //     }
    // }
}

#[test]
fn test_meta_move_meta_to_absolute_index() {
    // let meta_index = 3;
    // let absolute_index = MetaMove::meta_to_absolute_index(meta_index);
    // assert_eq!(absolute_index, [0,3]);

    // for i in 0..META_BOARD_SIZE {
    //     let absolute_index = MetaMove::meta_to_absolute_index(i);
    //     println!("{}: {} {} {}", i, absolute_index[0], absolute_index[1], absolute_index[2]);
    // }
}

#[test]
fn test_meta_move_shift_left() {
    let meta_move = MetaMove::from((8, 3));
    let shifted = meta_move.shift_left();
    assert_eq!(shifted, MetaMove::from((3, 0)));
}

// ######################################
// # MetaBoard
// ######################################

#[test]
fn test_meta_board_empty_positions() {
    let meta_board = MetaBoard::new();
    let empty_positions = meta_board.get_empty_positions();
    assert_eq!(empty_positions.len(), 9 * 9);
}

#[test]
fn test_meta_board_possible_moves() {
    let mut meta_board = MetaBoard::new();
    meta_board.set(MetaMove::from((8, 3)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 9);
}

#[test]
fn test_meta_board_possible_moves_2() {
    let mut meta_board = MetaBoard::new();
    meta_board.set(MetaMove::from((8, 8)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 8);
}

#[test]
fn test_meta_board_possible_moves_3() {
    let mut meta_board = MetaBoard::new();
    meta_board.set(MetaMove::from((8, 0)));
    meta_board.set(MetaMove::from((8, 1)));
    meta_board.set(MetaMove::from((8, 2)));
    meta_board.set(MetaMove::from((8, 3)));
    meta_board.set(MetaMove::from((8, 4)));
    meta_board.set(MetaMove::from((8, 5)));
    meta_board.set(MetaMove::from((8, 6)));
    meta_board.set(MetaMove::from((8, 7)));
    meta_board.set(MetaMove::from((8, 8)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 72);
}

#[test]
fn set_and_unset() {
    let mut meta_board = MetaBoard::new();
    let move_ = MetaMove::from((8, 3));
    meta_board.set(move_);
    assert_eq!(meta_board.boards[8].x, 1 << 3);
    meta_board.unset(Some(MetaMove::from((8, 0))));
    assert_eq!(meta_board.boards[8].x, 0);
}
