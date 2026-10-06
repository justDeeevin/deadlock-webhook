mod cli;
mod format;

use clap::Parser;
use cli::Args;
use format::format_steam;
use serenity::{
    all::{MessageFlags, Timestamp},
    builder::{CreateAllowedMentions, CreateEmbed, ExecuteWebhook},
    http::Http,
    model::webhook::Webhook,
};
use std::{env, fs};
use steam_rs::Steam;

const AVATAR_URL: &str = "https://project8-data.community.forum/assets/logo_alternate/icon.png";
const DEADLOCK_APPID: u32 = 1422450;
const SAVE_PATH: &str = if cfg!(debug_assertions) {
    "./last-post"
} else {
    "/var/lib/deadlock-webhook/last-post"
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let webhook_url = if let Ok(url) = env::var("WEBHOOK_URL") {
        url
    } else if let Ok(path) = env::var("WEBHOOK_URL_FILE") {
        fs::read_to_string(path)?
    } else {
        panic!("No webhook URL provided");
    };
    let role_id = std::env::var("ROLE_ID")
        .ok()
        .map(|s| s.parse::<u64>())
        .transpose()?;
    let args = Args::parse();

    let mut news = Steam::get_news_for_app(
        DEADLOCK_APPID,
        None,
        None,
        Some(args.index as u32 + 1),
        Some(vec!["steam_community_announcements"]),
    )
    .await?
    .newsitems;

    let latest = news.swap_remove(args.index);

    if !args.force
        && fs::read_to_string(SAVE_PATH)
            .as_deref()
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or_default()
            == latest.date
    {
        return Ok(());
    }

    let content = format_steam(&latest.contents);

    let allowed_mentions = CreateAllowedMentions::new();
    let mut req = ExecuteWebhook::new()
        .allowed_mentions(if let Some(id) = role_id {
            allowed_mentions.roles([id])
        } else {
            allowed_mentions.everyone(true)
        })
        .avatar_url(AVATAR_URL);

    let mention = if let Some(id) = role_id {
        format!("<@&{id}>")
    } else {
        "@everyone".into()
    };

    let plain_message_prepend = format!("{mention} **[{}]({})**\n\n", latest.title, latest.url);
    req = if content.len() <= 2000 - plain_message_prepend.len() {
        req.content(plain_message_prepend + &content)
            .flags(MessageFlags::SUPPRESS_EMBEDS)
    } else {
        req.content(mention).embed(
            CreateEmbed::new()
                .title(latest.title)
                .description(if content.len() <= 4096 {
                    content
                } else {
                    format!("{}…", &content[..4095])
                })
                .url(latest.url)
                .timestamp(Timestamp::from_unix_timestamp(latest.date as i64)?)
                .color(0xEFDEBF),
        )
    };

    if args.dry {
        return Ok(());
    }

    fs::write(SAVE_PATH, latest.date.to_string())?;

    let http = Http::new("");
    Webhook::from_url(&http, &webhook_url)
        .await?
        .execute(&http, true, req)
        .await?;

    println!("sent");

    Ok(())
}
