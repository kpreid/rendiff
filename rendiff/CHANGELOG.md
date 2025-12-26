# Changelog

## 0.2.2 (2025-12-26)

### Added

* `Threshold::remove_allowed_differences_from()`, which can be used in place of `Threshold::allows()` to give more information about comparison failures than just the boolean.

### Changed

* A `Threshold`’s allowance of large magnitude differences now also applies to small magnitude differences which were not covered by entries for smaller magnitudes.

### Removed

* Rust versions prior to 1.90.0 are no longer supported.

## 0.2.1 (2024-06-10)

This release only affects documentation.

* Add more documentation and more example code.
* Correct obsolete remark about `image` dependency in the package README.

## 0.2.0 (2024-06-09)

This release consists of the following **breaking changes**:

* No longer depends on the `image` package; all image types have been replaced with the much stabler [`imgref`](https://docs.rs/imgref/1/).
* `Difference` and `Threshold` no longer have public fields.
    * `Difference` provides accessors instead.
    * `Threshold` is write-only.

## 0.1.1 (2024-06-08)

### Added

* Difference images have been enhanced.
    * The expected input image is now displayed (in dark red) to give context for where the differences are.
    * The differences (now in cyan) are scaled up to make use of available dynamic range — small differences will be much more practical to view.
* `Threshold` may now contain entries of magnitude 255, which was previously rejected incorrectly.
* Expanded documentation, including further explanation of what applications `rendiff` is appropriate for, and example images.

### Removed

* Rust versions prior to 1.75.0 are no longer supported.

### Internal improvements

* No longer depends on the `itertools` package.

## 0.1.0 (2023-05-05)

Initial release.
