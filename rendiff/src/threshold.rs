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
    #[must_use]
    pub fn allows(&self, histogram: Histogram) -> bool {
        // Skip the first entry and always accept any number of zero-value differences.
        let mut checked_up_to = 1;
        // Loop over the thresholds, always in ascending order.
        for (&level, &count) in &self.0 {
            // Add 1 because the level value *includes* differences of that level, i.e.
            // level 1 should include checking histogram[1].
            let new_checked_up_to = usize::from(level) + 1;
            debug_assert!(new_checked_up_to > checked_up_to);
            let new_differences = histogram.0[checked_up_to..new_checked_up_to]
                .iter()
                .sum::<usize>();
            if new_differences > count {
                // TODO: Instead of failing immediately, buffer this and allow a later-checked
                // higher-difference entry to also permit lower differences.
                return false;
            }
            checked_up_to = new_checked_up_to;
        }

        // Finally, reject differences greater than any accepted.
        let remaining_differences = histogram.0[checked_up_to..].iter().sum::<usize>();
        if remaining_differences > 0 {
            return false;
        }

        true
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
        h[0] = 1000;
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
