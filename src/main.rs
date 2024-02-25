use std::env;
use std::error::Error;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{Args, Commands};
use crate::snapcast::Episode;
use clap::Parser;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::PrimitiveDateTime;
use ureq::{get, patch};
use url::Url;

mod cli;
mod snapcast;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Args {
        command: None,
        snadmin_feed_id: env::var("SNADMIN_FEED_ID")
            .expect("SNADMIN_FEED_ID required in environment"),
        snadmin_token: format!(
            "Bearer {}",
            env::var("SNADMIN_TOKEN").expect("SNADMIN_TOKEN required in environment")
        ),
        snadmin_base_url: env::var("SNADMIN_BASE_URL")
            .expect("SNADMIN_BASE_URL required in environment"),
    };

    Args::update_from(&mut args, env::args_os());

    match args.command.as_ref().unwrap() {
        Commands::List { sort, find } => handle_list(&args, sort, find),
        Commands::Info { episode_id } => match get_episode(&args, episode_id) {
            Ok(episode) => {
                println!("{:#?}", &episode);
                Ok(())
            }
            Err(e) => Err(e),
        },
        Commands::Update {
            episode_id,
            field,
            value,
        } => match get_episode(&args, episode_id) {
            Ok(episode) => handle_update(&args, episode, field, value),
            Err(e) => Err(e),
        },
        Commands::Download { episode_id } => match get_episode(&args, episode_id) {
            Ok(episode) => handle_download(episode),
            Err(e) => Err(e),
        },
    }
}

fn handle_download(episode: Episode) -> Result<(), Box<dyn Error>> {
    let media_url = Url::parse(&episode.media_url)?;
    let media_ext = Path::new(media_url.path()).extension().unwrap_or_default();

    let mut local_path = PathBuf::new();
    local_path.set_file_name(episode.uuid);
    local_path.set_extension(media_ext);

    let mut local_file = fs::File::create(&local_path)?;

    println!("Downloading {:?}", local_path);
    let resp = get(&episode.media_url).call()?;
    let len: usize = resp.header("Content-Length").unwrap().parse()?;
    let pb = ProgressBar::new(len as u64);
    // from the `download.rs` example in indicatif
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    std::io::copy(&mut pb.wrap_read(resp.into_reader()), &mut local_file)?;

    pb.finish();

    Ok(())
}

// TODO: Find option
fn handle_list(args: &Args, sort: &str, _find: &Option<String>) -> Result<(), Box<dyn Error>> {
    let mut episodes: Vec<Episode> = get(&format!(
        "{}/{}/episodes",
        args.snadmin_base_url, args.snadmin_feed_id
    ))
    .set("Authorization", args.snadmin_token.as_str())
    .call()?
    .into_json()?;

    episodes.sort_by(|a, b| match sort {
        "pub_date" => a.pub_date.cmp(&b.pub_date),
        "id" => a.id.cmp(&b.id),
        _ => unreachable!("Unreachable sort key branch."),
    });

    for episode in episodes {
        println!("{}", episode);
    }

    Ok(())
}

fn handle_update(
    args: &Args,
    episode: Episode,
    field: &String,
    value: &String,
) -> Result<(), Box<dyn Error>> {
    let new_value: serde_json::Value = match field.as_str() {
        // Convert [[HH:]MM:]SS to integer seconds. Looks less cool than the python version.
        "media_duration" => {
            let duration = value
                .split(':')
                .rev()
                .enumerate()
                .map(|(i, v)| u32::pow(60, i as u32) * v.parse::<u32>().unwrap())
                .sum::<u32>();
            json!(duration)
        }
        "pub_date" => {
            json!(PrimitiveDateTime::parse(
                value.as_str(),
                &format_description!("[year]-[month]-[day] [hour]:[minute]")
            )?
            .assume_offset(time::UtcOffset::current_local_offset().unwrap())
            .format(&Rfc3339)
            .unwrap())
        }
        _ => json!(value),
    };

    patch(&format!(
        "{}/{}/episode/{}",
        args.snadmin_base_url, args.snadmin_feed_id, episode.uuid
    ))
    .set("Authorization", args.snadmin_token.as_str())
    .send_json(json!({field: new_value}))?;

    Ok(())
}

fn get_episode(args: &Args, episode: &String) -> Result<Episode, Box<dyn Error>> {
    let episode = get(&format!(
        "{}/{}/episode/{episode}",
        args.snadmin_base_url, args.snadmin_feed_id
    ))
    .set("Authorization", args.snadmin_token.as_str())
    .call()?
    .into_json()?;
    Ok(episode)
}
