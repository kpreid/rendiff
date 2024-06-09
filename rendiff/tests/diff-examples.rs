//! Test that the algorithm produces the expected output for the files in `../example-comparisons/`.

// TODO: Add another test case for the behavior on images with transparency.

use std::path::Path;

#[test]
fn diff_example_robot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("example-comparisons");
    eprintln!("Dir: {root:?}");
    let input_actual = load_and_convert(&dbg!(root.join("robot-actual.png"))).unwrap();
    let input_expected = load_and_convert(&root.join("robot-exp.png")).unwrap();
    let expected_diff = load_and_convert(&root.join("robot-diff.png")).unwrap();

    let difference = rendiff::diff(input_actual.as_ref(), input_expected.as_ref());

    assert_eq!(
        difference.histogram(),
        rendiff::Histogram({
            let mut h = [0; 256];
            h[85] = 20;
            h[169] = 122;
            h[0] = ((input_actual.width() - 2) * (input_actual.height() - 2)) as usize
                - h.into_iter().sum::<usize>();
            h
        })
    );
    assert_eq!(
        expected_diff.as_ref(),
        difference.diff_image().expect("should have diff image")
    );
}

fn load_and_convert(path: &Path) -> Result<imgref::ImgVec<[u8; 4]>, image::ImageError> {
    Ok(interop::from_rgba(image::open(path)?.to_rgba8()))
}
