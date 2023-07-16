use crate::cli::cmds::{Commands, ItemKind, Opts};
use crate::config::Config;
use crate::db;
use crate::error::Error;
use crate::md;
use crate::Result;
use crossterm::event::Event;
use crossterm::terminal::*;
use crossterm::{cursor, event, queue};
use log::{debug, error, info, warn};
use std::{env, fs, io, io::Write, path::Path, process};
use termimad::{Area, MadView};

pub fn execute(opts: &Opts, cfg: &impl Config, db: &impl db::Database) -> Result<()> {
    if let Some(cmd) = &opts.command {
        match cmd {
            Commands::Topic { name } => {
                debug!("received topic command with name {:?}", name);
                if let Some(ref n) = name {
                    md!(
                        "created new topic **{:?}**: {:?}",
                        name,
                        db.topic(Some(n.as_str()))
                    );
                } else {
                    if let Some(t) = db.current_topic() {
                        md!("current topic is **{}**\n", t);
                    } else {
                        md!("no topic set\n");
                    }
                }
            }

            Commands::List { kind } => {
                let k = if let Some(kk) = kind {
                    kk
                } else {
                    &ItemKind::Topics {}
                };
                md!("## {:?}\n", k);
                match k {
                    ItemKind::Topics {} => {
                        for e in db.list(db::Entity::Topic).unwrap().iter() {
                            md::md!("* {}\n", e);
                        }
                    }
                    ItemKind::Notes {} => {
                        for e in db.list(db::Entity::Note).unwrap().iter() {
                            md::md!("* {}\n", e);
                        }
                    }
                }
            }

            Commands::Note { topic, amend } => {
                let editor = env::var("EDITOR")
                    .expect("EDITOR env var not set - don't know which editor to use!");
                if let Some(t) = topic {
                    db.topic(Some(&t.as_str())).expect("could not switch topic");
                }
                let mut note = cfg.git().repopath.to_path_buf();
                note.push(db.current_note());
                debug!("current note: {:?}:", note);
                let ret = process::Command::new(editor)
                    .args([note.as_os_str()])
                    .status()
                    .expect("could not launch {editor}");
                if ret.success() {
                    match db.commit(
                        &db.current_note(),
                        if let Some(a) = amend { *a } else { false },
                    ) {
                        Ok(()) => {
                            info!("committed successfully");
                        }
                        Err(e) => {
                            error!("failed to commit: {:?}", e);
                        }
                    }
                } else {
                    warn!("editing was aborted, discarding changes");
                }
            }

            Commands::Show { topic, .. } => {
                if let Some(t) = topic {
                    db.topic(Some(&t.as_str())).expect("could not switch topic");
                }
                let mut note = cfg.git().repopath.to_path_buf();
                note.push(db.current_note());
                debug!("current note: {:?}:", note);
                let _ = show_note(note.as_path());
            }

            _ => {}
        }
    }
    Ok(())
}

fn show_note(note: &Path) -> Result<()> {
    enable_raw_mode().expect("could not switch to terminal raw mode");
    let mut stdout = io::stdout();
    let mut area = Area::full_screen();
    area.pad_for_max_width(80);
    queue!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let mut view = MadView::from(
        fs::read_to_string(note)?,
        area.clone(),
        termimad::MadSkin::default(),
    );
    loop {
        view.write_on(&mut stdout)?;
        stdout.flush()?;
        match event::read()? {
            Event::Key(event::KeyEvent { code, .. }) => match code {
                event::KeyCode::Up => view.try_scroll_lines(-1),
                event::KeyCode::Down => view.try_scroll_lines(1),
                event::KeyCode::PageUp => view.try_scroll_pages(-1),
                event::KeyCode::PageDown => view.try_scroll_pages(1),
                _ => break,
            },
            Event::Resize(..) => {
                queue!(stdout, Clear(ClearType::All))?;
                view.resize(&area);
            }
            _ => {}
        }
    }
    disable_raw_mode()?;
    queue!(stdout, cursor::Show, LeaveAlternateScreen)?;
    stdout.flush()?;
    Err(Error::IoError(io::Error::new(
        io::ErrorKind::Other,
        "damnit",
    )))
}
