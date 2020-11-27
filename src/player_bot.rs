use std::fmt;
use std::io::{BufRead, Write};

const FIELD_SIZE: usize = 7;

pub fn main() {
    let mut field = Field::new();
    let mut play = true;
    while play {
        field.print();
        let mut column = 0;
        let mut repeat_input = true;
        while repeat_input {
            print!("Enter line number. (q to quit) > ");
            std::io::stdout().flush().expect("Failed to flush");
            let mut line = String::new();
            std::io::stdin().lock().read_line(&mut line).unwrap();
            line = line.trim().to_owned();

            if let Ok(user_column) = line.parse::<usize>() {
                if user_column < FIELD_SIZE {
                    column = user_column;
                    repeat_input = false;
                }
            } else if line == "q" {
                println!("bye");
                play = false;
                break;
            }
        }

        field.drop(column, Player::X);

        field.print();

        field.auto_drop();
    }
}

struct Field {
    field: [[Option<Player>; FIELD_SIZE]; FIELD_SIZE], // (0,0) is left bottom
    last_drop: usize,
}

impl Field {
    pub fn new() -> Self {
        Field {
            field: [[None; FIELD_SIZE]; FIELD_SIZE],
            last_drop: 0,
        }
    }

    fn drop(&mut self, column: usize, player: Player) {
        if column < FIELD_SIZE {
            for x in 0..FIELD_SIZE {
                for y in (0..FIELD_SIZE).rev() {
                    if self.field[x][y].is_none() && self.field[x][y - 1].is_some() {
                        self.field[x][y] = Some(player);
                    }
                }
            }
        }
    }

    fn auto_drop(&mut self) {
        self.drop(0, Player::O);
    }

    fn print(&self) {
        println!("{}\n", self);
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in 0..FIELD_SIZE {
            f.write_str(
                &(if self.last_drop == x {
                    "v".to_string()
                } else {
                    x.to_string()
                } + " "),
            )?;
        }
        for y in 0..FIELD_SIZE {
            for x in (0..FIELD_SIZE).rev() {
                let cell = self.field[x][y];
                if x == 0 {
                    f.write_str("─")?;
                }
                f.write_str(
                    &(match cell {
                        Some(Player::X) => "X",
                        Some(Player::O) => "O",
                        None => " ",
                    }
                    .to_string()
                        + " "),
                )?;
            }
            f.write_str("\n")?;
        }
        fmt::Result::Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Player {
    X,
    O,
}
