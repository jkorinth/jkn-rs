use clap::*;
use env_logger;
use termimad;
mod config;
mod db;
mod topic;

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

    /// Lists all topics.
    #[clap(alias = "l")]
    ListTopics {},

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

fn main() {
    env_logger::init();
    let cfg = config::Config::load().expect("could not load configuration");
    termimad::print_inline("***Hello***, **world**! `this` is nice.\n");
    //process::Command::new(editor).spawn().expect("could not launch {editor}").wait();
    cfg.save().expect("failed to save config");
    let db = db::Database::from_config(&cfg).expect("unable to open database");
    println!("current branch: {}", db.current_branch());
}
