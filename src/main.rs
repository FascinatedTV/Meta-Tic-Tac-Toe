use std::{fmt::Display, iter, usize};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const BOARD_SIZE: usize = 3;
const BOARD_SIZE_SQUARED: usize = usize::pow(BOARD_SIZE, 2);
const META_BOARD_DEPTH: usize = 2; // 0 = 3x3, 1 = 9x9, 2 = 27x27

const META_BOARD_SIZE: usize = usize::pow(BOARD_SIZE_SQUARED, META_BOARD_DEPTH as u32);
const META_BOARD_SIDE: usize = usize::pow(BOARD_SIZE, META_BOARD_DEPTH as u32);


const WINNING_POSITIONS: [u16; 8] = [
    0b111_000_000, 0b000_111_000, 0b000_000_111, // Zeilen
    0b100_100_100, 0b010_010_010, 0b001_001_001, // Spalten
    0b100_010_001, 0b001_010_100, // Diagonalen
];

#[derive(Clone, Copy, PartialEq)]
enum Value {
    X,
    O,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Value::X => "X",
            Value::O => "O",
        })
    }
}

impl Distribution<Value> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Value {
        if rng.gen() {
            Value::X
        } else {
            Value::O
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
        BitBoard {o: 0, x: 0}
    }

    fn set(&mut self, player: Value, position: usize) {
        let mask = 1 << position;
        match player {
            Value::X => self.x |= mask,
            Value::O => self.o |= mask,
        }
    }

    fn is_full(&self) -> bool {
        (self.x | self.o) == 0b111_111_111
    }

    fn can_set(&self) -> bool {
        !self.is_full() && self.get_winner().is_none()
    }

    fn get_empty_positions(&self) -> Vec<usize> {
        if !self.can_set(){
            return vec![];
        }
        iter::successors(Some(0), move |&i| Some(i + 1))
            .take(BOARD_SIZE_SQUARED)
            .filter(move |&i| (self.x | self.o) & (1 << i) == 0)
            .collect()
    }

