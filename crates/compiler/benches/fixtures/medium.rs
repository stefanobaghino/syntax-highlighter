//! Medium-sized Rust bench fixture: exercises items, generics,
//! match, closures, and macros. Intentionally self-contained (no
//! external crate imports) so the grammar sees the full surface
//! without a `use` cluster obscuring it.

use std::collections::HashMap;
use std::fmt::{self, Debug};

#[derive(Debug, Clone)]
pub struct Counter<'a, T: Clone + 'a> {
    name: &'a str,
    values: Vec<T>,
    histogram: HashMap<String, usize>,
}

impl<'a, T: Clone + Debug + 'a> Counter<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            values: Vec::new(),
            histogram: HashMap::new(),
        }
    }

    pub fn push(&mut self, v: T) {
        let key = format!("{:?}", v);
        *self.histogram.entry(key).or_insert(0) += 1;
        self.values.push(v);
    }

    pub fn summary(&self) -> String {
        let top = self
            .histogram
            .iter()
            .max_by_key(|(_, &n)| n)
            .map(|(k, n)| format!("{}={}", k, n))
            .unwrap_or_else(|| "<empty>".into());
        format!("{} (n={}, top={})", self.name, self.values.len(), top)
    }
}

pub fn classify(n: i64) -> &'static str {
    match n {
        i if i < 0 => "negative",
        0 => "zero",
        1..=9 => "single-digit",
        10..=99 => "two-digit",
        _ => "large",
    }
}

pub fn with_each<F: FnMut(i64)>(xs: &[i64], mut f: F) {
    for &x in xs {
        f(x);
    }
}

fn main() {
    let mut c: Counter<i64> = Counter::new("ages");
    let data = vec![1i64, 2, 2, 3, 10, 10, 10, -1];
    with_each(&data, |x| c.push(x));
    println!("{}", c.summary());
    for n in data {
        println!("{} -> {}", n, classify(n));
    }
}
