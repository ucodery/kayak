use std::fs;
use std::path::PathBuf;

use httpmock::prelude::*;
use rstest::rstest;

use kayak::warehouse;

#[rstest]
#[case("unknown")] // The UNKNOWN package has the lease metadata possible
#[case("0")] // Yes, PyPI has a package "0"
#[case("beautifulsoup4")] // The beautifulsoup4 project only provides the bs4 package
#[case("Pillow")] // The pillow project has many compiled extensions
fn fetch_unknown(#[case] pkg_name: &str) {
    let label = format!("local-warehouse-{}", pkg_name);
    // start test server
    let resync = option_env!("RESYNC_TEST_REQUESTS").is_some();
    let store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("requests_store");
    let server = MockServer::start();

    let recording = if resync {
        server.forward_to(warehouse::PYPI_URI, |rule| {
            rule.filter(|when| {
                when.any_request();
            });
        });
        Some(server.record(|rule| {
            rule.filter(|when| {
                when.any_request();
            });
        }))
    } else {
        let historic = fs::read_dir(&store_path).map(|dir|
            dir.into_iter().filter_map(|entry|
                if let Ok(Ok(f)) = entry.map(|file| file.file_name().into_string()) {
                    f.starts_with(&label).then(|| f)
                } else {
                    None
                }
            ).max()
        ).expect("error loading saved requests file").expect("error could not find a saved requests file");
        server.playback(store_path.join(historic));
        None
    };

    let url = server.base_url();

    // run test
    assert!(kayak::warehouse::Package::fetch(&url, pkg_name).is_ok());

    // close server
    if let Some(save) = recording {
        save.save_to(store_path, label).expect("error saving requests for offline testing");
    };
}
