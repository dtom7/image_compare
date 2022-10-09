# A simple image comparison tool for Rgb + alpha channel (32 bit-depth) PNG images


### Notes

Integration tests can be run only in parallel or in sequence

    cargo test --package image_compare --test integration_test -- --test-threads 1

To see println! & eprintln! messages, run with `nocapture`

    cargo test --package image_compare --test integration_test -- --test-threads 2 --nocapture



