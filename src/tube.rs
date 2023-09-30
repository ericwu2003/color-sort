use std::hash;

pub const TUBE_SIZE: usize = 4;
// color[3] is the "top" of the tube, while color[0] is the "bottom" of the tube.
#[derive(Clone)]
pub struct Tube {
    pub colors: [u8; TUBE_SIZE as usize],
    pub num_balls: u8,
}

impl hash::Hash for Tube {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
    pub fn empty_tube() -> Self {
        return Tube {
            colors: [0; TUBE_SIZE],
            num_balls: 0,
        };
    }

    pub fn new_from_str(tube_str: &str) -> Self {
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

    pub fn is_empty(&self) -> bool {
        return self.num_balls == 0;
    }

    pub fn is_full(&self) -> bool {
        return self.num_balls == TUBE_SIZE as u8;
    }

    pub fn is_solved(&self) -> bool {
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

    pub fn get_top_color(&self) -> u8 {
        return self.colors[(self.num_balls - 1) as usize];
    }
}
