//! A Serde deserializer for PEP-691 complaint package indexes, with a focus on how pypi.org
//! specifically encodes metadata
// Look at warehouse's _json_data for the practical implementation
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::str::FromStr;

use pep440::Version;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};
use trove_classifiers::Classifier;
use ureq;
use url::Url;

use super::distribution;

pub const MAJOR_API_VERSION: u8 = 1;
pub const MINOR_API_VERSION: u8 = 0;
pub const PYPI_URI: &str = "https://pypi.org";

#[derive(Debug)]
pub enum Error {
    NotFound,
    InvalidName,
    InvalidVersion,
}

impl From<ureq::Error> for Error {
    fn from(_err: ureq::Error) -> Error {
        Error::NotFound
    }
}

impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Error {
        Error::NotFound
    }
}

impl From<url::ParseError> for Error {
    fn from(_err: url::ParseError) -> Error {
        Error::NotFound
    }
}

impl From<distribution::Error> for Error {
    fn from(_err: distribution::Error) -> Error {
        Error::InvalidName
    }
}

/// The response from a Package Index root URL
#[derive(Debug)]
struct IndexRoot {
    api_version: String,
    projects: Vec<String>,
}

impl IndexRoot {
    fn fetch(index: &str) -> Result<Self, Error> {
        let index = Url::parse(index)?;
        if index.cannot_be_a_base() {
            return Err(Error::NotFound);
        }
        let response: IndexRoot = ureq::get(index.as_str()).call()?.into_json()?;
        Ok(response)
    }
}

pub fn fetch_index_version(index: &str) -> Result<String, Error> {
    let metadata = IndexRoot::fetch(index)?;
    Ok(metadata.api_version)
}

pub enum SupportLevel {
    /// This index is fully supported by this client
    Supported,
    /// This index may provide additional metadata that will be dropped by this client
    SomewhatSupported,
    /// This index is not compatible with this client
    Unsupported,
}

pub fn index_is_supported(index: &str) -> Result<SupportLevel, Error> {
    let api_version = fetch_index_version(index)?;
    let mut version_parts = api_version.split('.');
    let major = if let Ok(v) = version_parts
        .next()
        .ok_or(Error::InvalidVersion)?
        .parse::<u8>()
    {
        v
    } else {
        return Err(Error::InvalidVersion);
    };
    let minor = if let Ok(v) = version_parts
        .next()
        .ok_or(Error::InvalidVersion)?
        .parse::<u8>()
    {
        v
    } else {
        return Err(Error::InvalidVersion);
    };
    if version_parts.next().is_some() {
        return Err(Error::InvalidVersion);
    }
    if major > MAJOR_API_VERSION {
        Ok(SupportLevel::Unsupported)
    } else if minor > MINOR_API_VERSION {
        Ok(SupportLevel::SomewhatSupported)
    } else {
        Ok(SupportLevel::Supported)
    }
}

/// Retrieve the names of all projects hosted on this index
/// Names may or may not be normalized
pub fn fetch_projects(index: &str) -> Result<HashSet<String>, Error> {
    let metadata = IndexRoot::fetch(index)?;
    Ok(metadata.projects.into_iter().collect())
}

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

/// A Python package as returned by the JSON api
/// /pypi/{project}/json
#[derive(Debug)]
pub struct Package {
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub classifiers: Vec<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub docs_url: Option<String>,
    pub download_url: Option<String>,
    pub home_page: Option<String>,
    pub keywords: Option<String>,
    pub license: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub name: String,
    pub package_url: String,
    pub platform: Option<String>,
    pub project_url: String,
    pub project_urls: HashMap<String, String>,
    pub requires_dist: Vec<String>,
    pub requires_python: Option<String>,
    pub summary: Option<String>,
    pub versions: Vec<String>,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
}

impl Package {
    /// Retrieve package metadata from the package index
    pub fn fetch(index: &str, package: &str) -> Result<Self, Error> {
        let mut index = Url::parse(index)?;
        if index.cannot_be_a_base() {
            return Err(Error::NotFound);
        }
        let package = distribution::normalize_package_name(package)?;
        index.set_path(&format!("pypi/{package}/json"));
        let response: Package = ureq::get(index.as_str()).call()?.into_json()?;
        Ok(response)
    }

    /// Return validated versions of Package in comparison order
    ///
    /// Note that the order is not necessarily the same order as creation time
    /// and is also probably not in lexical order.
    pub fn ordered_versions(&self) -> Vec<Version> {
        let ordered_versions = self
            .versions
            .iter()
            .filter_map(|v| Version::parse(v))
            .collect::<BinaryHeap<Version>>();
        ordered_versions.into_sorted_vec()
    }

