type Field = [[Option<Square>; 3]; 3];

#[derive(Clone, Copy)]
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

    fn draw(&self) {
        for y in self.field {
            for s in y {
                if let Some(s) = s {
                    print!("{:?}", s)
                } else {
                    print!("[ ]")
                }
            }
            println!()
        }
    }
}

fn main() {
    let mut game = Game::new();

    game.draw();
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
