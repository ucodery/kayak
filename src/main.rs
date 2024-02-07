use crate::format::*;
use crate::warehouse::PackageVersion;
use clap::Parser;
use std::cmp::Ordering;

pub mod distribution;
pub mod format;
pub mod warehouse;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    package: String,

    #[arg(value_name = "VERSION")]
    package_version: Option<String>,

    #[arg(
        long,
        conflicts_with = "package_version",
        help = "list all versions of this package",
        long_help = "instead of displaying package details, list all versions available"
    )]
    versions: bool,

    #[arg(
        long,
        help = "list all wheel targets",
        long_help = "list all wheel distributions' compatibility tags. These tags can be passed back in as BDIST. Will normally suppress other package details"
    )]
    distributions: bool,

    #[arg(
        long,
        conflicts_with = "sdist",
        help = "display sdist link",
        long_help = "display the download link to this package version's sdist"
    )]
    bdist: Option<Option<String>>,
    #[arg(
        long,
        conflicts_with = "bdist",
        help = "display bdist link",
        long_help = "display the download link to this package version's wheel that matches BDIST. If BDIST is not given, the most appropriate wheel for the current platform is attempted to be chosen"
    )]
    sdist: bool,

    #[arg(
        long,
        short = 'r',
        help = "display the package's readme",
        long_help = "force the package's readme to display, otherwise requires verbosity 5 before being displayed"
    )]
    readme: bool,
    #[arg(
        long,
        short = 'd',
        help = "display the package's dependencies",
        long_help = "force the package's dependencies to display, otherwise requires verbosity 4 before being displayed"
    )]
    dependencies: bool,
    #[arg(
        long,
        short = 'a',
        help = "display the package's artifact types",
        long_help = "force the package's artifact types to display, otherwise requires verbosity 3 before being displayed"
    )]
    artifacts: bool,
    #[arg(
        long,
        short = 'c',
        help = "display the package's classifiers",
        long_help = "force the package's classifiers to display, otherwise requires verbosity 2 before being displayed"
    )]
    classifiers: bool,
    #[arg(
        long,
        short = 'k',
        help = "display the package's keywords",
        long_help = "force the package's keywords to display, otherwise requires verbosity 2 before being displayed"
    )]
    keywords: bool,
    #[arg(
        long,
        short = 'u',
        help = "display the package's URLs",
        long_help = "force the package's URLs to display, otherwise requires verbosity 1 before being displayed"
    )]
    urls: bool,
    #[arg(
        long,
        short = 'l',
        help = "display the package's license",
        long_help = "force the package's license to display, otherwise requires verbosity 1 before being displayed"
    )]
    license: bool,
    #[arg(
        long,
        short = 's',
        help = "display the package's summary",
        long_help = "force the package's summary to display. This happens by default, unless --quiet was set"
    )]
    summary: bool,
    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        help = "display more package details",
        long_help = "display more package details. This option can be passed up to 5 times, each time will display even more details",
    )]
    verbose: u8,
    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        help = "display less package details",
        long_help = "disable displaying any extra package details. This option can be passed up to 2 times, if passed twice an no other details are selected, the command will output nothing. This option overrides verbosity, but not explicit package detail options",
    )]
    quiet: u8,
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

fn main() {
    let cli = Cli::parse();

    let details = if cli.quiet > 1 {
        0
    } else if cli.quiet == 1 {
        1
    } else {
        cli.verbose + 2
    };

    if cli.versions {
        let package = warehouse::Package::fetch(warehouse::PYPI_URI, &cli.package).unwrap();
        println!("{}", format_package_versions(&package, details));
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
            format_package_version_details(
                &package_version,
                details,
                cli.summary,
                cli.license,
                cli.urls,
                cli.keywords,
                cli.classifiers,
                cli.artifacts,
                cli.dependencies,
                cli.readme
            )
        );
    }
}
