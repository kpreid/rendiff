//! Image manipulation functions built on [`imgref`].

use crate::RgbaPixel;

pub(crate) fn from_fn<T>(
    width: usize,
    height: usize,
    mut f: impl FnMut(usize, usize) -> T,
) -> imgref::ImgVec<T> {
    imgref::ImgVec::new(
        (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| f(x, y))
            .collect(),
        width,
        height,
    )
}

pub(crate) fn rgba_to_luma(pixel: RgbaPixel) -> u8 {
    // Legacy compatibility: this is the formula `image`'s internal `rgb_to_luma()` uses.
    // However, this is ill-founded, because sRGB encoded values are non-linear, so the weighting
    // effectively varies depending on the level. It gives us luma, not luminance (see
    // <https://en.wikipedia.org/wiki/Luma_(video)> for an explanation) but for our purposes,
    // this is not necessarily what we actually want.
    //
    // What we in principle should be doing instead is decoding (linearizing),
    // performing the weighted sum, then encoding again.
    // But that should not just replace this function; rather we should consider what each use case
    // actually needs.

    let [r, g, b, _a] = pixel;
    let luma: u32 = (2126 * u32::from(r) + 7152 * u32::from(g) + 722 * u32::from(b)) / 10000;

    debug_assert!(u8::try_from(luma).is_ok());
    #[allow(clippy::cast_possible_truncation)]
    {
        luma as u8
    }
}

#[cfg(test)]
pub(crate) fn luma_to_rgba(luma: u8) -> RgbaPixel {
    [luma, luma, luma, 255]
}
