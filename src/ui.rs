use crate::warehouse::DistributionUrl;

pub mod interactive;
pub mod pretty;
pub mod text;

/// Determine an appropriate icon for the url type
/// pypi.org implements icons for some url types
/// https://github.com/pypi/warehouse/blob/main/warehouse/templates/packaging/detail.html#L20
fn iconify_url(url: &str) -> String {
    match url.to_ascii_lowercase().as_str() {
        "package index" => "📦".to_string(),
        "download" => "⇩".to_string(),
        "home" | "homepage" | "home page" => "🏠".to_string(),
        "changelog" | "change log" | "changes" | "release notes" | "news" | "what's new"
        | "history" => "📜".to_string(),
        "docs" | "documentation" => "📄".to_string(),
        "bug" | "issue" | "tracker" | "report" => "🐞".to_string(),
        "funding" | "donate" | "donation" | "sponsor" => "💸".to_string(),
        "mastodon" => "🐘".to_string(),
        _ => "🔗".to_string(),
    }
}

fn summarize_artifacts<'a, A>(artifacts: A) -> String
where
    A: Iterator<Item = &'a DistributionUrl>,
{
    let mut sdist: u16 = 0;
    let mut universal: u16 = 0;
    let mut pure: u16 = 0;
    let mut plat: u16 = 0;
    for artifact in artifacts {
        if artifact.packagetype == "sdist" {
            sdist += 1;
        } else if artifact.packagetype == "bdist_wheel" {
            let compat = artifact.filename().unwrap().compatibility_tag;
            if compat.is_universal() {
                universal += 1;
            } else if compat.is_pure() {
                pure += 1;
            } else {
                plat += 1;
            }
        }
    }

    [
        (sdist > 0).then_some("sdist"),
        (universal > 0).then_some("universal wheel"),
        (pure > 1)
            .then_some("pure wheels")
            .or((pure > 0).then_some("pure wheel")),
        (plat > 1)
            .then_some("platform-specific wheels")
            .or((plat > 0).then_some("platform-specific wheel")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" and ")
}
