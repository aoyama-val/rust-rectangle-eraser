use rand::prelude::*;
use std::time;

pub const FPS: i32 = 30;
pub const FIELD_W: usize = 16;
pub const FIELD_H: usize = 16;
pub const MOVE_WAIT: i32 = 3;
pub const SHOOT_WAIT: i32 = 5;
pub const SCROLL_WAIT: i32 = 30;
pub const BULLET_COUNT_MAX: i32 = 4;
pub const BULLET_SPEED: i32 = 30;
pub const CELL_SIZE: i32 = 30;
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

pub struct Bullet {
    pub pos: Point,
    pub offset_y: i32,
    pub exist: bool,
}

impl Bullet {
    pub fn new(x: usize) -> Bullet {
        Bullet {
            pos: Point::new(x, FIELD_H - 2),
            offset_y: 0,
            exist: true,
        }
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub is_clear: bool,
    pub is_debug: bool,
    pub should_getline: bool,
    pub requested_sounds: Vec<&'static str>,
    pub width: usize,  // must be >= 2
    pub height: usize, // must be >= 2
    pub frame: i32,
    pub erased: Vec<Vec<bool>>,
    pub cursor: Point,
    pub erase_dir: Direction,
    pub field: [[char; FIELD_W]; FIELD_H],
    pub stage: Vec<String>,
    pub next_row: usize,
    pub player_x: usize,
    // pub player_offset: i32,
    pub move_dir: Direction,
    pub move_wait: i32,
    pub shoot_wait: i32,
    pub bullets: Vec<Bullet>,
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
            is_clear: false,
            is_debug: false,
            should_getline: true,
            requested_sounds: Vec::new(),
            width: width,
            height: height,
            frame: -1,
            erased: Vec::new(),
            cursor: Point::default(),
            erase_dir: Direction::Right,
            field: [[' '; FIELD_W]; FIELD_H],
            stage: Vec::new(),
            next_row: 0,
            player_x: FIELD_W / 2,
            // player_offset: 0,
            move_dir: Direction::Left,
            move_wait: 0,
            shoot_wait: 0,
            bullets: Vec::new(),
        };

        game.erased = vec![vec![false; game.width]; game.height];

        game.load_stage("resources/data/stage1.txt");

        // println!("STAGE");
        // for row in &game.stage {
        //     println!("{}", row);
        // }
        // println!("STAGE");

        game
    }

    pub fn toggle_debug(&mut self) {
        self.is_debug = !self.is_debug;
        println!("is_debug: {}", self.is_debug);
    }

    pub fn load_stage(&mut self, filename: &str) {
        let Ok(content) = std::fs::read_to_string(filename) else {
            panic!("Cannot load: {}", filename);
        };

        for (_, line) in content.lines().enumerate() {
            // println!("{}", line.len());
            let row = ((line.to_string() + &" ".repeat(FIELD_W as usize))[0..FIELD_W]).to_string();
            assert!(row.len() == FIELD_W);
            self.stage.push(row);
        }

        self.next_row = self.stage.len() - 1;
    }

    pub fn update(&mut self, command: Command) {
        self.frame += 1;

        if self.is_over || self.is_clear {
            return;
        }

        if self.frame % SCROLL_WAIT == 0 {
            self.scroll();
        }

        self.update_bullets();

        self.move_player();
        if self.shoot_wait > 0 {
            self.shoot_wait -= 1;
        }

        match command {
            Command::Shoot => self.shoot(),
            Command::Left | Command::Right => self.start_move_player(command),
            _ => {}
        }
    }

    pub fn scroll(&mut self) {
        if self.next_row == 0 {
            self.is_clear = true;
            return;
        }

        for y in (1..=(FIELD_H - 1)).rev() {
            self.field[y] = self.field[y - 1].clone();
        }
        for x in 0..FIELD_W {
            self.field[0][x] = self.stage[self.next_row].chars().nth(x).unwrap();
        }
        self.next_row -= 1;
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

    pub fn update_bullets(&mut self) {
        for bullet in &mut self.bullets {
            let mut fix_bullet = false;

            if bullet.pos.y >= 1 && self.field[bullet.pos.y - 1][bullet.pos.x] != ' ' {
                fix_bullet = true;
            }

            bullet.offset_y -= BULLET_SPEED;
            if bullet.offset_y < -CELL_SIZE {
                if bullet.pos.y == 0 {
                    bullet.exist = false;
                } else {
                    bullet.offset_y = 0;
                    bullet.pos.y -= 1;

                    if bullet.pos.y >= 1 && self.field[bullet.pos.y - 1][bullet.pos.x] != ' ' {
                        fix_bullet = true;
                    }
                }
            }

            if fix_bullet {
                self.field[bullet.pos.y][bullet.pos.x] = self.field[bullet.pos.y - 1][bullet.pos.x];
                bullet.exist = false;
            }
        }

        self.bullets.retain(|x| x.exist);
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
        println!("shoot");
        if self.shoot_wait > 0 {
            println!("shoot_wait = {}", self.shoot_wait);
            return;
        }
        if self.bullets.len() as i32 >= BULLET_COUNT_MAX {
            println!("len = {}", self.bullets.len());
            return;
        }
        let bullet = Bullet::new(self.player_x);
        self.bullets.push(bullet);
        self.shoot_wait = SHOOT_WAIT;
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