    fn get_winner(&self) -> Option<Value> {
        let x = self.x;
        let o = self.o;
        for &winning_position in WINNING_POSITIONS.iter() {
            if x & winning_position == winning_position {
                return Some(Value::X);
            }
            if o & winning_position == winning_position {
                return Some(Value::O);
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
#[derive(Clone, Copy, Debug, PartialEq)]
struct MetaMove {
    absolute_index: [usize; META_BOARD_DEPTH],
    meta_index: usize,
    board_index: usize,
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
        absolute_index.iter().fold(0, |acc, &index| {
            acc * BOARD_SIZE * BOARD_SIZE + index
        })
    }

    fn meta_to_absolute_index(meta_index: usize) -> [usize; META_BOARD_DEPTH] {
        let mut absolute_index = [0; META_BOARD_DEPTH];
        let mut meta_index = meta_index;
        for i in 0..META_BOARD_DEPTH {
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
    last_move: Option<(Value, MetaMove)>,
    boards: [BitBoard; META_BOARD_SIZE],
}


impl MetaBoard {
    fn new() -> Self {
        MetaBoard {
            boards: [BitBoard::new(); META_BOARD_SIZE],
            last_move: None,
        }
    }

    fn set(&mut self, player: Value, meta_move: MetaMove) {
        self.boards[meta_move.meta_index].set(player, meta_move.board_index);
        self.last_move = Some((player, meta_move));
    }

    

    fn get_empty_positions(&self) -> Vec<MetaMove> {
        self.boards.iter().enumerate().flat_map(|(meta_index, board)| {
            board.get_empty_positions().into_iter().map(move |board_index| From::from((meta_index, board_index)))
        }).collect()
    }

    fn get_winner(&self, index: &[usize]) -> Option<Value> {
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
                Value::X => board_x |= 1 << i,
                Value::O => board_o |= 1 << i,
            }
        }

        let board = BitBoard { x: board_x, o: board_o };
        board.get_winner()
    }

    fn get_possible_moves(&self) -> Vec<MetaMove> {
        if self.get_winner(&[]).is_some() {
            return vec![];
        }

        match self.last_move {
            Some((_, last_move)) => {
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
                        if self.get_winner(&last_move.absolute_index[..i]).is_some() || i == META_BOARD_DEPTH {
                            break;
                        }
                        i += 1;
                    }

                    if i == 0 {
                        return vec![];
                    }

                    for i in (0..i).rev() {
                        let start = MetaMove::absolute_index_to_meta(&last_move.absolute_index[..i]);
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
            },
            None => self.get_empty_positions(),
        }
    }

    fn join_vector(arr: &[String], join_char:&str, depth:usize) -> String {
        if depth == 0 {
            return arr[0].clone();
        }

        let mut result = String::new();
        let seg = arr.len() / 3;

        result.push_str(
            &MetaBoard::join_vector(&arr[..seg], join_char, depth - 1)
        );
        result.push_str(&join_char.to_string().repeat(depth));
        result.push_str(
            &MetaBoard::join_vector(&arr[seg..2*seg], join_char, depth - 1)
        );
        result.push_str(&join_char.to_string().repeat(depth));
        result.push_str(
            &MetaBoard::join_vector(&arr[2*seg..3*seg], join_char, depth - 1)
        );

        result
    }
}

impl Display for MetaBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut row_seperator = String::new();
        row_seperator.push('-');
        let vec = vec![String::from("---------"); 3 * META_BOARD_SIDE * (BOARD_SIZE + 1)];
        row_seperator.push_str(&MetaBoard::join_vector(vec.as_slice(), "+", META_BOARD_DEPTH));
        row_seperator.push('\n');

        let mut row_array: Vec<String> = vec![];
        for meta_row in 0..META_BOARD_SIDE {
            let mut col_string = String::new();
            for row in 0..BOARD_SIZE {
                col_string.push(' ');
                let mut col_array: Vec<String> = vec![];

                for meta_col in 0..META_BOARD_SIDE {
                    let mut part = String::new();
                    let board = &self.boards[meta_row * META_BOARD_SIDE + meta_col];
                    let bit_row = board.get_row(row);
                    for col in 0..BOARD_SIZE {
                        part.push(' ');
                        part.push(
                            match board.get_winner() {
                                Some(Value::X) => 'X',
                                Some(Value::O) => 'O',
                                None => bit_row[col],
                            }
                        );
                        part.push(' ');
                    }
                    col_array.push(part);
                }
                col_string.push_str(&MetaBoard::join_vector(&col_array, "|", META_BOARD_DEPTH));
                col_string.push('\n');
            }
            row_array.push(col_string);
        }

        write!(f, "{}", &MetaBoard::join_vector(row_array.as_slice(), &row_seperator, META_BOARD_DEPTH))
    }
}

// ######################################
// Player
// ######################################

trait Player {
    fn get_value(&self) -> Value;
    fn get_move(&self, meta_board: &MetaBoard) -> MetaMove;
}

struct RandomPlayer {
    value: Value,
}

impl Player for RandomPlayer {
    fn get_value(&self) -> Value {
        self.value
    }

    fn get_move(&self, meta_board: &MetaBoard) -> MetaMove {
        let possible_moves = meta_board.get_possible_moves();
        let mut rng = rand::thread_rng();
        possible_moves[rng.gen_range(0..possible_moves.len())]
    }
}

// struct MiniMaxPlayer {
//     value: Value,
//     depth: usize,
// }

struct MonteCarloPlayer {
    value: Value,
    depth: usize,
}

// impl Player for MonteCarloPlayer {
//     fn get_value(&self) -> Value {
//         self.value
//     }

//     fn get_move(&self, meta_board: &MetaBoard) -> MetaMove {
//         let mut rng = rand::thread_rng();
//         let mut best_move = None;
//         let mut best_score = std::i32::MIN;
//         let mut scores = vec![0; meta_board.get_possible_moves().len()];
//         for i in 0..1000 {
//             let mut meta_board_copy = meta_board.clone();
//             let mut current_player = self.value;
//             let mut current_move = None;
//             let mut current_score = 0;
//             loop {
//                 let possible_moves = meta_board_copy.get_possible_moves();
//                 if possible_moves.is_empty() {
//                     break;
//                 }
//                 let move_ = possible_moves[rng.gen_range(0..possible_moves.len())];
//                 meta_board_copy.set(current_player, move_);
//                 let winner = meta_board_copy.get_winner(&[]);
//                 if winner.is_some() {
//                     current_score = match winner.unwrap() {
//                         Value::X => 1,
//                         Value::O => -1,
//                     };
//                     break;
//                 }
//                 current_player = match current_player {
//                     Value::X => Value::O,
//                     Value::O => Value::X,
//                 };
//             }
//             scores[i % scores.len()] = current_score;
//             let score = scores.iter().sum();
//             if score > best_score {
//                 best_score = score;
//                 best_move = current_move;
//             }
//         }
//         best_move.unwrap()
//     }

// }

struct HumanPlayer {
    value: Value,
}

impl Player for HumanPlayer {
    fn get_value(&self) -> Value {
        self.value
    }

