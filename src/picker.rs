use crate::distribution;
use crate::package_inspect;
use crate::warehouse;

use anyhow::Result;

use std::cmp::Ordering;

// lazy loader for project metadata types
pub struct Project {
    package_selector: String,
    version_selector: Option<String>,
    distribution_selector: Option<String>,
    package: Option<warehouse::Package>,
    version: Option<warehouse::PackageVersion>,
    distribution: Option<warehouse::DistributionUrl>,
    import_package: Option<package_inspect::Package>,
}

impl Project {
    pub fn new(
        user_package: String,
        user_version: Option<String>,
        user_distribution: Option<String>,
    ) -> Self {
        Project {
            package_selector: user_package,
            version_selector: user_version,
            distribution_selector: user_distribution,
            package: None,
            version: None,
            distribution: None,
            import_package: None,
        }
    }

    pub fn version_was_selected(&self) -> bool {
        self.version_selector.is_some()
    }

    pub fn distribution_was_selected(&self) -> bool {
        self.distribution_selector.is_some()
    }

    pub fn package(&mut self) -> Result<&warehouse::Package> {
        if self.package.is_none() {
            self.package = Some(warehouse::Package::fetch(
                warehouse::PYPI_URI,
                &self.package_selector,
            )?)
        }
        Ok(self.package.as_ref().unwrap())
    }

    pub fn version(&mut self) -> Result<&warehouse::PackageVersion> {
        if self.version.is_none() {
            self.version = if let Some(version) = &self.version_selector {
                Some(warehouse::PackageVersion::fetch(
                    warehouse::PYPI_URI,
                    &self.package_selector,
                    version,
                )?)
            } else {
                Some(
                    self.package()?
                        .ordered_versions()
                        .iter()
                        .rev()
                        .filter_map(|v| {
                            warehouse::PackageVersion::fetch(
                                warehouse::PYPI_URI,
                                &self.package_selector,
                                &v.to_string(),
                            )
                            .ok()
                        })
                        .find(|v| !v.yanked)
                        .ok_or(warehouse::Error::NotFound)?,
                )
            };
        }
        Ok(self.version.as_ref().unwrap())
    }

    pub fn distribution(&mut self) -> Result<&warehouse::DistributionUrl> {
        if self.distribution.is_none() {
            self.distribution = if let Some(distribution) = &self.distribution_selector {
                if distribution == "sdist" {
                    self.select_sdist()
                } else {
                    self.select_bdist()
                }
            } else {
                self.pick_best_bdist()
            }
        }
        self.distribution
            .as_ref()
            .ok_or(distribution::Error::InvalidWheelName.into())
    }

    pub fn import_package(&mut self) -> Result<&package_inspect::Package> {
        if self.import_package.is_none() {
            if self.distribution_selector == Some("sdist".to_string()) {
                // cannot extract package from a source distribution
                return Err(warehouse::Error::InvalidName)?;
            } else if self.distribution()?.packagetype == "sdist" {
                // select a new distribution
                self.distribution = None;
                if self.distribution()?.packagetype == "sdist" {
                    // maybe there are no wheels
                    return Err(warehouse::Error::InvalidName)?;
                }
            }
            self.import_package = Some(package_inspect::fetch(&self.distribution()?.url)?);
        }
        Ok(self.import_package.as_ref().unwrap())
    }

    fn select_sdist(&mut self) -> Option<warehouse::DistributionUrl> {
        self.version()
            .ok()?
            .urls
            .iter()
            .find(|u| u.packagetype == "sdist")
            .to_owned()
            .cloned()
    }

    fn select_bdist(&mut self) -> Option<warehouse::DistributionUrl> {
        let requested =
            distribution::CompatibilityTag::from_tag(self.distribution_selector.as_ref()?)?;
        self.version()
            .ok()?
            .urls
            .iter()
            .filter(|u| u.packagetype == "bdist_wheel")
            .filter(|u| distribution::WheelName::from_filename(&u.filename).is_ok())
            .filter(|u| {
                distribution::WheelName::from_filename(&u.filename)
                    .unwrap()
                    .compatibility_tag
                    == requested
            })
            .max_by(|a, b| {
                distribution::WheelName::from_filename(&a.filename)
                    .unwrap()
                    .build_tag
                    .unwrap_or(distribution::BuildTag {
                        number: 0,
                        string: String::default(),
                    })
                    .cmp(
                        &distribution::WheelName::from_filename(&b.filename)
                            .unwrap()
                            .build_tag
                            .unwrap_or(distribution::BuildTag {
                                number: 0,
                                string: String::default(),
                            }),
                    )
            })
            .to_owned()
            .cloned()
    }

    fn pick_best_bdist(&mut self) -> Option<warehouse::DistributionUrl> {
        self.version()
            .ok()?
            .urls
            .iter()
            .filter(|u| u.packagetype == "bdist_wheel")
            .filter(|u| distribution::WheelName::from_filename(&u.filename).is_ok())
            .max_by(|a, b| {
                let a_wheel = distribution::WheelName::from_filename(&a.filename)
                    .unwrap()
                    .compatibility_tag;
                let b_wheel = distribution::WheelName::from_filename(&b.filename)
                    .unwrap()
                    .compatibility_tag;
                if a_wheel.is_universal() {
                    Ordering::Greater
                } else if b_wheel.is_universal() {
                    Ordering::Less
                } else if a_wheel.is_pure() {
                    Ordering::Greater
                } else if b_wheel.is_pure() {
                    Ordering::Less
                } else if a_wheel.for_any_platform() {
                    Ordering::Greater
                } else if b_wheel.for_any_platform() {
                    Ordering::Less
                } else if a_wheel.for_any_abi() {
                    Ordering::Greater
                } else if b_wheel.for_any_abi() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .to_owned()
            .cloned()
    }

    pub fn package_selector(&self) -> String {
        self.package_selector.clone()
    }

    pub fn version_selector(&self) -> Option<String> {
        self.version_selector.clone()
    }

    pub fn distribution_selector(&self) -> Option<String> {
        self.distribution_selector.clone()
    }
}
