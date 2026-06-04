#[cfg(debug_assertions)]
mod cli;
mod format;

use format::{format, format_steam};
use rss::Channel;
use scraper::{Html, Selector};
use serenity::{
    builder::{CreateAllowedMentions, CreateEmbed, ExecuteWebhook},
    http::Http,
    model::{Timestamp, webhook::Webhook},
};
use steam_rs::Steam;

const RSS_URL: &str = "https://forums.playdeadlock.com/forums/changelog.10/index.rss";
const WEBHOOK_URL: &str = "https://discord.com/api/webhooks/1425590300511830177/sDy9U1TanKfR2xfljLJ4j-cA3Kgqqethl_be6J7Go7pDzu-mjhnVONgJo3bA2A28pprr";
const DEADLOCK_APPID: u32 = 1422450;

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    #[cfg(debug_assertions)]
    let cli::Args { dry, index } = <cli::Args as clap::Parser>::parse();
    #[cfg(not(debug_assertions))]
    // FIXME: Is the first entry always the latest?
    let index = 0;

    let latest = Channel::read_from(reqwest::get(RSS_URL).await?.bytes().await?.as_ref())?
        .items
        .remove(index);
    let url = latest.link().unwrap();

    let page = Html::parse_document(&reqwest::get(url).await?.text().await?);

    let latest_message = {
        let message_selector = Selector::parse("div.message-main").unwrap();
        let mut messages = page.select(&message_selector);

        if latest.extensions["slash"]["comments"][0].value().unwrap() == "0" {
            messages.next()
        } else {
            messages.next_back()
        }
        .unwrap()
    };

    let timestamp = latest_message
        .select(&Selector::parse("time").unwrap())
        .next()
        .unwrap()
        .attr("data-timestamp")
        .unwrap()
        .parse()?;

    let body = latest_message
        .select(&Selector::parse("div.bbWrapper").unwrap())
        .next()
        .unwrap();

    let content = if body
        .select(&Selector::parse("div.fauxBlockLink").unwrap())
        .next()
        .is_some()
    {
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
        format(body)
    };

    let mut req =
        ExecuteWebhook::new().allowed_mentions(CreateAllowedMentions::new().everyone(true));
    req = if content.len() <= 2000 {
        req.content(format!("@everyone\n\n{content}"))
    } else {
        req.content("@everyone").embed(
            CreateEmbed::new()
                .title(latest.title().unwrap_or("Deadlock Patch Notes"))
                .description(&content[0..4096])
                .url(url)
                .timestamp(Timestamp::from_unix_timestamp(timestamp)?)
                .color(0xEFDEBF),
        )
    };

    #[cfg(debug_assertions)]
    if dry {
        return Ok(());
    }

    let http = Http::new("");
    Webhook::from_url(&http, WEBHOOK_URL)
        .await?
        .execute(&http, true, req)
        .await?;

    println!("Sent");

    Ok(())
}