    /// Return validated classifiers of Package
    ///
    /// This function may return less items than the classifiers field but
    /// note that pypi.org will reject uploads with classifiers that don't parse
    /// so the only way classifiers will get dropped here is if the version of
    /// trove-classifiers is out-of-date with pypi.org, or Package was pulled
    /// from a different package index that does not validate classifiers or uses
    /// some other set of classifiers.
    pub fn classifiers(&self) -> Vec<Classifier> {
        self.classifiers
            .iter()
            .filter_map(|c| Classifier::from_str(c).ok())
            .collect()
    }
}

impl<'de> Deserialize<'de> for Package {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Top {
            info: Info,
            //last_serial: usize,
            releases: HashMap<String, IgnoredAny>,
            //urls: Vec<DistributionUrl>,
            //vulnerabilities: Vec<IgnoredAny>,
        }

        #[derive(Deserialize)]
        struct Info {
            author: Option<String>,
            author_email: Option<String>,
            // deprecated
            //bugtrack_url: Option<String>,
            classifiers: Vec<String>,
            description: Option<String>,
            description_content_type: Option<String>,
            docs_url: Option<String>,
            download_url: Option<String>,
            // deprecated
            //downloads: HashMap<String, isize>,
            home_page: Option<String>,
            keywords: Option<String>,
            license: Option<String>,
            maintainer: Option<String>,
            maintainer_email: Option<String>,
            name: String,
            package_url: String,
            platform: Option<String>,
            project_url: String,
            project_urls: Option<HashMap<String, String>>,
            //release_url: Option<String>,
            requires_dist: Option<Vec<String>>,
            requires_python: Option<String>,
            summary: Option<String>,
            //version: String,
            yanked: bool,
            yanked_reason: Option<String>,
        }

        let top = Top::deserialize(deserializer)?;
        Ok(Package {
            author: top.info.author,
            author_email: top.info.author_email,
            classifiers: top.info.classifiers,
            description: top.info.description,
            description_content_type: top.info.description_content_type,
            docs_url: top.info.docs_url,
            download_url: top.info.download_url,
            home_page: top.info.home_page,
            keywords: top.info.keywords,
            license: top.info.license,
            maintainer: top.info.maintainer,
            maintainer_email: top.info.maintainer_email,
            name: top.info.name,
            package_url: top.info.package_url,
            platform: top.info.platform,
            project_url: top.info.project_url,
            project_urls: top.info.project_urls.unwrap_or_default(),
            requires_dist: top.info.requires_dist.unwrap_or_default(),
            requires_python: top.info.requires_python,
            summary: top.info.summary,
            // versions is yet-to-be implemented directly in the json API
            // https://github.com/pypi/warehouse/pull/12079
            versions: top
                .releases
                .keys()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
            yanked: top.info.yanked,
            yanked_reason: top.info.yanked_reason,
        })
    }
}

/// A Python package version as returned by the JSON api
/// /pypi/{project}/{version}/json
#[derive(Debug)]
pub struct PackageVersion {
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub classifiers: Vec<String>,
    pub description: Option<String>,
    pub description_content_type: Option<String>,
    pub docs_url: Option<String>,
    pub download_url: Option<String>,
    pub home_page: Option<String>,
    pub keywords: Option<String>,
    pub license: Option<String>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub name: String,
    pub package_url: String,
    pub platform: Option<String>,
    pub project_url: String,
    pub project_urls: HashMap<String, String>,
    pub release_url: Option<String>,
    pub requires_dist: Vec<String>,
    pub requires_python: Option<String>,
    pub summary: Option<String>,
    pub urls: Vec<DistributionUrl>,
    pub version: String,
    pub vulnerabilities: Vec<PackageVulnerability>,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
}

impl PackageVersion {
    /// Retrieve package version metadata from the package index
    pub fn fetch(index: &str, package: &str, version: &str) -> Result<Self, Error> {
        let mut index = Url::parse(index)?;
        if index.cannot_be_a_base() {
            return Err(Error::NotFound);
        }
        let package = distribution::normalize_package_name(package)?;
        let version = Version::parse(version)
            .ok_or(Error::InvalidVersion)?
            .normalize();
        index.set_path(&format!("pypi/{package}/{version}/json"));
        let response: PackageVersion = ureq::get(index.as_str()).call()?.into_json()?;
        Ok(response)
    }

