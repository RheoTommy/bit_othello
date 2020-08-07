#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Turn {
    Black,
    White,
}

type BitBoard = usize;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Board {
    pub turn_num: usize,
    pub turn: Turn,
    pub player_board: BitBoard,
    pub opponent_board: BitBoard,
}

impl Board {
    pub fn new() -> Self {
        Self {
            turn_num: 1,
            turn: Turn::Black,
            player_board: 0x0000000810000000,
            opponent_board: 0x0000001008000000,
        }
    }
}
