use std::collections::HashMap;

type Cell = char;

pub const EMPTY: Cell = ' ';

pub const FIELD_W: usize = 16;
pub const FIELD_H: usize = 16;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rectangle {
    pub left: usize,
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
}

#[derive(Default)]
pub struct Field {
    pub cells: [[Cell; FIELD_W]; FIELD_H],
}

impl Field {
    pub fn new() -> Field {
        Field {
            cells: [[EMPTY; FIELD_W]; FIELD_H],
        }
    }
    pub fn from_text(cells_text: &str) -> Field {
        let mut field = Field {
            cells: [[' '; FIELD_W]; FIELD_H],
        };

        let lines: Vec<&str> = cells_text.lines().collect();
        println!("{:?}", lines);
        for y in 0..FIELD_H {
            let line = lines[y].to_string();
            for x in 0..FIELD_W {
                if let Some(ch) = line.chars().nth(x) {
                    field.cells[y][x] = ch;
                } else {
                    field.cells[y][x] = EMPTY;
                }
            }
        }

        field
    }

    pub fn print_with_coord(&self) {
        print!("  ");
        for i in 0..self.cells[0].len() {
            print!("{:2}", i);
        }
        println!("");

        for (j, row) in self.cells.iter().enumerate() {
            print!("{:2}", j);
            for cell in row {
                print!("{:2}", cell);
            }
            println!("");
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        if 0 <= x && x < FIELD_W && 0 <= y && y < FIELD_H {
            self.cells[y][x]
        } else {
            EMPTY
        }
    }

    pub fn is_rectangle(&self, left: usize, top: usize, right: usize, bottom: usize) -> bool {
        // 幅または高さが1の矩形は除外
        if !(left < right && top < bottom) {
            return false;
        }

        for x in left..=right {
            // 上の辺がつながっているか判定
            if self.get(x, top) == EMPTY {
                return false;
            }
            // 上に飛び出ていないか判定
            if top >= 1 && self.get(x, top - 1) == self.get(x, top) {
                return false;
            }
        }

        for x in left..=right {
            if self.get(x, bottom) == EMPTY {
                return false;
            }
            if self.get(x, bottom + 1) == self.get(x, bottom) {
                return false;
            }
        }

        for y in top..=bottom {
            if self.get(left, y) == EMPTY {
                return false;
            }
            if left >= 1 && self.get(left - 1, y) == self.get(left, y) {
                return false;
            }
        }

        for y in top..=bottom {
            if self.get(right, y) == EMPTY {
                return false;
            }
            if self.get(right + 1, y) == self.get(right, y) {
                return false;
            }
        }

        return true;
    }

    pub fn find_corners(&self) -> (HashMap<char, (usize, usize)>, HashMap<char, (usize, usize)>) {
        let mut top_lefts: HashMap<char, (usize, usize)> = HashMap::new();
        let mut bottom_rights: HashMap<char, (usize, usize)> = HashMap::new();
        for y in 0..FIELD_H {
            for x in 0..FIELD_W {
                if self.get(x, y) == EMPTY {
                    continue;
                }
                if !top_lefts.contains_key(&self.get(x, y)) {
                    top_lefts.insert(self.get(x, y), (x, y));
                }
                let br = bottom_rights.get(&self.get(x, y));
                if br == None {
                    bottom_rights.insert(self.get(x, y), (x, y));
                } else {
                    let br = br.unwrap();
                    let mut new_br = *br;
                    if br.0 < x {
                        new_br.0 = x;
                    }
                    if br.1 < y {
                        new_br.1 = y;
                    }
                    bottom_rights.insert(self.get(x, y), new_br);
                }
            }
        }
        return (top_lefts, bottom_rights);
    }

    pub fn find_all_rectangles(&self) -> Vec<Rectangle> {
        let tlbr = self.find_corners();
        let top_lefts = tlbr.0;
        let bottom_rights = tlbr.1;
        println!("{:?}", top_lefts);
        println!("{:?}", bottom_rights);

        let mut answers = Vec::new();
        for (_, tl) in &top_lefts {
            for (_, br) in &bottom_rights {
                let left = tl.0;
                let top = tl.1;
                let right = br.0;
                let bottom = br.1;
                if self.is_rectangle(left, top, right, bottom) {
                    answers.push(Rectangle {
                        left,
                        top,
                        right,
                        bottom,
                    });
                }
            }
        }
        return answers;
    }
}

pub fn test() {
    #[rustfmt::skip]
    let cells_text: String =
          "                \n".to_string()
        + "                \n"
        + "                \n"
        + "   1112222      \n"
        + "   1  2  2      \n"
        + "   1  2  2      \n"
        + "   3334444      \n"
        + "   3555666      \n"
        + "   3577776      \n"
        + "   3577776      \n"
        + "   3577776      \n"
        + "                \n"
        + "                \n"
        + "                \n"
        + "                \n"
        + "                \n"
        + "";

    let field = Field::from_text(&cells_text);
    field.print_with_coord();
    let rectangles = field.find_all_rectangles();
    println!("len = {}", rectangles.len());
    for rectangle in rectangles {
        println!("{:?}", rectangle);
    }
}
