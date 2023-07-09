use clap::*;
use env_logger;
use log::*;
use std::env;
use std::process;
use termimad;
mod config;
mod db;
mod topic;
mod note;

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

#[derive(Subcommand)]
enum ItemKind {
    #[clap(alias = "t")]
    Topics {},
}

fn main() {
    env_logger::init();
    let opts = Opts::parse();
    let cfg = config::Config::load().expect("could not load configuration");
    cfg.save().expect("failed to save config");
    let db = db::Database::from_config(&cfg).expect("unable to open database");
    match &opts.command {
        Some(Commands::Topic { name }) => {
            debug!("received topic command with name {:?}", name);
            if let Some(n) = name {
                println!("{:?}", db.topic(Some(n.as_str())));
            } else {
                println!("current topic is {:?}", db.current_topic());
            }
        }
        Some(Commands::List { kind }) => {
            println!(
                "{:?}",
                match kind {
                    Some(ItemKind::Topics {}) => db.list(db::Entity::Topic),
                    None => db.list(db::Entity::Topic),
                }
            )
        }
        Some(Commands::Note { topic }) => {
            let editor = env::var("EDITOR").expect("EDITOR env var not set - don't know which editor to use!");
            if let Some(t) = topic {
                db.topic(Some(&t.as_str()));
            }
            let mut note = cfg.git.repopath.to_path_buf();
            note.push(db.current_note());
            debug!("current note: {:?}:", note);
            let ret = process::Command::new(editor)
                .args([note.as_os_str()])
                //.spawn()
                .status()
                .expect("could not launch {editor}");
            if ret.success() {
                match db.commit(&db.current_note()) {
                    Ok(()) => { info!("committed successfully"); }
                    Err(e) => { error!("failed to commit: {:?}", e); }
                }
            } else {
                warn!("editing was aborted, discarding changes");
            }
        }
        _ => {
            error!("unknown command");
        }
    }
    //termimad::print_inline("***Hello***, **world**! `this` is nice.\n");
    //println!("current branch: {}", db.current_branch());
}
