use crate::cpu::CPU;
use rand::Rng;
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
}
