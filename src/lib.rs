//! Calculates the difference between two lists using Paul Heckel's algorithm from 1978.
//! (<https://dl.acm.org/doi/10.1145/359460.359467>)

#[cfg(test)]
mod test;

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq)]
pub enum Change<T> {
    /// Delete value with the old position.
    Delete(T, usize),
    /// Insert value with the new position.
    Insert(T, usize),
    /// Unchanged value with the old and new positions.
    Unchanged(T, usize, usize),
}

#[derive(Debug)]
struct Line<'a, T>
where
    T: Eq + Hash,
{
    symbol_key: Option<&'a T>,
    other_position: Option<usize>,
}

#[derive(Copy, Clone, Debug)]
struct Symbol {
    new_copies: usize,
    old_copies: usize,
    old_position: usize,
}

impl<'a, T> Line<'a, T>
where
    T: Eq + Hash,
{
    fn from_key(key: &'a T) -> Self {
        Self {
            symbol_key: Some(key),
            other_position: None,
        }
    }

    fn from_position(position: usize) -> Self {
        Self {
            symbol_key: None,
            other_position: Some(position),
        }
    }
}

impl Symbol {
    fn new() -> Self {
        Self {
            new_copies: 0,
            old_copies: 0,
            old_position: 0,
        }
    }
}

fn connect_neighbours_ascending<T>(oa: &mut [Line<T>], na: &mut [Line<T>])
where
    T: Eq + Hash,
{
    for i in 0..na.len() - 1 {
        if let Some(p) = na[i].other_position
            && p < oa.len() - 1
            && same_symbol(&na[i + 1], &oa[p + 1])
        {
            oa[p + 1] = Line::from_position(i + 1);
            na[i + 1] = Line::from_position(p + 1);
        }
    }
}

fn connect_neighbours_descending<T>(oa: &mut [Line<T>], na: &mut [Line<T>])
where
    T: Eq + Hash,
{
    for i in (1..na.len()).rev() {
        if let Some(p) = na[i].other_position
            && p > 0
            && same_symbol(&na[i - 1], &oa[p - 1])
        {
            oa[p - 1] = Line::from_position(i - 1);
            na[i - 1] = Line::from_position(p - 1);
        }
    }
}

/// The difference between the two slices is calculated. The changes are in the order that
/// constructs the new version from the old version.
/// ```rust
/// # use heckel_diff::{diff, new_version, old_version};
/// # use heckel_diff::Change::{Delete, Insert, Unchanged};
/// # fn main() {
/// let old = vec!["MUCH", "WRITING", "IS", "LIKE", "SNOW", ",",
///               "A", "MASS", "OF", "LONG", "WORDS", "AND",
///               "PHRASES", "FALLS", "UPON", "THE", "RELEVANT",
///               "FACTS", "COVERING", "UP", "THE", "DETAILS", "."];
///let new = vec!["A", "MASS", "OF", "LATIN", "WORDS", "FALLS",
///               "UPON", "THE", "RELEVANT", "FACTS", "LIKE", "SOFT",
///               "SNOW", ",", "COVERING", "UP", "THE", "DETAILS", "."];
///let answer = vec![Delete(&"MUCH", 0), Delete(&"WRITING", 1), Delete(&"IS", 2),
///                  Unchanged(&"A",6, 0), Unchanged(&"MASS", 7, 1), Unchanged(&"OF", 8, 2),
///                  Insert(&"LATIN", 3), Delete(&"LONG", 9), Unchanged(&"WORDS", 10, 4),
///                  Delete(&"AND", 11), Delete(&"PHRASES", 12),Unchanged(&"FALLS", 13, 5),
///                  Unchanged(&"UPON", 14, 6), Unchanged(&"THE", 15, 7),
///                  Unchanged(&"RELEVANT", 16, 8), Unchanged(&"FACTS", 17, 9),
///                  Unchanged(&"LIKE", 3, 10), Insert(&"SOFT", 11), Unchanged(&"SNOW", 4, 12),
///                  Unchanged(&",", 5, 13), Unchanged(&"COVERING", 18, 14),
///                  Unchanged(&"UP", 19, 15), Unchanged(&"THE", 20, 16),
///                  Unchanged(&"DETAILS", 21, 17), Unchanged(&".", 22, 18)];
///let calculated = diff(&old, &new);
///let new_version: Vec<&str> =
///    new_version(calculated.as_slice()).into_iter().copied().collect();
///let old_version: Vec<&str> =
///    old_version(calculated.as_slice()).into_iter().copied().collect();
///
///assert_eq!(answer, calculated);
///assert_eq!(new, new_version);
///assert_eq!(old, old_version);
/// # }
/// ```
pub fn diff<'a, T>(old: &'a [T], new: &'a [T]) -> Vec<Change<&'a T>>
where
    T: Eq + Hash,
{
    let mut na: Vec<Line<T>> = new.iter().map(|v| Line::from_key(v)).collect();
    let mut oa: Vec<Line<T>> = old.iter().map(|v| Line::from_key(v)).collect();
    let mut symbol_table: HashMap<&T, Symbol> = HashMap::new();

    new.iter()
        .for_each(|v| symbol_table.entry(v).or_insert(Symbol::new()).new_copies += 1);
    old.iter().enumerate().for_each(|(i, v)| {
        let e = symbol_table.entry(v).or_insert(Symbol::new());
        e.old_copies += 1;
        e.old_position = i;
    });

    connect_ends(&mut oa, &mut na);
    make_connections(&mut oa, &mut na, &symbol_table);
    connect_neighbours_ascending(&mut oa, &mut na);
    connect_neighbours_descending(&mut oa, &mut na);
    encode_changes(old, new, &oa, &na)
}

