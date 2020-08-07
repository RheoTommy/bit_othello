use std::fmt::{Debug, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Turn {
    Black,
    White,
}

pub type BitBoard = usize;
pub type Coordinate = (usize, usize);
pub type ICoordinate = (isize, isize);

#[derive(Clone, Eq, PartialEq)]
pub struct Board {
    pub turn_num: usize,
    pub turn: Turn,
    pub player_board: BitBoard,
    pub opponent_board: BitBoard,
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let turn = self.turn;
        let (black, white) = match turn {
            Turn::Black => (self.player_board, self.opponent_board),
            Turn::White => (self.opponent_board, self.player_board),
        };

        let mut str = " |0|1|2|3|4|5|6|7|\n".to_string();

        for i in 0..8 {
            let mut s = format!("{}|", i);

            for j in 0..8 {
                let piece = if (black & 1 << (63 - i * 8 - j)) != 0 {
                    '○'
                } else if (white & 1 << (63 - i * 8 - j)) != 0 {
                    '●'
                } else {
                    ' '
                };

                s.push(piece);
                s.push('|');
            }

            str.push_str(&s);
            str.push_str("\n");
        }

        write!(f, "{}", str)
    }
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

    pub fn coordinate_to_bit(co: Coordinate) -> BitBoard {
        let mut mask = 0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
        let (i, j) = co;

        mask = mask >> (i * 8 + j);
        mask
    }
}
