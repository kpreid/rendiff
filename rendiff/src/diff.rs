use image::Rgba;
use image::{buffer::ConvertBuffer, GenericImageView, GrayImage, Pixel, RgbaImage};

use crate::Histogram;

/// Output of [`diff()`], a comparison between two images.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Difference {
    /// A histogram of magnitudes of the detected differences.
    pub histogram: Histogram,
    /// An image intended for human viewing of which pixels are different,
    /// or [`None`] if the images had different sizes.
    ///
    /// The precise content of this image is not specified. It will be 1:1 scale with the
    /// images being compared, but it may be larger or smaller due to treatment of the edges.
    pub diff_image: Option<RgbaImage>,
}

/// Compares two images with a neighborhood-sensitive comparison which counts one pixel worth of
/// displacement as not a difference.
///
/// See the [crate documentation](crate) for more details on the algorithm used.
///
/// This function does not have any options for ignoring small color differences; rather, the
/// result can be checked against a [`Threshold`](crate::Threshold).
///
/// Details:
///
/// * If the images have different sizes, then the result will always be the maximum difference.
/// * Differences in the alpha channel are counted the same as differences in luma; the maximum
///   of luma and alpha is used as the result.
#[must_use]
pub fn diff(actual: &RgbaImage, expected: &RgbaImage) -> Difference {
    if expected.dimensions() != actual.dimensions() {
        return Difference {
            // Count it as every pixel different.
            histogram: {
                let mut h = [0; 256];
                h[usize::from(u8::MAX)] = expected.len().max(actual.len())
                    / usize::from(expected.sample_layout().channels);
                Histogram(h)
            },
            diff_image: None,
        };
    }

    let hd1 = half_diff(expected, actual);
    let hd2 = half_diff(actual, expected);

    // Combine the two half_diff results: _both_ must be small for the output to be small.
    let combined_diff: GrayImage = GrayImage::from_fn(hd1.width(), hd1.height(), |x, y| {
        hd1.get_pixel(x, y)
            .map2(hd2.get_pixel(x, y), std::cmp::Ord::max)
    });

    // Compute a histogram of difference sizes.
    let mut histogram: [usize; 256] = [0; 256];
    for diff_value in combined_diff.pixels() {
        let diff_value: u8 = diff_value[0];
        histogram[diff_value as usize] += 1;
    }
    //eprintln!("{:?}", histogram);

    Difference {
        histogram: Histogram(histogram),
        diff_image: Some(combined_diff.convert()),
    }
}

/// Compare each pixel of `have` against a neighborhood of `want` (ignoring the edge).
/// Each pixel's color must be approximately equal to some pixel in the neighborhood.
///
/// This is "half" of the complete diffing process because the neighborhood comparison
/// could allow a 1-pixel line in `want` to completely vanish. By performing the same
/// comparison in both directions, we ensure that each color in each image must also
/// appear in the other image.
fn half_diff(have: &RgbaImage, want: &RgbaImage) -> GrayImage {
    let (width, height) = have.dimensions();
    let have_elems = have.view(1, 1, width - 2, height - 2);

    let mut buffer: Vec<u8> = Vec::new();
    for (x, y, hpixel) in have_elems.pixels() {
        // The x and y we get from the iterator start at (0, 0) ignoring our offset,
        // so when we use those same x,y as top-left corner of the neighborhood,
        // we get a centered neighborhood.
        let neighborhood = want.view(x, y, 3, 3);
        let minimum_diff_in_neighborhood: u8 = neighborhood
            .pixels()
            .map(|(_, _, wpixel)| pixel_diff(hpixel, wpixel))
            .min()
            .expect("neighborhood is never empty");
        buffer.push(minimum_diff_in_neighborhood);
    }

    GrayImage::from_raw(have_elems.width(), have_elems.height(), buffer).unwrap()
}

