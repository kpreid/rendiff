//! Image comparison (diffing) for computer graphics renderer test cases.
//!
//! The algorithm implemented in this library is intended to allow comparing images
//! of the same scene which were rendered using different algorithms, or different
//! hardware, causing small “rounding errors” in either color or spatial position.
//!
//! To use it, call [`diff()`] on your images, and test the result against a [`Threshold`].
// TODO: Add example code here.
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

mod diff;
pub use diff::*;

mod histogram;
pub use histogram::*;

mod threshold;
pub use threshold::*;
