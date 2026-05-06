use std::{collections::HashMap, ops, path::Path};

/// A borrowed view of one row in a [`DataFrame`].
pub struct Row<'a> {
    df: &'a DataFrame,
    idx: usize,
}

impl<'a> ops::Index<&str> for Row<'a> {
    type Output = f64;

    fn index(&self, col: &str) -> &f64 {
        let ci = self
            .df
            .col_index(col)
            .unwrap_or_else(|| panic!("column `{col}` not found"));
        &self.df.data[ci][self.idx]
    }
}

/// Column-major data frame loaded from a CSV file.
///
/// All values are stored as `f64`. Columns with non-numeric values are skipped.
#[derive(Clone)]
pub struct DataFrame {
    headers: Vec<String>,
    data: Vec<Vec<f64>>,
    len: usize,
}

impl DataFrame {
    /// Load a CSV file into a `DataFrame`.
    ///
    /// Rows where *any* column fails to parse as `f64` are silently dropped.
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_path(path)?;
        let headers: Vec<String> = rdr.headers()?.iter().map(str::to_owned).collect();
        let ncols = headers.len();
        let mut data: Vec<Vec<f64>> = vec![Vec::new(); ncols];

        for result in rdr.records() {
            let record = result?;
            let parsed: Option<Vec<f64>> = record
                .iter()
                .take(ncols)
                .map(|s| s.parse::<f64>().ok())
                .collect();
            if let Some(row) = parsed {
                for (ci, val) in row.into_iter().enumerate() {
                    data[ci].push(val);
                }
            }
        }

        let len = data.first().map_or(0, |c| c.len());
        Ok(DataFrame { headers, data, len })
    }

    /// Number of rows.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the frame has no rows.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Names of all columns in order.
    pub fn columns(&self) -> &[String] {
        &self.headers
    }

    /// Return a slice of values for a named column.
    pub fn col(&self, name: &str) -> &[f64] {
        let ci = self
            .col_index(name)
            .unwrap_or_else(|| panic!("column `{name}` not found"));
        &self.data[ci]
    }

    /// Borrow a row by index.
    pub fn row(&self, idx: usize) -> Row<'_> {
        assert!(idx < self.len, "row index {idx} out of bounds (len={})", self.len);
        Row { df: self, idx }
    }

    /// Keep only rows that satisfy `predicate`.
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(Row<'_>) -> bool,
    {
        let mask: Vec<bool> = (0..self.len).map(|i| predicate(self.row(i))).collect();
        let new_len = mask.iter().filter(|&&b| b).count();
        let mut new_data: Vec<Vec<f64>> = vec![Vec::with_capacity(new_len); self.data.len()];
        for i in 0..self.len {
            if mask[i] {
                for (ci, col) in self.data.iter().enumerate() {
                    new_data[ci].push(col[i]);
                }
            }
        }
        DataFrame { headers: self.headers.clone(), data: new_data, len: new_len }
    }

    /// Add a new column derived from existing columns.
    ///
    /// `f` receives a [`Row`] and should return the derived value.
    pub fn with_column<F>(self, name: &str, f: F) -> Self
    where
        F: Fn(Row<'_>) -> f64,
    {
        assert!(!self.headers.contains(&name.to_owned()), "column `{name}` already exists");
        let mut new_data = self.data.clone();
        let derived: Vec<f64> = (0..self.len).map(|i| f(self.row(i))).collect();
        new_data.push(derived);
        let mut new_headers = self.headers.clone();
        new_headers.push(name.to_owned());
        DataFrame { headers: new_headers, data: new_data, len: self.len }
    }

    /// Group rows by the unique sorted values in `key_col`.
    ///
    /// Returns a [`GroupedFrame`] where groups are sorted by their key value.
    pub fn group_by(&self, key_col: &str) -> GroupedFrame {
        let keys = self.col(key_col);

        // Collect unique keys, preserving first-occurrence order but then sorting.
        let mut seen: HashMap<u64, f64> = HashMap::new();
        for &k in keys {
            let bits = k.to_bits();
            seen.entry(bits).or_insert(k);
        }
        let mut unique: Vec<f64> = seen.into_values().collect();
        unique.sort_by(f64::total_cmp);

        // Build groups: for each unique key, collect matching row indices.
        let key_to_idx: HashMap<u64, usize> = unique
            .iter()
            .enumerate()
            .map(|(i, &k)| (k.to_bits(), i))
            .collect();

        let mut groups: Vec<Vec<usize>> = vec![Vec::new(); unique.len()];
        for i in 0..self.len {
            let gi = key_to_idx[&keys[i].to_bits()];
            groups[gi].push(i);
        }

        GroupedFrame { df: self.clone(), key_col: key_col.to_owned(), unique_keys: unique, groups }
    }

    fn col_index(&self, name: &str) -> Option<usize> {
        self.headers.iter().position(|h| h == name)
    }
}

/// A [`DataFrame`] grouped by a key column.
///
/// Groups are sorted by key value (ascending).
pub struct GroupedFrame {
    pub(crate) df: DataFrame,
    pub(crate) key_col: String,
    /// Sorted unique key values.
    pub(crate) unique_keys: Vec<f64>,
    /// `groups[i]` = row indices belonging to group `i`.
    pub(crate) groups: Vec<Vec<usize>>,
}

impl GroupedFrame {
    /// Number of groups (unique key values).
    pub fn num_groups(&self) -> usize {
        self.unique_keys.len()
    }

    /// Sorted key values (x-axis values).
    pub fn keys(&self) -> &[f64] {
        &self.unique_keys
    }

    /// Column name used for grouping.
    pub fn key_col(&self) -> &str {
        &self.key_col
    }

    /// Collect values of `col` for group `gi`.
    pub fn group_values(&self, gi: usize, col: &str) -> Vec<f64> {
        self.groups[gi]
            .iter()
            .map(|&ri| self.df.col(col)[ri])
            .collect()
    }
}
