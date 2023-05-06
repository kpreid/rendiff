use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use image::RgbaImage;
use rendiff::Threshold;

#[derive(Debug, Parser)]
struct Args {
    /// One of the image files to compare.
    actual: PathBuf,
    /// The other image file to compare.
    expected: PathBuf,

    /// Path to which to write an image visually depicting differences found.
    ///
    /// Output format is decided by the file extension; it can be any of the formats supported
    /// by <https://crates.io/crates/image> provided that this instance of `rendiff` was compiled
    /// with that support.
    #[arg(long = "diff-output", short = 'o', value_name = "PATH")]
    diff: Option<PathBuf>,
}

fn main() -> anyhow::Result<ExitCode> {
    let Args {
        actual,
        expected,
        diff: diff_path,
    } = Args::parse();

    let actual = open_with_context("actual image", &actual)?;
    let expected = open_with_context("expected image", &expected)?;

    let difference = rendiff::diff(&actual, &expected);

    if let (Some(diff_image), Some(diff_path)) = (difference.diff_image, diff_path) {
        diff_image
            .save(&diff_path)
            .with_context(|| format!("failed to write '{}'", diff_path.display()))?;
    }

    // TODO: args for threshold
    let equal = Threshold::no_bigger_than(0).allows(difference.histogram);

    // TODO: print the histogram

    Ok(if equal {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn open_with_context(description: &str, path: &Path) -> anyhow::Result<RgbaImage> {
    let image = image::open(path)
        .with_context(|| format!("failed to open {} '{}'", description, path.display()))?;
    match image {
        image::DynamicImage::ImageRgba8(rgba_image) => Ok(rgba_image),

        image::DynamicImage::ImageLuma8(_)
        | image::DynamicImage::ImageLumaA8(_)
        | image::DynamicImage::ImageRgb8(_) => {
            // Non-identity but non-lossy conversion
            Ok(image.to_rgba8())
        }

        other => {
            eprintln!("warning: converting image to RGBA8, which is lossy");
            Ok(other.to_rgba8())
        }
    }
}
