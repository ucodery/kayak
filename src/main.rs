#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use crate::format::*;
use crate::picker::{pick_dist, pick_version};
use clap::Parser;
use pep440::Version;
use termimad::*;

pub mod distribution;
pub mod format;
pub mod package_inspect;
pub mod picker;
pub mod warehouse;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(long_help = "the name of the python project to look up")]
    project: String,
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
        help = "list all versions of this project",
        long_help = "instead of displaying project details, list all versions available"
    )]
    versions: bool,

    #[arg(
        long,
        short = 's',
        help = "display the project's summary",
        long_help = "force the project's summary to display. This happens by default, unless --quiet\n\
                     was set"
    )]
    summary: bool,
    #[arg(
        long,
        short = 'l',
        help = "display the project's license",
        long_help = "force the project's license to display, otherwise requires verbosity 1 before\n\
                     being displayed"
    )]
    license: bool,
    #[arg(
        long,
        short = 'u',
        help = "display the project's URLs",
        long_help = "force the project's URLs to display, otherwise requires verbosity 1 before being\n\
                     displayed"
    )]
    urls: bool,
    #[arg(
        long,
        short = 'k',
        help = "display the project's keywords",
        long_help = "force the project's keywords to display, otherwise requires verbosity 2 before\n\
                     being displayed"
    )]
    keywords: bool,
    #[arg(
        long,
        short = 'c',
        help = "display the project's classifiers",
        long_help = "force the project's classifiers to display, otherwise requires verbosity 2 before\n\
                     being displayed"
    )]
    classifiers: bool,
    #[arg(
        long,
        short = 'a',
        action = clap::ArgAction::Count,
        help = "display the project's artifact types",
        long_help = "force the project's artifact types to display, otherwise requires verbosity 3\n\
                     before being displayed. This option can be passed up to 3 times, each time will\n\
                     display more details about the artifacts available. Verbosity of level 3 or\n\
                     higher will still only display the first level of artifact detail"
    )]
    artifacts: u8,
    #[arg(
        long,
        short = 'd',
        help = "display the project's dependencies",
        long_help = "force the project's dependencies to display, otherwise requires verbosity 4\n\
                     before being displayed"
    )]
    dependencies: bool,
    #[arg(
        long,
        short = 'r',
        action = clap::ArgAction::Count,
        help = "display the project's readme",
        long_help = "force the project's readme to display, otherwise requires verbosity 5 before\n\
                     being displayed. This option can be passed up to 2 times, if passed twice the\n\
                     readme will be styled if it is of a known content type"
    )]
    readme: u8,
    #[arg(
        long,
        short = 'p',
        help = "display the project's importable packages",
        long_help = "display the project's importable top-level names. Not displayed under any\n\
                     verbosity level"
    )]
    packages: bool,
    #[arg(
        long,
        short = 'e',
        help = "display the project's executable commands",
        long_help = "display the project's executable file names. Not displayed under any\n\
                     verbosity level"
    )]
    executables: bool,
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        help = "display more project details",
        long_help = "display more project details. This option can be passed up to 5 times, each time\n\
                     will display even more details",
    )]
    verbose: u8,
    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        help = "display less project details",
        long_help = "disable displaying any extra project details. This option can be passed up to 2\n\
                     times, if passed twice and no other details are selected, the command will output\n\
                     nothing. This option overrides verbosity, but not explicit project detail options",
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

    let details = match cli.quiet {
        2.. => 0,
        1 => 1,
        _ => cli.verbose + 2,
    };

    if cli.versions {
        return Ok(println!(
            "{}",
            format_package_versions(
                &warehouse::Package::fetch(warehouse::PYPI_URI, &cli.project)?,
                details
            )
        ));
    };

    let format_fields = format::FormatFields {
        detail_level: details,
        summary: cli.summary,
        license: cli.license,
        urls: cli.urls,
        keywords: cli.keywords,
        classifiers: cli.classifiers,
        artifacts: cli.artifacts,
        dependencies: cli.dependencies,
        readme: cli.readme,
        packages: cli.packages,
        executables: cli.executables,
    };

    let package_version = pick_version(cli.project, cli.package_version)?;
    let package_distribution = cli
        .dist
        .map(|d| pick_dist(&package_version, d))
        .transpose()?;

    let formatted =
        format_package_version_details(&package_version, package_distribution, format_fields);

    /*
    Ok(println!(
        "{}",
        formatted
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("")
            */

    for div in formatted.iter() {
        print_text(&div.to_string());
    }
    Ok(())
}
