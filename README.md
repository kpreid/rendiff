# `rendiff`

## Image comparison (diffing) for computer graphics renderer test cases

The algorithm implemented in this Rust library is intended to allow comparing images
of the same scene which were rendered using different algorithms, or different
hardware, causing small “rounding errors” in either color or spatial position that should be ignored as insignificant.

See [the library documentation](https://docs.rs/rendiff/latest/rendiff/) for details on the algorithm used.

Packages in this repository
---------------------------

* `rendiff/` is the library containing the algorithm.
* `cli/` (package name `rendiff-cli`) is a command-line tool to run the algorithm on pairs of image files.
* `interop/` is used only for shared code in this workspace.

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