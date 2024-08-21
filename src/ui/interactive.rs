use crate::ui::pretty::render;
use crate::{DisplayFields, Project};
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::layout::*;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::stdout;

enum UserInput {
    Quit,
    DoNothing,
    NewProject,
    NameOn,
    NameOff,
    VersionsOn,
    VersionsOff,
    TimeOn,
    TimeOff,
    SummaryOn,
    SummaryOff,
    LicenseOn,
    LicenseOff,
    UrlsOn,
    UrlsOff,
    KeywordsOn,
    KeywordsOff,
    ClassifiersOn,
    ClassifiersOff,
    ArtifactsOn,
    ArtifactsOff,
    DependenciesOn,
    DependenciesOff,
    ReadmeOn,
    ReadmeOff,
    PackagesOn,
    PackagesOff,
    ExecutablesOn,
    ExecutablesOff,
}

fn prompt_new_project<T: ratatui::backend::Backend>(
    terminal: &mut Terminal<T>,
) -> Result<Option<Project>> {
    let mut user_input = String::new();
    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new(user_input.clone())
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Color::Blue),
                    ),
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Max(3)])
                    .flex(Flex::Center)
                    .horizontal_margin(4)
                    .areas::<1>(frame.size())[0],
            );
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(key_char) => {
                            user_input.push(key_char);
                        }
                        KeyCode::Backspace => {
                            user_input.pop();
                        }
                        KeyCode::Enter => {
                            let mut requested_project = user_input.split_whitespace();
                            if let Some(name) = requested_project.next() {
                                let version = requested_project.next().map(str::to_string);
                                let distribution = requested_project.next().map(str::to_string);
                                return Ok(Some(Project::new(
                                    name.to_string(),
                                    version,
                                    distribution,
                                )));
                            } else {
                                // no string provided, continue to prompt
                                return Ok(None);
                            }
                        }
                        KeyCode::Esc => return Ok(None),
                        _ => (),
                    }
                }
            }
        }
    }
}

fn react() -> Result<UserInput> {
    if event::poll(std::time::Duration::from_millis(16))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(UserInput::Quit);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(UserInput::Quit);
                    }
                    KeyCode::Char(' ') => {
                        return Ok(UserInput::NewProject);
                    }
                    KeyCode::Char('n') => {
                        return Ok(UserInput::NameOn);
                    }
                    KeyCode::Char('N') => {
                        return Ok(UserInput::NameOff);
                    }
                    KeyCode::Char('v') => {
                        return Ok(UserInput::VersionsOn);
                    }
                    KeyCode::Char('V') => {
                        return Ok(UserInput::VersionsOff);
                    }
                    KeyCode::Char('t') => {
                        return Ok(UserInput::TimeOn);
                    }
                    KeyCode::Char('T') => {
                        return Ok(UserInput::TimeOff);
                    }
                    KeyCode::Char('s') => {
                        return Ok(UserInput::SummaryOn);
                    }
                    KeyCode::Char('S') => {
                        return Ok(UserInput::SummaryOff);
                    }
                    KeyCode::Char('l') => {
                        return Ok(UserInput::LicenseOn);
                    }
                    KeyCode::Char('L') => {
                        return Ok(UserInput::LicenseOff);
                    }
                    KeyCode::Char('u') => {
                        return Ok(UserInput::UrlsOn);
                    }
                    KeyCode::Char('U') => {
                        return Ok(UserInput::UrlsOff);
                    }
                    KeyCode::Char('k') => {
                        return Ok(UserInput::KeywordsOn);
                    }
                    KeyCode::Char('K') => {
                        return Ok(UserInput::KeywordsOff);
                    }
                    KeyCode::Char('c') => {
                        return Ok(UserInput::ClassifiersOn);
                    }
                    KeyCode::Char('C') => {
                        return Ok(UserInput::ClassifiersOff);
                    }
                    KeyCode::Char('a') => {
                        return Ok(UserInput::ArtifactsOn);
                    }
                    KeyCode::Char('A') => {
                        return Ok(UserInput::ArtifactsOff);
                    }
                    KeyCode::Char('d') => {
                        return Ok(UserInput::DependenciesOn);
                    }
                    KeyCode::Char('D') => {
                        return Ok(UserInput::DependenciesOff);
                    }
                    KeyCode::Char('r') => {
                        return Ok(UserInput::ReadmeOn);
                    }
                    KeyCode::Char('R') => {
                        return Ok(UserInput::ReadmeOff);
                    }
                    KeyCode::Char('p') => {
                        return Ok(UserInput::PackagesOn);
                    }
                    KeyCode::Char('P') => {
                        return Ok(UserInput::PackagesOff);
                    }
                    KeyCode::Char('e') => {
                        return Ok(UserInput::ExecutablesOn);
                    }
                    KeyCode::Char('E') => {
                        return Ok(UserInput::ExecutablesOff);
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(UserInput::DoNothing)
}

pub fn run(mut project: Project, display_fields: DisplayFields) -> Result<()> {
    let mut project = project;
    let mut display_fields = display_fields;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    loop {
        render(&mut terminal, &mut project, &display_fields)?;
        match react()? {
            UserInput::Quit => break,
            UserInput::NewProject => {
                if let Some(new_project) = prompt_new_project(&mut terminal)? {
                    project = new_project;
                };
            }
            UserInput::NameOn => {
                display_fields.name = true;
            }
            UserInput::NameOff => {
                display_fields.name = false;
            }
            UserInput::VersionsOn => {
                display_fields.versions = true;
            }
            UserInput::VersionsOff => {
                display_fields.versions = false;
            }
            UserInput::TimeOn => {
                display_fields.time = true;
            }
            UserInput::TimeOff => {
                display_fields.time = false;
            }
            UserInput::SummaryOn => {
                display_fields.summary = true;
            }
            UserInput::SummaryOff => {
                display_fields.summary = false;
            }
            UserInput::LicenseOn => {
                display_fields.license = true;
            }
            UserInput::LicenseOff => {
                display_fields.license = false;
            }
            UserInput::UrlsOn => {
                display_fields.urls = true;
            }
            UserInput::UrlsOff => {
                display_fields.urls = false;
            }
            UserInput::KeywordsOn => {
                display_fields.keywords = true;
            }
            UserInput::KeywordsOff => {
                display_fields.keywords = false;
            }
            UserInput::ClassifiersOn => {
                display_fields.classifiers = true;
            }
            UserInput::ClassifiersOff => {
                display_fields.classifiers = false;
            }
            UserInput::ArtifactsOn => {
                if display_fields.artifacts < 5 {
                    display_fields.artifacts += 1;
                }
            }
            UserInput::ArtifactsOff => {
                if display_fields.artifacts >= 0 {
                    display_fields.artifacts -= 1;
                }
            }
            UserInput::DependenciesOn => {
                display_fields.dependencies = true;
            }
            UserInput::DependenciesOff => {
                display_fields.dependencies = false;
            }
            UserInput::ReadmeOn => {
                if display_fields.readme < 3 {
                    display_fields.readme += 1;
                }
            }
            UserInput::ReadmeOff => {
                if display_fields.readme >= 0 {
                    display_fields.readme -= 1;
                }
            }
            UserInput::PackagesOn => {
                display_fields.packages = true;
            }
            UserInput::PackagesOff => {
                display_fields.packages = false;
            }
            UserInput::ExecutablesOn => {
                display_fields.executables = true;
            }
            UserInput::ExecutablesOff => {
                display_fields.executables = false;
            }
            UserInput::DoNothing => (),
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
