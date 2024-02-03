use clap::{arg, Parser, Subcommand};

use crate::snapcast::{Episode, DATABASE_FIELDS};

#[derive(Parser, Debug)]
#[command(name = "snapcast_admin")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    pub snadmin_feed_id: String,
    pub snadmin_token: String,
    pub snadmin_base_url: String,
}

#[allow(clippy::large_enum_variant)] // it is what it is
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print a list of all episodes.
    List {
        #[arg(
            short,
            long,
            value_name = "FIELD",
            value_parser = [ "id", "pub_date" ],
            default_value = "pub_date",
        )]
        /// sort results
        sort: String,
        #[arg(short, long, value_name = "TEXT")]
        /// filter output to results containing TEXT
        find: Option<String>,
    },
    /// Fetch information about an episode.
    Info {
        /// episode id
        episode_id: String,
        // #[arg(value_parser = EpisodeValueParser)]
        #[arg(skip)]
        episode: Option<Episode>,
    },
    /// Update a field of an episode.
    Update {
        /// episode id as a number
        episode_id: String,
        #[arg(value_parser = DATABASE_FIELDS)]
        /// the database field to update
        field: String,
        /// the new value to set the field to
        value: String,
    },
    // TODO: Delete command
}
