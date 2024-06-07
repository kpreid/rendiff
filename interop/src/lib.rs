/// Converts RGBA [`image`] image to [`imgref`] image with identical pixel bytes.
pub fn from_rgba(image: image::RgbaImage) -> imgref::ImgVec<[u8; 4]> {
    // These conversions cannot fail because if they didn't fit in `usize`,
    // the `buf` couldn't exist in memory.
    let width = usize::try_from(image.width()).expect("width too large");
    let height = usize::try_from(image.height()).expect("height too large");

    let buf = image
        .into_vec()
        .chunks(4)
        .map(|pixel| <[u8; 4]>::try_from(pixel).unwrap())
        .collect();

    imgref::ImgVec::new(buf, width, height)
}

/// Converts RGBA [`imgref`] image to [`image`] image with identical pixel bytes.
pub fn into_rgba(image: imgref::ImgVec<[u8; 4]>) -> image::RgbaImage {
    let (buf, width, height) = image.into_contiguous_buf();

    // These conversions cannot fail because `imgref` has an (undocumented)
    // restriction that the dimensions must fit in `u32` as well as `usize`.
    let width = u32::try_from(width).expect("width too large");
    let height = u32::try_from(height).expect("height too large");

    let buf = buf.into_iter().flatten().collect();

    image::RgbaImage::from_vec(width, height, buf).unwrap()
}
