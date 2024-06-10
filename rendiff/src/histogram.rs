use core::fmt;

#[cfg(doc)]
use crate::{Difference, Threshold};

/// A [histogram] of color differences.
///
/// Obtain this from [`Difference::histogram()`], and compare it to a [`Threshold`]
/// to decide whether your image comparison test passes or fails.
///
/// The histogram is represented as an array whose indices are the difference magnitudes,
/// and whose values are the count of how many differences of that magnitude were found.
/// (For example, the first (index `0`) element of the array contains a count of the pixels
/// that were considered equal.)
///
/// This type does not attribute any meaning, units, scale, or linearity to the difference
/// magnitudes;
/// only that there are 256 of them, they are ordered, and `0` is no difference.
///
/// [histogram]: https://en.wikipedia.org/wiki/Histogram
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_structs)]
pub struct Histogram(pub [usize; 256]);

impl Histogram {
    /// The histogram with all bins zero.
    pub const ZERO: Self = Self([0; 256]);

    /// Returns the maximum difference; that is, the index of the highest entry whose count
    /// is not zero.
    ///
    /// Returns zero if the histogram is entirely empty.
    ///
    /// # Example
    ///
    /// ```
    /// let mut histogram = rendiff::Histogram::ZERO;
    /// assert_eq!(histogram.max_difference(), 0);
    ///
    /// histogram.0[10] = 44;
    /// histogram.0[50] = 17;
    /// assert_eq!(histogram.max_difference(), 50);
    /// ```
    #[must_use]
    pub fn max_difference(&self) -> u8 {
        match self.0.iter().rposition(|&count| count != 0) {
            #[allow(clippy::cast_possible_truncation)] // impossible
            Some(position) => position as u8,
            None => 0,
        }
    }
}

impl fmt::Debug for Histogram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let alternate = f.alternate();

        write!(f, "Histogram(")?;

        let mut first = true;
        for (delta, count) in self
            .0
            .iter()
            .enumerate()
            .filter(|&(delta, &count)| count > 0 && (delta > 0 || alternate))
        {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "Δ{delta} ×{count}")?;
        }

        write!(f, ")")
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

    #[test]
    fn max_difference() {
        assert_eq!(Histogram::ZERO.max_difference(), 0);
        assert_eq!(Histogram([10; 256]).max_difference(), 255);
        assert_eq!(
            {
                let mut h = [0; 256];
                h[0] = 1000;
                h[10] = 5;
                h[50] = 1;
                Histogram(h)
            }
            .max_difference(),
            50,
        );
    }
}
