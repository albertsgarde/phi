use std::{
    fmt::Display,
    ops::{Add, AddAssign, Index, IndexMut},
};

use thiserror::Error;

use crate::{rule::Rule, Value};

#[derive(Clone, Debug, Error)]
#[error("Rule application at index {application_index} failed at rule index {rule_index} because rule value {rule_value} is greater than tape value {tape_value}.")]
pub struct ApplyRuleError {
    application_index: isize,
    rule_index: usize,
    rule_value: Value,
    tape_value: Value,
}

#[derive(Clone, Debug)]
pub struct Tape {
    positive_values: Vec<Value>,
    negative_values: Vec<Value>,
}

impl Tape {
    pub fn from_arrays<A1, A2>(positives: A1, negatives: A2) -> Self
    where
        A1: AsRef<[Value]>,
        A2: AsRef<[Value]>,
    {
        Tape {
            positive_values: positives.as_ref().iter().rev().copied().collect(),
            negative_values: negatives.as_ref().to_vec(),
        }
    }

    pub fn zero() -> Self {
        Self {
            positive_values: vec![],
            negative_values: vec![],
        }
    }

    pub fn range(&self) -> (isize, isize) {
        (
            -(self.negative_values.len() as isize),
            self.positive_values.len() as isize,
        )
    }

    fn internal_index(index: isize) -> (bool, usize) {
        if index >= 0 {
            (true, usize::try_from(index).unwrap())
        } else {
            (false, usize::try_from(-index - 1).unwrap())
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Value> + '_ {
        self.positive_values
            .iter()
            .copied()
            .rev()
            .chain(self.negative_values.iter().copied())
    }

    pub fn index_iter(&self) -> impl Iterator<Item = isize> {
        let (min, max) = self.range();
        (min..max).rev()
    }

    pub fn value(&self, rule: &Rule) -> f64 {
        self.iter()
            .zip(self.index_iter())
            .map(|(value, index)| {
                f64::from(value) * rule.base().powi(i32::try_from(index).unwrap())
            })
            .sum::<f64>()
    }

    pub fn apply(&self, rule: &Rule, index: isize) -> Result<Self, ApplyRuleError> {
        self.clone().apply_in_place(rule, index)
    }

    pub fn apply_in_place(mut self, rule: &Rule, index: isize) -> Result<Self, ApplyRuleError> {
        let rule_len = rule.len() as isize;
        assert!(rule_len > 0);
        self[index] += 1;
        for (rule_index, rule_value) in rule.iter().enumerate() {
            let tape_index = index - isize::try_from(rule_index + 1).unwrap();
            if self[tape_index] < rule_value {
                return Err(ApplyRuleError {
                    application_index: index,
                    rule_index,
                    rule_value,
                    tape_value: self[tape_index],
                });
            }
            self[tape_index] -= rule_value;
        }
        Ok(self)
    }

    pub fn is_valid(&self, rule: &Rule) -> bool {
        let max_allowed = rule.first();
        self.iter().all(|value| value <= max_allowed)
    }

    pub fn is_standard(&self, rule: &Rule) -> bool {
        if !self.is_valid(rule) {
            return false;
        }
        let rule_len = rule.len() as isize;
        assert!(rule_len > 0);
        let (min, max) = self.range();
        let mut cur = max;
        for (i, value) in self.index_iter().zip(self.iter()) {
            let rule_index = usize::try_from(cur - i - 1).unwrap();
            if let Some(rule_value) = rule.get(rule_index) {
                if value < rule_value {
                    cur = i;
                }
            } else {
                println!("{} {}", cur, i);
                return false;
            }
        }
        if cur - min == rule_len {
            return false;
        }
        assert!(cur - min < rule_len);
        true
    }

    pub fn standardize(&self, rule: &Rule) -> Self {
        self.clone().standardize_in_place(rule)
    }

    pub fn standardize_in_place(mut self, rule: &Rule) -> Self {
        assert!(self.is_valid(rule));
        let rule_max = rule.first();
        let rule_len = rule.len() as isize;
        assert!(rule_len > 0);
        let (min, max) = self.range();

        let mut cur = max;
        for i in self.index_iter() {
            let rule_index = usize::try_from(cur - i - 1).unwrap();
            if let Some(rule_value) = rule.get(rule_index) {
                if self[i] < rule_value {
                    cur = i;
                }
            } else {
                assert!(self[cur] < rule_max);
                self = self.apply_in_place(rule, cur).unwrap();
                cur = i;
            }
        }
        if cur - min - 1 == rule_len {
            self = self.apply_in_place(rule, cur).unwrap();
        }
        assert!(cur - min < rule_len);
        self
    }
}

impl Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first_positive) = self.positive_values.first() {
            for &value in self.positive_values[1..].iter().rev() {
                write!(f, "{value} ")?;
            }
            write!(f, "{first_positive}")?;
        } else {
            write!(f, "0")?;
        }
        if let Some(last_negative) = self.negative_values.last() {
            write!(f, ",")?;
            for &value in self.negative_values[..(self.negative_values.len() - 1)].iter() {
                write!(f, "{value} ")?;
            }
            write!(f, "{last_negative}")?;
        }
        Ok(())
    }
}

impl PartialEq<Tape> for Tape {
    fn eq(&self, other: &Tape) -> bool {
        let min_pos_len = self.positive_values.len().min(other.positive_values.len());
        let min_neg_len = self.negative_values.len().min(other.negative_values.len());
        self.positive_values[..min_pos_len] == other.positive_values[..min_pos_len]
            && self.positive_values[min_pos_len..]
                .iter()
                .chain(other.positive_values[min_pos_len..].iter())
                .all(|&x| x == 0)
            && self.negative_values[..min_neg_len] == other.negative_values[..min_neg_len]
            && self.negative_values[min_neg_len..]
                .iter()
                .chain(other.negative_values[min_neg_len..].iter())
                .all(|&x| x == 0)
    }
}

