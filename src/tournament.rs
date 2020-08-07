use crate::cpu::{cross_cpu, eval_cpu, mutate_cpu, CPU};
use rand::prelude::{SliceRandom, StdRng};
use rand::{Rng, SeedableRng};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    cpus: Vec<CPU>,
    generation: usize,
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
        log_file.write_all(str.as_bytes())
    }

    pub fn from_log_file(log_file: &mut File) -> std::io::Result<Self> {
        let mut buf = String::new();
        log_file.read_to_string(&mut buf)?;
        let tournament = serde_json::from_str(&buf)?;
        Ok(tournament)
    }

    fn select_parents(&self, selection_size: usize, depth: usize, rng: &mut impl Rng) -> Vec<&CPU> {
        let len = self.cpus.len();
        assert_eq!(len % 128, 0);

        let mut cpus = Vec::with_capacity(len / 128);
        let mut handles = Vec::with_capacity(128);

        for _ in 0..128 {
            let mut rng = StdRng::from_seed(rng.gen());
            let self_cpus = &self.cpus;

            let handle = std::thread::spawn(move || {
                let mut cpus = Vec::with_capacity(len / 128);

                for _ in 0..len / 128 {
                    let mut iter = self_cpus.choose_multiple(&mut rng, selection_size);
                    let first = iter.next().unwrap();
                    cpus.push(iter.fold(first, |a, b| eval_cpu(a, b, depth)))
                }

                cpus
            });

            handles.push(handle);
        }

        for handle in handles {
            cpus.append(handle.join().unwrap());
        }

        cpus
    }

    pub fn upgrade_generation(
        &mut self,
        selection_size: usize,
        depth: usize,
        cross_prob: f64,
        mutate_prob: f64,
        rng: &mut impl Rng,
    ) {
        let mut left = self.select_parents(selection_size, depth, rng);
        let right = left.split_off(left.len() / 2);

        let mut cpus = Vec::with_capacity(self.cpus.len());

        for i in 0..right.len() {
            cpus.push(cross_cpu(left[i], right[i], cross_prob, rng));
            cpus.push(cross_cpu(right[i], left[i], cross_prob, rng));
        }

        for cpu in &mut cpus {
            mutate_cpu(cpu, mutate_prob, rng);
        }

        self.cpus = cpus;
        self.generation += 1;
    }
}
