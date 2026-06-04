mod format;

use format::format;
use rss::Channel;
use scraper::{Html, Selector};
use serenity::{
    builder::{CreateAllowedMentions, CreateEmbed, ExecuteWebhook},
    http::Http,
    model::{Timestamp, webhook::Webhook},
};

const RSS_URL: &str = "https://forums.playdeadlock.com/forums/changelog.10/index.rss";
const WEBHOOK_URL: &str = "https://discord.com/api/webhooks/1425590300511830177/sDy9U1TanKfR2xfljLJ4j-cA3Kgqqethl_be6J7Go7pDzu-mjhnVONgJo3bA2A28pprr";

#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let index: usize = std::env::args()
        .nth(1)
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or_default(); // TODO: is the first one always the latest?

    let latest = Channel::read_from(reqwest::get(RSS_URL).await?.bytes().await?.as_ref())?
        .items
        .remove(index);
    let url = latest.link().unwrap();

    let page = Html::parse_fragment(&reqwest::get(url).await?.text().await?);

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

    let time = latest_message
        .select(&Selector::parse("time").unwrap())
        .next()
        .unwrap();

    let body = latest_message
        .select(&Selector::parse("div.bbWrapper").unwrap())
        .next()
        .unwrap();

    let content = if let Some(link) = body
        .select(&Selector::parse("div.fauxBlockLink").unwrap())
        .next()
    {
        todo!("steam announcement")
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
                .timestamp(Timestamp::parse(time.attr("datetime").unwrap())?)
                .color(0xEFDEBF),
        )
    };

    let http = Http::new("");
    Webhook::from_url(&http, WEBHOOK_URL)
        .await?
        .execute(&http, true, req)
        .await?;

    Ok(())
}
