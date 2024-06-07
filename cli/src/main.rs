use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use image::RgbaImage;
use rendiff::{Difference, Threshold};

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

    let actual = interop::from_rgba(open_with_context("actual image", &actual)?);
    let expected = interop::from_rgba(open_with_context("expected image", &expected)?);

    let difference = rendiff::diff(actual.as_ref(), expected.as_ref());

    if let (Some(diff_image), Some(diff_path)) = (&difference.diff_image, &diff_path) {
        interop::into_rgba(diff_image.clone())
            .save(&diff_path)
            .with_context(|| format!("failed to write '{}'", diff_path.display()))?;
    }

    print_results(&difference);

    // TODO: args for threshold
    let equal = Threshold::no_bigger_than(0).allows(difference.histogram);

    Ok(if equal {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

#[mutants::skip] // TODO: cli tests
fn print_results(difference: &Difference) {
    let Difference {
        histogram,
        diff_image: _,
        ..
    } = difference;
    eprintln!("{:#?}", histogram);
}

#[mutants::skip] // TODO: cli tests
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
