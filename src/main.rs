mod cli;
mod format;

use clap::Parser;
use format::{format_html, format_steam};
use rss::Channel;
use scraper::{Html, Selector};
use serenity::{
    all::MessageFlags,
    builder::{CreateAllowedMentions, CreateEmbed, ExecuteWebhook},
    http::Http,
    model::{Timestamp, webhook::Webhook},
};
use steam_rs::Steam;

use crate::cli::Args;

const RSS_URL: &str = "https://forums.playdeadlock.com/forums/changelog.10/index.rss";
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

    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL not set");
    let mention = std::env::var("ROLE_ID")
        .map(|id| format!("<@&{id}>"))
        .unwrap_or_else(|_| "@everyone".into());
    let args = Args::parse();

    let latest = Channel::read_from(reqwest::get(RSS_URL).await?.bytes().await?.as_ref())?
        .items
        .remove(args.index);
    let mut url = latest.link().unwrap();

    let page = Html::parse_document(&reqwest::get(url).await?.text().await?);
    let latest_message = page
        .select(&Selector::parse("div.message-main").unwrap())
        .next_back()
        .unwrap();

    let timestamp = latest_message
        .select(&Selector::parse("time").unwrap())
        .next()
        .unwrap()
        .attr("data-timestamp")
        .unwrap()
        .parse()?;

    if !args.force
        && timestamp
            <= std::fs::read_to_string(SAVE_PATH)
                .unwrap_or_else(|_| "0".to_string())
                .parse()?
    {
        return Ok(());
    }

    let body = latest_message
        .select(&Selector::parse("div.bbWrapper").unwrap())
        .next()
        .unwrap();
    let content = if let Some(link) = body
        .select(&Selector::parse("div.fauxBlockLink").unwrap())
        .next()
    {
        url = link.attr("data-url").unwrap();
        format_steam(
            &Steam::get_news_for_app(
                DEADLOCK_APPID,
                None,
                None,
                None,
                Some(vec!["steam_community_announcements"]),
            )
            .await?
            .newsitems[0]
                .contents,
        )
    } else {
        format_html(body)
    };

    let mut req = ExecuteWebhook::new()
        .allowed_mentions(CreateAllowedMentions::new().everyone(true))
        .avatar_url(AVATAR_URL);
    let plain_message_prepend = format!("{mention} **[Deadlock Patch Notes]({url})**\n\n");
    req = if content.len() <= 2000 - plain_message_prepend.len() {
        req.content(plain_message_prepend + &content)
            .flags(MessageFlags::SUPPRESS_EMBEDS)
    } else {
        req.content(mention).embed(
            CreateEmbed::new()
                .title(latest.title().unwrap_or("Deadlock Patch Notes"))
                .description(if content.len() <= 4096 {
                    content
                } else {
                    format!("{}…", &content[..4095])
                })
                .url(url)
                .timestamp(Timestamp::from_unix_timestamp(timestamp)?)
                .color(0xEFDEBF),
        )
    };

    if args.dry {
        return Ok(());
    }

    std::fs::write(SAVE_PATH, timestamp.to_string())?;

    let http = Http::new("");
    Webhook::from_url(&http, &webhook_url)
        .await?
        .execute(&http, true, req)
        .await?;

    println!("sent");

    Ok(())
}
