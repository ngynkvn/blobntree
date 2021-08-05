use std::time::Instant;

pub struct Game {
    pub ticks: usize,
    pub render_ticks: usize,
    pub start_system_time: Instant,
    pub running: bool,
    pub time: Instant,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            ticks: 0,
            render_ticks: 0,
            start_system_time: Instant::now(),
            running: true,
            time: Instant::now(),
        }
    }
}
