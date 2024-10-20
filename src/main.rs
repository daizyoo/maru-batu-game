mod online;

use online::online;
use serde::{Deserialize, Serialize};

type Field = [[Option<Square>; 3]; 3];

// todo いい感じの関数作りたい
const INPUT_MAP: [(usize, usize); 9] = [
    (0, 0),
    (0, 1),
    (0, 2),
    (1, 0),
    (1, 1),
    (1, 2),
    (2, 0),
    (2, 1),
    (2, 2),
];

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Square {
    Maru,
    Batu,
}

struct Game {
    field: Field,
    turn: bool,
}

impl Game {
    fn new() -> Game {
        Game {
            field: [[None; 3]; 3],
            turn: true,
        }
    }
}

impl GameF for Game {
    fn field(&self) -> &Field {
        &self.field
    }
    fn field_mut(&mut self) -> &mut Field {
        &mut self.field
    }
    fn turn_square(&self) -> Square {
        if self.turn {
            Square::Maru
        } else {
            Square::Batu
        }
    }

    fn start(&mut self) {
        loop {
            self.draw();
            if !self.turn(input()) {
                println!("input continue: not number");
                continue;
            }

            if self.check() {
                break;
            }

            self.turn = !self.turn;
        }

        self.draw();

        println!("winner: {:?}!", self.turn_square())
    }
}

#[tokio::main]
async fn main() {
    print!("\x1b[?25l");
    clear();

    loop {
        println!(">1: normal game");
        println!(">2: online game");
        match 2 {
            1 => Game::new().start(),
            2 => online().await,
            _ => {
                println!("input continue");
                continue;
            }
        }
        break;
    }

    quit();
}

fn quit() {
    print!("\x1b[?25h");
}

fn clear() {
    print!("\x1b[2J");
    print!("\x1b[H");
}

pub fn input<T: std::str::FromStr>() -> T {
    loop {
        let mut line = String::new();

        if std::io::stdin().read_line(&mut line).is_err() {
            println!("input continue");
            continue;
        }
        if let Ok(t) = line.trim().parse() {
            return t;
        }
        println!("parse error")
    }
}

trait GameF {
    fn field_mut(&mut self) -> &mut Field;
    fn field(&self) -> &Field;

    fn turn_square(&self) -> Square;

    fn turn(&mut self, num: usize) -> bool {
        if let Some((x, y)) = INPUT_MAP.get(num - 1) {
            if self.field()[*x][*y].is_none() {
                self.field_mut()[*x][*y] = Some(self.turn_square());
            }
            return true;
        }
        false
    }

    fn check(&mut self) -> bool {
        let field = *self.field();
        let mut line = [None; 3];
        let mut check_counts: Vec<usize> = Vec::new();

        let count = |line: [Option<Square>; 3]| {
            line.iter()
                .filter(|&&s| match s {
                    Some(s) => s == self.turn_square(),
                    None => false,
                })
                .count()
        };

        // horizontal
        for y in field {
            check_counts.push(count(y));
        }

        // vertical
        for i in 0..3 {
            for (x, y) in field.iter().enumerate() {
                line[x] = y[i];
            }
            check_counts.push(count(line));
        }

        // diagonal
        for (l, i) in (0..3).enumerate() {
            line[l] = field[i][i]
        }
        check_counts.push(count(line));

        for (i, (y, x)) in (0..3).zip((0..3).rev()).enumerate() {
            line[i] = field[y][x]
        }
        check_counts.push(count(line));

        for len in check_counts {
            if len == 3 {
                return true;
            }
        }

        false
    }

    fn draw(&self) {
        clear();

        for y in self.field() {
            for square in y {
                match square {
                    Some(s) => print!("{:?}", s),
                    None => print!("[ ]"),
                }
            }
            println!()
        }
        println!("turn: {:?}", self.turn_square());
    }

    fn start(&mut self);
}

impl std::fmt::Debug for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Maru => "[o]",
                Self::Batu => "[x]",
            }
        )
    }
}
