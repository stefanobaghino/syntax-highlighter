//! Generated Rust bench fixture. Mix of items, generics, match,
//! closures, macros and trait impls. Parses to completion under
//! grammars/rust.peg — real crates stress the grammar similarly.

use std::collections::{HashMap, HashSet, BTreeMap};
use std::fmt::{self, Debug, Display};
use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity { Info, Warn, Error(String) }

pub trait Emit<T: Debug> {
    fn emit(&mut self, value: T);
    fn count(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct Counter0<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> Counter0<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            field_0: Vec::new(),
            field_1: Vec::new(),
            index: HashMap::new(),
            tags: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub fn push<U: Into<T>>(&mut self, slot: usize, u: U) -> Result<&mut Self, String> {
        let value: T = u.into();
        let key = format!("{:?}", value);
        *self.index.entry(key).or_insert(0) += 1;
        match slot {
            0 => self.field_0.push(value),
            1 => self.field_1.push(value),
            other => return Err(format!("bad slot {}", other)),
        }
        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut lens: Vec<usize> = vec![
            self.field_0.len(),
            self.field_1.len(),
        ];
        lens.sort_by(|a, b| b.cmp(a));
        let top = lens.first().copied().unwrap_or(0);
        format!("{}(top={}, tags={})", self.name, top, self.tags.len())
    }

    pub fn tag(&mut self, t: impl Into<String>) -> &mut Self {
        self.tags.insert(t.into());
        self
    }
}

impl<'a, T: Clone + Debug + 'a> Emit<T> for Counter0<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_0(xs: &[i64]) -> Vec<(i64, &'static str)> {
    xs.iter()
        .map(|&n| (n, match n {
            i if i < 0 => "negative",
            0 => "zero",
            1..=9 => "single-digit",
            10..=99 => "two-digit",
            _ => "large",
        }))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub struct Bucket1<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> Bucket1<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            field_0: Vec::new(),
            field_1: Vec::new(),
            field_2: Vec::new(),
            index: HashMap::new(),
            tags: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub fn push<U: Into<T>>(&mut self, slot: usize, u: U) -> Result<&mut Self, String> {
        let value: T = u.into();
        let key = format!("{:?}", value);
        *self.index.entry(key).or_insert(0) += 1;
        match slot {
            0 => self.field_0.push(value),
            1 => self.field_1.push(value),
            2 => self.field_2.push(value),
            other => return Err(format!("bad slot {}", other)),
        }
        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut lens: Vec<usize> = vec![
            self.field_0.len(),
            self.field_1.len(),
            self.field_2.len(),
        ];
        lens.sort_by(|a, b| b.cmp(a));
        let top = lens.first().copied().unwrap_or(0);
        format!("{}(top={}, tags={})", self.name, top, self.tags.len())
    }

    pub fn tag(&mut self, t: impl Into<String>) -> &mut Self {
        self.tags.insert(t.into());
        self
    }
}

impl<'a, T: Clone + Debug + 'a> Emit<T> for Bucket1<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_1(xs: &[i64]) -> Vec<(i64, &'static str)> {
    xs.iter()
        .map(|&n| (n, match n {
            i if i < 0 => "negative",
            0 => "zero",
            1..=9 => "single-digit",
            10..=99 => "two-digit",
            _ => "large",
        }))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub struct Ledger2<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> Ledger2<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            field_0: Vec::new(),
            field_1: Vec::new(),
            field_2: Vec::new(),
            field_3: Vec::new(),
            index: HashMap::new(),
            tags: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub fn push<U: Into<T>>(&mut self, slot: usize, u: U) -> Result<&mut Self, String> {
        let value: T = u.into();
        let key = format!("{:?}", value);
        *self.index.entry(key).or_insert(0) += 1;
        match slot {
            0 => self.field_0.push(value),
            1 => self.field_1.push(value),
            2 => self.field_2.push(value),
            3 => self.field_3.push(value),
            other => return Err(format!("bad slot {}", other)),
        }
        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut lens: Vec<usize> = vec![
            self.field_0.len(),
            self.field_1.len(),
            self.field_2.len(),
            self.field_3.len(),
        ];
        lens.sort_by(|a, b| b.cmp(a));
        let top = lens.first().copied().unwrap_or(0);
        format!("{}(top={}, tags={})", self.name, top, self.tags.len())
    }

    pub fn tag(&mut self, t: impl Into<String>) -> &mut Self {
        self.tags.insert(t.into());
        self
    }
}

impl<'a, T: Clone + Debug + 'a> Emit<T> for Ledger2<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_2(xs: &[i64]) -> Vec<(i64, &'static str)> {
    xs.iter()
        .map(|&n| (n, match n {
            i if i < 0 => "negative",
            0 => "zero",
            1..=9 => "single-digit",
            10..=99 => "two-digit",
            _ => "large",
        }))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub struct Registry3<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> Registry3<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            field_0: Vec::new(),
            field_1: Vec::new(),
            index: HashMap::new(),
            tags: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub fn push<U: Into<T>>(&mut self, slot: usize, u: U) -> Result<&mut Self, String> {
        let value: T = u.into();
        let key = format!("{:?}", value);
        *self.index.entry(key).or_insert(0) += 1;
        match slot {
            0 => self.field_0.push(value),
            1 => self.field_1.push(value),
            other => return Err(format!("bad slot {}", other)),
        }
        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut lens: Vec<usize> = vec![
            self.field_0.len(),
            self.field_1.len(),
        ];
        lens.sort_by(|a, b| b.cmp(a));
        let top = lens.first().copied().unwrap_or(0);
        format!("{}(top={}, tags={})", self.name, top, self.tags.len())
    }

    pub fn tag(&mut self, t: impl Into<String>) -> &mut Self {
        self.tags.insert(t.into());
        self
    }
}

impl<'a, T: Clone + Debug + 'a> Emit<T> for Registry3<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_3(xs: &[i64]) -> Vec<(i64, &'static str)> {
    xs.iter()
        .map(|&n| (n, match n {
            i if i < 0 => "negative",
            0 => "zero",
            1..=9 => "single-digit",
            10..=99 => "two-digit",
            _ => "large",
        }))
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
pub struct Cache4<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> Cache4<'a, T> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            field_0: Vec::new(),
            field_1: Vec::new(),
            field_2: Vec::new(),
            index: HashMap::new(),
            tags: HashSet::new(),
            _marker: PhantomData,
        }
    }

    pub fn push<U: Into<T>>(&mut self, slot: usize, u: U) -> Result<&mut Self, String> {
        let value: T = u.into();
        let key = format!("{:?}", value);
        *self.index.entry(key).or_insert(0) += 1;
        match slot {
            0 => self.field_0.push(value),
            1 => self.field_1.push(value),
            2 => self.field_2.push(value),
            other => return Err(format!("bad slot {}", other)),
        }
        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut lens: Vec<usize> = vec![
            self.field_0.len(),
            self.field_1.len(),
            self.field_2.len(),
        ];
        lens.sort_by(|a, b| b.cmp(a));
        let top = lens.first().copied().unwrap_or(0);
        format!("{}(top={}, tags={})", self.name, top, self.tags.len())
    }

    pub fn tag(&mut self, t: impl Into<String>) -> &mut Self {
        self.tags.insert(t.into());
        self
    }
}

impl<'a, T: Clone + Debug + 'a> Emit<T> for Cache4<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_4(xs: &[i64]) -> Vec<(i64, &'static str)> {
    xs.iter()
        .map(|&n| (n, match n {
            i if i < 0 => "negative",
            0 => "zero",
            1..=9 => "single-digit",
            10..=99 => "two-digit",
            _ => "large",
        }))
        .collect::<Vec<_>>()
}

fn main() {
    let mut c = Counter0::<i64>::new("xs");
    let data: Vec<i64> = vec![-5, -1, 0, 1, 2, 3, 9, 10, 42, 99, 100];
    for n in &data {
        c.emit(*n);
    }
    println!("{} (n={})", c.summary(), c.count());
    for (n, label) in demo_0(&data) {
        println!("{} -> {}", n, label);
    }
}
