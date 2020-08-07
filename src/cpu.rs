use crate::board::{Board, Choice, Coordinate, JudgeResult, Player};
use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};

const WEIGHT_LEN: usize = 11;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CPU {
    pub stage1: [i8; WEIGHT_LEN],
    pub stage2: [i8; WEIGHT_LEN],
    pub stage3: [i8; WEIGHT_LEN],
    pub stage4: [i8; WEIGHT_LEN],
}

impl CPU {
    pub fn log(&self, log_file: &mut File) -> std::io::Result<()> {
        let str = serde_json::to_string(self)?;
        log_file.write_all(format!("\n{}", str).as_bytes())
    }

    pub fn from_log_file(log_file: &mut File) -> std::io::Result<Self> {
        let mut buf = String::new();
        log_file.read_to_string(&mut buf)?;
        let buf = buf.lines().last().unwrap_or("Invalid");
        let cpu = serde_json::from_str(&buf)?;
        Ok(cpu)
    }

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
        let mut max_score = -(1 << 61);

        if board.is_skip() {
            let mut board_clone = board.clone();
            return match board_clone.update(Choice::Skip).unwrap() {
                JudgeResult::Continue => -self.eval_node(&board_clone, depth - 1, max_score),
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

    pub fn choose_best(&self, board: &Board, depth: usize) -> Choice {
        if board.is_skip() {
            return Choice::Skip;
        }

        let legal = board.make_legal_board();
        let mut max_score = -(1 << 62);
        let mut best_choice = Choice::Skip;

        for k in 0..64 {
            if (legal & 1 << (63 - k)) != 0 {
                let (i, j) = (k / 8, k % 8);
                let choice = Choice::Coordinate((i, j));
                let mut board_clone = board.clone();

                match board_clone.update(choice).unwrap() {
                    JudgeResult::Draw => {
                        if max_score < 0 {
                            max_score = 0;
                            best_choice = choice;
                        }
                    }
                    JudgeResult::Win(winner) => {
                        if winner == board.player {
                            return choice;
                        }

                        if max_score < -(1 << 60) {
                            max_score = -(1 << 60);
                            best_choice = choice;
                        }
                    }
                    JudgeResult::Continue => {
                        let next_score = -self.eval_node(&board_clone, depth - 1, max_score);

                        if max_score < next_score {
                            max_score = next_score;
                            best_choice = choice;
                        }
                    }
                }
            }
        }

        best_choice
    }
}

pub fn eval_cpu<'a>(black: &'a CPU, white: &'a CPU, depth: usize) -> (&'a CPU, usize, usize) {
    let mut board = Board::new();
    let mut winner = black;

    loop {
        let next = if board.player == Player::Black {
            black.choose_best(&board, depth)
        } else {
            white.choose_best(&board, depth)
        };

        match board.update(next).unwrap() {
            JudgeResult::Continue => continue,
            JudgeResult::Win(w) => {
                winner = if w == Player::Black { black } else { white };
                break;
            }
            _ => break,
        }
    }

    let (b, w) = board.calc_now_score();
    (winner, b as usize, w as usize)
}

pub fn cross_cpu(left: &CPU, right: &CPU, cross_prob: f64, rng: &mut impl Rng) -> CPU {
    if rng.gen::<f64>() >= cross_prob {
        return left.clone();
    }

    let mut stage1 = [0; WEIGHT_LEN];
    let mut stage2 = [0; WEIGHT_LEN];
    let mut stage3 = [0; WEIGHT_LEN];
    let mut stage4 = [0; WEIGHT_LEN];

    let i = rng.gen_range(0, WEIGHT_LEN);
    let j = rng.gen_range(i, WEIGHT_LEN);

    for k in 0..WEIGHT_LEN {
        if k < i {
            stage1[k] = left.stage1[k];
            stage2[k] = left.stage2[k];
            stage3[k] = left.stage3[k];
            stage4[k] = left.stage4[k];
        } else if k < j {
            stage1[k] = right.stage1[k];
            stage2[k] = right.stage2[k];
            stage3[k] = right.stage3[k];
            stage4[k] = right.stage4[k];
        } else {
            stage1[k] = left.stage1[k];
            stage2[k] = left.stage2[k];
            stage3[k] = left.stage3[k];
            stage4[k] = left.stage4[k];
        }
    }

    CPU {
        stage1,
        stage2,
        stage3,
        stage4,
    }
}

pub fn mutate_cpu(cpu: &mut CPU, mutate_prob: f64, rng: &mut impl Rng) {
    if rng.gen::<f64>() < mutate_prob {
        for i in 0..WEIGHT_LEN {
            cpu.stage1[i] = cpu.stage1[i].wrapping_add(rng.gen());
            cpu.stage2[i] = cpu.stage2[i].wrapping_add(rng.gen());
            cpu.stage3[i] = cpu.stage3[i].wrapping_add(rng.gen());
            cpu.stage4[i] = cpu.stage4[i].wrapping_add(rng.gen());
        }
    }
}
