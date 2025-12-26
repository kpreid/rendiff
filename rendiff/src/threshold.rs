use std::collections::BTreeMap;

use crate::Histogram;

/// A bound upon pixel differences observed in a [`Histogram`](crate::Histogram),
/// which you may use to define the pass/fail criterion for your image comparison test.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::exhaustive_structs)]
pub struct Threshold(BTreeMap<u8, usize>);

impl Threshold {
    /// Creates a [`Threshold`] from a list of (magnitude, count) pairs.
    ///
    /// Each pair means “There may be up to &lt;count&gt; differences of magnitude
    /// &lt;magnitude&gt; or less”.
    /// Lower magnitudes that are accepted by a lower entry don't count towards
    /// the limit at a higher magnitude.
    /// Differences of zero are always accepted.
    ///
    /// # Panics
    ///
    /// All magnitudes must be greater than zero (which would have no effect if it were
    /// permitted).
    #[must_use]
    pub fn new(data: impl IntoIterator<Item = (u8, usize)>) -> Self {
        Self(
            data.into_iter()
                .map(|kv @ (key, _)| {
                    assert!(key > 0, "putting 0 ({kv:?}) in Threshold is redundant");
                    kv
                })
                .collect(),
        )
    }

    /// Allow any number of pixel differences not exceeding `magnitude`.
    ///
    /// # Example
    ///
    /// ```
    /// use rendiff::{Histogram, Threshold};
    ///
    /// let threshold = Threshold::no_bigger_than(5);
    ///
    /// assert!(threshold.allows(Histogram::ZERO));
    ///
    /// // Differences greater than 5 are not permitted, no matter how few they are.
    /// assert!(!threshold.allows(Histogram([1; 256])));
    ///
    /// // Any differences are permitted if they are less than 5.
    /// assert!(threshold.allows(Histogram({
    ///     let mut table = [0; 256];
    ///     table[0] = 1000;
    ///     table[2] = 100;
    ///     table[4] = 10;
    ///     table
    /// })));
    /// ```
    #[must_use]
    pub fn no_bigger_than(magnitude: u8) -> Self {
        if magnitude == 0 {
            Self::new([])
        } else {
            Self::new([(magnitude, usize::MAX)])
        }
    }

    /// Returns whether the differences described by the given [`Histogram`] are permitted
    /// by this [`Threshold`].
    ///
    /// This is equivalent to `self.remove_allowed_differences_from(histogram) == Histogram::ZERO`.
    #[must_use]
    pub fn allows(&self, histogram: Histogram) -> bool {
        self.remove_allowed_differences_from(histogram) == Histogram::ZERO
    }

    /// Modify the given [`Histogram`] so that it does not include any of the differences which
    /// this [`Threshold`] permits. The return value is the remaining, disallowed differences.
    ///
    /// # Example
    ///
    /// ```
    /// use rendiff::{Histogram, Threshold};
    ///
    /// let histogram = Histogram({
    ///     let mut table = [0; 256];
    ///     table[1] = 1000;
    ///     table[2] = 100;
    ///     table[4] = 10;
    ///     table
    /// });
    ///
    /// // The histogram is allowed by this threshold, so `remove_allowed_differences_from`
    /// // returns zero.
    /// assert_eq!(
    ///     Threshold::no_bigger_than(5).remove_allowed_differences_from(histogram),
    ///     Histogram::ZERO,
    /// );
    ///
    /// // This threshold is too small in both magnitude and count, so there is a remainder
    /// // in bin 1 and bin 4 is passed through fully.
    /// assert_eq!(
    ///     Threshold::new([(3, 1050)]).remove_allowed_differences_from(histogram),
    ///     Histogram({
    ///         let mut table = [0; 256];
    ///         table[1] = 50; // residual small difference
    ///         table[4] = 10; // above threshold
    ///         table
    ///     }),
    /// );
    /// ```
    #[must_use]
    pub fn remove_allowed_differences_from(&self, mut histogram: Histogram) -> Histogram {
        // Always accept any number of zero-value differences.
        histogram.0[0] = 0;

        // Loop over the thresholds, in ascending order so that we prefer spending low-magnitude
        // allowance on low-magnitude differences instead of spending high-magnitude allowance on
        // low-magnitude differences.
        for (&allowed_magnitude, &(mut allowed_count)) in &self.0 {
            // Slice the portion of the histogram that is affected.
            let affected_region_of_histogram = &mut histogram.0[1..=usize::from(allowed_magnitude)];

            // Subtract all allowed differences from both the histogram and the threshold.
            // This is done in descending order of magnitude, so that the final result will favor
            // reporting excess small differences not covered by a high-magnitude allowance over
            // reporting excess large differences that look like they should have been covered.
            for h_count in affected_region_of_histogram.iter_mut().rev() {
                let deduction: usize = (*h_count).min(allowed_count);
                *h_count -= deduction;
                allowed_count -= deduction;
            }
        }

        histogram
    }
}

impl From<u8> for Threshold {
    fn from(level: u8) -> Self {
        Self::no_bigger_than(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H1: Histogram = {
        let mut h = [0; 256];
        h[0] = 1000; // never matters
        h[1] = 30;
        h[10] = 5;
        h[50] = 1;
        h[100] = 1;
        Histogram(h)
    };

    #[test]
    fn simple_threshold() {
        assert_eq!(
            (
                Threshold::no_bigger_than(99).allows(H1),
                Threshold::no_bigger_than(100).allows(H1)
            ),
            (false, true)
        );
    }

    #[test]
    fn exact_fit() {
        assert!(Threshold::new([(1, 30), (10, 5), (50, 1), (100, 1)]).allows(H1));
    }

    #[test]
    fn almost_exact_fit() {
        // fails because not allowing two in the 50-100 range
        assert!(!Threshold::new([(1, 30), (10, 5), (100, 1)]).allows(H1));
    }

    #[test]
    fn higher_value_threshold_can_apply_to_lower_error() {
        assert!(
            Threshold::new([
                (1, 10),    // fewer 1s than H1 contains
                (100, 100)  // should allow everything
            ])
            .allows(H1)
        );
    }

    #[test]
    fn total_count() {
        assert_eq!(
            (
                Threshold::new([(100, 36)]).allows(H1),
                Threshold::new([(100, 37)]).allows(H1)
            ),
            (false, true)
        );
    }

    #[test]
    fn max_threshold_allows_max_diff() {
        assert!(Threshold::new([(255, 10)]).allows({
            let mut h = [0; 256];
            h[255] = 10;
            Histogram(h)
        }));
        assert!(!Threshold::new([(255, 10)]).allows({
            let mut h = [0; 256];
            h[255] = 11;
            Histogram(h)
        }));
    }
}
