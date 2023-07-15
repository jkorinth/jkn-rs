use clap::*;
use env_logger;
use jkn::*;
use jkn::config::Config;
use jkn::config::ConfigImpl;
use log::*;
use std::env;
use std::process;

#[derive(Parser)]
#[command(version)]
struct Opts {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Shows current topic.
    #[clap(alias = "t")]
    Topic {
        /// if set, switches current topic
        name: Option<String>,
    },

    /// Lists all elements of a kind.
    #[clap(alias = "l")]
    List {
        #[command(subcommand)]
        kind: Option<ItemKind>,
    },

    /// Shows journal of current topic.
    #[clap(alias = "j")]
    Journal {
        /// select topic
        #[arg(short, long)]
        topic: Option<String>,
        /// show only date and summary of each entry
        #[arg(short, long)]
        brief: bool,
    },

    /// Take a note in the current topic
    #[clap(alias = "n")]
    Note {
        /// select topic
        #[arg(short, long)]
        topic: Option<String>,
    },
}

impl Commands {
    pub fn exec(&self, cfg: &Box<dyn Config>, db: &mut db::Database) {
        match self {
            Commands::Topic { name } => {
                debug!("received topic command with name {:?}", name);
                if let Some(n) = name {
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
                md!("## {:?}\n", &kind.as_ref().unwrap_or(&ItemKind::Topics {}));
                for e in db.list(db::Entity::Topic).unwrap().iter() {
                    md::md!("* {}\n", e);
                }
            }

            Commands::Note { topic } => {
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
                    match db.commit(&db.current_note()) {
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

            _ => {}
        }
    }
}

#[derive(Debug, Subcommand)]
enum ItemKind {
    #[clap(alias = "t")]
    Topics {},
}

fn main() {
    env_logger::init();
    let opts = Opts::parse();
    let cfg = ConfigImpl::load().expect("could not load configuration");
    cfg.save().expect("failed to save config");
    let mut db = jkn::db::Database::from_config(&cfg).expect("unable to open database");
    if let Some(cmd) = opts.command {
        cmd.exec(&cfg, &mut db);
    }
}
