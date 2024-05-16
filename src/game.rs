use std::{error::Error, fmt, ops::{Index, RangeBounds}, vec};

// #############################
// Change these value to change the depth of the game
// The depth of the game is the number of boards that are nested in each other
   const META_DEPTH: usize = 1;
// #############################


// #############################
// #                           #
// #      Fixed Constants      #
// #                           #
// #############################
pub const BOARD_SIZE: usize = 3;
pub const BOARD_SIZE_SQUARED: usize = BOARD_SIZE * BOARD_SIZE;
pub const META_SIZE: usize = BOARD_SIZE_SQUARED.pow(META_DEPTH as u32);
pub const DISPLAY_SIZE: usize = Board::calculate_display_size();
const WINNING_POSITIONS: [u16; 8] = [
    0b111_000_000, 0b000_111_000, 0b000_000_111, // Zeilen
    0b100_100_100, 0b010_010_010, 0b001_001_001, // Spalten
    0b100_010_001, 0b001_010_100, // Diagonalen
];


#[derive(Clone, Copy, PartialEq)]
pub enum PlayerMarker {
    X,
    O,
    Empty,
}

impl PlayerMarker {
    pub fn to_char(&self) -> char {
        match self {
            PlayerMarker::X => 'X',
            PlayerMarker::O => 'O',
            PlayerMarker::Empty => '_',
        }
    }

