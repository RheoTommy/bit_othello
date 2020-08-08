use bit_othello::board::{Board, Choice, JudgeResult, Player};
use bit_othello::config::Config;
use bit_othello::cpu::{eval_cpu, CPU};
use bit_othello::tournament::Tournament;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::env::args;
use std::fs::{File, OpenOptions};
use std::io::stdin;

const CONFIG_FILE_NAME: &str = "config.json";

fn main() {
    let mut config_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(CONFIG_FILE_NAME)
        .unwrap();

    let config = Config::from_log_file(&mut config_file).unwrap_or(Config::new());
    config.log(&mut config_file).unwrap();

    let mut winner_latest_log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .write(true)
        .open(config.winner_latest_file_name)
        .unwrap();
    let mut tournament_latest_log_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(config.tournament_latest_file_name)
        .unwrap();
    let mut rng = ThreadRng::default();

    let arg = args().skip(1).next().expect("No args!");
    match arg {
        a if &a == "simulate" => simulate(&mut winner_latest_log_file, config.simulation_depth),
        a if &a == "learn" => learn(
            &mut tournament_latest_log_file,
            &mut winner_latest_log_file,
            config.log_tournament_generation,
            config.tournament_size,
            // config.select_tournament_size,
            config.learning_depth,
            // config.cross_prob,
            config.mutate_prob,
            &mut rng,
        ),
        _ => unimplemented!(),
    }
}

pub fn learn(
    tournament_log_file: &mut File,
    winner_latest_log_file: &mut File,
    log_tournament_generation: usize,
    tournament_size: usize,
    // selection_size: usize,
    learn_depth: usize,
    // cross_prob: f64,
    mutate_prob: f64,
    rng: &mut impl Rng,
) {
    let tournament = Tournament::from_log_file(tournament_log_file);
    let mut tournament = if let Err(e) = tournament {
        eprintln!("{}", e);
        Tournament::new_random(tournament_size, rng)
    } else {
        tournament.unwrap()
    };

    loop {
        println!("Now generation: {}", tournament.generation);
        tournament.upgrade_generation_alpha(learn_depth, mutate_prob, rng);

        if tournament.generation % log_tournament_generation == 0 {
            tournament.log(tournament_log_file).unwrap();
            let mut iter = tournament.cpus.iter();
            let first = iter.next().unwrap();
            iter.fold(first, |a, b| eval_cpu(a, b, learn_depth).0)
                .log(winner_latest_log_file)
                .unwrap();
        }
    }
}

pub fn simulate(winner_log_file: &mut File, simulate_depth: usize) {
    let mut board = Board::new();

    let cpu = CPU::from_log_file(winner_log_file);
    let cpu = if let Err(e) = cpu {
        eprintln!("{}", e);
        CPU::new_alpha()
    } else {
        cpu.unwrap()
    };

    loop {
        eprintln!("{:?}score: {}\n", board, cpu.eval_board(&board));

        let next = if board.player == Player::Black {
            cpu.choose_best(&board, simulate_depth)
        } else {
            input()
        };

        match board.update(next) {
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
            Ok(JudgeResult::Continue) => continue,
            Ok(JudgeResult::Draw) => {
                eprintln!("{:?}", board);
                println!("Draw!");
                break;
            }
            Ok(JudgeResult::Win(winner)) => {
                eprintln!("{:?}", board);
                println!("{:?} wins!", winner);
                break;
            }
        }
    }
}

pub fn input() -> Choice {
    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
    let buf = buf.trim();

    match buf {
        "s" => Choice::Skip,
        _ => {
            if buf.chars().count() != 2 {
                eprintln!("Wrong input length");
                return input();
            }

            if buf.chars().map(|c| c.to_digit(10)).any(|o| o.is_none()) {
                eprintln!("Wrong input");
                return input();
            }

            let v = buf
                .chars()
                .map(|c| c.to_digit(10).unwrap() as usize)
                .collect::<Vec<_>>();
            Choice::Coordinate((v[0], v[1]))
        }
    }
}
