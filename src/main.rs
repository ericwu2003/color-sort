use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};

pub const TUBE_SIZE: usize = 4;

// color[3] is the "top" of the tube, while color[0] is the "bottom" of the tube.
#[derive(Clone)]
struct Tube {
    colors: [i32; TUBE_SIZE],
    num_balls: usize,
}

impl Tube {
    fn empty_tube() -> Self {
        return Tube {
            colors: [0; TUBE_SIZE],
            num_balls: 0,
        };
    }

    fn new_from_str(tube_str: &str) -> Self {
        assert!(tube_str.len() == TUBE_SIZE);

        let mut colors = [0; TUBE_SIZE];

        for (i, char) in tube_str.chars().enumerate() {
            colors[i] = i32::from_str_radix(&char.to_string(), 36).unwrap();
        }

        return Tube {
            colors,
            num_balls: TUBE_SIZE,
        };
    }

    fn is_empty(&self) -> bool {
        return self.num_balls == 0;
    }

    fn is_full(&self) -> bool {
        return self.num_balls == TUBE_SIZE;
    }

    fn is_solved(&self) -> bool {
        if self.num_balls != TUBE_SIZE {
            return false;
        }
        for i in 0..TUBE_SIZE - 1 {
            if self.colors[i] != self.colors[i + 1] {
                return false;
            }
        }
        return true;
    }

    fn get_top_color(&self) -> i32 {
        return self.colors[(self.num_balls - 1) as usize];
    }
}

#[derive(Clone)]
struct GameState(Vec<Tube>);

impl GameState {
    fn new_from_file() -> Self {
        // the first line of the file indicates the number of completely empty tubes
        // the following lines of the file each represent a tube
        let file = File::open("src/input.txt").unwrap();
        let mut lines: io::Lines<io::BufReader<File>> = io::BufReader::new(file).lines();

        let num_empty_tubes = u32::from_str_radix(&lines.next().unwrap().unwrap(), 10).unwrap();

        let mut game_state: Vec<Tube> = Vec::new();

        for line in lines {
            let line = line.unwrap();
            game_state.push(Tube::new_from_str(&line));
        }
        for _ in 0..num_empty_tubes {
            game_state.push(Tube::empty_tube());
        }

        return GameState(game_state);
    }

    fn num_tubes(&self) -> usize {
        self.0.len()
    }

    fn is_solved(&self) -> bool {
        self.0
            .iter()
            .all(|tube| tube.is_empty() || tube.is_solved())
    }

    fn get_legal_moves(&self) -> Vec<Move> {
        let tubes = &self.0;
        let mut legal_moves = Vec::new();
        for (source_index, source_tube) in tubes.iter().enumerate() {
            for (target_index, target_tube) in tubes.iter().enumerate() {
                if self.is_legal_move(source_index, target_index) {
                    legal_moves.push(Move {
                        from: source_index,
                        to: target_index,
                    });
                }
            }
        }

        return legal_moves;
    }

    fn is_legal_move(&self, from: usize, to: usize) -> bool {
        let source_tube_opt = self.0.get(from);
        let source_tube;
        match source_tube_opt {
            None => {
                return false;
            }
            Some(t) => {
                source_tube = t;
            }
        }
        let target_tube_opt = self.0.get(to);
        let target_tube;
        match target_tube_opt {
            None => {
                return false;
            }
            Some(t) => {
                target_tube = t;
            }
        }
        if from == to || source_tube.is_empty() || target_tube.is_full() {
            // cannot move ball from source to target
            return false;
        }
        if !target_tube.is_empty() && source_tube.get_top_color() != target_tube.get_top_color() {
            // cannot move ball from source to target because of color mismatch
            return false;
        }
        return true;
    }

    fn apply_move(&mut self, m: Move) {
        // this function assumes that m is a legal move and moves a ball from m.from to m.to
        if !self.is_legal_move(m.from, m.to) {
            panic!();
        }
        let tubes = &mut self.0;
        let target_color = tubes.get(m.from).unwrap().get_top_color();
        let target_tube = tubes.get_mut(m.to).unwrap();
        target_tube.num_balls += 1;
        target_tube.colors[target_tube.num_balls - 1] = target_color;

        let source_tube = tubes.get_mut(m.from).unwrap();
        source_tube.num_balls -= 1;
    }

    fn search_for_solution(&self, move_history: &mut Vec<Move>) -> bool {
        // this function returns true if it finds a solution, false otherwise

        let last_move;
        if move_history.is_empty() {
            last_move = None;
        } else {
            // This option will be None if move_history is empty
            last_move = move_history.get(move_history.len() - 1).map(|x| x.clone());
        }

        if self.is_solved() {
            println!("SOLVED");
            dbg!(move_history);
            return true;
        }

        for m in self.get_legal_moves() {
            if let Some(last_move) = last_move {
                if last_move.to == m.from && last_move.from == m.to {
                    continue;
                }
            }

            let mut new_gs = self.clone();
            new_gs.apply_move(m);
            move_history.push(m);
            if new_gs.search_for_solution(move_history) {
                return true;
            }
            move_history.pop();
        }

        return false;
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tubes = &self.0;
        writeln!(f, "##### Total of {} tubes", tubes.len())?;
        for tube in tubes {
            for ball_position in 0..TUBE_SIZE {
                if ball_position >= tube.num_balls {
                    write!(f, "_")?;
                } else {
                    write!(f, "{}", tube.colors[ball_position])?;
                }
            }
            write!(f, "\n")?;
        }
        writeln!(f, "#####")?;
        return Ok(());
    }
}

#[derive(fmt::Debug, Clone, Copy)]
struct Move {
    from: usize,
    to: usize,
}

fn main() {
    let gs = GameState::new_from_file();
    println!("{}", gs);
    // dbg!(gs.get_legal_moves());
    // for m in gs.get_legal_moves() {
    //     let mut new_gs = gs.clone();
    //     new_gs.apply_move(m);
    //     println!("{}", new_gs);
    // }

    gs.search_for_solution(&mut Vec::new());
}
