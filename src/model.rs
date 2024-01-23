use rand::prelude::*;
use std::time;

use crate::field::*;

pub const FPS: i32 = 30;
pub const MOVE_WAIT: i32 = 3;
pub const SHOOT_WAIT: i32 = 5;
pub const SCROLL_WAIT: i32 = 30;
pub const BULLET_COUNT_MAX: i32 = 4;
pub const BULLET_SPEED: i32 = 30;
pub const CELL_SIZE: i32 = 30;
pub const CELL_SIZEu32: u32 = CELL_SIZE as u32;
pub const ERASING: Cell = '*';
pub const ERASE_WAIT: i32 = 1;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Left,
    Right,
    Up,
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

pub struct ErasingEffect {
    pub exist: bool,
    pub cursor: Point,
    pub erase_wait: i32,
    pub dir: Direction,
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub is_clear: bool,
    pub is_debug: bool,
    pub should_getline: bool,
    pub requested_sounds: Vec<&'static str>,
    pub frame: i32,
    pub field: Field,
    pub stage: Vec<String>,
    pub next_row: usize,
    pub player_x: usize,
    // pub player_offset: i32,
    pub move_dir: Direction,
    pub move_wait: i32,
    pub shoot_wait: i32,
    pub scroll_wait: i32,
    pub bullets: Vec<Bullet>,
    pub erasing_effects: Vec<ErasingEffect>,
}

impl Game {
    pub fn new() -> Self {
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
            frame: -1,
            field: Field::new(),
            stage: Vec::new(),
            next_row: 0,
            player_x: FIELD_W / 2,
            // player_offset: 0,
            move_dir: Direction::Left,
            move_wait: 0,
            shoot_wait: 0,
            scroll_wait: SCROLL_WAIT,
            bullets: Vec::new(),
            erasing_effects: Vec::new(),
        };

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

        if command == Command::Up {
            if self.scroll_wait > 5 {
                self.scroll_wait = 5;
            }
        }

        if self.scroll_wait > 0 {
            self.scroll_wait -= 1;
            if self.scroll_wait == 0 {
                self.scroll();
                self.scroll_wait = SCROLL_WAIT;
            }
        }

        self.update_bullets();
        self.update_erasing_effects();

        self.move_player();
        if self.shoot_wait > 0 {
            self.shoot_wait -= 1;
        }

        match command {
            Command::Shoot => self.shoot(),
            Command::Left | Command::Right => self.start_move_player(command),
            _ => {}
        }

        self.bullets.retain(|x| x.exist);
        self.erasing_effects.retain(|x| x.exist);
    }

    pub fn scroll(&mut self) {
        if self.next_row == 0 {
            self.is_clear = true;
            return;
        }

        // 1つ上の行をコピー
        for y in (1..=(FIELD_H - 1)).rev() {
            for x in 0..FIELD_W {
                if self.field.cells[y][x] != ERASING && self.field.cells[y - 1][x] != ERASING {
                    self.field.cells[y][x] = self.field.cells[y - 1][x];
                }
            }
        }

        // ステージデータから1行読み込んでフィールドの一番上にセット
        for x in 0..FIELD_W {
            self.field.cells[0][x] = self.stage[self.next_row].chars().nth(x).unwrap();
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
        for i in 0..self.bullets.len() {
            let mut fix_bullet = false;
            let bullet_pos = self.bullets[i].pos;
            if bullet_pos.y >= 1 && self.field.cells[bullet_pos.y - 1][bullet_pos.x] != EMPTY {
                fix_bullet = true;
            }

            self.bullets[i].offset_y -= BULLET_SPEED;
            if self.bullets[i].offset_y < -CELL_SIZE {
                if bullet_pos.y == 0 {
                    self.bullets[i].exist = false;
                } else {
                    self.bullets[i].offset_y = 0;
                    self.bullets[i].pos.y -= 1;

                    if bullet_pos.y >= 1
                        && self.field.cells[bullet_pos.y - 1][bullet_pos.x] != EMPTY
                    {
                        fix_bullet = true;
                    }
                }
            }

            if fix_bullet {
                self.field.cells[bullet_pos.y][bullet_pos.x] =
                    self.field.cells[bullet_pos.y - 1][bullet_pos.x];
                self.bullets[i].exist = false;

                self.erase_rectangle(bullet_pos);
            }
        }
    }

    pub fn update_erasing_effects(&mut self) {
        for effect in &mut self.erasing_effects {
            if effect.erase_wait > 0 {
                effect.erase_wait -= 1;
                continue;
            }

            self.field.cells[effect.cursor.y][effect.cursor.x] = EMPTY;

            // 渦巻き状に消す
            if effect.dir == Direction::Right {
                if effect.cursor.x + 1 < FIELD_W
                    && self.field.cells[effect.cursor.y][effect.cursor.x + 1] == ERASING
                {
                    effect.cursor.x += 1;
                } else {
                    effect.dir = Direction::Down;
                    effect.cursor.y += 1;
                }
            } else if effect.dir == Direction::Down {
                if effect.cursor.y + 1 < FIELD_H
                    && self.field.cells[effect.cursor.y + 1][effect.cursor.x] == ERASING
                {
                    effect.cursor.y += 1;
                } else {
                    effect.dir = Direction::Left;
                    effect.cursor.x -= 1;
                }
            } else if effect.dir == Direction::Left {
                if effect.cursor.x >= 1
                    && self.field.cells[effect.cursor.y][effect.cursor.x - 1] == ERASING
                {
                    effect.cursor.x -= 1;
                } else {
                    effect.dir = Direction::Up;
                    effect.cursor.y -= 1;
                }
            } else if effect.dir == Direction::Up {
                if effect.cursor.y >= 1
                    && self.field.cells[effect.cursor.y - 1][effect.cursor.x] == ERASING
                {
                    effect.cursor.y -= 1;
                } else {
                    effect.dir = Direction::Right;
                    effect.cursor.x += 1;
                }
            }
            if self.field.cells[effect.cursor.y][effect.cursor.x] == EMPTY {
                effect.exist = false;
            }

            effect.erase_wait = ERASE_WAIT;
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

    pub fn erase_rectangle(&mut self, bullet_pos: Point) {
        let r = self
            .field
            .find_rectangle_to_be_erased(bullet_pos.x, bullet_pos.y);
        if let Some(r) = r {
            for y in r.top..=r.bottom {
                for x in r.left..=r.right {
                    self.field.cells[y][x] = ERASING;
                }
            }
            self.erasing_effects.push(ErasingEffect {
                exist: true,
                cursor: Point::new(r.left, r.bottom),
                erase_wait: ERASE_WAIT,
                dir: Direction::Up,
                left: r.left,
                top: r.top,
                right: r.right,
                bottom: r.bottom,
            });
        }
    }
}
