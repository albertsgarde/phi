use std::ops::Index;

use itertools::Itertools;

use crate::Value;

#[derive(Clone, Debug)]
pub struct Rule {
    values: Vec<Value>,
}

impl Rule {
    pub fn from_array<A>(values: A) -> Option<Self>
    where
        A: AsRef<[Value]>,
    {
        let values = values.as_ref();
        if values.iter().tuple_windows().any(|(a, b)| a < b) {
            None
        } else {
            let result: Vec<_> = values.iter().copied().take_while(|&v| v != 0).collect();
            if result.is_empty() {
                None
            } else {
                Some(Rule { values: result })
            }
        }
    }

    pub fn first(&self) -> Value {
        self.values.first().copied().unwrap()
    }

    pub fn phi(&self) -> Self {
        Rule { values: vec![1, 1] }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.get(index).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.values.iter().copied()
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Eq for Rule {}

impl Index<usize> for Rule {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values.get(index).unwrap_or(&0)
    }
}
