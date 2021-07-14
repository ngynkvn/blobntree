use crate::Game;

pub enum State {
    Idle,
    Run,
    Peck,
}

pub struct PlayerState {
    pub state: State,
    pub position: (i32, i32),
    pub velocity: (i32, i32),
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            state: State::Idle,
            position: (100, 100),
            velocity: (0, 0),
        }
    }
    pub fn update(&mut self) {
        let (mut player_x, mut player_y) = self.position;
        let (mut pv_x, mut pv_y) = self.velocity;
        //player movement
        pv_x = pv_x.min(20);
        pv_x = (pv_x as f32 * 0.98) as i32;
        player_x += pv_x;
        player_y += pv_y;

        //gravity
        if player_y > 200 {
            player_y = 200;
            pv_y = 0;
        } else {
            pv_y += 10;
        }
        self.velocity = (pv_x, pv_y);
        self.position = (player_x, player_y);
    }
}
