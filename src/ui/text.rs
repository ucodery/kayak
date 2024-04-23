use crate::package_inspect;
use crate::ui::*;
use crate::warehouse::{DistributionUrl, Error, PackageVersion};
use crate::{DisplayFields, Project};
use chrono::{DateTime, Utc};
use std::iter;
use termimad::*;

fn format_name_version(version: &PackageVersion) -> String {
    if let Some(_reason) = &version.yanked_reason {
        format!("{}@{} [YANKED]", &version.name, &version.version)
    } else {
        format!("{}@{}", &version.name, &version.version)
    }
}

fn format_dist_time(version: &PackageVersion, distribution: Option<&DistributionUrl>) -> String {
    if let Some(dist) = distribution {
        format!("{}@{}", format_dist(dist, 0), dist.upload_time)
    } else if let Some(time) = version
        .urls
        .iter()
        .filter_map(|u| u.upload_time_iso_8601.parse::<DateTime<Utc>>().ok())
        .min()
    {
        format!("  {}", time.format("%Y-%m-%dT%H:%M:%S"))
    } else {
        "".to_string()
    }
}

fn format_summary(version: &PackageVersion) -> String {
    format!("  {}", version.summary.clone().unwrap_or_default())
}

fn format_license_copyright(version: &PackageVersion) -> String {
    if let Some(license) = &version.license {
        if let Some(author_email) = &version.author_email {
            format!("  {license} © {}", author_email.replace('"', ""))
        } else {
            format!("  {license}")
        }
    } else if let Some(author_email) = &version.author_email {
        format!("  © {}", author_email.replace('"', ""))
    } else {
        "".to_string()
    }
}

fn format_urls(version: &PackageVersion) -> Vec<String> {
    iter::once("Links".to_string())
        .chain(
            iter::once((&"Package Index".to_string(), &version.project_url))
                .chain(version.project_urls.iter())
                .map(|url| format!("  {}  {}", iconify_url(url), url.1)),
        )
        .collect()
}

fn format_keywords(version: &PackageVersion) -> Vec<String> {
    let keywords = &version.keywords();
    if !keywords.is_empty() {
        vec!["Keywords".to_string(), format!("  {}", keywords.join(", "))]
    } else {
        vec![]
    }
}

fn format_classifiers(version: &PackageVersion) -> Vec<String> {
    if !&version.classifiers.is_empty() {
        iter::once("Classifiers".to_string())
            .chain(version.classifiers.iter().map(|c| format!("  {c}")))
            .collect()
    } else {
        vec![]
    }
}

fn format_dist(dist: &DistributionUrl, details: u8) -> String {
    if dist.packagetype == "sdist" {
        if details > 3 {
            format!("  sdist {} {}", dist.upload_time, dist.url)
        } else if details == 3 {
            format!("  sdist {}", dist.url)
        } else {
            "  sdist".to_string()
        }
    } else if dist.packagetype == "bdist_wheel" {
        if details > 3 {
            format!(
                "  {} {} {}",
                dist.filename().unwrap().compatibility_tag,
                dist.upload_time,
                dist.url
            )
        } else if details == 3 {
            format!(
                "  {} {}",
                dist.filename().unwrap().compatibility_tag,
                dist.url
            )
        } else {
            format!("  {}", dist.filename().unwrap().compatibility_tag)
        }
    } else {
        "".to_string()
    }
}

fn format_distributions(distributions: &[DistributionUrl], details: u8) -> Vec<String> {
    let sdist = distributions.iter().any(|u| u.packagetype == "sdist");
    let wheel = distributions.iter().any(|u| u.packagetype == "bdist_wheel");
    if !(sdist || wheel) {
        return vec![];
    };

    let header = "Distribution Types".to_string();
    if details == 1 {
        vec![
            header,
            format!("  {}", summarize_artifacts(distributions.iter())),
        ]
    } else {
        iter::once(header)
            .chain(distributions.iter().map(|u| format_dist(u, details)))
            .collect()
    }
}

