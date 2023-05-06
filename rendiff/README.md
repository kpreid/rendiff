# `rendiff`

## Image comparison (diffing) for computer graphics renderer test cases

The algorithm implemented in this Rust library is intended to allow comparing images
of the same scene which were rendered using different algorithms, or different
hardware, causing small “rounding errors” in either color or spatial position that should be ignored as insignificant.

See the library documentation for details on the algorithm used.

`rendiff` provides _only_ a comparison algorithm which can use a pass/fail criterion, and a visual representation of the diff; it does not provide any test-framework features like loading expected images and overwriting them, or producing visual reports.

Stability
---------

`rendiff` is being used within one of my other projects (`all-is-cubes`) and reliably performs its function. However, there are several features that it lacks, which may result in API changes in future versions:

* Customizing the pixel comparison (color value distance) function.
* More effective comparison of antialiased images.
* Processing images of greater than 8 bits-per-component.
* Not requiring the input to use the image type from [the `image` library](https://docs.rs/image/latest/image/).

License
-------

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.