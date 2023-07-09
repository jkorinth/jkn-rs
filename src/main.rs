use clap::*;
use env_logger;
use termimad;
mod config;
mod db;
mod topic;
use log::*;

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
        _ => {
            error!("unknown command");
        }
    }
    //termimad::print_inline("***Hello***, **world**! `this` is nice.\n");
    //process::Command::new(editor).spawn().expect("could not launch {editor}").wait();
    //println!("current branch: {}", db.current_branch());
}
