use crate::distribution::WheelName;
use crate::warehouse::{Package, PackageVersion};
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
        println!("{:?}", keywords);
        format!("\n    {}", keywords.join(", ")).magenta().bold()
    } else {
        "".normal()
    }
}

fn format_classifiers(version: &PackageVersion) -> ColoredString {
    if !&version.classifiers.is_empty() {
        format!("\n    {}", &version.classifiers.join("\n    ")).magenta()
    } else {
        "".normal()
    }
}

fn format_distributions(version: &PackageVersion, details: u8) -> Vec<ColoredString> {
    let sdist = version.urls.iter().any(|u| u.packagetype == "sdist");
    let wheel = version.urls.iter().any(|u| u.packagetype == "bdist_wheel");
    if !(sdist || wheel) {
        return vec!["".normal()];
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
            .filter(|u| u.packagetype == "sdist")
            .flat_map(|u| {
                if details > 2 {
                    vec!["\n    sdist ".cyan(), u.url.cyan().underline()]
                } else {
                    vec!["\n    sdist".cyan()]
                }
            })
            .chain(
                version
                    .urls
                    .iter()
                    .filter(|u| u.filename().is_ok())
                    .flat_map(|u| {
                        if details > 2 {
                            vec![
                                format!(
                                    "\n    {} ",
                                    u.filename().unwrap().compatibility_tag
                                )
                                .cyan(),
                                u.url.cyan().underline(),
                            ]
                        } else {
                            vec![format!(
                                "\n    {}",
                                u.filename().unwrap().compatibility_tag
                            )
                            .cyan()]
                        }
                    }),
            )
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

pub fn format_package_version_details(
    version: &PackageVersion,
    details: u8,
    include_summary: bool,
    include_license: bool,
    include_urls: bool,
    include_keywords: bool,
    include_classifiers: bool,
    include_artifacts: u8,
    include_dependencies: bool,
    include_readme: bool,
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
        display.push(format_classifiers(version));
    };

    if details >= 5 || include_artifacts >= 1 {
        display.extend(format_distributions(version, include_artifacts));
    };

    if details >= 6 || include_dependencies {
        display.extend(format_dependencies(version));
    };

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
