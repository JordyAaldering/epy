use std::{collections::HashMap, path::Path};

use serde::de::DeserializeOwned;
use statrs::statistics::{Data, OrderStatistics, Statistics};

#[derive(Clone)]
/// A typed collection of CSV rows loaded via `serde`.
pub struct DataFrame<T> {
    pub(crate) rows: Vec<T>,
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

pub struct GroupedSummaryBand {
    pub keys: Vec<f64>,
    pub centers: Vec<f64>,
    pub lowers: Vec<f64>,
    pub uppers: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum AggregationMode {
    #[default]
    Quartiles,
    MeanStd {
        scale: f64
    },
}

#[derive(Clone, Copy)]
struct BandStats {
    center: f64,
    lower: f64,
    upper: f64,
}

fn summarize_band(values: &[f64], mode: AggregationMode) -> BandStats {
    assert!(!values.is_empty());
    match mode {
        AggregationMode::Quartiles => {
            let mut data = Data::new(values.to_vec());
            let center = data.median();
            let lower = data.lower_quartile();
            let upper = data.upper_quartile();
            BandStats { center, lower, upper }
        }
        AggregationMode::MeanStd { scale } => {
            let center = values.mean();
            let stddev = if values.len() > 1 {
                values.std_dev()
            } else {
                0.0
            };
            let spread = scale * stddev;
            BandStats {
                center,
                lower: center - spread,
                upper: center + spread,
            }
        }
    }
}

impl<T> DataFrame<T> {
    /// Load a CSV file into a [`DataFrame`].
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: DeserializeOwned,
    {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut rows = Vec::new();

        for result in rdr.deserialize() {
            rows.push(result?);
        }

        Ok(DataFrame { rows })
    }

    /// Create a [`DataFrame`] from a vector of rows.
    pub fn from_vec(rows: Vec<T>) -> Self {
        DataFrame { rows }
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Borrow all rows.
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Borrow a row by index.
    pub fn row(&self, idx: usize) -> &T {
        assert!(idx < self.len(), "row index {idx} out of bounds (len={})", self.len());
        &self.rows[idx]
    }

    pub fn map<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, &mut T),
    {
        self.rows.iter_mut()
            .enumerate()
            .for_each(|(i, row)|
                f(i, row)
            );
        self
    }

    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, &T) -> bool,
    {
        self.rows = self.rows.drain(..)
            .enumerate()
            .filter_map(|(i, x)|
                f(i, &x).then_some(x)
            )
            .collect();
        self
    }

    pub fn fold<F, U>(&self, init: U, f: F) -> U
    where
        F: Fn(U, &T) -> U,
    {
        self.rows.iter()
            .fold(init, |acc, row|
                f(acc, row)
            )
    }

    /// Group rows by the unique sorted values produced by `key_selector`.
    ///
    /// Returns a [`GroupedFrame`] where groups are sorted by their key value.
    pub fn group_by<F>(self, key_selector: F) -> GroupedFrame<T>
    where
        F: Fn(&T) -> f64,
    {
        let mut seen: HashMap<u64, f64> = HashMap::new();
        for row in &self.rows {
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
        for (i, row) in self.rows.iter().enumerate() {
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

    /// Compute center and lower/upper bands for each group using `selector`.
    pub fn summarize_by_group(
        &self,
        selector: &dyn Fn(&T) -> f64,
        mode: AggregationMode,
    ) -> GroupedSummaryBand {
        let mut centers = Vec::with_capacity(self.num_groups());
        let mut lowers = Vec::with_capacity(self.num_groups());
        let mut uppers = Vec::with_capacity(self.num_groups());

        for group in &self.groups {
            let vals = group
                .iter()
                .map(|&ri| selector(&self.df.rows[ri]))
                .collect::<Vec<_>>();
            let stats = summarize_band(&vals, mode);
            centers.push(stats.center);
            lowers.push(stats.lower);
            uppers.push(stats.upper);
        }

        GroupedSummaryBand {
            keys: self.keys().to_vec(),
            centers,
            lowers,
            uppers,
        }
    }

    /// Regroup rows from a single existing group by another numeric key.
    ///
    /// The returned [`GroupedFrame`] contains only rows from `group_index`.
    pub fn subgroup_by<F>(&self, group_index: usize, key_selector: F) -> GroupedFrame<T>
    where
        T: Clone,
        F: Fn(&T) -> f64,
    {
        let rows = self.groups.get(group_index)
            .expect(&format!("group index {} out of bounds (len={})", group_index, self.num_groups()))
            .iter()
            .map(|&ri| self.df.rows[ri].clone())
            .collect();
        DataFrame { rows }.group_by(key_selector)
    }
}
