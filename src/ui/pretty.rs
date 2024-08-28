use crate::ui::*;
use crate::warehouse::{DistributionUrl, PackageVersion};
use crate::{DisplayFields, Project};
use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use ratatui::layout::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::TerminalOptions;
use ratatui::Viewport;
use std::io::stdout;
use std::iter;

fn render_name_versions<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    let package = project.package()?;
    let mut versions = package
        .ordered_versions()
        .iter()
        .map(|v| v.normalize())
        .collect::<Vec<_>>();
    versions.reverse();

    if display_fields.name {
        Ok(Some((
            Constraint::Min(2),
            Paragraph::new(vec![
                Line::from(Span::styled(
                    package.name.to_string(),
                    Style::new().bold().reversed(),
                ))
                .centered(),
                Line::from(versions.join(", ")),
            ])
            .wrap(Wrap { trim: false }),
        )))
    } else {
        Ok(Some((
            Constraint::Min(1),
            Paragraph::new(Line::from(versions.join(", "))).wrap(Wrap { trim: false }),
        )))
    }
}

fn render_name_version<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.name {
        return Ok(None);
    }
    let version = project.version()?;
    let name = Line::from(Span::styled(
        version.name.to_string(),
        Style::new().bold().reversed(),
    ));
    let ver = if let Some(_reason) = &version.yanked_reason {
        Line::from(Span::styled(
            format!("{} [YANKED]", version.version),
            Style::new().bold().white().on_red(),
        ))
    } else {
        Line::from(Span::styled(
            version.version.to_string(),
            Style::new().bold().reversed(),
        ))
    };

    Ok(Some((
        Constraint::Length(2),
        Paragraph::new(vec![name, ver]).centered(),
    )))
}

fn render_distribution<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.name || !project.distribution_was_selected() {
        Ok(None)
    } else {
        let dist = if let Ok(d) = project.distribution()?.filename() {
            d.compatibility_tag.to_string()
        } else {
            "sdist".to_string()
        };
        Ok(Some((
            Constraint::Length(1),
            Paragraph::new(Line::from(Span::styled(
                dist,
                Style::new().bold().reversed(),
            )))
            .centered(),
        )))
    }
}

fn render_time<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.time {
        Ok(None)
    } else if project.distribution_was_selected() {
        Ok(Some((
            Constraint::Length(1),
            Paragraph::new(Line::from(Span::styled(
                project.distribution()?.upload_time.clone(),
                Style::new().bold().reversed(),
            )))
            .centered(),
        )))
    } else {
        // cannot use project.distribution() as this should report the earliest upload time, not
        // the time of the best-fit distribution
        if let Some(time) = project
            .version()?
            .urls
            .iter()
            .filter_map(|u| u.upload_time_iso_8601.parse::<DateTime<Utc>>().ok())
            .min()
        {
            Ok(Some((
                Constraint::Length(1),
                Paragraph::new(Line::from(Span::styled(
                    time.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    Style::new().bold().reversed(),
                )))
                .centered(),
            )))
        } else {
            Ok(None)
        }
    }
}

fn render_license_copyright<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.license {
        return Ok(None);
    }
    let constraint = Constraint::Length(3);
    let license = project
        .version()?
        .license
        .as_ref()
        .map(|license| Span::raw(String::from(license)));
    let author = project
        .version()?
        .author_email
        .as_ref()
        .map(|author_email| Span::raw(format!(" © {}", author_email.replace('"', ""))));

    match (license, author) {
        (None, None) => Ok(None),
        (None, Some(paragraph)) | (Some(paragraph), None) => Ok(Some((
            constraint,
            Paragraph::new(paragraph)
                .block(Block::default().borders(Borders::ALL))
                .centered(),
        ))),
        (Some(license), Some(author)) => Ok(Some((
            constraint,
            Paragraph::new(Line::from(vec![license, author]))
                .block(Block::default().borders(Borders::ALL))
                .centered(),
        ))),
    }
}

fn render_summary<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.summary {
        Ok(None)
    } else {
        Ok(project.version()?.summary.as_ref().map(|summary| {
            (
                Constraint::Length(3),
                Paragraph::new(Line::from(String::from(summary)))
                    .centered()
                    .block(Block::default().borders(Borders::ALL)),
            )
        }))
    }
}

