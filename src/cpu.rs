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

    /// 盤面を次の手の人視点で評価
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

    /// Nodeを次の人の手視点で評価
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

    /// 次の一手として最適なものを選ぶ
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

    /// 遺伝子を一定の確率にランダムで乱数にする
    pub fn mutate_cpu(&mut self, mutate_prob: f64, rng: &mut impl Rng) {
        fn mutate(weights: &mut [i8], mutate_prob: f64, rng: &mut impl Rng) {
            for weight in weights {
                if mutate_prob < rng.gen::<f64>() {
                    *weight = rng.gen();
                }
            }
        }

        mutate(&mut self.stage1[..], mutate_prob, rng);
        mutate(&mut self.stage2[..], mutate_prob, rng);
        mutate(&mut self.stage3[..], mutate_prob, rng);
        mutate(&mut self.stage4[..], mutate_prob, rng);
    }
}

/// 2つのCPUのうち優秀な方を返す
pub fn eval_cpu<'a>(black: &'a CPU, white: &'a CPU, depth: usize) -> &'a CPU {
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

    winner
}

/// ランダムな2点で遺伝子を入れ替えたCPUを作成する
pub fn two_point_cross(left: &CPU, right: &CPU, rng: &mut impl Rng) -> CPU {
    let i = rng.gen_range(0, WEIGHT_LEN);
    let j = rng.gen_range(i, WEIGHT_LEN);

    let mut stage1 = [0; WEIGHT_LEN];
    let mut stage2 = [0; WEIGHT_LEN];
    let mut stage3 = [0; WEIGHT_LEN];
    let mut stage4 = [0; WEIGHT_LEN];

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

/// 各遺伝子をランダムに選択したCPUを作る
pub fn random_cross(left: &CPU, right: &CPU, rng: &mut impl Rng) -> CPU {
    let mut stage1 = [0; WEIGHT_LEN];
    let mut stage2 = [0; WEIGHT_LEN];
    let mut stage3 = [0; WEIGHT_LEN];
    let mut stage4 = [0; WEIGHT_LEN];

    for k in 0..WEIGHT_LEN {
        if rng.gen::<bool>() {
            stage1[k] = left.stage1[k];
            stage2[k] = left.stage2[k];
            stage3[k] = left.stage3[k];
            stage4[k] = left.stage4[k];
        } else {
            stage1[k] = right.stage1[k];
            stage2[k] = right.stage2[k];
            stage3[k] = right.stage3[k];
            stage4[k] = right.stage4[k];
        }
    }

    CPU {
        stage1,
        stage2,
        stage3,
        stage4,
    }
}
