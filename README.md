# The Portable Network Graphics container format

This crate handles [PNG](https://en.wikipedia.org/wiki/PNG), [APNG](https://en.wikipedia.org/wiki/APNG), and [JNG](https://en.wikipedia.org/wiki/JPEG_Network_Graphics) files.
Maybe [MNG](https://en.wikipedia.org/wiki/Multiple-image_Network_Graphics) in the future.

Note that this crate only concerns itself with the ['container'](https://en.wikipedia.org/wiki/Container_format) aspect of a PNG/APNG/etc file.
It implements structs and enums for working with chunks and their data, but leaves the decoding of that data into images up to another crate.
