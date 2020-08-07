use crate::board::{Board, Choice, Coordinate, JudgeResult};
use rand::Rng;

const WEIGHT_LEN: usize = 11;

#[derive(Clone, Debug)]
pub struct CPU {
    stage1: [i8; WEIGHT_LEN],
    stage2: [i8; WEIGHT_LEN],
    stage3: [i8; WEIGHT_LEN],
    stage4: [i8; WEIGHT_LEN],
}

impl CPU {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let mut stage1 = [0; WEIGHT_LEN];
        let mut stage2 = [0; WEIGHT_LEN];
        let mut stage3 = [0; WEIGHT_LEN];
        let mut stage4 = [0; WEIGHT_LEN];

        for i in 0..WEIGHT_LEN {
            stage1[i] = rng.gen();
            stage2[i] = rng.gen();
            stage3[i] = rng.gen();
            stage4[i] = rng.gen();
        }

        Self {
            stage1,
            stage2,
            stage3,
            stage4,
        }
    }

    pub fn eval_board(&self, board: &Board) -> isize {
        fn mirror(i: usize) -> usize {
            match i {
                0..=3 => i,
                4 => 3,
                5 => 2,
                6 => 1,
                7 => 0,
                _ => unimplemented!(),
            }
        }

        fn co_to_index(co: Coordinate) -> usize {
            let (i, j) = co;
            let (i, j) = (mirror(i), mirror(j));
            let (i, j) = if i >= j { (i, j) } else { (j, i) };

            match (i, j) {
                (0, 0) => 0,
                (1, k) => 1 + k,
                (2, k) => 3 + k,
                (3, k) => 6 + k,
                _ => unimplemented!(),
            }
        }

        let weights = if board.turn < 15 {
            self.stage1
        } else if board.turn < 30 {
            self.stage2
        } else if board.turn < 45 {
            self.stage3
        } else {
            self.stage4
        };

        let mut score = 0;

        for k in 0..64 {
            let (i, j) = (k / 8, k % 8);
            let index = co_to_index((i, j));

            score += match board.player_board & 1 << (63 - k) {
                0 => 0,
                _ => weights[index] as isize,
            }
        }

        score += board.make_legal_board().count_ones() as isize * weights[10] as isize;

        score
    }

    pub fn eval_node(&self, board: &Board, depth: usize, alpha: isize) -> isize {
        if depth == 0 {
            return self.eval_board(board);
        }

        let legal = board.make_legal_board();
        let mut max_score = -(1 << 62);

        if board.is_skip() {
            let mut board_clone = board.clone();
            return match board_clone.update(Choice::Skip).unwrap() {
                JudgeResult::Continue => -self.eval_node(board_clone, depth - 1, max_score),
                JudgeResult::Draw => 0,
                JudgeResult::Win(winner) => {
                    if winner == board.player {
                        1 << 60
                    } else {
                        -(1 << 60)
                    }
                }
            };
        }

        for k in 0..64 {
            if (legal & 1 << (63 - k)) != 0 {
                let (i, j) = (k / 8, k % 8);
                let choice = Choice::Coordinate((i, j));
                let mut board_clone = board.clone();

                match board_clone.update(choice).unwrap() {
                    JudgeResult::Draw => {
                        max_score = (max_score).max(0);
                    }
                    JudgeResult::Win(winner) => {
                        if winner == board.player {
                            return 1 << 60;
                        } else {
                            max_score = (max_score).max(-(1 << 60));
                        }
                    }
                    JudgeResult::Continue => {
                        max_score =
                            (max_score).max(-self.eval_node(&board_clone, depth - 1, max_score))
                    }
                }
            }

            if alpha >= -max_score {
                return alpha;
            }
        }

        max_score
    }
}