    pub fn to_other(&self) -> Self {
        match self {
            PlayerMarker::X => PlayerMarker::O,
            PlayerMarker::O => PlayerMarker::X,
            PlayerMarker::Empty => PlayerMarker::Empty,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InvalidMoveError {
    pub message: String,
}

impl fmt::Display for InvalidMoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InvalidMoveError {}

// #############################
// #                           #
// #         MetaMove          #
// #                           #
// #############################

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct MetaMove {
    pub absolute_index: [usize; META_DEPTH],
    index: usize
}

impl MetaMove {
    pub fn new(absolute_index: &[usize]) -> Self {
        if absolute_index.len() > META_DEPTH {
            panic!("Invalid index length");
        }
        MetaMove {
            absolute_index: absolute_index.try_into().unwrap(),
            index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
    }

    pub fn new_empty() -> Self {
        MetaMove {
            absolute_index: [0; META_DEPTH],
            index: 0,
        }
    }

    pub fn push(&mut self, index: usize) {
        if self.index >= META_DEPTH {
            panic!("Index is full");
        }
        self.absolute_index[self.index] = index;
        self.index += 1;
    }

    pub fn pop(&mut self) -> usize {
        self.index -= 1;
        self.absolute_index[self.index]
    }

    pub fn shift_left(&self) -> MetaMove {
        let mut new_index = self.absolute_index;
        new_index.rotate_left(1);
        MetaMove::new(new_index.as_slice())    
    }
}
// #############################
// #                           #
// #       PossibleMoves       #
// #                           #
// #############################

pub struct PossibleMoves {
    moves: [MetaMove; META_SIZE],
    index: usize,
}

impl PossibleMoves {
    pub fn new() -> PossibleMoves {
        PossibleMoves {
            moves : [MetaMove::default(); META_SIZE],
            index : 0,
        }
    }

    pub fn len(&self) -> usize {
        self.index
    }

    pub fn push(&mut self, move_: MetaMove) {
        self.moves[self.index] = move_;
        self.index += 1;
    }

    pub fn clear(&mut self ) {
        self.index = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.index == 0
    }
}

impl<'a> IntoIterator for &'a PossibleMoves {
    type Item = &'a MetaMove;
    type IntoIter = PossibleMovesIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PossibleMovesIterator {
            possible_moves: &self,
            current_index: 0,
        }
    }
}

pub struct PossibleMovesIterator<'a> {
    possible_moves: &'a PossibleMoves,
    current_index: usize,
}

impl<'a> Iterator for PossibleMovesIterator<'a> {
    type Item = &'a MetaMove;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.possible_moves.index {
            let result = &self.possible_moves.moves[self.current_index];
            self.current_index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl RangeBounds<usize> for PossibleMoves {
    fn start_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Included(&0)
    }

    fn end_bound(&self) -> std::ops::Bound<&usize> {
        std::ops::Bound::Excluded(&self.index)
    }
}

impl Index<usize> for PossibleMoves {
    type Output = MetaMove;

    fn index(&self, index: usize) -> &Self::Output {
        &self.moves[index]
    }
}

// #############################
// #                           #
// #         BitBoard          #
// #                           #
// #############################

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BitBoard {
    x: u16,
    o: u16,
}

impl BitBoard {

    pub fn new() -> Self {
        BitBoard {
            x: 0,
            o: 0,
        }
    }


    fn get(&self, index: usize) -> PlayerMarker {
        let mask = 1 << index;
        if self.x & mask != 0 {
            PlayerMarker::X
        } else if self.o & mask != 0 {
            PlayerMarker::O
        } else {
            PlayerMarker::Empty
        }
    }

    fn set(&mut self, index: usize, player: PlayerMarker) -> Result<PlayerMarker, InvalidMoveError> {
        let mask = 1 << index;

        if (self.x | self.o) & mask > 0 {
            return Err(InvalidMoveError { message: String::from("Move was already played") });
        }

        match player {
            PlayerMarker::X => self.x |= mask,
            PlayerMarker::O => self.o |= mask,
            PlayerMarker::Empty => {}
        };

        Ok(self.get_winner())
    }

    fn unset(&mut self, index: &[usize]) {
        if index.len() != 1 {
            return;
        }
        let index = index[0];
        let mask = !(1 << index);
        self.x &= mask;
        self.o &= mask;
    }

    fn get_empty_positions(&self, _index: &[usize], possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) {
        if self.get_winner() != PlayerMarker::Empty {
            return;
        }

        for i in 0..BOARD_SIZE_SQUARED {
            let mask = 1 << i;
            if self.x & mask == 0 && self.o & mask == 0 {
                next_move.push(i);
                possible_moves.push(next_move.clone());
                next_move.pop();
            }
        }
    }

    fn get_winner(&self) -> PlayerMarker {
        for &pos in WINNING_POSITIONS.iter() {
            if self.x & pos == pos {
                return PlayerMarker::X;
            } else if self.o & pos == pos {
                return PlayerMarker::O;
            }
        }
        PlayerMarker::Empty
    }

    fn can_set(&self) -> bool {
        self.get_winner() == PlayerMarker::Empty && (self.x | self.o) != 0b111_111_111
    }
}

// #############################
// #                           #
// #         MetaBoard         #
// #                           #
// #############################

#[derive(Clone, PartialEq)]
pub struct MetaBoard {
    pub board: BitBoard,
    pub sub_boards: Box<[Board; BOARD_SIZE_SQUARED]>,
}

impl MetaBoard {

    fn get(&self, index: &[usize]) -> Result<PlayerMarker, InvalidMoveError> {
        if index.len() <= 1 {
            return Ok(self.board.get(index[0]));
        }
        let sub_board = self.sub_boards.get(index[0]).unwrap();
        sub_board.get(&index[1..])
    }

    fn set(&mut self, index: &[usize], player: PlayerMarker) -> Result<PlayerMarker, InvalidMoveError> {
        if index.len() <= 1 {
            return Err(InvalidMoveError {
                message: "Index is too short".to_string(),
            });
        }

        let spec_index = index[0];

        if self.board.get(spec_index) != PlayerMarker::Empty {
            return Err(InvalidMoveError {
                message: "Board is already won".to_string(),
            });
        }

        let sub_board = self.sub_boards.get_mut(spec_index).unwrap();
        match sub_board.set(&index[1..], player) {
            Ok(marker) => {
                if marker != PlayerMarker::Empty {
                    self.board.set(spec_index, marker)?;
                }
                Ok(self.board.get_winner())
            }
            Err(e) => Err(e),
        }
    }

    fn unset(&mut self, index: &[usize]) {
        if index.len() <= 1 {
            return;
        }
        let spec_index = index[0];
        let sub_board = self.sub_boards.get_mut(spec_index).unwrap();
        sub_board.unset(&index[1..]);
        self.board.unset(&[spec_index]);
    }

    fn get_empty_positions(&self, index: &[usize], possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) {
        if self.get_winner() != PlayerMarker::Empty {
            return;
        }

        if index.is_empty() || self.board.get(index[0]) != PlayerMarker::Empty {
            for (i, sub_board) in self.sub_boards.iter().enumerate() {
                if self.board.get(i) != PlayerMarker::Empty {
                    continue;
                }
    
                next_move.push(i);
                sub_board.get_empty_positions(&[], possible_moves, next_move);
                next_move.pop();
            }
            return;
        }

        let spec_index = index[0];
        let sub_board = self.sub_boards.get(spec_index).unwrap();

        next_move.push(spec_index);
        sub_board.get_empty_positions(&index[1..], possible_moves, next_move);
        next_move.pop();
    }

    fn get_winner(&self) -> PlayerMarker {
        self.board.get_winner()
    }

    fn can_set(&self) -> bool {
        self.get_winner() == PlayerMarker::Empty && self.sub_boards.iter().any(|board| board.can_set())
    }
    
}

// #############################
// #                           #
// #           Board           #
// #                           #
// #############################

#[derive(Clone, PartialEq)]
pub enum Board {
    BitBoard(BitBoard),
    MetaBoard(MetaBoard),
}

impl Board{
    pub fn new() -> Self {
        Board::create_board(META_DEPTH)
    }

    pub fn create_board(depth: usize) -> Self {
        if depth == 1 {
            Board::BitBoard(BitBoard::new())
        } else {
            Board::MetaBoard(MetaBoard {
                board: BitBoard::new(),
                sub_boards: Box::new([(); BOARD_SIZE_SQUARED].map(|_| Board::create_board(depth - 1))),
            })
        }
    }

    fn set(&mut self, meta_move: &[usize], player: PlayerMarker) -> Result<PlayerMarker, InvalidMoveError> {
        if meta_move.len() == 0 {
            return Err(InvalidMoveError {
                message: "Index is empty".to_string(),
            });
        }

        match self {
            Board::MetaBoard(meta_board) => meta_board.set(meta_move, player),
            Board::BitBoard(bit_board) => 
                if meta_move.len() == 1 {
                    bit_board.set(meta_move[0], player)
                } else {
                    Err(InvalidMoveError { message: String::from("Invalid Index") })
                }
        }
    }

    fn unset(&mut self, meta_move: &[usize]) {
        if meta_move.len() == 0 {
            panic!("Index is empty")
        }

        match self {
            Board::BitBoard(bit_board) => bit_board.unset(meta_move),
            Board::MetaBoard(meta_board) => meta_board.unset(meta_move)
        }
    }

    fn get_empty_positions(&self, index: &[usize], possible_moves: &mut PossibleMoves, next_move: &mut MetaMove){
        match self {
            Board::BitBoard(bit_board) => bit_board.get_empty_positions(index, possible_moves, next_move),
            Board::MetaBoard(meta_board) => meta_board.get_empty_positions(index, possible_moves, next_move),
        }
    }

    fn get(&self, index: &[usize]) -> Result<PlayerMarker, InvalidMoveError> {
        if index.len() == 0 {
            panic!("Index is empty")
        }
        match self {
            Board::MetaBoard(meta_board) => meta_board.get(index),
            Board::BitBoard(bit_board) => 
                if index.len() == 1 {
                    Ok(bit_board.get(index[0]))
                } else {
                    Err(InvalidMoveError { message: String::from("Invalid Index") })
                },
        }
    }

    pub fn get_winner(&self) -> PlayerMarker {
        match self {
            Board::BitBoard(bit_board) => bit_board.get_winner(),
            Board::MetaBoard(meta_board) => meta_board.get_winner(),
        }
    }

    pub fn can_set(&self) -> bool {
        match self {
            Board::BitBoard(bit_board) => bit_board.can_set(),
            Board::MetaBoard(meta_board) => meta_board.can_set(),
        }
    }

}

// #############################
// #                           #
// #           Display         #
// #                           #
// #############################

impl Board {
    const fn calculate_display_size() -> usize {
        let mut current = 1;
        let mut index = 0;
        loop {
            if META_DEPTH == index {
                return current;
            }
            
            if let Some(val) = current.checked_mul(BOARD_SIZE) {
                if let Some(val2) = val.checked_add(index * 2) {
                    current = val2;
                } else {
                    panic!("overflow");
                }
            } else {
                panic!("overflow");
            }

            index += 1;
        }
    }

    fn fill_board(&self, array: &mut [Vec<char>], top_left: (usize, usize), depth: usize, display_size: usize) {
        match self {
            Board::BitBoard(bitboard) => {
                bitboard.fill_board(array, top_left)
            },
            Board::MetaBoard(metaboard) => {
                metaboard.fill_board(array, top_left, depth, display_size)
            },
        }
    }
}

impl BitBoard {
    fn fill_board(&self, array: &mut [Vec<char>], (top, left): (usize, usize)) {
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let pos = i * BOARD_SIZE + j;
                let mask = 1 << pos;
                let symbol = if self.x & mask != 0 {
                    'X'
                } else if self.o & mask != 0 {
                    'O'
                } else {
                    '-'
                };
                array[top + i][left + j] = symbol;
            }
        }
    }
}

