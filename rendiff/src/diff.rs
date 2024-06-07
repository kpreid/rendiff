use imgref::{ImgRef, ImgVec};

use crate::{Histogram, RgbaPixel};

/// Output of [`diff()`], a comparison between two images.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct Difference {
    // TODO: Make these fields private so we have more flexibility.
    /// A histogram of magnitudes of the detected differences.
    pub histogram: Histogram,

    /// An sRGB RGBA image intended for human viewing of which pixels are different,
    /// or [`None`] if the images had different sizes.
    ///
    /// The precise content of this image is not specified. It will be 1:1 scale with the
    /// images being compared, but it may be larger or smaller due to treatment of the edges.
    ///
    /// Currently, the red channel contains data from the input `expected` image,
    /// and the blue and green channels contain differences, scaled up for high visibility.
    pub diff_image: Option<imgref::ImgVec<RgbaPixel>>,
}

/// Compares two RGBA images with a neighborhood-sensitive comparison which counts one pixel worth
/// of displacement as not a difference.
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
pub fn diff(actual: ImgRef<'_, RgbaPixel>, expected: ImgRef<'_, RgbaPixel>) -> Difference {
    if dimensions(expected) != dimensions(actual) {
        return Difference {
            // Count it as every pixel different.
            histogram: {
                let mut h = [0; 256];
                h[usize::from(u8::MAX)] = expected.pixels().len().max(actual.pixels().len());
                Histogram(h)
            },
            diff_image: None,
        };
    }

    let hd1 = half_diff(expected, actual);
    let hd2 = half_diff(actual, expected);

    // Combine the two half_diff results: _both_ must be small for the output to be small.
    let raw_diff_image: ImgVec<u8> = ImgVec::new(
        (0..hd1.height())
            .flat_map(|y| {
                (0..hd1.width()).map({
                    let hd1 = &hd1;
                    let hd2 = &hd2;
                    move |x| core::cmp::max(hd1[(x, y)], hd2[(x, y)])
                })
            })
            .collect(),
        hd1.width(),
        hd1.height(),
    );

    // Compute a histogram of difference sizes.
    let mut histogram: [usize; 256] = [0; 256];
    for diff_value in raw_diff_image.pixels() {
        histogram[usize::from(diff_value)] += 1;
    }
    let histogram = Histogram(histogram);

    Difference {
        histogram,
        diff_image: Some(crate::visualize::visualize(
            expected,
            raw_diff_image.as_ref(),
            &histogram,
        )),
    }
}

fn dimensions<T>(image: imgref::ImgRef<'_, T>) -> [usize; 2] {
    [image.width(), image.height()]
}

/// Compare each pixel of `have` against a neighborhood of `want` (ignoring the edge).
/// Each pixel's color must be approximately equal to some pixel in the neighborhood.
///
/// This is "half" of the complete diffing process because the neighborhood comparison
/// could allow a 1-pixel line in `want` to completely vanish. By performing the same
/// comparison in both directions, we ensure that each color in each image must also
/// appear in the other image.
fn half_diff(have: ImgRef<'_, RgbaPixel>, want: ImgRef<'_, RgbaPixel>) -> ImgVec<u8> {
    let have_elems = have.sub_image(1, 1, have.width() - 2, have.height() - 2);

    let mut buffer: Vec<u8> = Vec::new();
    for (x, y, have_pixel) in have_elems
        .rows()
        .enumerate()
        .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, &pixel)| (x, y, pixel)))
    {
        // The x and y we get from the enumerate()s start at (0, 0) ignoring our offset,
        // so when we use those same x,y as top-left corner of the neighborhood,
        // we get a centered neighborhood.
        let neighborhood = want.sub_image(x, y, 3, 3);
        let minimum_diff_in_neighborhood: u8 = neighborhood
            .pixels()
            .map(|want_pixel| pixel_diff(have_pixel, want_pixel))
            .min()
            .expect("neighborhood is never empty");
        buffer.push(minimum_diff_in_neighborhood);
    }

    ImgVec::new(buffer, have_elems.width(), have_elems.height())
}

