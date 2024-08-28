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
use std::iter;

fn render_menu(frame: &mut Frame, area: Rect) {
    // anchor the quit and help commands, so they are always visable
    let [controls_area, help_area, quit_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Max(11),
            Constraint::Max(11),
        ])
        .areas::<3>(area);

    // All branches in [run] should be covered here
    let quit_content = Paragraph::new(String::from("q: quit"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    let help_content = Paragraph::new(String::from("?: help"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT));
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
    let controls_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            controls_text
                .iter()
                .map(|s| Constraint::Max((s.len() + 3).try_into().unwrap()))
                .chain(iter::once(Constraint::Fill(1)))
                .collect::<Vec<Constraint>>(),
        )
        .split(controls_area);

    for (c, control_text) in controls_text.into_iter().enumerate() {
        frame.render_widget(
            Paragraph::new(control_text)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::TOP | Borders::LEFT)),
            controls_areas[c],
        );
    }
    // connect last control to quit anchor when there is too much space
    frame.render_widget(
        Block::default().borders(Borders::TOP),
        controls_areas[controls_areas.len() - 1],
    );
    frame.render_widget(help_content, help_area);
    frame.render_widget(quit_content, quit_area);
}

fn render_interactive_help(frame: &mut Frame, area: Rect) {
    let controls_text = [
        [
            String::from("name"),
            String::from("on: n off: N"),
            String::from("display the name and version of the currenly loaded project"),
        ],
        [
            String::from("versions"),
            String::from("on: v off: V"),
            String::from("instead of displaying project details, list all versions available"),
        ],
        [
            String::from("time"),
            String::from("on: t off: T"),
            String::from("display the project's release timestamp"),
        ],
        [
            String::from("summary"),
            String::from("on: s off: S"),
            String::from("display the project's summary"),
        ],
        [
            String::from("license"),
            String::from("on: l off: L"),
            String::from("display the project's license and copyright"),
        ],
        [
            String::from("urls"),
            String::from("on: u off: U"),
            String::from("display the project's URLs"),
        ],
        [
            String::from("keywords"),
            String::from("on: k off: K"),
            String::from("display the project's keywords"),
        ],
        [
            String::from("classifiers"),
            String::from("on: c off: C"),
            String::from("display the project's classifiers"),
        ],
        [
            String::from("artifacts"),
            String::from("more: a less: A"),
            String::from("display the project's distribution artifacts;  \
                          initially a summary of artifact flavors is displayed;  \
                          with more details, all artifacts are displayed with their target platform;  \
                          with even more deails, links to file downloads are displayed;  \
                          with the most deails, the timestamp of each file upload is displayed"),
        ],
        [
            String::from("dependencies"),
            String::from("on: d off: D"),
            String::from("display the project's dependencies on other projects"),
        ],
        [
            String::from("readme"),
            String::from("more: r less: R"),
            String::from("display the project's README;  \
                          initially the raw text is displayed;  \
                          with more details, if the readme is of a known MIME type, it will be styled before displaying"),
        ],
        [
            String::from("packages"),
            String::from("on: p off: P"),
            String::from("display the project's importable top-level names"),
        ],
        [
            String::from("executables"),
            String::from("on: e off: E"),
            String::from("display the project's executable file names"),
        ],
    ];

    let controls_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Fill(1); controls_text.len()])
        .split(area);

    for (c, control_text) in controls_text.into_iter().enumerate() {
        let control_sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(16), Constraint::Max(9), Constraint::Fill(1)])
            .split(controls_areas[c]);
        for (s, segment) in control_text.into_iter().enumerate() {
            frame.render_widget(
                Paragraph::new(segment)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .block(Block::default().borders(Borders::ALL)),
                control_sections[s],
            );
        }
    }
}

fn render_interactive_help_menu(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(String::from("PRESS ANY KEY TO EXIT"))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)),
        area,
    );
}

fn render_new_project_prompt(frame: &mut Frame, area: Rect, current_input: String) {
    // input pop-up goes "above the fold"
    let prompt_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Max(4),
            Constraint::Percentage(50),
        ])
        .horizontal_margin(4)
        .split(area)[1];
    frame.render_widget(Clear, prompt_area);
    frame.render_widget(
        Paragraph::new(current_input)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Color::Blue),
            ),
        prompt_area,
    );
}

fn render_new_project_prompt_menu(frame: &mut Frame, area: Rect) {
    // anchor the quit and enter commands, so they are always visable
    let [enter_area, usage_area, quit_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Max(31),
            Constraint::Fill(1),
            Constraint::Max(24),
        ])
        .areas::<3>(area);

    let enter_content = Paragraph::new(String::from("<ENTER>: lookup new project"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    let usage_content = Paragraph::new(String::from("project_name [version] [distribution]"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    let quit_content = Paragraph::new(String::from("<ESC>: cancel lookup"))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));

    frame.render_widget(enter_content, enter_area);
    frame.render_widget(usage_content, usage_area);
    frame.render_widget(quit_content, quit_area);
}

pub fn run(project: Project, display_fields: DisplayFields) -> Result<()> {
    let mut project = project;
    let mut display_fields = display_fields;
    let mut displaying_help = false;
    let mut gathering_user_input = false;
    let mut user_input = String::new();

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
            if displaying_help {
                render_interactive_help(frame, display);
                render_interactive_help_menu(frame, dock);
            } else {
                render(frame, display, &mut project, &display_fields);
                if gathering_user_input {
                    render_new_project_prompt(frame, display, user_input.clone());
                    render_new_project_prompt_menu(frame, dock);
                } else {
                    render_menu(frame, dock);
                }
            }
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // CTRL-C always quits, check first
                    if let KeyCode::Char('c') = key.code {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            break;
                        }
                    }
                    if displaying_help {
                        displaying_help = false;
                    } else if gathering_user_input {
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
                                    project = Project::new(name.to_string(), version, distribution);
                                }
                                // else no string provided, continue to prompt
                                user_input.clear();
                                gathering_user_input = false;
                            }
                            KeyCode::Esc => {
                                gathering_user_input = false;
                            }
                            _ => (),
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char('?') => {
                                displaying_help = true;
                            }
                            KeyCode::Char(' ') => {
                                gathering_user_input = true;
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
                                if display_fields.artifacts < 4 {
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
                                if display_fields.readme < 2 {
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
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
