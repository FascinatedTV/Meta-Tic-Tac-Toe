const BOARD_SIZE: usize = 3;
const BOARD_SIZE_SQUARED: usize = BOARD_SIZE * BOARD_SIZE;
const META_DEPTH: usize = 2;
const WINNING_POSITIONS: [u16; 8] = [
    0b111_000_000, 0b000_111_000, 0b000_000_111, // Zeilen
    0b100_100_100, 0b010_010_010, 0b001_001_001, // Spalten
    0b100_010_001, 0b001_010_100, // Diagonalen
];

#[derive(Clone, Copy, PartialEq)]
enum Player {
    X,
    O,
    Empty,
}

impl Player {
    fn to_char(&self) -> char {
        match self {
            Player::X => 'X',
            Player::O => 'O',
            Player::Empty => '_',
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct BitBoard {
    x: u16,
    o: u16,
}

impl BitBoard {
    fn new() -> Self {
        BitBoard {
            x: 0,
            o: 0,
        }
    }

    fn get(&self, index: usize) -> Player {
        let mask = 1 << index;
        if self.x & mask != 0 {
            Player::X
        } else if self.o & mask != 0 {
            Player::O
        } else {
            Player::Empty
        }
    }

    fn set(&mut self, index: usize, player: Player) {
        let mask = 1 << index;
        match player {
            Player::X => self.x |= mask,
            Player::O => self.o |= mask,
            Player::Empty => {
                self.x &= !mask;
                self.o &= !mask;
            }
        }
    }

    fn is_winning(&self, player: Player) -> bool {
        let mask = match player {
            Player::X => self.x,
            Player::O => self.o,
            Player::Empty => 0,
        };
        WINNING_POSITIONS.iter().any(|&winning_position| mask & winning_position == winning_position)
    }
}

#[derive(Clone, PartialEq)]
enum Board {
    BitBoard(BitBoard),
    MetaBoard(Box<[Board; BOARD_SIZE_SQUARED]>),
}

impl From<BitBoard> for Board {
    fn from(bit_board: BitBoard) -> Self {
        Board::BitBoard(bit_board)
    }
}

impl From<Box<[Board; BOARD_SIZE_SQUARED]>> for Board {
    fn from(meta_board: Box<[Board; BOARD_SIZE_SQUARED]>) -> Self {
        Board::MetaBoard(meta_board)
    }
}

impl Board {

    fn new(depth: usize) -> Self {
        if depth == 0 {
            Board::BitBoard(BitBoard::new())
        } else {
            let mut meta_board = Vec::with_capacity(BOARD_SIZE_SQUARED);
            for _ in 0..BOARD_SIZE_SQUARED {
                meta_board.push(Board::new(depth - 1));
            }
            Board::MetaBoard(meta_board)
        }
    }

    fn get(&self, index: &[usize]) -> Player {
        match self {
            Board::BitBoard(bit_board) => bit_board.get(index[0]),
            Board::MetaBoard(meta_board) => meta_board[index[0]].get(&index[1..]),
        }
    }

    fn set(&mut self, index: usize, player: Player) {
        match self {
            Board::BitBoard(bit_board) => bit_board.set(index, player),
            Board::MetaBoard(meta_board) => {
                for board in meta_board.iter_mut() {
                    board.set(index, player);
                }
            }
        }
    }

    fn is_winning(&self, player: Player) -> bool {
        match self {
            Board::BitBoard(bit_board) => bit_board.is_winning(player),
            Board::MetaBoard(meta_board) => {
                for board in meta_board.iter() {
                    if board.is_winning(player) {
                        return true;
                    }
                }
                false
            }
        }
    }
}






fn main() {

}