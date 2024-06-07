use imgref::{ImgRef, ImgVec};

use crate::{Histogram, RgbaPixel};

/// Take the raw absolute-difference values and visualize them
/// (by making small values more visible).
pub(crate) fn visualize(
    reference: ImgRef<'_, RgbaPixel>,
    raw_diff_image: ImgRef<'_, u8>,
    histogram: &Histogram,
) -> ImgVec<RgbaPixel> {
    // Validate the assumption our `(x + 1, y + 1)` coordinate lookups are making.
    // This will fail if we change how the diff algorithm works and don't update this.
    debug_assert_eq!(
        (reference.width(), reference.height()),
        (raw_diff_image.width() + 2, raw_diff_image.height() + 2)
    );

    let max_difference = f64::from(histogram.max_difference());

    crate::image::from_fn(raw_diff_image.width(), raw_diff_image.height(), |x, y| {
        // TODO: this should be re-encoded luminance, not luma
        let reference_value = crate::image::rgba_to_luma(reference[(x + 1, y + 1)]);

        // Scale up the diff values to maximize contrast
        let raw_diff_value = raw_diff_image[(x, y)];
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let amplified_difference = (f64::from(raw_diff_value) / max_difference * 255.0) as u8;

        [
            // Make the reference image low-contrast (in the red channel and scaled down),
            // so that it doesn't distract from the diff pixels but just gives visual context
            // for the spatial position of the differences.
            reference_value / 3,
            amplified_difference,
            amplified_difference,
            255,
        ]
    })
}