/// Compare two pixel values and produce a difference magnitude.
///
/// TODO: This function should be replaceable by the caller of `diff()` instead.
fn pixel_diff(a: Rgba<u8>, b: Rgba<u8>) -> u8 {
    // Diff each channel independently, then convert the difference to luma.
    // Note: this is not theoretically correct in that sRGB nonlinearity
    // means we're under-counting the brightness difference, but `image`
    // is also doing it with linear arithmetic anyway:
    // <https://docs.rs/image/0.24.2/src/image/color.rs.html#473>
    let channel_diffs = a.map2(&b, u8::abs_diff);
    let color_diff = channel_diffs.to_luma()[0];
    let alpha_diff = channel_diffs[3];
    color_diff.max(alpha_diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Threshold;
    use image::{buffer::ConvertBuffer, ImageBuffer, Pixel, Rgba};

    /// Run diff() against two images defined as vectors,
    /// with an added border.
    fn diff_vecs<P: Pixel>(
        (width, height): (u32, u32),
        expected_data: Vec<P::Subpixel>,
        actual_data: Vec<P::Subpixel>,
        border_value: P,
    ) -> Difference
    where
        ImageBuffer<P, Vec<P::Subpixel>>: ConvertBuffer<RgbaImage>,
    {
        let expected = add_border(
            border_value,
            ImageBuffer::from_raw(width, height, expected_data)
                .expect("wrong expected_data length"),
        );
        let actual = add_border(
            border_value,
            ImageBuffer::from_raw(width, height, actual_data).expect("wrong actual_data length"),
        );
        diff(&expected.convert(), &actual.convert())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn add_border<P: Pixel>(
        border_value: P,
        image: ImageBuffer<P, Vec<P::Subpixel>>,
    ) -> ImageBuffer<P, Vec<P::Subpixel>> {
        let (width, height) = image.dimensions();
        ImageBuffer::from_fn(width + 2, height + 2, |x, y| {
            if (1..=width).contains(&x) && (1..=height).contains(&y) {
                *image.get_pixel(x - 1, y - 1)
            } else {
                border_value
            }
        })
    }

    #[test]
    fn simple_equality() {
        let image: RgbaImage =
            GrayImage::from_raw(3, 3, vec![0, 255, 128, 0, 255, 12, 255, 13, 99])
                .unwrap()
                .convert();
        let diff_result = dbg!(diff(&image, &image));

        let mut expected_histogram = [0; 256];
        expected_histogram[0] = 1;
        assert_eq!(diff_result.histogram, Histogram(expected_histogram));

        assert!(Threshold::no_bigger_than(0).allows(diff_result.histogram));
        assert!(Threshold::no_bigger_than(5).allows(diff_result.histogram));
        assert!(Threshold::no_bigger_than(254).allows(diff_result.histogram));
    }

    #[test]
    fn simple_inequality() {
        let delta = 55u8;
        let dred = 11; // delta scaled down by being on red channel only

        let expected_diff_image: RgbaImage =
            RgbaImage::from_raw(1, 1, vec![dred, dred, dred, 255]).unwrap();

        let mut expected_histogram = [0; 256];
        expected_histogram[usize::from(dred)] = 1;

        // Try both orders; result should be symmetric
        let diff_result = dbg!(diff_vecs(
            (1, 1),
            vec![0, 0, 0, 255],
            vec![delta, 0, 0, 255],
            Rgba([0, 0, 0, 255]),
        ));
        assert_eq!(
            diff_result,
            diff_vecs(
                (1, 1),
                vec![delta, 0, 0, 255],
                vec![0, 0, 0, 255],
                Rgba([0, 0, 0, 255])
            )
        );

        assert_eq!(
            diff_result,
            Difference {
                histogram: Histogram(expected_histogram),
                diff_image: Some(expected_diff_image)
            }
        );

        assert_eq!(
            (
                Threshold::no_bigger_than(dred - 1).allows(diff_result.histogram),
                Threshold::no_bigger_than(dred).allows(diff_result.histogram),
            ),
            (false, true)
        );
    }

    #[test]
    fn mismatched_sizes() {
        let expected = ImageBuffer::from_raw(1, 1, vec![0, 0, 0, 255u8]).unwrap();
        let actual = ImageBuffer::from_raw(1, 2, vec![0, 0, 0, 255, 0, 0, 0, 255u8]).unwrap();
        assert_eq!(
            diff(&expected, &actual),
            Difference {
                histogram: {
                    let mut h = [0; 256];
                    h[255] = 2;
                    Histogram(h)
                },
                diff_image: None
            }
        );
    }
}
