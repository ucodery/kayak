use crate::distribution;
use crate::warehouse;

use std::cmp::Ordering;

pub fn pick_version(
    package: String,
    version: Option<String>,
) -> Result<warehouse::PackageVersion, warehouse::Error> {
    match version {
        Some(v) => Ok(warehouse::PackageVersion::fetch(
            warehouse::PYPI_URI,
            &package,
            &v,
        )?),
        None => Ok(warehouse::Package::fetch(warehouse::PYPI_URI, &package)?
            .ordered_versions()
            .iter()
            .rev()
            .filter_map(|v| {
                warehouse::PackageVersion::fetch(warehouse::PYPI_URI, &package, &v.to_string()).ok()
            })
            .find(|v| !v.yanked)
            .ok_or(warehouse::Error::NotFound)?),
    }
}

fn select_bdist(
    version: &warehouse::PackageVersion,
    requested: distribution::CompatibilityTag,
) -> Option<&warehouse::DistributionUrl> {
    version
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
}

pub fn pick_best_bdist(version: &warehouse::PackageVersion) -> Option<&warehouse::DistributionUrl> {
    version
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
}

pub fn pick_dist(
    version: &warehouse::PackageVersion,
    distribution: String,
) -> Result<&warehouse::DistributionUrl, warehouse::Error> {
    if distribution == "sdist" {
        version
            .urls
            .iter()
            .find(|u| u.packagetype == "sdist")
            .ok_or(warehouse::Error::NotFound)
    } else {
        let compat = distribution::CompatibilityTag::from_tag(&distribution)
            .ok_or(warehouse::Error::InvalidName)?;
        select_bdist(version, compat).ok_or(warehouse::Error::NotFound)
    }
}
