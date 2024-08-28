use std::iter;
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

fn render_menu(
    mut frame: &mut Frame,
    area: Rect,
) -> () {
    // All branches in [run] should be covered here
    let quit_text = String::from("q: quit");
    let help_text = String::from("?: help");
    let controls_text = [
        String::from("<SPACE>: new project"),
        String::from("n[N]: [no] name"),
        String::from("v[V]: [not] all versions"),
        String::from("t[T]: [no] time"),
        String::from("s[S]: [no] summary"),
        String::from("l[L]: [no] license"),
        String::from("u[N]: [no] urls"),
        String::from("k[K]: [no] keywords"),
        String::from("c[C]: [no] classifiers"),
        String::from("a[A]+: [less] artifacts"),
        String::from("d[D]: [no] dependencies"),
        String::from("r[R]+: [less] readme"),
        String::from("p[P]: [no] packages"),
        String::from("e[E]: [no] executables"),
    ];

    // anchor the quit and help commands, so they are always visable
    let [controls_area, help_area, quit_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Max(11), Constraint::Max(11)])
        .areas::<3>(area);
    let controls_constraints: Vec<Constraint> = controls_text
        .iter()
        .map(|s| Constraint::Max((s.len() + 3).try_into().unwrap()))
        .chain(
            iter::once(Constraint::Fill(1))
        )
        .collect();
    let controls_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(controls_constraints)
        .split(controls_area);

    for (c, control_text) in controls_text.into_iter().enumerate() {
        frame.render_widget(
            Paragraph::new(control_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::TOP | Borders::LEFT)),
            controls_areas[c]
        );
    }
    // connect last control to quit anchor when there is too much space
    frame.render_widget(
        Block::default().borders(Borders::TOP),
        controls_areas[controls_areas.len() - 1]
    );
    frame.render_widget(
        Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP | Borders::LEFT)),
        help_area
    );
    frame.render_widget(
        Paragraph::new(quit_text)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)),
        quit_area
    );
}

fn render_interactive_help<T: ratatui::backend::Backend>(
    terminal: &mut Terminal<T>,
    //mut frame: &mut Frame,
    //area: Rect,
) -> Result<Option<Project>> {
    Ok(None)
}

fn prompt_new_project<T: ratatui::backend::Backend>(
    terminal: &mut Terminal<T>,
    //mut frame: &mut Frame,
    //area: Rect,
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
                // error pop-up goes "above the fold"
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Max(4), Constraint::Percentage(50)])
                    .horizontal_margin(4)
                    .split(frame.area())[1],
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

pub fn run(mut project: Project, display_fields: DisplayFields) -> Result<()> {
    let mut project = project;
    let mut display_fields = display_fields;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            // anchor menu to the bottom
            let [display, dock] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Max(2)])
                .areas::<2>(frame.area());
            render(frame, display, &mut project, &display_fields);
            render_menu(frame, dock);
        });
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('?') => {
                            render_interactive_help(&mut terminal)?;
                        }
                        KeyCode::Char(' ') => {
                            if let Some(new_project) = prompt_new_project(&mut terminal)? {
                                project = new_project;
                            };
                        }
                        KeyCode::Char('n') => {
                            display_fields.name = true;
                        }
                        KeyCode::Char('N') => {
                            display_fields.name = false;
                        }
                        KeyCode::Char('v') => {
                            display_fields.versions = true;
                        }
                        KeyCode::Char('V') => {
                            display_fields.versions = false;
                        }
                        KeyCode::Char('t') => {
                            display_fields.time = true;
                        }
                        KeyCode::Char('T') => {
                            display_fields.time = false;
                        }
                        KeyCode::Char('s') => {
                            display_fields.summary = true;
                        }
                        KeyCode::Char('S') => {
                            display_fields.summary = false;
                        }
                        KeyCode::Char('l') => {
                            display_fields.license = true;
                        }
                        KeyCode::Char('L') => {
                            display_fields.license = false;
                        }
                        KeyCode::Char('u') => {
                            display_fields.urls = true;
                        }
                        KeyCode::Char('U') => {
                            display_fields.urls = false;
                        }
                        KeyCode::Char('k') => {
                            display_fields.keywords = true;
                        }
                        KeyCode::Char('K') => {
                            display_fields.keywords = false;
                        }
                        KeyCode::Char('c') => {
                            display_fields.classifiers = true;
                        }
                        KeyCode::Char('C') => {
                            display_fields.classifiers = false;
                        }
                        KeyCode::Char('a') => {
                            if display_fields.artifacts < 5 {
                                display_fields.artifacts += 1;
                            }
                        }
                        KeyCode::Char('A') => {
                            if display_fields.artifacts > 0 {
                                display_fields.artifacts -= 1;
                            }
                        }
                        KeyCode::Char('d') => {
                            display_fields.dependencies = true;
                        }
                        KeyCode::Char('D') => {
                            display_fields.dependencies = false;
                        }
                        KeyCode::Char('r') => {
                            if display_fields.readme < 3 {
                                display_fields.readme += 1;
                            }
                        }
                        KeyCode::Char('R') => {
                            if display_fields.readme > 0 {
                                display_fields.readme -= 1;
                            }
                        }
                        KeyCode::Char('p') => {
                            display_fields.packages = true;
                        }
                        KeyCode::Char('P') => {
                            display_fields.packages = false;
                        }
                        KeyCode::Char('e') => {
                            display_fields.executables = true;
                        }
                        KeyCode::Char('E') => {
                            display_fields.executables = false;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
