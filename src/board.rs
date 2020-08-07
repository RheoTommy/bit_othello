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

    fn coordinate_to_bit(co: Coordinate) -> BitBoard {
        let (i, j) = co;
        1 << (63 - i * 8 - j)
    }

    pub fn make_legal_board(&self) -> BitBoard {
        let horizontal_watch_board = self.opponent_board & 0x7e7e7e7e7e7e7e7e;
        let vertical_watch_board = self.opponent_board & 0x00FFFFFFFFFFFF00;
        let all_side_watch_board = self.opponent_board & 0x007e7e7e7e7e7e00;

        // 空きマス列挙
        let blank_board = !(self.player_board | self.opponent_board);

        let mut tmp;

        let mut legal_board;

        // 左
        tmp = horizontal_watch_board & (self.player_board << 1);
        tmp |= horizontal_watch_board & (tmp << 1);
        tmp |= horizontal_watch_board & (tmp << 1);
        tmp |= horizontal_watch_board & (tmp << 1);
        tmp |= horizontal_watch_board & (tmp << 1);
        tmp |= horizontal_watch_board & (tmp << 1);
        legal_board = blank_board & (tmp << 1);

        // 右
        tmp = horizontal_watch_board & (self.player_board >> 1);
        tmp |= horizontal_watch_board & (tmp >> 1);
        tmp |= horizontal_watch_board & (tmp >> 1);
        tmp |= horizontal_watch_board & (tmp >> 1);
        tmp |= horizontal_watch_board & (tmp >> 1);
        tmp |= horizontal_watch_board & (tmp >> 1);
        legal_board |= blank_board & (tmp >> 1);

        // 上
        tmp = vertical_watch_board & (self.player_board << 8);
        tmp |= vertical_watch_board & (tmp << 8);
        tmp |= vertical_watch_board & (tmp << 8);
        tmp |= vertical_watch_board & (tmp << 8);
        tmp |= vertical_watch_board & (tmp << 8);
        tmp |= vertical_watch_board & (tmp << 8);
        legal_board |= blank_board & (tmp << 8);

        // 下
        tmp = vertical_watch_board & (self.player_board >> 8);
        tmp |= vertical_watch_board & (tmp >> 8);
        tmp |= vertical_watch_board & (tmp >> 8);
        tmp |= vertical_watch_board & (tmp >> 8);
        tmp |= vertical_watch_board & (tmp >> 8);
        tmp |= vertical_watch_board & (tmp >> 8);
        legal_board |= blank_board & (tmp >> 8);

        // 右上
        tmp = all_side_watch_board & (self.player_board << 7);
        tmp |= all_side_watch_board & (tmp << 7);
        tmp |= all_side_watch_board & (tmp << 7);
        tmp |= all_side_watch_board & (tmp << 7);
        tmp |= all_side_watch_board & (tmp << 7);
        tmp |= all_side_watch_board & (tmp << 7);
        legal_board |= blank_board & (tmp << 7);

        // 左上
        tmp = all_side_watch_board & (self.player_board << 9);
        tmp |= all_side_watch_board & (tmp << 9);
        tmp |= all_side_watch_board & (tmp << 9);
        tmp |= all_side_watch_board & (tmp << 9);
        tmp |= all_side_watch_board & (tmp << 9);
        tmp |= all_side_watch_board & (tmp << 9);
        legal_board |= blank_board & (tmp << 9);

        // 右下
        tmp = all_side_watch_board & (self.player_board >> 9);
        tmp |= all_side_watch_board & (tmp >> 9);
        tmp |= all_side_watch_board & (tmp >> 9);
        tmp |= all_side_watch_board & (tmp >> 9);
        tmp |= all_side_watch_board & (tmp >> 9);
        tmp |= all_side_watch_board & (tmp >> 9);
        legal_board |= blank_board & (tmp >> 9);

        // 左下
        tmp = all_side_watch_board & (self.player_board >> 7);
        tmp |= all_side_watch_board & (tmp >> 7);
        tmp |= all_side_watch_board & (tmp >> 7);
        tmp |= all_side_watch_board & (tmp >> 7);
        tmp |= all_side_watch_board & (tmp >> 7);
        tmp |= all_side_watch_board & (tmp >> 7);
        legal_board |= blank_board & (tmp >> 7);

        legal_board
    }

    pub fn is_possible(&self, co: Coordinate) -> bool {
        (Board::coordinate_to_bit(co) & self.make_legal_board()) != 0
    }

    fn reverse(&mut self, put: BitBoard) {
        let mut rev = 0;
        for k in 0..8 {
            let rev_ = 0;
            let mut mask = Board::transfer(put, k);

            while (mask != 0) && ((mask & self.opponent_board) != 0) {
                rev |= mask;
                mask = Board::transfer(mask, k);
            }

            if (mask & self.player_board) != 0 {
                rev |= rev_;
            }
        }

        self.player_board ^= put | rev;
        self.opponent_board ^= rev;
        self.turn_num += 1;
    }

    fn transfer(put: BitBoard, k: usize) -> BitBoard {
        match k {
            0 => (put << 8) & 0xffffffffffffff00,
            1 => (put << 7) & 0x7f7f7f7f7f7f7f00,
            2 => (put >> 1) & 0x7f7f7f7f7f7f7f7f,
            3 => (put >> 9) & 0x007f7f7f7f7f7f7f,
            4 => (put >> 8) & 0x00ffffffffffffff,
            5 => (put >> 7) & 0x00fefefefefefefe,
            6 => (put << 1) & 0xfefefefefefefefe,
            7 => (put << 9) & 0xfefefefefefefe00,
            _ => unimplemented!(),
        }
    }
}

mod tests {
    use crate::board::Board;

    #[test]
    fn coordinate_to_bit_test() {
        let mask = Board::coordinate_to_bit((0, 0));

        assert_eq!(
            0b_10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            mask
        );

        let mask = Board::coordinate_to_bit((5, 4));
        assert_eq!(
            0b_00000000_00000000_00000000_00000000_00000000_00001000_00000000_00000000,
            mask
        );
    }

    #[test]
    fn make_legal_board_test() {
        let default_board = Board::new();
        let legal_board = default_board.make_legal_board();

        assert_eq!(
            0b_00000000_00000000_00010000_00100000_00000100_00001000_00000000_00000000,
            legal_board
        )
    }

    #[test]
    fn is_possible_test() {
        let default_board = Board::new();
        assert!(default_board.is_possible((3, 2)));
        assert!(!default_board.is_possible((0, 0)));
    }

    #[test]
    fn reverse_test() {
        let mut default_board = Board::new();
        default_board.reverse(Board::coordinate_to_bit((3, 2)));
        eprintln!("{:?}", default_board);
        assert_eq!(
            0b_00000000_00000000_00000000_00111000_00010000_00000000_00000000_00000000,
            default_board.player_board
        );
    }
}