fn render_urls<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.urls {
        return Ok(None);
    }
    let version: &PackageVersion = project.version()?;
    let size = version.project_urls.len() + 3; // plus project URL plus box
    Ok(Some((
        Constraint::Max(size.try_into().unwrap()),
        Paragraph::new(
            iter::once((&"Package Index".to_string(), &version.project_url))
                .chain(version.project_urls.iter())
                .map(|url| {
                    Line::from(vec![
                        iconify_url(url).into(),
                        "  ".into(),
                        Span::styled(
                            url.1.to_string(),
                            Style::new().blue().add_modifier(Modifier::UNDERLINED),
                        ),
                    ])
                })
                .collect::<Vec<_>>(),
        )
        .block(Block::default().title("Links").borders(Borders::ALL)),
    )))
}

fn render_keywords<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.keywords {
        return Ok(None);
    }
    let keywords = project.version()?.keywords();
    if !keywords.is_empty() {
        Ok(Some((
            Constraint::Length(3),
            Paragraph::new(Line::from(keywords.join(", ")))
                .block(Block::default().title("Keywords").borders(Borders::ALL)),
        )))
    } else {
        Ok(None)
    }
}

fn render_classifiers<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.classifiers {
        return Ok(None);
    }
    let classifiers = project.version()?.classifiers();
    if !classifiers.is_empty() {
        let size = classifiers.len() + 2;
        Ok(Some((
            Constraint::Max(size.try_into().unwrap()),
            Paragraph::new(
                classifiers
                    .iter()
                    .map(|c| Line::from(c.to_string()))
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().title("Classifiers").borders(Borders::ALL)),
        )))
    } else {
        Ok(None)
    }
}

fn render_artifacts<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if display_fields.artifacts == 0 {
        return Ok(None);
    }
    let artifacts: Box<dyn Iterator<Item = &DistributionUrl>> =
        if project.distribution_was_selected() {
            Box::new(iter::once(project.distribution()?))
        } else {
            Box::new(project.version()?.urls.iter())
        };

    let mut render: Option<(Constraint, Paragraph)> = None;
    if display_fields.artifacts == 1 {
        let line = summarize_artifacts(artifacts);
        if !line.is_empty() {
            render = Some((
                Constraint::Length(3),
                Paragraph::new(Line::from(line.to_string())).block(
                    Block::default()
                        .title("Distribution Types")
                        .borders(Borders::ALL),
                ),
            ));
        }
    } else {
        let lines = artifacts
            .filter_map(|artifact| {
                let tag = if let Ok(dist) = artifact.filename() {
                    Span::raw(dist.compatibility_tag.to_string())
                } else if artifact.packagetype == "sdist" {
                    Span::raw(String::from("sdist"))
                } else {
                    return None;
                };

                if display_fields.artifacts > 3 {
                    Some(Line::from(vec![
                        tag,
                        " ".into(),
                        artifact.upload_time.clone().into(),
                        " ".into(),
                        Span::styled(
                            artifact.url.clone(),
                            Style::new().blue().add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                } else if display_fields.artifacts == 3 {
                    Some(Line::from(vec![
                        tag,
                        " ".into(),
                        Span::styled(
                            artifact.url.clone(),
                            Style::new().blue().add_modifier(Modifier::UNDERLINED),
                        ),
                    ]))
                } else {
                    Some(Line::from(tag))
                }
            })
            .collect::<Vec<_>>();
        if !lines.is_empty() {
            render = Some((
                // TODO: *2 and trim:false allows long url to wrap to the next line, but leaves
                // excess space when not needed
                //Constraint::Max((lines.len() * 2 + 2).try_into().unwrap()),
                Constraint::Max((lines.len() + 2).try_into().unwrap()),
                Paragraph::new(lines)
                    .block(Block::default().title("Downloads").borders(Borders::ALL)), //.wrap(Wrap { trim: false }),
            ));
        }
    }

    Ok(render)
}

fn render_dependencies<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.dependencies {
        return Ok(None);
    }
    let dependencies = project
        .version()?
        .requires_python
        .clone()
        .into_iter()
        .map(|p| format!("python{p}"))
        .chain(
            project
                .version()?
                .requires_dist
                .iter()
                .map(|d| d.to_string()),
        )
        .map(Line::from)
        .collect::<Vec<_>>();
    if !dependencies.is_empty() {
        Ok(Some((
            Constraint::Max(dependencies.len().try_into().unwrap()),
            Paragraph::new(dependencies)
                .block(Block::default().title("Dependencies").borders(Borders::ALL))
                .wrap(Wrap { trim: false }),
        )))
    } else {
        Ok(None)
    }
}

fn render_packages<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.packages {
        return Ok(None);
    }
    let mut packages = project
        .import_package()?
        .provides_packages()
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>();
    if !packages.is_empty() {
        packages.sort_unstable();
        Ok(Some((
            Constraint::Length(3),
            Paragraph::new(Line::from(packages.join(", ")))
                .block(
                    Block::default()
                        .title("Importable Packages")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false }),
        )))
    } else {
        Ok(None)
    }
}

