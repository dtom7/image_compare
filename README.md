# A simple image comparison tool for Rgb + alpha channel (32 bit-depth) PNG images

This tool is heavily inspired by the Java image comparison library -> https://github.com/romankh3/image-comparison


### Notes

Minimun Rust version to build this package is 1.63.0

This tool has been tested in Windows 10/11 64-bit platform only

Integration tests can be run only in parallel or in sequence

    cargo test --package image_compare --test integration_test -- --test-threads 1

To see println! & eprintln! messages, run with `nocapture`

    cargo test --package image_compare --test integration_test -- --test-threads 2 --nocapture



