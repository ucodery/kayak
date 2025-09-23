use std::fs;
use std::path::PathBuf;

use httpmock::prelude::*;
use rstest::rstest;

use kayak::simple;

// XXX: The saved requests file for these tests are generated differently
// - httpmock doesn't seem to forward Accept headers to to a proxy server,
//   even when it does record them. Captured responses always have content-type: text/html
//   whatever header is used in the actual request.
// - because of this, the yaml is patched with correct content returned by curl
// - the response from the simple index root is also trimmed down, as reporting all
//   projects hosted on pypi.org currently takes up 32Mb

#[rstest]
fn fetch_index_version() {
    let label = "local-simple-root";
    // start test server
    let resync = option_env!("RESYNC_TEST_REQUESTS_FOR_SIMPLE").is_some();
    let store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("requests_store");
    let server = MockServer::start();

    let recording = if resync {
        server.forward_to(simple::PYPI_SIMPLE_URI, |rule| {
            rule.filter(|when| {
                when.any_request();
            });
        });
        Some(server.record(|rule| {
            rule.record_request_header("Accept")
                .filter(|when| {
                when.any_request();
            });
        }))
    } else {
        let historic = fs::read_dir(&store_path).map(|dir|
            dir.into_iter().filter_map(|entry|
                if let Ok(Ok(f)) = entry.map(|file| file.file_name().into_string()) {
                    f.starts_with(label).then(|| f)
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
    let index_version = kayak::simple::fetch_index_version(&url);
    assert!(index_version.is_ok());
    assert!(index_version.unwrap() == "1.4");

    // close server
    if let Some(save) = recording {
        save.save_to(store_path, label).expect("error saving requests for offline testing");
    };
}

#[rstest]
fn index_supported() {
    let label = "local-simple-root";
    // start test server
    let resync = option_env!("RESYNC_TEST_REQUESTS_FOR_SIMPLE").is_some();
    let store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("requests_store");
    let server = MockServer::start();

    let recording = if resync {
        server.forward_to(simple::PYPI_SIMPLE_URI, |rule| {
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
                    f.starts_with(label).then(|| f)
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
    let supported_index = kayak::simple::index_is_supported(&url);
    assert!(supported_index.is_ok());
    assert!(matches!(supported_index.unwrap(), kayak::simple::SupportLevel::Supported));

    // close server
    if let Some(save) = recording {
        save.save_to(store_path, label).expect("error saving requests for offline testing");
    };
}

#[rstest]
fn fetch_projects() {
    let label = "local-simple-root";
    // start test server
    let resync = option_env!("RESYNC_TEST_REQUESTS_FOR_SIMPLE").is_some();
    let store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("requests_store");
    let server = MockServer::start();

    let recording = if resync {
        server.forward_to(simple::PYPI_SIMPLE_URI, |rule| {
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
    let maybe_packages = kayak::simple::fetch_projects(&url);
    assert!(maybe_packages.is_ok());
    let packages = maybe_packages.unwrap();
    // pick some real package names to check
    assert!(packages.contains("setuptools"));
    assert!(packages.contains("numpy"));
    assert!(packages.contains("pytest"));
    assert!(packages.contains("000"));

    // close server
    if let Some(save) = recording {
        save.save_to(store_path, label).expect("error saving requests for offline testing");
    };
}