fn connect_ends<T>(oa: &mut Vec<Line<T>>, na: &mut Vec<Line<T>>)
where
    T: Eq + Hash,
{
    na.insert(0, Line::from_position(0));
    oa.insert(0, Line::from_position(0));
    na.push(Line::from_position(oa.len()));
    oa.push(Line::from_position(na.len() - 1));
}

fn encode_changes<'a, T>(
    old: &'a [T],
    new: &'a [T],
    oa: &[Line<T>],
    na: &[Line<T>],
) -> Vec<Change<&'a T>>
where
    T: Eq + Hash,
{
    let mut old_position = 1;
    let mut result = vec![];

    for i in 1..na.len() - 1 {
        if na[i].other_position.is_none() {
            result.push(Change::Insert(&new[i - 1], i - 1))
        } else {
            let other_position = na[i].other_position.unwrap();

            for j in old_position..other_position {
                if oa[j].other_position.is_none() {
                    result.push(Change::Delete(&old[j - 1], j - 1));
                }

                old_position += 1;
            }

            result.push(Change::Unchanged(&new[i - 1], other_position - 1, i - 1));
        }
    }

    for i in old_position..old.len() {
        if oa[i].other_position.is_none() {
            result.push(Change::Delete(&old[i - 1], i - 1));
        }
    }

    result
}

fn make_connections<T>(oa: &mut [Line<T>], na: &mut [Line<T>], symbol_table: &HashMap<&T, Symbol>)
where
    T: Eq + Hash,
{
    for i in 1..na.len() - 1 {
        if let Some(s) = na[i]
            .symbol_key
            .and_then(|k| symbol_table.get(k))
            .filter(|s| s.old_copies == 1 && s.new_copies == 1)
        {
            na[i] = Line::from_position(s.old_position + 1);
            oa[s.old_position + 1] = Line::from_position(i);
        }
    }
}

fn new_position<T>(change: &Change<&T>) -> usize {
    match change {
        Change::Delete(_, _) => usize::MAX,
        Change::Insert(_, p) => *p,
        Change::Unchanged(_, _, p) => *p,
    }
}

/// Produces the new version of the list from the calculated changes.
pub fn new_version<'a, T>(changes: &'a [Change<&T>]) -> Vec<&'a T> {
    let mut filtered: Vec<&Change<&T>> = changes
        .iter()
        .flat_map(|c| match c {
            Change::Delete(_, _) => None,
            Change::Insert(_, _) => Some(c),
            Change::Unchanged(_, _, _) => Some(c),
        })
        .collect();

    filtered.sort_by_key(|c| new_position(c));

    changes
        .iter()
        .flat_map(|c| match c {
            Change::Delete(_, _) => None,
            Change::Insert(v, _) => Some(*v),
            Change::Unchanged(v, _, _) => Some(*v),
        })
        .collect()
}

fn old_position<T>(change: &Change<&T>) -> usize {
    match change {
        Change::Delete(_, p) => *p,
        Change::Insert(_, _) => usize::MAX,
        Change::Unchanged(_, p, _) => *p,
    }
}

/// Produces the old version of the list from the calculated changes.
pub fn old_version<'a, T>(changes: &'a [Change<&T>]) -> Vec<&'a T> {
    let mut filtered: Vec<&Change<&T>> = changes
        .iter()
        .flat_map(|c| match c {
            Change::Delete(_, _) => Some(c),
            Change::Insert(_, _) => None,
            Change::Unchanged(_, _, _) => Some(c),
        })
        .collect();

    filtered.sort_by_key(|c| old_position(c));

    filtered
        .iter()
        .flat_map(|c| match c {
            Change::Delete(v, _) => Some(*v),
            Change::Insert(_, _) => None,
            Change::Unchanged(v, _, _) => Some(*v),
        })
        .collect()
}

fn same_symbol<T>(line1: &Line<T>, line2: &Line<T>) -> bool
where
    T: Eq + Hash,
{
    line1.symbol_key.is_some()
        && line2.symbol_key.is_some()
        && line1.symbol_key.unwrap() == line2.symbol_key.unwrap()
}
