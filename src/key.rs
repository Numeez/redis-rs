use std::fmt::Display;

#[derive(Clone, Debug)]
pub struct Key {
    score: f64,
    name: String,
}

impl Key {
    pub fn new(score: f64, name: &str) -> Key {
        Self {
            score: score,
            name: name.to_string(),
        }
    }
    pub fn score(&self) -> f64 {
        self.score
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .total_cmp(&other.score)
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl Eq for Key {}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        let result = self.cmp(other);
        match result {
            std::cmp::Ordering::Equal => true,
            _ => false,
        }
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Score: {} Name: {}", self.score, self.name)
    }
}
