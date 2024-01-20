use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const WORLD_W: f32 = 600.0;
pub const WORLD_H: f32 = 600.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
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
        };

        game.erased = vec![vec![false; game.width]; game.height];

        game
    }

    pub fn update(&mut self, command: Command) {
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

        match command {
            Command::None => {}
        }
    }
}
