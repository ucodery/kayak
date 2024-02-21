use crate::distribution::WheelName;
use crate::package_inspect;
use crate::picker::pick_best_bdist;
use crate::warehouse::{DistributionUrl, Package, PackageVersion};
use colored::*;
use std::iter;

fn format_name_version(version: &PackageVersion) -> ColoredString {
    if let Some(_reason) = &version.yanked_reason {
        format!("    {}@{} [YANKED]", &version.name, &version.version).red()
    } else {
        format!("    {}@{}", &version.name, &version.version).yellow()
    }
}

fn format_summary(version: &PackageVersion) -> ColoredString {
    format!("\n    {}", version.summary.clone().unwrap_or_default()).normal()
}

fn format_license_copyright(version: &PackageVersion) -> ColoredString {
    if let Some(license) = &version.license {
        if let Some(author_email) = &version.author_email {
            format!("\n    {license} © {}", author_email.replace('"', "")).yellow()
        } else {
            format!("\n    {license}").yellow()
        }
    } else if let Some(author_email) = &version.author_email {
        format!("\n    © {}", author_email.replace('"', "")).yellow()
    } else {
        "".normal()
    }
}

fn format_urls(version: &PackageVersion) -> Vec<ColoredString> {
    iter::once((&"Package Index".to_string(), &version.project_url))
        .chain(version.project_urls.iter())
        .flat_map(|url| {
            [
                // pypi.org implements icons for some url types
                // https://github.com/pypi/warehouse/blob/main/warehouse/templates/packaging/detail.html#L20
                match url.0.to_ascii_lowercase().as_str() {
                    "package index" => "\n    📦 ".normal(),
                    "download" => "\n    ⇩ ".normal(),
                    "home" | "homepage" | "home page" => "\n    🏠 ".normal(),
                    "changelog" | "change log" | "changes" | "release notes" | "news"
                    | "what's new" | "history" => "\n    📜 ".normal(),
                    "docs" | "documentation" => "\n    📄 ".normal(),
                    "bug" | "issue" | "tracker" | "report" => "\n    🐞 ".normal(),
                    "funding" | "donate" | "donation" | "sponsor" => "\n    💸 ".normal(),
                    "mastodon" => "\n    🐘 ".normal(),
                    _ => "\n    🔗 ".normal(),
                },
                url.1.blue().underline(),
            ]
        })
        .collect()
}

fn format_keywords(version: &PackageVersion) -> ColoredString {
    let keywords = &version.keywords();
    if !keywords.is_empty() {
        format!("\n    {}", keywords.join(", ")).magenta().bold()
    } else {
        "".normal()
    }
}

fn format_classifiers(version: &PackageVersion) -> Vec<ColoredString> {
    if !&version.classifiers.is_empty() {
        version
            .classifiers
            .iter()
            .map(|c| format!("\n    {c}").magenta())
            .collect()
    } else {
        vec![]
    }
}

fn format_bdist(bdist: &DistributionUrl, details: u8) -> Vec<ColoredString> {
    if let Ok(filename) = bdist.filename() {
        if details > 2 {
            vec![
                format!("\n    {} ", filename.compatibility_tag).cyan(),
                bdist.url.cyan().underline(),
            ]
        } else {
            vec![format!("\n    {}", filename.compatibility_tag).cyan()]
        }
    } else {
        vec![]
    }
}

fn format_sdist(sdist: &DistributionUrl, details: u8) -> Vec<ColoredString> {
    if details > 2 {
        vec!["\n    sdist ".cyan(), sdist.url.cyan().underline()]
    } else {
        vec!["\n    sdist".cyan()]
    }
}

fn format_distribution(distribution: &DistributionUrl, details: u8) -> Vec<ColoredString> {
    if distribution.packagetype == "sdist" {
        format_sdist(distribution, details)
    } else if distribution.packagetype == "bdist_wheel" {
        format_bdist(distribution, details)
    } else {
        vec![]
    }
}