fn format_dependencies(version: &PackageVersion) -> Vec<String> {
    let dependencies = iter::once("Dependencies".to_string())
        .chain(
            version
                .requires_python
                .clone()
                .into_iter()
                .map(|p| format!("  python{p}"))
                .chain(version.requires_dist.iter().map(|d| format!("  {d}"))),
        )
        .collect::<Vec<_>>();
    if dependencies.len() == 1 {
        vec![]
    } else {
        dependencies
    }
}

fn format_readme(version: &PackageVersion, style: bool) -> String {
    if style {
        if let Some(Ok(content_type)) = version.description_content_type() {
            if content_type.essence_str() == "text/markdown" {
                return MadSkin::default()
                    .term_text(&version.description.clone().unwrap_or_default())
                    .to_string();
            };
        };
    };
    version.description.clone().unwrap_or_default().to_string()
}

fn format_packages(distribution: &DistributionUrl) -> Vec<String> {
    if distribution.packagetype == "sdist" {
        // don't know how to extract packages from an sdist
        return vec![];
    };
    if let Ok(inspect) = package_inspect::fetch(&distribution.url) {
        iter::once("Importable Packages".to_string())
            .chain(inspect.provides_packages().iter().map(|p| format!("  {p}")))
            .collect()
    } else {
        vec![]
    }
}

fn format_executables(distribution: &DistributionUrl) -> Vec<String> {
    if distribution.packagetype == "sdist" {
        // don't know how to extract packages from an sdist
        return vec![];
    };
    if let Ok(inspect) = package_inspect::fetch(&distribution.url) {
        iter::once("Executable Commands".to_string())
            .chain(
                inspect
                    .provides_executables()
                    .iter()
                    .chain(inspect.console_scripts().iter())
                    .map(|p| format!("  {p}")),
            )
            .collect()
    } else {
        vec![]
    }
}

fn format_package_version_details(
    mut project: Project,
    display_fields: DisplayFields,
) -> Result<String, Error> {
    let mut display = Vec::new();

    if display_fields.name {
        display.push(format_name_version(project.version()?));
    };

    if display_fields.time || project.distribution_was_selected() {
        let dist = (project.distribution_was_selected()).then_some(project.distribution()?.clone());
        display.push(format_dist_time(project.version()?, dist.as_ref()));
    };

    if display_fields.license {
        display.push(format_license_copyright(project.version()?));
    };

    if display_fields.summary {
        display.push(format_summary(project.version()?));
    };

    if display_fields.urls {
        display.extend(format_urls(project.version()?));
    };

    if display_fields.keywords {
        display.extend(format_keywords(project.version()?));
    };

    if display_fields.classifiers {
        display.extend(format_classifiers(project.version()?));
    };

    if display_fields.artifacts >= 1 {
        if project.distribution_was_selected() {
            display.extend(format_distributions(
                &vec![project.distribution()?.clone()],
                display_fields.artifacts,
            ));
        } else {
            display.extend(format_distributions(
                &project.version()?.urls,
                display_fields.artifacts,
            ));
        };
    };

    if display_fields.dependencies {
        display.extend(format_dependencies(project.version()?));
    };

    if display_fields.packages {
        display.extend(format_packages(project.distribution()?));
    }

    if display_fields.executables {
        display.extend(format_executables(project.distribution()?));
    }

    if display_fields.readme >= 1 {
        let render_readme = display_fields.readme >= 2;
        display.push(format_readme(project.version()?, render_readme));
    };

    Ok(display.join("\n"))
}

fn format_package_versions(
    mut project: Project,
    display_fields: DisplayFields,
) -> Result<String, Error> {
    let package = project.package()?;
    let name = if display_fields.name {
        format!("{}\n", &package.name)
    } else {
        "".to_string()
    };
    let mut versions: Vec<String> = package
        .ordered_versions()
        .iter()
        .map(|v| v.normalize())
        .collect();
    versions.reverse();
    Ok(format!("{name}{}", versions.join(", ")))
}

pub fn display(project: Project, display_fields: DisplayFields) -> Result<(), Error> {
    if display_fields.versions {
        println!("{}", format_package_versions(project, display_fields)?);
    } else {
        println!(
            "{}",
            format_package_version_details(project, display_fields)?
        );
    }
    Ok(())
}