/// Compare two pixel values and produce a difference magnitude.
///
/// TODO: This function should be replaceable by the caller of `diff()` instead,
/// allowing the caller to choose a perceptual or encoded difference function,
/// and choose how they wish to treat alpha.
fn pixel_diff(a: RgbaPixel, b: RgbaPixel) -> u8 {
    // Diff each channel independently, then convert the difference to luma.
    // Note: this is a very naive comparison, but
    let r_diff = a[0].abs_diff(b[0]);
    let g_diff = a[1].abs_diff(b[1]);
    let b_diff = a[2].abs_diff(b[2]);
    let a_diff = a[3].abs_diff(b[3]);

    let color_diff = crate::image::rgba_to_luma([r_diff, g_diff, b_diff, 255]);

    color_diff.max(a_diff).min(255)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::luma_to_rgba;
    use crate::Threshold;
    use imgref::{Img, ImgExt as _};

    /// Run [`diff()`] against two images defined as vectors,
    /// with an added border.
    fn diff_vecs(
        (width, height): (usize, usize),
        actual_data: Vec<RgbaPixel>,
        expected_data: Vec<RgbaPixel>,
        border_value: RgbaPixel,
    ) -> Difference {
        let expected = add_border(border_value, ImgVec::new(expected_data, width, height));
        let actual = add_border(border_value, ImgVec::new(actual_data, width, height));
        diff(actual.as_ref(), expected.as_ref())
    }

    #[allow(clippy::needless_pass_by_value)]
    fn add_border<P: Copy>(border_value: P, image: ImgVec<P>) -> ImgVec<P> {
        let width = image.width();
        let height = image.height();
        crate::image::from_fn(width + 2, height + 2, |x, y| {
            if (1..=width).contains(&x) && (1..=height).contains(&y) {
                image[(x - 1, y - 1)]
            } else {
                border_value
            }
        })
    }

    #[test]
    fn simple_equality() {
        let image = Img::new(
            [0, 255, 128, 0, 255, 12, 255, 13, 99].map(luma_to_rgba),
            3,
            3,
        );
        let image = image.as_ref();
        let diff_result = dbg!(diff(image, image));

        let mut expected_histogram = [0; 256];
        expected_histogram[0] = 1;
        assert_eq!(diff_result.histogram, Histogram(expected_histogram));

        assert!(Threshold::no_bigger_than(0).allows(diff_result.histogram));
        assert!(Threshold::no_bigger_than(5).allows(diff_result.histogram));
        assert!(Threshold::no_bigger_than(254).allows(diff_result.histogram));
    }

    #[test]
    fn simple_inequality_thoroughly_examined() {
        let base_pixel_value = 100u8;
        let delta = 55u8;
        let dred = 11; // delta scaled down by being on red channel of the test image only
        let display_scale = 3; // input image is divided by this when put in diff image

        let mut expected_histogram = [0; 256];
        expected_histogram[usize::from(dred)] = 1;

        // Try both orders; result should be symmetric except for the diff image
        let result_of_negative_difference = dbg!(diff_vecs(
            (1, 1),
            vec![[base_pixel_value, base_pixel_value, base_pixel_value, 255]],
            vec![[
                base_pixel_value + delta,
                base_pixel_value,
                base_pixel_value,
                255
            ]],
            [base_pixel_value, base_pixel_value, base_pixel_value, 255],
        ));
        let result_of_positive_difference = dbg!(diff_vecs(
            (1, 1),
            vec![[
                base_pixel_value + delta,
                base_pixel_value,
                base_pixel_value,
                255
            ]],
            vec![[base_pixel_value, base_pixel_value, base_pixel_value, 255]],
            [base_pixel_value, base_pixel_value, base_pixel_value, 255],
        ));

        // Note that the diff image is constructed using the expected image, not actual.
        assert_eq!(
            result_of_positive_difference,
            Difference {
                histogram: Histogram(expected_histogram),
                diff_image: Some(ImgVec::new(
                    vec![[(base_pixel_value) / display_scale, 255, 255, 255]],
                    1,
                    1,
                ))
            }
        );
        assert_eq!(
            result_of_negative_difference,
            Difference {
                histogram: Histogram(expected_histogram),
                diff_image: Some(ImgVec::new(
                    vec![[(base_pixel_value + dred) / display_scale, 255, 255, 255]],
                    1,
                    1,
                ))
            }
        );

        assert_eq!(
            (
                Threshold::no_bigger_than(dred - 1).allows(result_of_positive_difference.histogram),
                Threshold::no_bigger_than(dred).allows(result_of_positive_difference.histogram),
            ),
            (false, true)
        );
    }

    /// Test that the diff image is 2 pixels smaller, as expected.
    ///
    /// TODO: We should have image-comparison tests applying to the diff image.
    /// Once we do, this test is moot.
    #[test]
    fn diff_image_size() {
        let image1 = crate::image::from_fn(10, 10, |_, _| [1, 2, 3, 255]);
        let image2 = crate::image::from_fn(10, 10, |_, _| [100, 200, 255, 255]);
        let diff_result = diff(image1.as_ref(), image2.as_ref());

        let diff_image = diff_result.diff_image.unwrap();
        assert_eq!((diff_image.width(), diff_image.height()), (8, 8));
    }

    #[test]
    fn mismatched_sizes() {
        let expected = ImgRef::new(&[[0, 0, 0, 255u8]], 1, 1);
        let actual = ImgRef::new(&[[0, 0, 0, 255], [0, 0, 0, 255u8]], 1, 2);
        assert_eq!(
            diff(actual, expected),
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
