use std::fmt;

use itertools::Itertools;

/// A histogram of luminance difference values.
///
/// For example, the zero element of the array contains a count of the pixels
/// that were considered equal.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_structs)]
pub struct Histogram(pub [usize; 256]);

impl Histogram {
    /// The histogram with all bins zero.
    pub const ZERO: Self = Self([0; 256]);
}

impl fmt::Debug for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Histogram({})",
            self.0
                .into_iter()
                .enumerate()
                .filter(|&(delta, count)| count > 0 && (delta > 0 || f.alternate()))
                .map(|(delta, count)| format!("Δ{delta} ×{count}"))
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_empty() {
        let zero = Histogram::ZERO;
        assert_eq!(format!("{zero:?}"), "Histogram()");
        assert_eq!(format!("{zero:#?}"), "Histogram()");
    }

    #[test]
    fn fmt_nonempty() {
        let histogram = {
            let mut h = [0; 256];
            h[0] = 1000;
            h[10] = 5;
            h[50] = 1;
            Histogram(h)
        };
        assert_eq!(format!("{histogram:?}"), "Histogram(Δ10 ×5, Δ50 ×1)");
        assert_eq!(
            format!("{histogram:#?}"),
            "Histogram(Δ0 ×1000, Δ10 ×5, Δ50 ×1)"
        );
    }
}
