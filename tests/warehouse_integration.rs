use kayak::warehouse;

/// The UNKNOWN package has the lease metadata possible
#[test]
fn fetch_unknown() {
    assert!(kayak::warehouse::Package::fetch(kayak::warehouse::PYPI_URI, "unknown").is_ok());
}
