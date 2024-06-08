//! Test that the algorithm produces the expected output for the files in `../example-comparisons/`.

// TODO: Add another test case for the behavior on images with transparency.

use std::path::Path;

#[test]
fn diff_example_robot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("example-comparisons");
    eprintln!("Dir: {root:?}");
    let input_actual = image::open(dbg!(root.join("robot-actual.png"))).unwrap();
    let input_expected = image::open(root.join("robot-exp.png")).unwrap();
    let expected_diff = image::open(root.join("robot-diff.png")).unwrap();

    let difference = rendiff::diff(&input_actual.to_rgba8(), &input_expected.to_rgba8());

    assert_eq!(
        difference.histogram,
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
        expected_diff.to_rgba8(),
        difference.diff_image.expect("should have diff image")
    );
}
