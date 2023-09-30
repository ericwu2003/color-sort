pub const MAX_SEARCH_DEPTH: usize = 200;

use crate::move_type::{Move, MoveHistory};
use crate::tube::{Tube, TUBE_SIZE};
use dashmap::DashSet;
use std::collections::{BinaryHeap, VecDeque};
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufRead};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::{fmt, thread};

const THREAD_COUNT: usize = 8;
static IS_SOLVED_FLAG: AtomicBool = AtomicBool::new(false);
const THREAD_SYNC_INTERVAL: usize = 8192; // number of loop iterations a thread will run before syncing with global queue
const LOGGING_INTERVAL: usize = 32768;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct GameState(Vec<Tube>);

pub struct GameStateWithHistory(GameState, MoveHistory);

impl Ord for GameStateWithHistory {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.1.len().cmp(&self.1.len())
    }
}

impl PartialOrd for GameStateWithHistory {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.1.len().partial_cmp(&self.1.len())
    }
}

impl PartialEq for GameStateWithHistory {
    fn eq(&self, other: &Self) -> bool {
        self.1.len() == other.1.len()
    }
}

impl Eq for GameStateWithHistory {}

impl From<(GameState, MoveHistory)> for GameStateWithHistory {
    fn from(value: (GameState, MoveHistory)) -> Self {
        GameStateWithHistory(value.0, value.1)
    }
}

impl GameState {
    pub fn new_from_file() -> Self {
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

    pub fn search_for_solution(&self) {
        let visited_states_global: Arc<DashSet<GameState>> = Arc::new(DashSet::new());
        let queue_global: Arc<Mutex<BinaryHeap<GameStateWithHistory>>> =
            Arc::new(Mutex::new(BinaryHeap::new()));

        let initial_state = &self.clone();

        visited_states_global.insert(initial_state.clone());

        let mut handles = Vec::with_capacity(THREAD_COUNT);

        for thread_index in 0..THREAD_COUNT {
            let mut curr_state: GameState = initial_state.clone();
            let visited_states_global = Arc::clone(&visited_states_global);
            let queue_global = Arc::clone(&queue_global);
            let is_solved_flag = (&IS_SOLVED_FLAG).clone();
            is_solved_flag.store(false, Ordering::SeqCst);

            let mut loop_index = 0;
            let mut move_history: MoveHistory = MoveHistory::new();
            let mut queue_to_consume_local = VecDeque::new();
            let mut queue_to_produce_local = VecDeque::new();
            // let mut visited_states_local = HashSet::new();
            if thread_index == 0 {
                queue_to_consume_local.push_back((curr_state.clone(), MoveHistory::new()));
            }

            handles.push(thread::spawn(move || loop {
                for m in curr_state.get_legal_moves() {
                    let mut new_gs: GameState = curr_state.clone();

                    new_gs.apply_move(m);

                    if !visited_states_global.contains(&new_gs)
                        && move_history.len() < MAX_SEARCH_DEPTH
                    {
                        let mut move_history_copy = move_history.clone();
                        move_history_copy.push(m);
                        visited_states_global.insert(new_gs.clone());
                        queue_to_produce_local.push_back((new_gs, move_history_copy));
                    }
                }

                if queue_to_consume_local.is_empty() {
                    if is_solved_flag.load(Ordering::Relaxed) {
                        return;
                    } else {
                        // sync the global and local queues here
                        let mut queue_global = queue_global.lock().unwrap();

                        for state in queue_to_produce_local {
                            queue_global.push(GameStateWithHistory::from(state));
                        }
                        queue_to_produce_local = VecDeque::new();
                        while queue_to_consume_local.len() < THREAD_SYNC_INTERVAL {
                            match queue_global.pop() {
                                Some(state) => {
                                    queue_to_consume_local.push_back((state.0, state.1));
                                }
                                None => break,
                            }
                        }
                        if queue_to_consume_local.is_empty() {
                            continue;
                        }
                    }
                }
                (curr_state, move_history) = queue_to_consume_local.pop_front().unwrap();

                if loop_index % LOGGING_INTERVAL == 0 {
                    println!(
                        "thread {} visited {} states current depth is {}",
                        thread_index,
                        visited_states_global.len(),
                        move_history.len(),
                    );
                }

                if curr_state.is_solved() {
                    dbg!(&move_history);
                    println!("SOLVED in {} moves", move_history.len());
                    is_solved_flag.store(true, Ordering::SeqCst);
                    return;
                }

                loop_index += 1;
            }))
        }

        for handle in handles {
            let res = handle.join();
            if res.is_err() {
                dbg!(res);
            }
        }
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
