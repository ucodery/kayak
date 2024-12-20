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

fn encode_cli(project: &mut Project, display_fields: &DisplayFields) -> String {
    let mut cli = String::from("kayak");
    if display_fields.versions {
        cli += " versions";
        if !display_fields.name {
            cli += " -qq";
        }
    } else {
        cli += " ";
        cli += &project.package_selector();
        if let Some(version) = project.version_selector() {
            cli += " ";
            cli += &version;
        }
        if project.is_distribution_loaded() {
            cli += " ";
            if let Ok(d) = &project.distribution().unwrap().filename() {
                cli += &d.compatibility_tag.to_string();
            } else {
                cli += "sdist";
            }
        }
        if !display_fields.name {
            cli += " -qq";
        }
        if !display_fields.time && project.distribution_selector().is_some() {
            cli += " -q";
        }
        if display_fields.summary {
            cli += " --summary";
        }
        if display_fields.license {
            cli += " --license";
        }
        if display_fields.urls {
            cli += " --urls";
        }
        if display_fields.keywords {
            cli += " --keywords";
        }
        if display_fields.classifiers {
            cli += " --classifiers";
        }
        match display_fields.artifacts {
            0 => (),
            1 => cli += " --artifacts",
            _ => {
                cli += " -";
                cli += &"a".repeat(display_fields.artifacts.into());
            }
        }
        if display_fields.dependencies {
            cli += " --dependencies";
        }
        match display_fields.readme {
            0 => (),
            1 => cli += " --readme",
            _ => {
                cli += " -";
                cli += &"r".repeat(display_fields.readme.into());
            }
        }
        if display_fields.packages {
            cli += " --packages";
        }
        if display_fields.executables {
            cli += " --executables";
        }
    }
    cli
}

fn render_popup(frame: &mut Frame, area: Rect, message: String, is_error: bool) {
    // info pop-up goes "above the fold", error pop-up goes "below the fold"
    let constraints = if is_error {
        [
            Constraint::Percentage(50),
            Constraint::Max(4),
            Constraint::Fill(1),
        ]
    } else {
        [
            Constraint::Fill(1),
            Constraint::Max(4),
            Constraint::Percentage(50),
        ]
    };
    let block = if is_error {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::Red)
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Color::Blue)
    };
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .horizontal_margin(4)
        .split(area)[1];

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

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
        // session commands
        // TODO: CTRL-C
        // TODO: ?
        [
            String::from("print"),
            // issue#3 String::from("preview: CTRL-P exit: CTRL-SHIFT-P"),
            String::from("CTRL-p"),
            String::from("display the `kayak` command that will recreate the currently displayed project information. \
                         the `--format` is explicitly left out"),
                         // issue#3 previewing the print will maintain the current interactive session, while exiting will clear \
                         // issue#3 the screen and show only the command. \
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

fn render_no_commands_menu(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(String::from("PRESS ANY KEY TO CLOSE"))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)),
        area,
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

fn render_messages(
    frame: &mut Frame,
    area: Rect,
    project: &mut Option<Project>,
    display_fields: &DisplayFields,
    messages: &Messages,
) {
    // floating boxes are rendered over the main display; if render is not called, the main display will disappear
    if let Some(prj) = project {
        // render should have already been tried before trying to render messages, the bigger goal here is to render the popups
        let _ = render(frame, area, prj, display_fields);
    }
    match messages {
        Messages::Info(msg) => render_popup(frame, area, msg.to_string(), false),
        Messages::Error(err) => render_popup(frame, area, err.to_string(), true),
        Messages::InfoError((msg, err)) => {
            render_popup(frame, area, msg.to_string(), false);
            render_popup(frame, area, err.to_string(), true);
        }
    }
}

enum Messages {
    Info(String),
    Error(String),
    InfoError((String, String)),
}

enum DisplayMode {
    Help,
    Info(Messages),
    Input(Messages),
    Normal,
}

