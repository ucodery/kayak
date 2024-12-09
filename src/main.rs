#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]

use crate::picker::Project;
use crate::ui::{interactive, pretty, text};
use anyhow::Result;
use clap::{Parser, ValueEnum};
use pep440::Version;

pub mod distribution;
pub mod package_inspect;
pub mod picker;
pub mod ui;
pub mod warehouse;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(
        long_help = "the name of the python project to look up",
        // what I want: required_unless_present_and_eq_all([("format", "interactive")])
        required_unless_present_all = ["format"],
        required_if_eq_any = [
            ("format", "text"),
            ("format", "pretty"),
        ]
    )]
    project: Option<String>,
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
                     before being displayed. This option can be passed up to 4 times, each time will\n\
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

    #[arg(
        long,
        value_enum,
        default_value_t=Format::Pretty,
        help = "output format to display project key-data",
        long_help = "select the output format:\n\
                     pretty: write key-data using tables and colors directly to stdout\n\
                     interactive: write key-data using tables and colors to an alternate screen.\n\
                     \t\tthis mode can accept further command to update the display interactively",
    )]
    format: Format,
}

#[derive(ValueEnum, Debug, Clone)]
enum Format {
    //Plain,
    Text,
    Pretty,
    Interactive,
    //Json,
}

#[derive(Debug)]
pub struct DisplayFields {
    pub name: bool,
    pub versions: bool,
    pub time: bool,
    pub summary: bool,
    pub license: bool,
    pub urls: bool,
    pub keywords: bool,
    pub classifiers: bool,
    pub artifacts: u8,
    pub dependencies: bool,
    pub readme: u8,
    pub packages: bool,
    pub executables: bool,
}

fn main() -> Result<()> {
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

    // quiet and verbosity are quick ways to turn on/off output
    // map them to real fields here
    let display_fields = DisplayFields {
        name: cli.quiet < 2,
        versions: cli.versions,
        time: cli.dist.is_some() || (cli.verbose >= 1 && cli.quiet < 1),
        summary: cli.quiet < 1 || cli.summary,
        license: cli.verbose >= 1 && cli.quiet < 1 || cli.license,
        urls: cli.verbose >= 1 && cli.quiet < 1 || cli.urls,
        keywords: cli.verbose >= 2 && cli.quiet < 1 || cli.keywords,
        classifiers: cli.verbose >= 2 && cli.quiet < 1 || cli.classifiers,
        artifacts: if cli.artifacts > 0 {
            cli.artifacts
        } else if cli.verbose >= 3 && cli.quiet < 1 {
            1
        } else {
            0
        },
        dependencies: cli.verbose >= 4 && cli.quiet < 1 || cli.dependencies,
        readme: if cli.readme > 0 {
            cli.readme
        } else if cli.verbose >= 5 && cli.quiet < 1 {
            1
        } else {
            0
        },
        packages: cli.packages,
        executables: cli.executables,
    };

    let project = cli.project.map(|p| Project::new(p, cli.package_version, cli.dist));

    match cli.format {
        Format::Text => text::display(project.expect("a project is requred to output text"), display_fields)?,
        Format::Pretty => pretty::display(project.expect("a project is requred to pretty print text"), display_fields)?,
        Format::Interactive => interactive::run(project, display_fields)?,
    };

    Ok(())
}
