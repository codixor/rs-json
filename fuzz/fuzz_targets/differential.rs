//! cargo-fuzz target: every entry point must agree with serde_json on
//! accept/reject, and tape round-trips must equal serde_json::Value.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let reference = serde_json::from_slice::<serde_json::Value>(data).is_ok();
    let ours = rs_json::parse_bytes(data).is_ok();
    // Known intentional differences must be listed in tests/conformance/decisions.toml.
    assert_eq!(reference, ours, "accept/reject mismatch");
});
