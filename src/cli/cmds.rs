use clap::*;

#[derive(Parser)]
#[command(version)]
pub struct Opts {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Shows current topic.
    #[clap(alias = "t")]
    Topic {
        /// if set, switches current topic
        name: Option<String>,
    },

    /// Lists all elements of a kind\.
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
        /// amend last note?
        #[arg(short, long)]
        amend: Option<bool>,
    },

    /// Show the current note
    #[clap(alias = "s")]
    Show {
        /// select topic
        #[arg(short, long)]
        topic: Option<String>,
        /// show latest, if today's note is not available?
        #[arg(long)]
        only_latest: Option<bool>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ItemKind {
    /// list the currently defined topics
    #[clap(alias = "t")]
    Topics {},
    /// list all entries in the current topic
    #[clap(alias = "e")]
    Notes {},
}
