use kayak::warehouse;

/// The UNKNOWN package has the lease metadata possible
#[test]
fn fetch_unknown() {
    assert!(kayak::warehouse::fetch_package("unknown".to_string()).is_ok());
}