impl MetaBoard {
    fn fill_board(&self, array: &mut [Vec<char>], (top, left): (usize, usize), depth: usize, display_size: usize) {
        let sub_size = (display_size - depth * 2) / BOARD_SIZE;
        for i in 0..BOARD_SIZE {
            for j in 0..BOARD_SIZE {
                let index: usize = i * BOARD_SIZE + j;
                let sub_top = top + i * sub_size + i * depth;
                let sub_left = left + j * sub_size + j * depth;

                if self.board.get(index) != PlayerMarker::Empty {
                    let symbol = self.board.get(index).to_char();

                    array[sub_top][sub_left] = symbol;
                    array[sub_top][sub_left + sub_size] = symbol;
                    array[sub_top + sub_size][sub_left + sub_size] = symbol;
                    array[sub_top + sub_size][sub_left] = symbol;
                    array[sub_top + sub_size / 2][sub_left + sub_size / 2] = symbol;
                } else {
                    self.sub_boards[index].fill_board(array, (sub_top, sub_left), depth - 1, sub_size);
                }

                // self.sub_boards[index].fill_board(array, (sub_top, sub_left), depth - 1, sub_size);
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut array = vec![vec![' '; DISPLAY_SIZE]; DISPLAY_SIZE];
        self.fill_board(&mut array, (0, 0), META_DEPTH, DISPLAY_SIZE);

        for line in array {
            for char in line {
                write!(f, "{}", char)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// #############################
// #                           #
// #        GameState          #
// #                           #
// #############################

#[derive(Clone)]
pub struct GameState {
    pub board: Board,
    pub current_player: PlayerMarker,
    pub last_move: Option<MetaMove>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: Board::new(),
            current_player: PlayerMarker::X,
            last_move: None,
        }
    }

    pub fn get_winner(&self) -> PlayerMarker {
        self.board.get_winner()
    }

    pub fn set(&mut self, meta_move: MetaMove) -> Result<PlayerMarker, InvalidMoveError> {

        match self.board.set(meta_move.absolute_index.as_slice(), self.current_player){
            Ok(marker) => {
                self.current_player = self.current_player.to_other();
                self.last_move = Some(meta_move);
                return Ok(marker);
            }
            Err(e) => return Err(e),
        }
    }

    pub fn unset(&mut self, previous_move: Option<MetaMove>) {
        if let Some(last_move) = &self.last_move {
            self.board.unset(last_move.absolute_index.as_slice());
            self.current_player = self.current_player.to_other();
            self.last_move = previous_move;
        }
    }

    pub fn get_possible_moves(&self, possible_moves: &mut PossibleMoves, next_move: &mut MetaMove) {
        
        let mut next_index: &[usize] = &[];
        let temp;
        if let Some(last_move) = &self.last_move {
            temp = last_move.shift_left();
            next_index = temp.absolute_index.as_slice();
        }
        
        possible_moves.clear();
        next_move.clear();
        self.board.get_empty_positions(next_index, possible_moves, next_move);
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.board.fmt(f)
    }
}
