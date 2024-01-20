use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const FIELD_W: usize = 16;
pub const FIELD_H: usize = 256;
pub const MOVE_WAIT: i32 = 3;
pub const CELL_SIZE: i32 = 20;
pub const CELL_SIZEu32: u32 = CELL_SIZE as u32;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
    Shoot,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub should_getline: bool,
    pub requested_sounds: Vec<&'static str>,
    pub width: usize,  // must be >= 2
    pub height: usize, // must be >= 2
    pub frame: i32,
    pub erased: Vec<Vec<bool>>,
    pub cursor: Point,
    pub erase_dir: Direction,
    pub field: [[i32; FIELD_W]; FIELD_H],
    pub player_x: usize,
    // pub player_offset: i32,
    pub move_dir: Direction,
    pub move_wait: i32,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        // let rng = StdRng::seed_from_u64(0);

        let mut game = Game {
            rng: rng,
            is_over: false,
            should_getline: true,
            requested_sounds: Vec::new(),
            width: width,
            height: height,
            frame: -1,
            erased: Vec::new(),
            cursor: Point::default(),
            erase_dir: Direction::Right,
            field: [[0; FIELD_W]; FIELD_H],
            player_x: FIELD_W / 2,
            // player_offset: 0,
            move_dir: Direction::Left,
            move_wait: 0,
        };

        game.erased = vec![vec![false; game.width]; game.height];

        game
    }

    pub fn update(&mut self, command: Command) {
        self.frame += 1;

        if self.is_over {
            return;
        }

        self.move_player();

        match command {
            Command::Shoot => self.shoot(),
            Command::Left | Command::Right => self.start_move_player(command),
            _ => {}
        }
    }

    // 移動中のアニメーション処理
    pub fn move_player(&mut self) {
        if self.move_wait > 0 {
            self.move_wait -= 1;
            if self.move_dir == Direction::Left {
                // self.player_offset -= self.move_wait / MOVE_WAIT;
                // if self.player_offset == 0 {
                if self.move_wait == 0 {
                    self.player_x -= 1;
                }
            } else if self.move_dir == Direction::Right {
                // self.player_offset += self.move_wait / MOVE_WAIT;
                // if self.player_offset == 0 {
                if self.move_wait == 0 {
                    self.player_x += 1;
                }
            }
        }
    }

    pub fn start_move_player(&mut self, command: Command) {
        if self.move_wait == 0 {
            match command {
                Command::Left => {
                    if self.player_x >= 1 {
                        self.move_dir = Direction::Left;
                        self.move_wait = MOVE_WAIT;
                    }
                }
                Command::Right => {
                    if self.player_x + 1 < FIELD_W {
                        self.move_dir = Direction::Right;
                        self.move_wait = MOVE_WAIT;
                    }
                }
                _ => return,
            };
        }
    }

    pub fn shoot(&mut self) {
        if self.move_wait == 0 {
            println!("shoot");
        }
    }

    pub fn update_erase(&mut self, command: Command) {
        self.frame += 1;

        if self.is_over {
            return;
        }

        // 渦巻き状に消していく
        if self.frame % 8 == 0 && self.frame != 0 {
            self.erased[self.cursor.y][self.cursor.x] = true;
            println!("erase {:?}", self.cursor);

            if self.erase_dir == Direction::Right {
                if self.cursor.x + 1 < self.width && !self.erased[self.cursor.y][self.cursor.x + 1]
                {
                    self.cursor.x += 1;
                } else {
                    self.erase_dir = Direction::Down;
                    self.cursor.y += 1;
                }
            } else if self.erase_dir == Direction::Down {
                if self.cursor.y + 1 < self.height && !self.erased[self.cursor.y + 1][self.cursor.x]
                {
                    self.cursor.y += 1;
                } else {
                    self.erase_dir = Direction::Left;
                    self.cursor.x -= 1;
                }
            } else if self.erase_dir == Direction::Left {
                if self.cursor.x >= 1 && !self.erased[self.cursor.y][self.cursor.x - 1] {
                    self.cursor.x -= 1;
                } else {
                    self.erase_dir = Direction::Up;
                    self.cursor.y -= 1;
                }
            } else if self.erase_dir == Direction::Up {
                if self.cursor.y >= 1 && !self.erased[self.cursor.y - 1][self.cursor.x] {
                    self.cursor.y -= 1;
                } else {
                    self.erase_dir = Direction::Right;
                    self.cursor.x += 1;
                }
            }
            if self.erased[self.cursor.y][self.cursor.x] {
                self.is_over = true;
            }
        }
    }
}

fn clamp<T: PartialOrd>(min: T, value: T, max: T) -> T {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}
