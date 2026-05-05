#[derive(Debug, Clone)]
pub struct IQR {
   pub median: f64,
   pub q1: f64,
   pub q3: f64,
}

pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

pub fn median(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    if n % 2 == 0 {
        (xs[n / 2 - 1] + xs[n / 2]) / 2.0
    } else {
        xs[n / 2]
    }
}

/// First quartile (Q1): median of the lower half.
pub fn q1(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    let lower = &xs[..n / 2];
    median(lower)
}

/// Third quartile (Q3): median of the upper half.
pub fn q3(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut xs = xs.to_vec();
    xs.sort_by(f64::total_cmp);
    let n = xs.len();
    let upper = &xs[(n + 1) / 2..];
    median(upper)
}
