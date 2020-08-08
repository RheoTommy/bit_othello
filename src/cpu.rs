use crate::board::{Board, Choice, Coordinate, JudgeResult, Player};
use rand::Rng;
use std::fs::File;
use std::io::{Read, Write};

const WEIGHT_LEN: usize = 11;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct CPU {
    pub stage1: [i8; WEIGHT_LEN],
    pub stage2: [i8; WEIGHT_LEN],
    pub stage3: [i8; WEIGHT_LEN],
    pub stage4: [i8; WEIGHT_LEN],
}

impl CPU {
    /// ファイルに改行を加えて出力
    pub fn log(&self, log_file: &mut File) -> std::io::Result<()> {
        let str = serde_json::to_string(self)?;
        log_file.write_all(format!("\n{}", str).as_bytes())?;
        log_file.flush()
    }

    /// ファイルから文字列として入力を受け取り、最後の行をパースして返す
    pub fn from_log_file(log_file: &mut File) -> std::io::Result<Self> {
        let mut buf = String::new();
        log_file.read_to_string(&mut buf)?;
        let buf = buf.lines().last().unwrap_or("Invalid"); // "Invalid"ならパースに失敗するので
        let cpu = serde_json::from_str(&buf)?;
        Ok(cpu)
    }

    /// 乱数生成器を受け取り、ランダムに値を決めたCPUをつくる
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

    /// そこそこ強いであろう値を持つCPU
    pub fn new_alpha() -> Self {
        // let weight = [120, -20, -40, 20, -5, 15, 5, -5, 3, 3, 0];
        let weight = [120, -12, -15, 0, -3, 0, -1, -3, -1, -1, 0];

        Self {
            stage1: weight,
            stage2: weight,
            stage3: weight,
            stage4: weight,
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

            // 自分の駒なら加点
            score += match board.player_board & 1 << (63 - k) {
                0 => 0,
                _ => weights[index] as isize,
            };

            // 敵の駒なら減点
            score -= match board.opponent_board & 1 << (63 - k) {
                0 => 0,
                _ => weights[index] as isize,
            };
        }

        // 次に打てる手の数に応じて加点
        score += board.make_legal_board().count_ones() as isize * weights[10] as isize;

        score
    }

    pub fn eval_node(&self, board: &Board, depth: usize, alpha: isize) -> isize {
        if depth == 0 {
            return self.eval_board(board);
        }

        if board.is_skip() {
            let mut board_clone = board.clone();
            return match board_clone.update(Choice::Skip).unwrap() {
                JudgeResult::Win(winner) => {
                    if winner == board.player {
                        1 << 60
                    } else {
                        -(1 << 60)
                    }
                }
                JudgeResult::Draw => 0,
                JudgeResult::Continue => -self.eval_node(&board_clone, depth - 1, -(1 << 62)),
            };
        }

        let legal = board.make_legal_board();
        let mut max_score = -(1 << 61);

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
                        let opponent_score = self.eval_node(&board_clone, depth - 1, max_score);

                        max_score = (max_score).max(-opponent_score);
                    }
                }

                // すでに確定した最低点より低い点数を含むNodeは枝刈り
                if alpha >= -max_score {
                    return max_score;
                }
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
    let (b, w) = if board.player == Player::Black {
        (b, w)
    } else {
        (w, b)
    };
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

pub fn cross_cpu_alpha(left: &CPU, right: &CPU, l: usize, r: usize, rng: &mut impl Rng) -> CPU {
    let mut stage1 = [0; WEIGHT_LEN];
    let mut stage2 = [0; WEIGHT_LEN];
    let mut stage3 = [0; WEIGHT_LEN];
    let mut stage4 = [0; WEIGHT_LEN];

    for k in 0..WEIGHT_LEN {
        stage1[k] = cross_calc(left.stage1[k], l, right.stage1[k], r, rng);
        stage2[k] = cross_calc(left.stage2[k], l, right.stage2[k], r, rng);
        stage3[k] = cross_calc(left.stage3[k], l, right.stage3[k], r, rng);
        stage4[k] = cross_calc(left.stage4[k], l, right.stage4[k], r, rng);
    }

    CPU {
        stage1,
        stage2,
        stage3,
        stage4,
    }
}

fn cross_calc(a: i8, aa: usize, b: i8, bb: usize, rng: &mut impl Rng) -> i8 {
    let a = a as isize;
    let aa = aa as isize;
    let b = b as isize;
    let bb = bb as isize;

    (((a * aa + b * bb) / (aa + bb)) as f64 * (1.5 + 0.1 * (1.0 - 2.0 * rng.gen::<f64>()))) as i8
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
