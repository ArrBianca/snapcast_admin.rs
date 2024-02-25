use std::fmt::Formatter;

use serde::{Deserialize, Serialize};
use terminal_size::Width;
use time::macros::format_description;

pub static DATABASE_FIELDS: [&str; 13] = [
    "title",
    "subtitle",
    "description",
    "media_url",
    "media_size",
    "media_type",
    "media_duration",
    "pub_date",
    "link",
    "image",
    "episode_type",
    "season",
    "episode",
];
#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Episode {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub media_url: String,
    pub media_size: i32,
    pub media_type: String,
    pub media_duration: Option<i32>,
    #[serde(with = "time::serde::iso8601")]
    pub pub_date: time::OffsetDateTime,
    pub link: Option<String>,
    pub image: Option<String>,
    pub episode_type: Option<String>,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub uuid: String,
    pub podcast_uuid: String,
}

impl std::fmt::Display for Episode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let duration = self.media_duration.unwrap_or_default();
        let ymd = format_description!("[year]-[month]-[day]");
        // f"{fwtruncate(ep.title, width - 38)}" if width else f"{ep.title}",
        let title = match terminal_size::terminal_size() {
            Some((Width(w), _h)) => self
                .title
                .chars()
                .take((w as usize) - 38)
                .collect::<String>(),
            None => self.title.clone(),
        };
        write!(
            f,
            "{id:3}│ {pub_date}| {hh:2}:{mm:02}:{ss:02}│ {media_size:6.2} MB│ {title}",
            id = self.id,
            // pub_date = self.pub_date.format("%Y-%m-%d"),
            pub_date = self.pub_date.format(&ymd).unwrap(),
            hh = duration / 60 / 60,
            mm = duration / 60 % 60,
            ss = duration % 60,
            media_size = self.media_size as f64 / 1000000.0,
            title = title
        )
    }
}

// Alas
// impl From<String> for Episode {
//     fn from(value: String) -> Self {
//         get_episode(&value).unwrap_or_else(|err| {
//             eprintln!("Error fetching Episode with ID {}: {}", value, err);
//             process::exit(1);
//         })
//     }
// }
