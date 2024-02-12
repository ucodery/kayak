use crate::format::*;
use crate::picker::pick_version;
use clap::Parser;
use pep440::Version;

pub mod distribution;
pub mod format;
pub mod picker;
pub mod warehouse;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    package: String,

    #[arg(
        value_name = "VERSION",
        long_help = "if not specified, the greatest stable version is automatically retrieved"
    )]
    package_version: Option<String>,
    #[arg(
        value_name = "DIST",
        long_help = "if not specified, a suitable distribution will be automatically retrieved.\n\
                     If any level of artifact metadata is to be displayed, metadata will only be\n\
                     displayed for the specified distribution, otherwise for all distributions the\n\
                     particular version provides"
    )]
    dist: Option<String>,

    #[arg(
        long,
        conflicts_with = "package_version",
        help = "list all versions of this package",
        long_help = "instead of displaying package details, list all versions available"
    )]
    versions: bool,

    #[arg(
        long,
        short = 's',
        help = "display the package's summary",
        long_help = "force the package's summary to display. This happens by default, unless --quiet\n\
                     was set"
    )]
    summary: bool,
    #[arg(
        long,
        short = 'l',
        help = "display the package's license",
        long_help = "force the package's license to display, otherwise requires verbosity 1 before\n\
                     being displayed"
    )]
    license: bool,
    #[arg(
        long,
        short = 'u',
        help = "display the package's URLs",
        long_help = "force the package's URLs to display, otherwise requires verbosity 1 before being\n\
                     displayed"
    )]
    urls: bool,
    #[arg(
        long,
        short = 'k',
        help = "display the package's keywords",
        long_help = "force the package's keywords to display, otherwise requires verbosity 2 before\n\
                     being displayed"
    )]
    keywords: bool,
    #[arg(
        long,
        short = 'c',
        help = "display the package's classifiers",
        long_help = "force the package's classifiers to display, otherwise requires verbosity 2 before\n\
                     being displayed"
    )]
    classifiers: bool,
    #[arg(
        long,
        short = 'a',
        action = clap::ArgAction::Count,
        help = "display the package's artifact types",
        long_help = "force the package's artifact types to display, otherwise requires verbosity 3\n\
                     before being displayed. This option can be passed up to 3 times, each time will\n\
                     display more details about the artifacts available. Verbosity of level 3 or\n\
                     higher will still only display the first level of artifact detail"
    )]
    artifacts: u8,
    #[arg(
        long,
        short = 'd',
        help = "display the package's dependencies",
        long_help = "force the package's dependencies to display, otherwise requires verbosity 4\n\
                     before being displayed"
    )]
    dependencies: bool,
    #[arg(
        long,
        short = 'r',
        help = "display the package's readme",
        long_help = "force the package's readme to display, otherwise requires verbosity 5 before\n\
                     being displayed"
    )]
    readme: bool,
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        help = "display more package details",
        long_help = "display more package details. This option can be passed up to 5 times, each time\n\
                     will display even more details",
    )]
    verbose: u8,
    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        help = "display less package details",
        long_help = "disable displaying any extra package details. This option can be passed up to 2\n\
                     times, if passed twice and no other details are selected, the command will output\n\
                     nothing. This option overrides verbosity, but not explicit package detail options",
    )]
    quiet: u8,
}

fn main() -> Result<(), warehouse::Error> {
    let cli = Cli::parse();

    // do sanity checks before making network requests
    if let Some(v) = &cli.package_version {
        Version::parse(v).ok_or(warehouse::Error::InvalidVersion)?;
    };
    if let Some(d) = &cli.dist {
        if d != "sdist" {
            distribution::CompatibilityTag::from_tag(d).ok_or(warehouse::Error::InvalidVersion)?;
        };
    };

    let details = if cli.quiet > 1 {
        0
    } else if cli.quiet == 1 {
        1
    } else {
        cli.verbose + 2
    };

    if cli.versions {
        return Ok(println!(
            "{}",
            format_package_versions(
                &warehouse::Package::fetch(warehouse::PYPI_URI, &cli.package)?,
                details
            )
        ));
    };

    let package_version = pick_version(cli.package, cli.package_version)?;

    let formatted = format_package_version_details(
        &package_version,
        cli.dist,
        details,
        cli.summary,
        cli.license,
        cli.urls,
        cli.keywords,
        cli.classifiers,
        cli.artifacts,
        cli.dependencies,
        cli.readme,
    );

    Ok(println!(
        "{}",
        formatted
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("")
    ))
}
