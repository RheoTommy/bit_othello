use crate::cpu::{eval_cpu, random_cross, two_point_cross, CPU};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub cpus: Vec<CPU>,
    pub generation: usize,
}

impl Tournament {
    pub fn new_random(tournament_size: usize, rng: &mut impl Rng) -> Self {
        let mut cpus = Vec::with_capacity(tournament_size);
        for _ in 0..tournament_size {
            cpus.push(CPU::new_random(rng));
        }

        Self {
            cpus,
            generation: 1,
        }
    }

    pub fn log(&self, log_file: &mut File) -> std::io::Result<()> {
        let str = serde_json::to_string(self)?;
        log_file.set_len(0)?;
        log_file.write_all(str.as_bytes())?;
        log_file.flush()
    }

    pub fn from_log_file(log_file: &mut File) -> std::io::Result<Self> {
        let mut buf = String::new();
        log_file.read_to_string(&mut buf)?;
        let tournament = serde_json::from_str(&buf)?;
        Ok(tournament)
    }

    pub fn upgrade_generation(&mut self, mutate_prob: f64, depth: usize, rng: &mut impl Rng) {
        assert_eq!(self.cpus.len(), 4096);

        let mut handles = Vec::with_capacity(4096);
        let mut cpus = Vec::with_capacity(4096);

        for i in 0..64 {
            let thread_cpu = self.cpus.clone();
            let mut rng = StdRng::from_seed(rng.gen());

            let handle = std::thread::spawn(move || {
                let thread_cpu = &thread_cpu[i * 64..(i + 1) * 64];
                let mut cpus = Vec::with_capacity(64);

                for i in 0..8 {
                    let tournament_cpus = &thread_cpu[i * 8..(i + 1) * 8];
                    let mut win_score = [0; 8];

                    for i in 0..8 {
                        for j in 0..8 {
                            if i <= j {
                                continue;
                            }

                            let left = &tournament_cpus[i];
                            let right = &tournament_cpus[j];

                            let winner = eval_cpu(left, right, depth);
                            if winner == left {
                                win_score[i] += 1;
                            } else {
                                win_score[j] += 1;
                            }
                        }
                    }

                    let mut sort_by_strong = win_score.iter().enumerate().collect::<Vec<_>>();
                    sort_by_strong.sort_by_key(|(_cpu_index, &win_num)| -win_num);
                    let sort_by_strong = sort_by_strong
                        .into_iter()
                        .map(|(cpu_index, _win_num)| &tournament_cpus[cpu_index])
                        .collect::<Vec<_>>();

                    let mut cpu_vec = Vec::with_capacity(64);
                    // 最優秀1体
                    cpu_vec.push(sort_by_strong[0].clone());

                    // 2点交叉3体
                    cpu_vec.push(two_point_cross(
                        sort_by_strong[0],
                        sort_by_strong[1],
                        &mut rng,
                    ));
                    cpu_vec.push(two_point_cross(
                        sort_by_strong[0],
                        sort_by_strong[2],
                        &mut rng,
                    ));
                    cpu_vec.push(two_point_cross(
                        sort_by_strong[1],
                        sort_by_strong[2],
                        &mut rng,
                    ));

                    // ランダム交叉1体
                    cpu_vec.push(random_cross(sort_by_strong[0], sort_by_strong[1], &mut rng));
                    cpu_vec.push(random_cross(sort_by_strong[0], sort_by_strong[2], &mut rng));
                    cpu_vec.push(random_cross(sort_by_strong[1], sort_by_strong[2], &mut rng));

                    // ランダム1体
                    cpu_vec.push(CPU::new_random(&mut rng));

                    cpus.append(&mut cpu_vec);
                }

                cpus
            });

            handles.push(handle);
        }

        for handle in handles {
            cpus.append(&mut handle.join().unwrap());
        }

        for cpu in &mut cpus {
            cpu.mutate_cpu(mutate_prob, rng);
        }

        self.cpus = cpus;
        self.generation += 1;
    }
}
