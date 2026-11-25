# JSON conformance

The reader suite pins [JSONTestSuite at 1ef36fa](https://github.com/nst/JSONTestSuite/tree/1ef36fa01286573e846ac449e8683f8833c5b26a/test_parsing).
The 95 required-valid and 188 required-invalid cases follow the upstream
expectations. [decisions.toml](decisions.toml) records the result for each of
the 35 implementation-defined cases.

GitHub CI checks out the fixtures and runs the conformance test explicitly.
To run it locally, set `RS_JSON_CONFORMANCE_DIR` to the pinned checkout's
`test_parsing` directory and run:

```sh
cargo test --locked --all-features --test conformance -- --ignored
```

The test compares the DOM result with slice and reader events, raw token
contents, and error positions at buffer sizes 1, 7, 64, and 8192. The regular
`read` integration tests also cover short reads, interrupts and I/O errors
without external fixtures.
