//! Image comparison (diffing) for computer graphics renderer test cases.
//!
//! The algorithm implemented in this library is intended to allow comparing images
//! of the same scene which were rendered using different algorithms, or different
//! hardware, causing small “rounding errors” in either color or spatial position.
//!
//! To use it, call [`diff()`] on your images, and test the result against a [`Threshold`].
//!
//! ## When to use this library
//!
//! `rendiff` is *not* a “perceptual” difference algorithm; it does not attempt to calculate
//! how visible a difference is to the eye.
//! Rather, it is intended to allow for various kinds of numerical error or arbitrary
//! choice in rendering, which happen to also be perceptually insignificant:
//!
//! * Spatial: A line or curve being considered to fall on one side or another of a pixel center.
//! * Color: A color value being rounded up or down.
//!
//! This algorithm will always ignore spatial errors which meet the following criteria:
//!
//! * the spatial offset is at most 1 pixel,
//! * there are no 1-pixel-sized shapes that vanish entirely
//!   (e.g. imagine a very narrow, pointy, non-axis-aligned triangle;
//!   its sharp end turns into a series of disconnected dots), and
//! * there isn't antialiasing which introduces miscellaneous intermediate shades.
//!
//! Therefore, `rendiff` is ideal for comparing non-antialiased renderings of “vector” graphics.
//! In other situations, and for color rounding differences, you must tune the
//! [`Threshold`] for allowed differences.
//!
//! `rendiff` is unsuitable for comparing images which have strong noise (e.g. halftone)
//! or spatial displacements of more than one pixel.
//!
//! ## Example output
//!
//! These two cartoon robot head images are the inputs.
//! The third image is the visual output of the diff function;
//! red comes from the input and cyan marks differences.
//!
//! <div style="background: gray; text-align: center; padding: 0.7em;">
//!
//! ![robot-exp]
//! ![robot-actual]
//! ![robot-diff]
//!
//! </div>
//!
//! Note that the eyes’ shapes differ slightly, and this is ignored, but the gaps at the bottom
//! and between the teeth are highlighted, because they are places where colors are present in
//! one image but entirely absent in the other.
//! These are examples of the type of rendering bug which `rendiff` is designed to catch.
//!
//! ## Example usage
//!
//! ```
//! use rendiff::Threshold;
//!
//! // In a real application, you would load the expected image from disk and
//! // the actual image from the output of your renderer or other image processor.
//! // For this example, we'll embed some very simple images as text.
//!
//! fn ascii_image(s: &str) -> Vec<[u8; 4]> {
//!     s.chars().map(|ch| {
//!         let gray = u8::try_from(ch.to_digit(10).unwrap()).unwrap();
//!         [gray, gray, gray, 255]
//!     }).collect()
//! }
//!
//! let expected_image = imgref::ImgVec::new(
//!     ascii_image("\
//!         00000000\
//!         00000000\
//!         00229900\
//!         00229900\
//!         00229900\
//!         00000000\
//!         00000000\
//!     "),
//!     8, 6
//! );
//! let actual_image = imgref::ImgVec::new(
//!     ascii_image("\
//!         00000000\
//!         00000000\
//!         00449990\
//!         00449990\
//!         00449990\
//!         00000000\
//!         00000000\
//!     "),
//!     8, 6
//! );
//!
//! let difference = rendiff::diff(actual_image.as_ref(), expected_image.as_ref());
//!
//! // `difference` describes the differences found but does not define success or failure.
//! // To do that, you must use a `Threshold`, or examine the `histogram()` yourself.
//!
//! assert!(Threshold::no_bigger_than(2).allows(difference.histogram()));
//! assert!(!Threshold::no_bigger_than(1).allows(difference.histogram()));
//!
//! let diff_image = difference.diff_image();
//! // You can put `diff_image` in your test report.
//! ```
//!
//! ## Principle of operation
//!
//! Suppose we are comparing two images, A and B.
//! For each pixel in A (except for the perimeter),
//! a neighborhood around the corresponding pixel in B is compared, and the _smallest_
//! color difference is taken to be the difference value for that pixel in A.
//! Then, the same process is repeated, swapping the roles of the two images, and the
//! final difference value for each pixel is the maximum of those two.
//!
//! The pixel differences are then compiled into a difference image (for user viewing)
//! and a histogram (for pass/fail conditions).
//!
//! The effect of this strategy is that any feature in the image, such as the edge of a
//! shape, can be displaced by up to the neighborhood size (currently fixed to 1 pixel
//! radius, i.e. a 3×3 neighborhood) in any direction, thus
//! tolerating different choices of rounding into the pixel grid, as long as the color is
//! the same.
//!
//! This algorithm does not inherently accept differences in antialiased images, because
//! depending on how an edge lands with respect to the pixel grid, the color may be
//! different. A future version of this library may solve that problem by accepting any
//! color which is a blend of colors found in the neighborhood.
//!
#![doc = ::embed_doc_image::embed_image!("robot-actual", "example-comparisons/robot-actual.png")]
#![doc = ::embed_doc_image::embed_image!("robot-diff", "example-comparisons/robot-diff.png")]
#![doc = ::embed_doc_image::embed_image!("robot-exp", "example-comparisons/robot-exp.png")]
//!

// This list is sorted.
#![forbid(rust_2018_idioms)]
#![forbid(unsafe_code)]
#![warn(clippy::exhaustive_enums)]
#![warn(clippy::exhaustive_structs)]
#![warn(clippy::modulo_arithmetic)]
#![warn(clippy::pedantic)]
#![warn(clippy::unnecessary_self_imports)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(noop_method_call)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unused_lifetimes)]

/// `rendiff` uses image types from `imgref`.
pub use ::imgref;

type RgbaPixel = [u8; 4];

mod image;

mod diff;
pub use diff::*;

mod histogram;
pub use histogram::*;

mod threshold;
pub use threshold::*;

mod visualize;
