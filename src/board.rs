use crate::board::Player::Black;
use std::fmt::{Debug, Formatter};
use std::mem::swap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn next(&self) -> Self {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

pub type BitBoard = usize;
pub type Coordinate = (usize, usize);
pub type ICoordinate = (isize, isize);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum JudgeResult {
    Continue,
    Draw,
    Win(Player),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Choice {
    Skip,
    Coordinate(Coordinate),
}

#[derive(Clone, Eq, PartialEq)]
pub struct Board {
    pub turn: usize,
    pub player: Player,
    pub player_board: BitBoard,
    pub opponent_board: BitBoard,
}

impl Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let turn = self.player;
        let (black, white) = match turn {
            Player::Black => (self.player_board, self.opponent_board),
            Player::White => (self.opponent_board, self.player_board),
        };

        let legal = self.make_legal_board();

        let dialog = format!("<Turn: {}>\nNext Player: {:?}", self.turn, self.player);

        let mut str = " |0|1|2|3|4|5|6|7|\n".to_string();

        for i in 0..8 {
            let mut s = format!("{}|", i);

            for j in 0..8 {
                let piece = if (black & 1 << (63 - i * 8 - j)) != 0 {
                    '○'
                } else if (white & 1 << (63 - i * 8 - j)) != 0 {
                    '●'
                } else if (legal & 1 << (63 - i * 8 - j)) != 0 {
                    '.'
                } else {
                    ' '
                };

                s.push(piece);
                s.push('|');
            }

            str.push_str(&s);
            str.push_str("\n");
        }

        write!(f, "{}\n{}", dialog, str)
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            turn: 1,
            player: Player::Black,
            player_board: 0x0000000810000000,
            opponent_board: 0x0000001008000000,
        }
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

    fn coordinate_to_bit(co: Coordinate) -> BitBoard {
        let (i, j) = co;
        1 << (63 - i * 8 - j)
    }

    pub fn is_possible(&self, put: BitBoard) -> bool {
        (put & self.make_legal_board()) != 0
    }

    fn skip(&mut self) {
        self.turn += 1;
        self.player = self.player.next();
        swap(&mut self.opponent_board, &mut self.player_board);
    }

    pub fn update(&mut self, choice: Choice) -> Result<JudgeResult, &'static str> {
        if let Choice::Skip = choice {
            self.skip();
            return Ok(JudgeResult::Continue);
        }

        let co = match choice {
            Choice::Skip => unreachable!(),
            Choice::Coordinate(co) => co,
        };

        let (i, j) = co;
        let put = Board::coordinate_to_bit(co);

        if !(i < 8 && j < 8) {
            return Err("Out of index!");
        }

        if !self.is_possible(put) {
            return Err("Impposible Choice!");
        }

        self.reverse(put);
    
        swap(&mut self.opponent_board, &mut self.player_board);
        self.player = self.player.next();
        self.turn += 1;

        if self.is_game_finished() {
            let (player, opponent) = self.calc_now_score();
            return if player == opponent {
                Ok(JudgeResult::Draw)
            } else if player > opponent {
                Ok(JudgeResult::Win(self.player))
            } else {
                Ok(JudgeResult::Win(self.player.next()))
            };
        }

        Ok(JudgeResult::Continue)
    }

    fn reverse(&mut self, put: BitBoard) {
        let mut rev = 0;
        for k in 0..8 {
            let mut rev_ = 0;
            let mut mask = Board::transfer(put, k);

            while (mask != 0) && ((mask & self.opponent_board) != 0) {
                rev_ |= mask;
                mask = Board::transfer(mask, k);
            }

            if (mask & self.player_board) != 0 {
                rev |= rev_;
            }
        }

        self.player_board ^= put | rev;
        self.opponent_board ^= rev;
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

    pub fn is_skip(&self) -> bool {
        let player_legal_board = self.make_legal_board();

        let opponent_board = Self {
            player: Black,
            turn: 0,
            player_board: self.opponent_board,
            opponent_board: self.player_board,
        };

        let opponent_legal_board = opponent_board.make_legal_board();

        player_legal_board == 0x0000000000000000 && opponent_legal_board != 0x0000000000000000
    }

    fn is_game_finished(&self) -> bool {
        let player_legal_board = self.make_legal_board();

        let opponent_board = Self {
            player: Black,
            turn: 0,
            player_board: self.opponent_board,
            opponent_board: self.player_board,
        };

        let opponent_legal_board = opponent_board.make_legal_board();

        player_legal_board == 0x0000000000000000 && opponent_legal_board == 0x0000000000000000
    }

    pub fn calc_now_score(&self) -> (u32, u32) {
        let player_num = self.player_board.count_ones();
        let opponent_num = self.opponent_board.count_ones();

        (player_num, opponent_num)
    }
}
