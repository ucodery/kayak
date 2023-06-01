use crate::warehouse::PackageVersion;
use clap::Parser;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::iter;

pub mod distribution;
pub mod warehouse;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    package: String,

    #[arg(value_name = "VERSION")]
    package_version: Option<String>,

    #[arg(long, conflicts_with = "package_version")]
    versions: bool,

    #[arg(long)]
    distributions: bool,

    #[arg(long)]
    readme: bool,

    #[arg(long, conflicts_with = "sdist")]
    bdist: Option<Option<String>>,
    #[arg(long, conflicts_with = "bdist")]
    sdist: bool,
}

fn select_bdist(
    version: &PackageVersion,
    requested: distribution::CompatibilityTag,
) -> Option<&warehouse::DistributionUrl> {
    version
        .urls
        .iter()
        .filter(|u| u.packagetype == "bdist_wheel")
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

fn find_best_bdist(version: &PackageVersion) -> Option<&warehouse::DistributionUrl> {
    version
        .urls
        .iter()
        .filter(|u| u.packagetype == "bdist_wheel")
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

fn format_package_version_details(version: &PackageVersion, include_readme: bool) -> String {
    let name = &version.name;
    let ver = &version.version;
    let yanked = if let Some(yanked) = &version.yanked_reason {
        format!("Yanked: {}\n", yanked)
    } else {
        "".to_string()
    };
    let summary = &version.summary.clone().unwrap_or_default();
    let license = &version.license.clone().unwrap_or_default();
    let author = &version.author_email.clone().unwrap_or_default();
    let keywords = &version.keywords.clone().unwrap_or_default();
    let classifiers = version
        .classifiers()
        .iter()
        .map(|t| t.as_ref())
        .collect::<Vec<&str>>()
        .join("\n    ");
    let links = iter::once((&"Package Index".to_string(), &version.project_url))
        .chain(version.project_urls.iter())
        .map(|url| format!("{}: {}", url.0, url.1))
        .collect::<Vec<String>>()
        .join("\n    ");
    let mut dists = version
        .urls
        .iter()
        .map(|u| u.packagetype.clone())
        .collect::<Vec<String>>();
    dists.sort();
    dists.dedup();
    let dist_types = dists.join(", ");
    let bdists = version
        .urls
        .iter()
        .filter_map(|u| u.filename().ok())
        .map(|d| d.compatibility_tag)
        .collect::<HashSet<distribution::CompatibilityTag>>();
    let mut universal = false;
    let mut python_to_platform = HashMap::<String, Option<Vec<String>>>::new();
    for bdist in bdists {
        if bdist.is_universal() {
            universal = true;
            break;
        }
        for python_version in bdist.python_tags() {
            if bdist.for_any_platform() {
                python_to_platform.insert(python_version.to_string(), None);
            } else if python_to_platform.contains_key(python_version) {
                if python_to_platform[python_version].is_some() {
                    python_to_platform
                        .get_mut(python_version)
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .extend(bdist.platform_tags().iter().map(|p| p.to_string()));
                }; // else is platform agnostic
            } else {
                python_to_platform.insert(
                    python_version.to_string(),
                    Some(
                        bdist
                            .platform_tags()
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<String>>(),
                    ),
                );
            };
        }
    }
    let outcome = if universal {
        "universal".to_string()
    } else {
        let mut all_pythons = Vec::new();
        let pure_py3 =
            python_to_platform.contains_key("py3") && python_to_platform["py3"].is_none();
        let pure_py2 =
            python_to_platform.contains_key("py2") && python_to_platform["py2"].is_none();
        for (python, supported_platforms) in python_to_platform.iter().filter(|(p, _p)| {
            !((pure_py3 && distribution::split_python_tag(p).1.starts_with('3') && p != &"py3")
                || (pure_py2
                    && distribution::split_python_tag(p).1.starts_with('2')
                    && p != &"py2"))
        }) {
            if let Some(platforms) = supported_platforms {
                all_pythons.push(format!("{} for {}", python, platforms.join(" ")));
            } else {
                all_pythons.push(python.clone());
            }
        }
        all_pythons.join("\n               ")
    };
    let requires_python = &version
        .requires_python
        .clone()
        .unwrap_or_else(|| "any".to_string());
    let dependencies = &version.requires_dist.clone().join(" ");
    let readme = if include_readme {
        version.description.clone().unwrap_or_default()
    } else {
        String::default()
    };
    format!(
        "Name: {name}\n\
             Version: {ver}\n\
             {yanked}\
             Summary: {summary}\n\
             License: {license}\n\
             Author: {author}\n\
             Keywords: {keywords}\n\
             Classifiers:\n    {classifiers}\n\
             Links:\n    {links}\n\
             Distribution types: {dist_types}\n\
             Wheel targets: {outcome}\n\
             Requires Python: {requires_python}\n\
             Depends On: {dependencies}\n\
             {readme}"
    )
}

fn main() {
    let cli = Cli::parse();

    if cli.versions {
        let package = warehouse::Package::fetch(warehouse::PYPI_URI, &cli.package).unwrap();
        let mut versions: Vec<String> = package
            .ordered_versions()
            .iter()
            .map(|v| v.normalize())
            .collect();
        versions.reverse();
        println!("{}", versions.join(", "));
        return;
    }

    let package_version = if let Some(version) = cli.package_version {
        warehouse::PackageVersion::fetch(warehouse::PYPI_URI, &cli.package, &version)
            .expect("Could not find that package version")
    } else {
        let package = warehouse::Package::fetch(warehouse::PYPI_URI, &cli.package)
            .expect("Could not find that package");
        let version = package.ordered_versions().last().unwrap().to_string();
        warehouse::PackageVersion::fetch(warehouse::PYPI_URI, &cli.package, &version)
            .expect("Could not find that package version")
    };

    if cli.distributions {
        let dists = package_version
            .urls
            .iter()
            .filter_map(|u| u.filename().ok())
            .map(|d| d.compatibility_tag.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        println!("{dists}");
    } else if let Some(bdist) = cli.bdist {
        if let Some(specific) = bdist {
            let requested = distribution::CompatibilityTag::from_tag(&specific)
                .expect("Specified bdist should be a valid compatibility tag");
            let found = select_bdist(&package_version, requested);
            if let Some(dist) = found {
                println!("{}", dist.url);
            } else {
                println!("NOT FOUND");
            }
        } else if let Some(dist) = find_best_bdist(&package_version) {
            println!("{}", dist.url);
        } else {
            println!("NO WHEELS FOUND");
        }
    } else if cli.sdist {
        if let Some(dist) = package_version
            .urls
            .iter()
            .find(|u| u.packagetype == "sdist")
        {
            println!("{}", dist.url);
        } else {
            println!("NOT FOUND");
        }
    } else {
        println!(
            "{}",
            format_package_version_details(&package_version, cli.readme)
        );
    }
}