    fn get_move(&self, meta_board: &MetaBoard) -> MetaMove {
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
                println!("Invalid input. Please enter a number between 0 and {}.", possible_moves.len() - 1);
            }
        }
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
            current_player: rand::random()
        }
    }

    fn start(&mut self) {
        println!("Starting Game");
        println!("{}", self.meta_board);
        loop {
            let current_player = if self.current_player % 2 == 0 {
                &self.player_one
            } else {
                &self.player_two
            };

            let move_ = current_player.get_move(&self.meta_board);
            self.meta_board.set(current_player.get_value(), move_);
            
            println!("Player {} played {}", current_player.get_value(), move_);
            println!("{}", self.meta_board);

            if self.meta_board.get_possible_moves().len() <= 0 {
                break;
            }
            self.current_player = (self.current_player + 1) % 2;
        }

        match self.meta_board.get_winner(&[]) {
            Some(Value::X) => println!("Player X won!"),
            Some(Value::O) => println!("Player O won!"),
            None => println!("It's a draw!"),
        }
    }
}

// ############################################################################
// ############################################################################
// # Main Function
// ############################################################################
// ############################################################################

fn main() {
    // let mut meta_board = MetaBoard::new();
    // meta_board.set(Value::X, From::from((8, 8)));
    // print!("{}", meta_board);
    // let possible_moves = meta_board.get_possible_moves();
    // for move_ in possible_moves {
    //     println!("{}", move_);
    // }

    let now = std::time::Instant::now();
    let player_one = RandomPlayer { value: Value::X };
    let player_two = RandomPlayer { value: Value::O };
    let mut game = Game::new(player_one, player_two);
    game.start();

    println!("Time: {} micro s", now.elapsed().as_micros());
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
        assert_eq!(bitboard.get_empty_positions(), (i..9).collect::<Vec<usize>>());
        bitboard.set(Value::X, i);
    }
    assert!(bitboard.get_empty_positions().is_empty());
}

#[test]
fn test_bitboard_full() {
    let mut bitboard = BitBoard::new();

    for i in 0..9 {
        assert!(!bitboard.is_full());
        bitboard.set(Value::X, i);
    }
    assert!(bitboard.is_full());
}

// ######################################
// # MetaMove
// ######################################

#[test]
fn test_meta_move_absolute_index_to_meta() {
    let absolute_index = [2, 3];
    let meta_index = MetaMove::absolute_index_to_meta(&absolute_index);
    assert_eq!(meta_index, 2 * 3 * 3 + 3);
}

// #[test]
// fn test_meta_move_meta_to_absolute_index() {
//     let meta_index = 3;
//     let absolute_index = MetaMove::meta_to_absolute_index(meta_index);
//     assert_eq!(absolute_index, [3,3]);
// }

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
    meta_board.set(Value::X, MetaMove::from((8, 3)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 9);
}

#[test]
fn test_meta_board_possible_moves_2() {
    let mut meta_board = MetaBoard::new();
    meta_board.set(Value::X, MetaMove::from((8, 8)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 8);
}

#[test]
fn test_meta_board_possible_moves_3() {
    let mut meta_board = MetaBoard::new();
    meta_board.set(Value::X, MetaMove::from((8, 0)));
    meta_board.set(Value::X, MetaMove::from((8, 1)));
    meta_board.set(Value::X, MetaMove::from((8, 2)));
    meta_board.set(Value::X, MetaMove::from((8, 3)));
    meta_board.set(Value::X, MetaMove::from((8, 4)));
    meta_board.set(Value::X, MetaMove::from((8, 5)));
    meta_board.set(Value::X, MetaMove::from((8, 6)));
    meta_board.set(Value::X, MetaMove::from((8, 7)));
    meta_board.set(Value::X, MetaMove::from((8, 8)));
    let possible_moves = meta_board.get_possible_moves();
    assert_eq!(possible_moves.len(), 72);
}