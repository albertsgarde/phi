use std::ops::Index;

use itertools::Itertools;

use crate::Value;

#[derive(Clone, Debug)]
pub struct Rule {
    values: Vec<Value>,
    base: f64,
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
                let rule_base = calculate_rule_base(values);
                Some(Rule {
                    values: result,
                    base: rule_base,
                })
            }
        }
    }

    pub fn first(&self) -> Value {
        self.values.first().copied().unwrap()
    }

    pub fn base(&self) -> f64 {
        self.base
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        self.values.get(index).copied()
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = Value> + '_ {
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
        self.values.get(index).unwrap_or(&0)
    }
}

fn evaluate_rule_polynomial(rule: &[Value], x: f64) -> f64 {
    let degree = i32::try_from(rule.len()).unwrap();
    -x.powi(degree)
        + rule
            .iter()
            .copied()
            .rev()
            .enumerate()
            .map(|(i, v)| f64::from(v) * x.powi(i32::try_from(i).unwrap()))
            .sum::<f64>()
}

fn calculate_rule_base(rule: &[Value]) -> f64 {
    let mut min = f64::from(rule[0]);
    let mut max = min + 1.;
    let mut min_value = evaluate_rule_polynomial(rule, min);
    let mut max_value = evaluate_rule_polynomial(rule, max);
    loop {
        assert!(min_value >= 0.);
        assert!(max_value <= 0.);
        if min == max {
            return min;
        }
        let mid = (min + max) / 2.;
        if mid == min || mid == max {
            return mid;
        }
        let mid_value = evaluate_rule_polynomial(rule, mid);
        if mid_value > 0. {
            min = mid;
            min_value = mid_value;
        } else if mid_value < 0. {
            max = mid;
            max_value = mid_value;
        } else {
            return mid;
        }
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn rule_base_whole() {
        let rule = Rule::from_array([1]).unwrap();
        assert_relative_eq!(rule.base(), 1.0);
        let rule = Rule::from_array([2]).unwrap();
        assert_relative_eq!(rule.base(), 2.0);
        let rule = Rule::from_array([3]).unwrap();
        assert_relative_eq!(rule.base(), 3.0);
    }

    #[test]
    fn rule_base_phi() {
        let rule = Rule::from_array([1, 1]).unwrap();
        let phi = (1. + 5_f64.sqrt()) / 2.;
        assert_relative_eq!(rule.base(), phi);

        let rule = Rule::from_array([1, 1, 0]).unwrap();
        let phi = (1. + 5_f64.sqrt()) / 2.;
        assert_relative_eq!(rule.base(), phi);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            timeout: 10,
            ..ProptestConfig::default()
        })]

        #[test]
        fn rule_from_array(values in proptest::collection::vec(1u32..=100, 0..10)) {
            let rule = Rule::from_array(values.clone());
            if values.is_empty() || values.iter().tuple_windows().any(|(a, b)| a < b) {
                prop_assert!(rule.is_none());
            } else {
                rule.unwrap();
            }
        }

        #[test]
        fn rule_base(values in proptest::collection::vec(1u32..=100, 1..10)) {
            let rule = Rule::from_array(values.clone());
            if values.is_empty() || values.iter().tuple_windows().any(|(a, b)| a < b) {
                prop_assert!(rule.is_none());
            } else {
                let rule = rule.unwrap();
                let first = rule.first();
                let base = rule.base();
                assert!(base >= f64::from(first));
                assert!(base < f64::from(first) + 1.);
            }
        }
    }
}
