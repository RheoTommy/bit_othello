use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub log_tournament_generation: usize,
    pub tournament_size: usize,
    pub select_tournament_size: usize,
    pub learning_depth: usize,
    pub simulation_depth: usize,
    pub cross_prob: f64,
    pub mutate_prob: f64,
    pub tournament_latest_file_name: String,
    pub winner_latest_file_name: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            log_tournament_generation: 25,
            tournament_size: 4096,
            select_tournament_size: 4,
            learning_depth: 1,
            simulation_depth: 8,
            cross_prob: 0.75,
            mutate_prob: 0.025,
            tournament_latest_file_name: "tournament_latest.json".to_string(),
            winner_latest_file_name: "winner_latest.json".to_string(),
        }
    }

    pub fn log(&self, log_file: &Path) -> std::io::Result<()> {
        let mut log_file = if log_file.exists() {
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(log_file)?
        } else {
            File::create(log_file)?
        };

        log_file.set_len(0)?;

        let json = serde_json::to_string(self)?;
        log_file.write_all(json.as_bytes())?;
        log_file.write_all("\n".as_bytes())?;
        log_file.flush()
    }

    pub fn from_log_file(log_file: &Path) -> std::io::Result<Self> {
        let mut log_file = File::open(log_file)?;
        let mut buf = String::new();
        log_file.read_to_string(&mut buf)?;
        let tournament = serde_json::from_str(&buf)?;
        Ok(tournament)
    }
}
