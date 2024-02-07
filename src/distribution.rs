use std::fmt;

use pep440::Version;
use regex::Regex;

/// Normalize the package name
/// https://packaging.python.org/en/latest/specifications/name-normalization/
/// An Error is returned if the name is not valid to begin with
pub fn normalize_package_name(name: &str) -> Result<String, Error> {
    let valid_name = Regex::new(r"^([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9._-]*[a-zA-Z0-9])$").unwrap();
    if !valid_name.is_match(name) {
        return Err(Error::InvalidPackageName);
    }

    let separators = Regex::new(r"[-_.]+").unwrap();
    let normalized = separators.replace_all(name, "-");
    Ok(normalized.to_ascii_lowercase())
}

#[derive(Debug)]
pub enum Error {
    InvalidWheelName,
    InvalidPackageName,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct BuildTag {
    pub number: usize,
    pub string: String,
}

impl BuildTag {
    fn parse(buildtag: &str) -> Option<Self> {
        let number = buildtag
            .chars()
            .take_while(|b| b.is_ascii_digit())
            .collect::<String>()
            .parse::<usize>()
            .ok()?;
        let string = buildtag
            .chars()
            .skip_while(|b| b.is_ascii_digit())
            .collect::<String>();
        Some(Self { number, string })
    }
}

impl fmt::Display for BuildTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.number, self.string)
    }
}

/// A PEP-425 Compatibility Tag
#[derive(PartialEq, Eq, Hash)]
pub struct CompatibilityTag {
    python_tag: Vec<String>,
    api_tag: Option<Vec<String>>,
    platform_tag: Option<Vec<String>>,
}

impl CompatibilityTag {
    pub fn from_parts(python_part: &str, api_part: &str, platform_part: &str) -> Option<Self> {
        // this function does what verification it can, but the valid values for each part are
        // entirely defined by the particular Python implementation it is representing and there is
        // no definitive list or limit to these implementations
        if python_part.is_empty()
            || api_part.is_empty()
            || platform_part.is_empty()
            || api_part
                .chars()
                .any(|c| !(c.is_alphanumeric() || c == '_' || c == '.'))
            || platform_part
                .chars()
                .any(|c| !(c.is_alphanumeric() || c == '_' || c == '.'))
        {
            // this restriction is not explicit in PEP-425 but implied by the FAQ
            // "Why normalise hyphens and other non-alphanumeric characters to underscores?"
            return None;
        };

        // TODO: "py" pure-python tags cannot be combined with api or platform tags
        let python_tag = python_part
            .split('.')
            .map(|p| p.to_string())
            .collect::<Vec<String>>();

        // TODO: "none" tag cannot be concatenated with other api tags
        let api_tag: Option<Vec<String>> = if api_part == "none" {
            None
        } else {
            Some(
                api_part
                    .split('.')
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>(),
            )
        };

        // TODO: "any" tag cannot be concatenated with other platform tags
        let platform_tag: Option<Vec<String>> = if platform_part == "any" {
            None
        } else {
            Some(
                platform_part
                    .split('.')
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>(),
            )
        };

        if api_tag.is_some() && platform_tag.is_none() {
            None
        } else {
            Some(Self {
                python_tag,
                api_tag,
                platform_tag,
            })
        }
    }

    pub fn from_tag(tag: &str) -> Option<Self> {
        let mut tags = tag.splitn(3, '-');
        let py = tags.next()?;
        let api = tags.next()?;
        let plat = tags.next()?;
        Self::from_parts(py, api, plat)
    }

    pub fn python_tags(&self) -> Vec<&str> {
        self.python_tag.iter().map(AsRef::as_ref).collect()
    }

    pub fn api_tags(&self) -> Vec<&str> {
        if let Some(api_tag) = &self.api_tag {
            api_tag.iter().map(AsRef::as_ref).collect()
        } else {
            vec!["none"]
        }
    }

    pub fn platform_tags(&self) -> Vec<&str> {
        if let Some(platform_tag) = &self.platform_tag {
            platform_tag.iter().map(AsRef::as_ref).collect()
        } else {
            vec!["any"]
        }
    }

    pub fn is_universal(&self) -> bool {
        self.is_pure() && self.python_tag == vec!["py2", "py3"]
    }

    pub fn is_pure(&self) -> bool {
        self.for_any_platform() && self.for_any_abi()
    }

    pub fn for_any_platform(&self) -> bool {
        self.platform_tag.is_none()
    }

    pub fn for_any_abi(&self) -> bool {
        self.api_tag.is_none()
    }
}

pub fn split_python_tag(python_tag: &str) -> (String, String) {
    let implementation = python_tag
        .chars()
        .take_while(|b| !b.is_ascii_digit())
        .collect::<String>();
    let version = python_tag
        .chars()
        .skip_while(|b| !b.is_ascii_digit())
        .collect::<String>();
    (implementation, version)
}

impl fmt::Display for CompatibilityTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let python_tag = self.python_tag.join(".");
        let api_tag = if let Some(tag) = &self.api_tag {
            tag.join(".")
        } else {
            "none".to_string()
        };
        let platform_tag = if let Some(tag) = &self.platform_tag {
            tag.join(".")
        } else {
            "any".to_string()
        };
        write!(f, "{}-{}-{}", python_tag, api_tag, platform_tag)
    }
}

pub struct WheelName {
    distribution: String,
    version: Version,
    pub build_tag: Option<BuildTag>,
    pub compatibility_tag: CompatibilityTag,
}

impl WheelName {
    /// Parse a PEP-427 compliant wheel name
    pub fn from_filename(filename: &str) -> Result<Self, Error> {
        if !filename.ends_with(".whl") {
            return Err(Error::InvalidWheelName);
        }
        let name_parts = filename.strip_suffix(".whl").unwrap();
        let compat_and_name: Vec<&str> = name_parts.rsplitn(4, '-').collect();
        if compat_and_name.len() != 4 {
            return Err(Error::InvalidWheelName);
        }
        let compatibility_tag = CompatibilityTag::from_parts(
            compat_and_name[2],
            compat_and_name[1],
            compat_and_name[0],
        )
        .ok_or(Error::InvalidWheelName)?;
        let name_and_maybe_build: Vec<&str> = compat_and_name[3].split('-').collect();
        if name_and_maybe_build.len() < 2 || name_and_maybe_build.len() > 3 {
            return Err(Error::InvalidWheelName);
        }
        let distribution = name_and_maybe_build[0].to_string();
        let version = Version::parse(name_and_maybe_build[1]).ok_or(Error::InvalidWheelName)?;
        let build_tag: Option<BuildTag> = if name_and_maybe_build.len() == 3 {
            Some(BuildTag::parse(name_and_maybe_build[2]).ok_or(Error::InvalidWheelName)?)
        } else {
            None
        };
        Ok(WheelName {
            distribution,
            version,
            build_tag,
            compatibility_tag,
        })
    }
}

impl fmt::Display for WheelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maybe_build_tag = if let Some(some_build_tag) = &self.build_tag {
            format!("-{}", some_build_tag)
        } else {
            "".to_string()
        };
        write!(
            f,
            "{}-{}{}-{}",
            self.distribution, self.version, maybe_build_tag, self.compatibility_tag
        )
    }
}
