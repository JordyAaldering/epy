use std::{collections::HashMap, path::Path};

use serde::de::DeserializeOwned;

#[derive(Clone)]
/// A typed collection of CSV rows loaded via `serde`.
pub struct DataFrame<T> {
    data: Vec<T>,
}

/// A [`DataFrame`] grouped by a numeric key.
///
/// Groups are sorted by key value (ascending).
pub struct GroupedFrame<T> {
    pub(crate) df: DataFrame<T>,
    /// Sorted unique key values.
    pub(crate) unique_keys: Vec<f64>,
    /// `groups[i]` = row indices belonging to group `i`.
    pub(crate) groups: Vec<Vec<usize>>,
}

impl<T> DataFrame<T> {
    /// Load a CSV file into a `DataFrame`.
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned,
    {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut data = Vec::new();

        for result in rdr.deserialize() {
            let row: T = result?;
            data.push(row);
        }

        Ok(DataFrame { data })
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Borrow all rows.
    pub fn rows(&self) -> &[T] {
        &self.data
    }

    /// Borrow a row by index.
    pub fn row(&self, idx: usize) -> &T {
        assert!(idx < self.len(), "row index {idx} out of bounds (len={})", self.len());
        &self.data[idx]
    }

    /// Keep only rows that satisfy `predicate`.
    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        self.data.retain(|row| predicate(row));
        self
    }

    /// Group rows by the unique sorted values produced by `key_selector`.
    ///
    /// Returns a [`GroupedFrame`] where groups are sorted by their key value.
    pub fn group_by<F>(self, key_selector: F) -> GroupedFrame<T>
    where
        F: Fn(&T) -> f64,
    {
        let mut seen: HashMap<u64, f64> = HashMap::new();
        for row in &self.data {
            let k = key_selector(row);
            let bits = k.to_bits();
            seen.entry(bits).or_insert(k);
        }
        let mut unique: Vec<f64> = seen.into_values().collect();
        unique.sort_by(f64::total_cmp);

        let key_to_idx: HashMap<u64, usize> = unique
            .iter()
            .enumerate()
            .map(|(i, &k)| (k.to_bits(), i))
            .collect();

        let mut groups: Vec<Vec<usize>> = vec![Vec::new(); unique.len()];
        for (i, row) in self.data.iter().enumerate() {
            let gi = key_to_idx[&key_selector(row).to_bits()];
            groups[gi].push(i);
        }

        GroupedFrame { df: self, unique_keys: unique, groups }
    }
}

impl<T> GroupedFrame<T> {
    /// Number of groups (unique key values).
    pub fn num_groups(&self) -> usize {
        self.unique_keys.len()
    }

    /// Sorted key values (x-axis values).
    pub fn keys(&self) -> &[f64] {
        &self.unique_keys
    }

    /// Collect values produced by `selector` for group `gi`.
    pub fn group_values<F>(&self, gi: usize, selector: &F) -> Vec<f64>
    where
        F: Fn(&T) -> f64 + ?Sized,
    {
        self.groups[gi]
            .iter()
            .map(|&ri| selector(&self.df.data[ri]))
            .collect()
    }
}
