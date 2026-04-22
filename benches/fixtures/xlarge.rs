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
pub struct XCounter0<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCounter0<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCounter0<'a, T> {
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
pub struct XBucket1<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBucket1<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBucket1<'a, T> {
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
pub struct XLedger2<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XLedger2<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XLedger2<'a, T> {
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
pub struct XRegistry3<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XRegistry3<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XRegistry3<'a, T> {
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
pub struct XCache4<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCache4<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCache4<'a, T> {
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

#[derive(Debug, Clone)]
pub struct XJournal5<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XJournal5<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XJournal5<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_5(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XPool6<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XPool6<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XPool6<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_6(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XQueue7<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XQueue7<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XQueue7<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_7(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XSlab8<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XSlab8<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XSlab8<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_8(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XStore9<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XStore9<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XStore9<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_9(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XIndex10<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XIndex10<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XIndex10<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_10(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XGraph11<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XGraph11<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XGraph11<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_11(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XScanner12<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XScanner12<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XScanner12<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_12(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XMonitor13<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XMonitor13<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XMonitor13<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_13(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XTracker14<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XTracker14<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XTracker14<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_14(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBroker15<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBroker15<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBroker15<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_15(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCounter16<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCounter16<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCounter16<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_16(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBucket17<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBucket17<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBucket17<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_17(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XLedger18<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XLedger18<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XLedger18<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_18(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XRegistry19<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XRegistry19<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XRegistry19<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_19(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCache20<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCache20<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCache20<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_20(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XJournal21<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XJournal21<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XJournal21<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_21(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XPool22<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XPool22<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XPool22<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_22(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XQueue23<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XQueue23<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XQueue23<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_23(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XSlab24<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XSlab24<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XSlab24<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_24(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XStore25<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XStore25<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XStore25<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_25(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XIndex26<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XIndex26<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XIndex26<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_26(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XGraph27<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XGraph27<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XGraph27<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_27(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XScanner28<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XScanner28<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XScanner28<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_28(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XMonitor29<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XMonitor29<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XMonitor29<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_29(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XTracker30<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XTracker30<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XTracker30<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_30(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBroker31<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBroker31<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBroker31<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_31(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCounter32<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCounter32<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCounter32<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_32(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBucket33<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBucket33<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBucket33<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_33(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XLedger34<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XLedger34<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XLedger34<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_34(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XRegistry35<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XRegistry35<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XRegistry35<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_35(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCache36<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCache36<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCache36<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_36(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XJournal37<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XJournal37<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XJournal37<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_37(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XPool38<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XPool38<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XPool38<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_38(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XQueue39<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XQueue39<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XQueue39<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_39(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XSlab40<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XSlab40<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XSlab40<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_40(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XStore41<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XStore41<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XStore41<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_41(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XIndex42<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XIndex42<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XIndex42<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_42(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XGraph43<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XGraph43<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XGraph43<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_43(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XScanner44<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XScanner44<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XScanner44<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_44(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XMonitor45<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XMonitor45<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XMonitor45<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_45(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XTracker46<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XTracker46<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XTracker46<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_46(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBroker47<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBroker47<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBroker47<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_47(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCounter48<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCounter48<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCounter48<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_48(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XBucket49<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XBucket49<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XBucket49<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_49(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XLedger50<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XLedger50<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XLedger50<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_50(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XRegistry51<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XRegistry51<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XRegistry51<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_51(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XCache52<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XCache52<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XCache52<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_52(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XJournal53<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XJournal53<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XJournal53<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_53(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XPool54<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XPool54<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XPool54<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_54(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XQueue55<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XQueue55<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XQueue55<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_55(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XSlab56<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XSlab56<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XSlab56<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_56(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XStore57<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XStore57<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XStore57<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_57(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XIndex58<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XIndex58<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XIndex58<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_58(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
pub struct XGraph59<'a, T: Clone + Debug + 'a> {
    pub name: &'a str,
    pub field_0: Vec<T>,
    pub field_1: Vec<T>,
    pub field_2: Vec<T>,
    pub field_3: Vec<T>,
    pub index: HashMap<String, usize>,
    pub tags: HashSet<String>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Clone + Debug + 'a> XGraph59<'a, T> {
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

impl<'a, T: Clone + Debug + 'a> Emit<T> for XGraph59<'a, T> {
    fn emit(&mut self, value: T) {
        let _ = self.push(0, value);
    }
    fn count(&self) -> usize {
        self.index.values().sum()
    }
}

pub fn demo_59(xs: &[i64]) -> Vec<(i64, &'static str)> {
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
    let mut c = XCounter0::<i64>::new("xs");
    let data: Vec<i64> = vec![-5, -1, 0, 1, 2, 3, 9, 10, 42, 99, 100];
    for n in &data {
        c.emit(*n);
    }
    println!("{} (n={})", c.summary(), c.count());
    for (n, label) in demo_0(&data) {
        println!("{} -> {}", n, label);
    }
}
