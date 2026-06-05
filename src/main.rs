mod cli;
mod format;

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

const RSS_URL: &str = "https://forums.playdeadlock.com/forums/changelog.10/index.rss";
const AVATAR_URL: &str = "https://project8-data.community.forum/assets/logo_alternate/icon.png";
const DEADLOCK_APPID: u32 = 1422450;
const SAVE_PATH: &str = if cfg!(debug_assertions) {
    "./last-post"
} else {
    "/var/cache/deadlock-webhook/last-post"
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL not set");
    let mention = std::env::var("ROLE_ID")
        .map(|id| format!("<@&{id}>"))
        .unwrap_or_else(|_| "@everyone".into());
    let cli::Args { dry, index, force } = <cli::Args as clap::Parser>::parse();

    let latest = Channel::read_from(reqwest::get(RSS_URL).await?.bytes().await?.as_ref())?
        .items
        .remove(index);
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

    if !force
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
                // Doesn't seem to work but I'm keeping it here anyway
                Some(vec!["steam_community_announcements"]),
            )
            .await?
            .newsitems
            .remove(0)
            .contents,
        )
    } else {
        format_html(body)
    };

    let mut req = ExecuteWebhook::new()
        .allowed_mentions(CreateAllowedMentions::new().everyone(true))
        // FIXME: seemingly doesn't work?
        // the url is correct. maybe rules about avatar images?
        .avatar_url(AVATAR_URL);
    // FIXME: subtract prepends
    req = if content.len() <= 2000 {
        req.content(format!(
            "{mention} **[Deadlock Patch Notes]({url})**\n\n{content}"
        ))
        .flags(MessageFlags::SUPPRESS_EMBEDS)
    } else {
        req.content(mention).embed(
            CreateEmbed::new()
                .title(latest.title().unwrap_or("Deadlock Patch Notes"))
                .description(&content[0..4096]) // TODO: truncate with ellipsis
                .url(url)
                .timestamp(Timestamp::from_unix_timestamp(timestamp)?)
                .color(0xEFDEBF),
        )
    };

    if dry {
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