fn format_distributions(version: &PackageVersion, details: u8) -> Vec<ColoredString> {
    let sdist = version.urls.iter().any(|u| u.packagetype == "sdist");
    let wheel = version.urls.iter().any(|u| u.packagetype == "bdist_wheel");
    if !(sdist || wheel) {
        return vec![];
    };

    if details < 2 {
        let formatted_sdist = if !sdist {
            ""
        } else if wheel {
            "source, "
        } else {
            "source"
        };

        let formatted_wheel = if !wheel {
            "".to_string()
        } else {
            let version_wheels = version
                .urls
                .iter()
                .filter_map(|u| u.filename().ok())
                .collect::<Vec<WheelName>>();
            if version_wheels
                .iter()
                .any(|d| d.compatibility_tag.is_universal())
            {
                "universal wheel".to_string()
            } else {
                let python_versions = version_wheels
                    .iter()
                    .flat_map(|d| d.compatibility_tag.python_tags())
                    .collect::<Vec<&str>>();
                let wheel_type = if version_wheels.iter().any(|d| d.compatibility_tag.is_pure()) {
                    if python_versions.len() == 1 {
                        "pure wheel"
                    } else {
                        "pure wheels"
                    }
                } else {
                    "platform-specific wheel"
                };
                if python_versions.contains(&"py3") {
                    wheel_type.to_string()
                } else {
                    format!("{} for {}", wheel_type, python_versions[0])
                }
            }
        };
        vec![format!("\n    {formatted_sdist}{formatted_wheel}").cyan()]
    } else {
        version
            .urls
            .iter()
            .flat_map(|u| {
                if u.packagetype == "sdist" {
                    format_sdist(u, details)
                } else if u.packagetype == "bdist_wheel" {
                    format_bdist(u, details)
                } else {
                    vec![]
                }
            })
            .collect()
    }
}

fn format_dependencies(version: &PackageVersion) -> Vec<ColoredString> {
    let mut deps = Vec::new();
    if let Some(requires_python) = &version.requires_python {
        deps.push(format!("\n    python{}", requires_python).green())
    };
    if !&version.requires_dist.is_empty() {
        deps.extend(
            version
                .requires_dist
                .iter()
                .map(|d| format!("\n    {d}").green()),
        )
    };
    deps
}

fn format_readme(version: &PackageVersion) -> ColoredString {
    format!("\n{}", version.description.clone().unwrap_or_default()).normal()
}

fn format_packages(distribution: &DistributionUrl) -> Vec<ColoredString> {
    if let Ok(inspect) = package_inspect::fetch(&distribution.url) {
        inspect
            .provides_packages()
            .iter()
            .map(|p| format!("\n    ■ {p}").red())
            .collect()
    } else {
        vec![]
    }
}

fn format_executables(distribution: &DistributionUrl) -> Vec<ColoredString> {
    if let Ok(inspect) = package_inspect::fetch(&distribution.url) {
        inspect
            .provides_executables()
            .iter()
            .chain(inspect.console_scripts().iter())
            .map(|p| format!("\n    ▶ {p}").red())
            .collect()
    } else {
        vec![]
    }
}

pub fn format_package_version_details(
    version: &PackageVersion,
    distribution: Option<&DistributionUrl>,
    details: u8,
    include_summary: bool,
    include_license: bool,
    include_urls: bool,
    include_keywords: bool,
    include_classifiers: bool,
    include_artifacts: u8,
    include_dependencies: bool,
    include_readme: bool,
    include_packages: bool,
    include_executables: bool,
) -> Vec<ColoredString> {
    let mut display = Vec::new();

    if details >= 1 {
        display.push(format_name_version(version));
    };

    if details >= 3 || include_license {
        display.push(format_license_copyright(version));
    };

    if details >= 2 || include_summary {
        display.push(format_summary(version));
    };

    if details >= 3 || include_urls {
        display.extend(format_urls(version));
    };

    if details >= 4 || include_keywords {
        display.push(format_keywords(version));
    };

    if details >= 4 || include_classifiers {
        display.extend(format_classifiers(version));
    };

    if details >= 5 || include_artifacts >= 1 {
        if let Some(dist) = distribution {
            display.extend(format_distribution(dist, include_artifacts));
        } else {
            display.extend(format_distributions(version, include_artifacts));
        };
    };

    if details >= 6 || include_dependencies {
        display.extend(format_dependencies(version));
    };

    if include_packages {
        if let Some(dist) = distribution {
            if dist.packagetype != "sdist" {
                display.extend(format_packages(dist));
            };
        } else if let Some(dist) = pick_best_bdist(version) {
            display.extend(format_packages(dist));
        };
    }

    if include_executables {
        if let Some(dist) = distribution {
            if dist.packagetype != "sdist" {
                display.extend(format_executables(dist));
            };
        } else if let Some(dist) = pick_best_bdist(version) {
            display.extend(format_executables(dist));
        };
    }

    if details >= 7 || include_readme {
        display.push(format_readme(version));
    };

    display
}

pub fn format_package_versions(package: &Package, details: u8) -> String {
    let name = if details >= 1 {
        format!("    {}\n", package.name).yellow()
    } else {
        "".normal()
    };
    let mut versions: Vec<String> = package
        .ordered_versions()
        .iter()
        .map(|v| v.normalize())
        .collect();
    versions.reverse();
    format!("{name}{}", versions.join(", "))
}
