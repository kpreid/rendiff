use image::{GrayImage, Pixel, Rgba, RgbaImage};

use crate::Histogram;

/// Take the raw absolute-difference values and visualize them
/// (by making small values more visible).
pub(crate) fn visualize(
    reference: &RgbaImage,
    raw_diff_image: &GrayImage,
    histogram: &Histogram,
) -> RgbaImage {
    // Validate the assumption our `(x + 1, y + 1)` coordinate lookups are making.
    // This will fail if we change how the diff algorithm works and don't update this.
    debug_assert_eq!(
        (reference.width(), reference.height()),
        (raw_diff_image.width() + 2, raw_diff_image.height() + 2)
    );

    let max_difference = f64::from(histogram.max_difference());

    RgbaImage::from_fn(raw_diff_image.width(), raw_diff_image.height(), |x, y| {
        let image::Luma([reference_value]) = reference.get_pixel(x + 1, y + 1).to_luma();

        // Scale up the diff values to maximize contrast
        let &image::Luma([raw_diff_value]) = raw_diff_image.get_pixel(x, y);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let amplified_difference = (f64::from(raw_diff_value) / max_difference * 255.0) as u8;

        Rgba([
            // Make the reference image low-contrast (in the red channel and scaled down),
            // so that it doesn't distract from the diff pixels but just gives visual context
            // for the spatial position of the differences.
            reference_value / 3,
            amplified_difference,
            amplified_difference,
            255,
        ])
    })
}