pub fn run(project: Option<Project>, display_fields: DisplayFields) -> Result<()> {
    let mut project = project;
    let mut project_loads = false;
    let mut last_good_project: Option<Project> = None;
    let mut display_fields = display_fields;
    let mut mode = if project.is_some() {
        DisplayMode::Normal
    } else {
        DisplayMode::Input(Messages::Info(String::new()))
    };

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

            match &mode {
                DisplayMode::Help => {
                    render_interactive_help(frame, display);
                    render_no_commands_menu(frame, dock);
                }
                DisplayMode::Info(info) => {
                    render_messages(frame, display, &mut project, &display_fields, info);
                    render_no_commands_menu(frame, dock);
                }
                DisplayMode::Input(input) => {
                    render_messages(frame, display, &mut project, &display_fields, input);
                    render_new_project_prompt_menu(frame, dock);
                }
                DisplayMode::Normal => {
                    let prj = &mut project
                        .as_mut()
                        .expect("only attempt to render project after a selection has been made");
                    match render(frame, display, prj, &display_fields) {
                        Ok(()) => {
                            project_loads = true;
                        }
                        Err(err) => {
                            project = last_good_project.take();
                            mode = DisplayMode::Info(Messages::Error(err.to_string()));
                        }
                    }
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
                    match &mut mode {
                        DisplayMode::Help => {
                            mode = if project.is_some() {
                                DisplayMode::Normal
                            } else {
                                DisplayMode::Input(Messages::Info(String::new()))
                            };
                        }
                        DisplayMode::Info(_) => {
                            mode = if project.is_some() {
                                DisplayMode::Normal
                            } else {
                                DisplayMode::Input(Messages::Info(String::new()))
                            };
                        }
                        DisplayMode::Input(user_progress) => {
                            match key.code {
                                KeyCode::Char(key_char) => match user_progress {
                                    Messages::Info(user_input)
                                    | Messages::InfoError((user_input, _)) => {
                                        user_input.push(key_char)
                                    }
                                    Messages::Error(user_error) => {
                                        mode = DisplayMode::Input(Messages::InfoError((
                                            key_char.to_string(),
                                            user_error.to_string(),
                                        )))
                                    }
                                },
                                KeyCode::Backspace => {
                                    match user_progress {
                                        Messages::Info(user_input)
                                        | Messages::InfoError((user_input, _)) => {
                                            user_input.pop();
                                        }
                                        Messages::Error(_) => (), // backspace with no current input
                                    }
                                }
                                KeyCode::Enter => {
                                    mode = match user_progress {
                                        Messages::Info(user_input)
                                        | Messages::InfoError((user_input, _)) => {
                                            let mut requested_project =
                                                user_input.split_whitespace();
                                            if let Some(name) = requested_project.next() {
                                                let version =
                                                    requested_project.next().map(str::to_string);
                                                let distribution =
                                                    requested_project.next().map(str::to_string);
                                                if project_loads {
                                                    last_good_project = project;
                                                }
                                                project = Some(Project::new(
                                                    name.to_string(),
                                                    version,
                                                    distribution,
                                                ));
                                                DisplayMode::Normal
                                            } else {
                                                DisplayMode::Input(Messages::InfoError((
                                                    user_input.to_string(),
                                                    String::from(
                                                        "please enter the name of a package",
                                                    ),
                                                )))
                                            }
                                        }
                                        Messages::Error(_) => DisplayMode::Input(Messages::Error(
                                            String::from("please enter the name of a package"),
                                        )),
                                    };
                                }
                                KeyCode::Esc => {
                                    if project.is_some() {
                                        mode = DisplayMode::Normal;
                                    } else {
                                        mode = match user_progress {
                                            Messages::Info(user_input)
                                            | Messages::InfoError((user_input, _)) => {
                                                DisplayMode::Input(Messages::InfoError((
                                                    user_input.to_string(),
                                                    String::from(
                                                        "please enter the name of a package",
                                                    ),
                                                )))
                                            }
                                            Messages::Error(_) => {
                                                DisplayMode::Input(Messages::Error(String::from(
                                                    "please enter the name of a package",
                                                )))
                                            }
                                        };
                                    }
                                }
                                _ => (),
                            }
                        }
                        DisplayMode::Normal => match key.code {
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Char('?') => {
                                mode = DisplayMode::Help;
                            }
                            KeyCode::Char(' ') => {
                                mode = DisplayMode::Input(Messages::Info(String::new()));
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
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    mode = DisplayMode::Info(Messages::Info(encode_cli(
                                        &mut project.as_mut().expect(
                                            "normal mode should alway have a project loaded",
                                        ),
                                        &display_fields,
                                    )));
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        break;
                                    }
                                } else {
                                    display_fields.packages = true;
                                }
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
                        },
                    }
                }
            }
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
