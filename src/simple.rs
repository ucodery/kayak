//! A Serde deserializer for PEP-691 complaint Python package indexes
//! https://packaging.python.org/en/latest/specifications/simple-repository-api/
use std::collections::HashSet;
use std::fmt;

use anyhow::Result as aResult;
use serde::{Deserialize, Deserializer};
use ureq;
use url::Url;

use super::warehouse::Error;

pub const MAJOR_API_VERSION: u8 = 1;
pub const MINOR_API_VERSION: u8 = 4;

pub const PYPI_SIMPLE_URI: &str = "https://pypi.org/simple/";

// needs to be kept in sync with `MAJOR_API_VERSION`
const SIMPLE_JSON_MIME: &str = "application/vnd.pypi.simple.v1+json";

/// The response from a Package Index root URL
#[derive(Deserialize)]
struct IndexRoot {
    meta: IndexRootMeta,
    projects: Vec<SimpleProject>,
}

#[derive(Deserialize)]
struct IndexRootMeta {
    #[serde(rename = "_last-serial")]
    last_serial: usize,
    #[serde(rename = "api-version")]
    api_version: String,
}

#[derive(Deserialize)]
struct SimpleProject {
    #[serde(rename = "_last-serial")]
    last_serial: usize,
    name: String,
}

impl IndexRoot {
    fn fetch(index: &str) -> aResult<Self> {
        let index = Url::parse(index)?;
        if index.cannot_be_a_base() {
            return Err(Error::NotFound)?;
        }
        let response: IndexRoot = ureq::get(index.as_str()).header("Accept", SIMPLE_JSON_MIME).call()?.body_mut().read_json()?;
        Ok(response)
    }
}

pub fn fetch_index_version(index: &str) -> aResult<String> {
    let metadata = IndexRoot::fetch(index)?;
    Ok(metadata.meta.api_version)
}

pub enum SupportLevel {
    /// This index is fully supported by this client
    Supported,
    /// This index may provide additional metadata that will be dropped by this client
    SomewhatSupported,
    /// This index is not compatible with this client
    Unsupported,
}

pub fn index_is_supported(index: &str) -> aResult<SupportLevel> {
    let api_version = fetch_index_version(index)?;
    let mut version_parts = api_version.split('.');
    let major = if let Ok(v) = version_parts
        .next()
        .ok_or(Error::InvalidVersion)?
        .parse::<u8>()
    {
        v
    } else {
        return Err(Error::InvalidVersion)?;
    };
    let minor = if let Ok(v) = version_parts
        .next()
        .ok_or(Error::InvalidVersion)?
        .parse::<u8>()
    {
        v
    } else {
        return Err(Error::InvalidVersion)?;
    };
    if version_parts.next().is_some() {
        Err(Error::InvalidVersion)?;
    }
    if major != MAJOR_API_VERSION {
        Ok(SupportLevel::Unsupported)
    } else if minor > MINOR_API_VERSION {
        Ok(SupportLevel::SomewhatSupported)
    } else {
        Ok(SupportLevel::Supported)
    }
}

/// Retrieve the names of all projects hosted on this index
/// Names may or may not be normalized
pub fn fetch_projects(index: &str) -> aResult<HashSet<String>> {
    let metadata = IndexRoot::fetch(index)?;
    Ok(metadata.projects.into_iter().map(|p| p.name).collect())
}

/*
impl<'de> Deserialize<'de> for IndexRoot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Top {
            meta: Meta,
            projects: Vec<Project>,
        }

        #[derive(Deserialize)]
        struct Meta {
            api_version: String,
        }

        #[derive(Deserialize)]
        struct Project {
            name: String,
        }

        let top = Top::deserialize(deserializer)?;
        Ok(Self {
            api_version: top.meta.api_version,
            projects: top.projects.into_iter().map(|p| p.name).collect(),
        })
    }
}
*/