fn render_executables<'a>(
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if !display_fields.executables {
        return Ok(None);
    }
    let package = project.import_package()?;
    let executables = package
        .provides_executables()
        .iter()
        .chain(package.console_scripts().iter())
        .map(|p| p.to_string())
        .collect::<Vec<_>>();
    if !executables.is_empty() {
        Ok(Some((
            Constraint::Length(3),
            Paragraph::new(Line::from(executables.join(", ")))
                .block(
                    Block::default()
                        .title("Executable Commands")
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false }),
        )))
    } else {
        Ok(None)
    }
}

fn render_readme<'a>(
    // TODO: cannot render md within ratatui as escape codes don't work
    display_fields: &DisplayFields,
    project: &mut Project,
) -> Result<Option<(Constraint, Paragraph<'a>)>> {
    if display_fields.readme == 0 {
        return Ok(None);
    }
    if let Some(readme) = &project.version()?.description {
        return Ok(Some((
            Constraint::Fill(1),
            Paragraph::new(readme.to_string()).wrap(Wrap { trim: false }),
        )));
    }
    Ok(None)
}

fn render_recoverable_error(frame: &mut Frame, area: Rect, error: Error) {
    frame.render_widget(
        Paragraph::new(error.to_string())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::Red),
            ),
        // error pop-up goes "below the fold"
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Max(4),
                Constraint::Fill(1),
            ])
            .horizontal_margin(4)
            .split(area)[1],
    );
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    project: &mut Project,
    display_fields: &DisplayFields,
) {
    let mut constraints = Vec::new();
    let mut components = Vec::new();

    if display_fields.versions {
        match render_name_versions(display_fields, project) {
            Ok(Some((constraint, component))) => {
                constraints.push(constraint);
                components.push(component);
            }
            Ok(None) => (),
            Err(error) => {
                render_recoverable_error(frame, area, error);
            }
        };
    } else {
        for render_field in [
            render_name_version,
            render_distribution,
            render_time,
            render_license_copyright,
            render_summary,
            render_urls,
            render_keywords,
            render_classifiers,
            render_artifacts,
            render_dependencies,
            render_packages,
            render_executables,
            render_readme,
        ] {
            match render_field(display_fields, project) {
                Ok(Some((constraint, component))) => {
                    constraints.push(constraint);
                    components.push(component);
                }
                Ok(None) => (),
                Err(error) => {
                    render_recoverable_error(frame, area, error);
                }
            };
        }
    }

    let page = Layout::new(Direction::Vertical, constraints)
        .flex(Flex::Start)
        .split(area);
    for (p, component) in components.iter().enumerate() {
        frame.render_widget(component, page[p]);
    }
}

pub fn display(mut project: Project, display_fields: DisplayFields) -> Result<()> {
    let backend = CrosstermBackend::new(stdout());
    let options = TerminalOptions {
        viewport: Viewport::Inline(backend.size()?.height),
    };
    let mut terminal = Terminal::with_options(backend, options)?;
    terminal.draw(|frame| {
        render(frame, frame.area(), &mut project, &display_fields);
    })?;
    println!();
    Ok(())
}
