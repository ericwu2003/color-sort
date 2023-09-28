use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead};

pub const TUBE_SIZE: usize = 4;
pub const MAX_SEARCH_DEPTH: usize = 200;

// color[3] is the "top" of the tube, while color[0] is the "bottom" of the tube.
#[derive(Clone)]
struct Tube {
    colors: [u8; TUBE_SIZE as usize],
    num_balls: u8,
}

impl Hash for Tube {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.num_balls.hash(state);
        self.colors[0..self.num_balls as usize].hash(state);
    }
}

impl PartialEq for Tube {
    fn eq(&self, other: &Self) -> bool {
        self.num_balls == other.num_balls
            && self.colors[0..self.num_balls as usize] == other.colors[0..self.num_balls as usize]
    }
}

impl Eq for Tube {}

impl Tube {
    fn empty_tube() -> Self {
        return Tube {
            colors: [0; TUBE_SIZE],
            num_balls: 0,
        };
    }

    fn new_from_str(tube_str: &str) -> Self {
        assert!(tube_str.len() == TUBE_SIZE);

        let mut colors = [0u8; TUBE_SIZE];

        for (i, char) in tube_str.chars().enumerate() {
            colors[i] = u8::from_str_radix(&char.to_string(), 36).unwrap();
        }

        return Tube {
            colors,
            num_balls: TUBE_SIZE as u8,
        };
    }

    fn is_empty(&self) -> bool {
        return self.num_balls == 0;
    }

    fn is_full(&self) -> bool {
        return self.num_balls == TUBE_SIZE as u8;
    }

    fn is_solved(&self) -> bool {
        if self.num_balls != TUBE_SIZE as u8 {
            return false;
        }
        for i in 0..TUBE_SIZE - 1 {
            if self.colors[i] != self.colors[i + 1] {
                return false;
            }
        }
        return true;
    }

    fn get_top_color(&self) -> u8 {
        return self.colors[(self.num_balls - 1) as usize];
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
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
        let mut legal_moves = Vec::new();
        for source_index in 0..self.0.len() {
            for target_index in 0..self.0.len() {
                if self.is_legal_move(source_index, target_index) {
                    legal_moves.push(Move {
                        from: source_index as u8,
                        to: target_index as u8,
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
        if !self.is_legal_move(m.from as usize, m.to as usize) {
            panic!();
        }
        let tubes = &mut self.0;
        let target_color = tubes.get(m.from as usize).unwrap().get_top_color();
        let target_tube = tubes.get_mut(m.to as usize).unwrap();
        target_tube.num_balls += 1;
        target_tube.colors[(target_tube.num_balls - 1) as usize] = target_color;

        let source_tube = tubes.get_mut(m.from as usize).unwrap();
        source_tube.num_balls -= 1;
    }

    fn search_for_solution(&self) {
        // this function returns true if it finds a solution, false otherwise

        let mut exploration_queue: VecDeque<(GameState, Vec<Move>)> = VecDeque::new();
        let mut visited_states: HashSet<GameState> = HashSet::new();

        exploration_queue.push_back((self.clone(), Vec::new()));
        visited_states.insert(self.clone());
        let mut loop_index = 0;
        while !exploration_queue.is_empty() {
            let (curr_state, move_history) = exploration_queue.pop_front().unwrap();

            loop_index += 1;
            if loop_index % 32768 == 0 {
                println!(
                    "visited {} states, queue is {} elements long, current depth is {}",
                    visited_states.len(),
                    exploration_queue.len(),
                    move_history.len(),
                );
            }
            if curr_state.is_solved() {
                dbg!(&move_history);
                println!("SOLVED in {} moves", move_history.len());
                return;
            }

            for m in curr_state.get_legal_moves() {
                let mut new_gs: GameState = curr_state.clone();

                new_gs.apply_move(m);

                if !visited_states.contains(&new_gs) && move_history.len() < MAX_SEARCH_DEPTH {
                    let mut move_history_copy = move_history.clone();
                    move_history_copy.push(m);
                    visited_states.insert(new_gs.clone());
                    exploration_queue.push_back((new_gs, move_history_copy));
                }
            }
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tubes = &self.0;
        writeln!(f, "##### Total of {} tubes", tubes.len())?;
        for tube in tubes {
            for ball_position in 0..TUBE_SIZE {
                if ball_position >= tube.num_balls as usize {
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

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tubes = &self.0;
        writeln!(f, "##### Total of {} tubes", tubes.len())?;
        for tube in tubes {
            for ball_position in 0..TUBE_SIZE {
                if ball_position >= tube.num_balls as usize {
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

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Move from {} to {}", self.from, self.to)?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Move {
    from: u8,
    to: u8,
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

    gs.search_for_solution();

    // println!("{:?}", visited_states);
}
