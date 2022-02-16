#![no_main]
use libfuzzer_sys::fuzz_target;
use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        let problem = s.contains("Lorem ipsum dolor sit amet") && s.contains("Hello world!");
        assert!(!problem);
    }
});