impl Eq for Tape {}

impl Index<isize> for Tape {
    type Output = Value;

    fn index(&self, index: isize) -> &Self::Output {
        match Self::internal_index(index) {
            (true, index) => self.positive_values.get(index).unwrap_or(&0),
            (false, index) => self.negative_values.get(index).unwrap_or(&0),
        }
    }
}

impl IndexMut<isize> for Tape {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        let (positive, index) = Self::internal_index(index);
        let array = if positive {
            &mut self.positive_values
        } else {
            &mut self.negative_values
        };
        if array.len() <= index {
            array.resize(index + 1, 0);
        }
        array.get_mut(index).unwrap()
    }
}

impl AddAssign<Tape> for Tape {
    fn add_assign(&mut self, rhs: Tape) {
        for (self_array, rhs_array) in [
            (&mut self.positive_values, &rhs.positive_values),
            (&mut self.negative_values, &rhs.negative_values),
        ] {
            if rhs_array.len() > self_array.len() {
                self_array.resize(rhs_array.len(), 0);
            }
            for (value, &rhs_value) in self_array.iter_mut().zip(rhs_array.iter()) {
                *value += rhs_value
            }
        }
    }
}

impl Add<Tape> for Tape {
    type Output = Tape;

    fn add(mut self, rhs: Tape) -> Self::Output {
        self += rhs;
        self
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn from_arrays() {
        let x = Tape::from_arrays([1, 2, 3], [4, 5, 6]);
        assert_eq!(x.positive_values, vec![3, 2, 1]);
        assert_eq!(x.negative_values, vec![4, 5, 6]);
    }

    #[test]
    fn eq() {
        let x = Tape::from_arrays([0, 1, 2, 3], [4, 5, 6]);
        let y = Tape::from_arrays([1, 2, 3], [4, 5, 6, 0]);
        assert_eq!(x, y);
    }

    #[test]
    fn add() {
        let x = Tape::from_arrays([1, 2], [3, 4, 5, 6]);
        let y = Tape::from_arrays([1, 2, 3, 4], [5]);
        let z = x + y;
        assert_eq!(z.positive_values, vec![6, 4, 2, 1]);
        assert_eq!(z.negative_values, vec![8, 4, 5, 6]);
        assert_eq!(z, Tape::from_arrays([1, 2, 4, 6], [8, 4, 5, 6]));
    }

    #[test]
    fn is_valid() {
        let rule = Rule::from_array([1, 1]).unwrap();
        assert!(!Tape::from_arrays([1, 2], [3, 4]).is_valid(&rule));
        assert!(Tape::from_arrays([1, 1], [1, 1]).is_valid(&rule));
        assert!(Tape::from_arrays([1, 0], [1, 0]).is_valid(&rule));
    }

    #[test]
    fn is_standard() {
        let rule = Rule::from_array([1, 1]).unwrap();
        assert!(!Tape::from_arrays([1, 1], [1, 1]).is_standard(&rule));
        assert!(Tape::from_arrays([1, 0], [1]).is_standard(&rule));
        assert!(!Tape::from_arrays([1, 1, 1], [1, 1, 1]).is_standard(&rule));
        assert!(!Tape::from_arrays([1, 1, 1], [1, 1, 0]).is_standard(&rule));
        assert!(!Tape::from_arrays([1, 0, 1], [0, 1, 1]).is_standard(&rule));
    }

    #[test]
    fn apply_rule_1() {
        let tape = Tape::from_arrays([5], [2]);
        let rule = Rule::from_array([2]).unwrap();
        let index = 0;
        let result = tape.apply(&rule, index).unwrap();
        assert_relative_eq!(tape.value(&rule), 6.);
        assert_relative_eq!(result.value(&rule), 6.);
    }

    proptest! {
        #[test]
        fn apply_rule(tape_negatives in proptest::collection::vec(0u32..=100, 0..10),
                tape_positives in proptest::collection::vec(0u32..=100, 0..10),
                rule_values in proptest::collection::vec(1u32..=10, 1..10),
                index in -10isize..10) {
            let tape = Tape::from_arrays(tape_positives, tape_negatives);
            if let Some(rule) = Rule::from_array(rule_values) {
                if let Ok(result) = tape.apply(&rule, index) {
                    let tape_value = tape.value(&rule);
                    let result_value = result.value(&rule);
                    assert_relative_eq!(result_value, tape_value);
                }
            }
        }

        #[test]
        fn standardize(max in 1u32..=20, tape_negatives in proptest::collection::vec(0u32..=20, 0..10),
        tape_positives in proptest::collection::vec(0u32..=20, 0..10), rule_values in proptest::collection::vec(1u32..=10, 0..10)) {
            let tape_negatives: Vec<_> = tape_negatives.iter().map(|&x| x % max).collect();
            let tape_positives: Vec<_> = tape_positives.iter().map(|&x| x % max).collect();
            let tape = Tape::from_arrays(tape_positives, tape_negatives);
            let rule_values: Vec<_> = std::iter::once(max).chain(rule_values.iter().map(|&x| x % max)).collect();
            if let Some(rule) = Rule::from_array(rule_values) {
                let result = tape.standardize(&rule);
                let tape_value = tape.value(&rule);
                let result_value = result.value(&rule);
                assert_relative_eq!(result_value, tape_value);
            }
        }
    }
}
