//! Descriptive statistics: median, quartiles.

/// Compute the median of a slice of values.
///
/// Returns `0.0` for an empty slice.
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 0 {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    } else {
        v[n / 2]
    }
}

/// Compute the mean of a slice of values.
///
/// Returns `0.0` for an empty slice.
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// First quartile (Q1): median of the lower half.
///
/// Uses the "exclusive" method (the median itself is excluded from both halves).
pub fn q1(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let lower = &v[..n / 2];
    median(lower)
}

/// Third quartile (Q3): median of the upper half.
pub fn q3(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    // Skip the median element for odd-length vectors.
    let upper = &v[(n + 1) / 2..];
    median(upper)
}

/// Per-group statistics produced by [`crate::data::GroupedFrame`].
#[derive(Debug, Clone)]
pub struct GroupStats {
    pub x: f64,
    pub median: f64,
    pub q1: f64,
    pub q3: f64,
}

impl GroupStats {
    pub fn iqr(&self) -> f64 {
        self.q3 - self.q1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_odd() {
        assert_eq!(median(&[3.0, 1.0, 2.0]), 2.0);
    }

    #[test]
    fn test_median_even() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
    }

    #[test]
    fn test_q1_q3() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert_eq!(q1(&v), 2.5);
        assert_eq!(q3(&v), 6.5);
    }

    #[test]
    fn test_median_empty() {
        assert_eq!(median(&[]), 0.0);
    }
}