    /// Return the validated classifiers set on Package
    ///
    /// This function may return less results than the classifiers field but
    /// note that pypi.org will reject uploads with classifiers that don't parse
    /// so the only way classifiers will get dropped here is if the version of
    /// trove-classifiers is out-of-date with pypi.org, or Package was pulled
    /// from a different package index that does not validate classifiers or uses
    /// some other set of classifiers.
    pub fn classifiers(&self) -> Vec<Classifier> {
        self.classifiers
            .iter()
            .filter_map(|c| Classifier::from_str(c).ok())
            .collect()
    }

    pub fn version(&self) -> Result<Version, Error> {
        Version::parse(&self.version).ok_or(Error::InvalidVersion)
    }
}

impl<'de> Deserialize<'de> for PackageVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Top {
            info: Info,
            //last_serial: usize,
            urls: Vec<DistributionUrl>,
            vulnerabilities: Vec<PackageVulnerability>,
        }

        #[derive(Deserialize)]
        struct Info {
            author: Option<String>,
            author_email: Option<String>,
            // deprecated
            //bugtrack_url: Option<String>,
            classifiers: Vec<String>,
            description: Option<String>,
            description_content_type: Option<String>,
            docs_url: Option<String>,
            download_url: Option<String>,
            //downloads: HashMap<String, isize>,
            home_page: Option<String>,
            keywords: Option<String>,
            license: Option<String>,
            maintainer: Option<String>,
            maintainer_email: Option<String>,
            name: String,
            package_url: String,
            platform: Option<String>,
            project_url: String,
            project_urls: Option<HashMap<String, String>>,
            release_url: Option<String>,
            requires_dist: Option<Vec<String>>,
            requires_python: Option<String>,
            summary: Option<String>,
            version: String,
            yanked: bool,
            yanked_reason: Option<String>,
        }

        let top = Top::deserialize(deserializer)?;
        Ok(PackageVersion {
            author: top.info.author,
            author_email: top.info.author_email,
            classifiers: top.info.classifiers,
            description: top.info.description,
            description_content_type: top.info.description_content_type,
            docs_url: top.info.docs_url,
            download_url: top.info.download_url,
            home_page: top.info.home_page,
            keywords: top.info.keywords,
            license: top.info.license,
            maintainer: top.info.maintainer,
            maintainer_email: top.info.maintainer_email,
            name: top.info.name,
            package_url: top.info.package_url,
            platform: top.info.platform,
            project_url: top.info.project_url,
            project_urls: top.info.project_urls.unwrap_or_default(),
            release_url: top.info.release_url,
            requires_dist: top.info.requires_dist.unwrap_or_default(),
            requires_python: top.info.requires_python,
            summary: top.info.summary,
            urls: top.urls,
            version: top.info.version,
            vulnerabilities: top.vulnerabilities,
            yanked: top.info.yanked,
            yanked_reason: top.info.yanked_reason,
        })
    }
}

#[derive(Debug)]
pub struct DistributionUrl {
    pub digests: DistributionDigest,
    pub filename: String,
    pub md5_digest: String,
    pub packagetype: String,
    pub python_version: String,
    pub requires_python: Option<String>,
    pub size: usize,
    pub upload_time: String,
    pub upload_time_iso_8601: String,
    pub url: String,
    pub yanked: bool,
    pub yanked_reason: Option<String>,
}

impl DistributionUrl {
    pub fn filename(&self) -> Result<distribution::WheelName, Error> {
        Ok(distribution::WheelName::from_filename(&self.filename)?)
    }
}

impl<'de> Deserialize<'de> for DistributionUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Url {
            //comment_text: Option<String>,
            digests: DistributionDigest,
            //downloads: HashMap<String, isize>,
            filename: String,
            //has_sig: bool,
            md5_digest: String,
            packagetype: String,
            python_version: String,
            requires_python: Option<String>,
            size: usize,
            upload_time: String,
            upload_time_iso_8601: String,
            url: String,
            yanked: bool,
            yanked_reason: Option<String>,
        }

        let url = Url::deserialize(deserializer)?;
        Ok(DistributionUrl {
            digests: url.digests,
            filename: url.filename,
            md5_digest: url.md5_digest,
            packagetype: url.packagetype,
            python_version: url.python_version,
            requires_python: url.requires_python,
            size: url.size,
            upload_time: url.upload_time,
            upload_time_iso_8601: url.upload_time_iso_8601,
            url: url.url,
            yanked: url.yanked,
            yanked_reason: url.yanked_reason,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct DistributionDigest {
    pub blake2b_256: String,
    pub md5: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub struct PackageVulnerability {
    pub id: String,
    pub source: String,
    pub link: String,
    pub aliases: Vec<String>,
    pub details: String,
    pub summary: Option<String>,
    pub fixed_in: Vec<String>,
    pub withdrawn: Option<String>,
}
