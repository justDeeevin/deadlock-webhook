use rss::Channel;
use scraper::{ElementRef, Html, Selector};
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

    let timestamp = latest_message
        .select(&Selector::parse("time").unwrap())
        .next()
        .unwrap()
        .attr("data-timestamp")
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
        get_contents(body)
    };

    let mut req =
        ExecuteWebhook::new().allowed_mentions(CreateAllowedMentions::new().everyone(true));
    req = if content.len() <= 2000 {
        req.content("@everyone\n\n".to_string() + &content)
    } else if content.len() <= 4096 {
        req.content("@everyone").embed(
            CreateEmbed::new()
                .title(latest.title().unwrap_or("Deadlock Patch Notes"))
                .description(content)
                .url(url)
                .timestamp(Timestamp::from_unix_timestamp(timestamp.parse()?)?)
                .color(0xEFDEBF),
        )
    } else {
        todo!("too long!")
    };

    let http = Http::new("");
    Webhook::from_url(&http, WEBHOOK_URL)
        .await?
        .execute(&http, true, req)
        .await?;

    Ok(())
}

fn get_contents(element: ElementRef) -> String {
    element
        .inner_html()
        .replace("<br>", "\n")
        .replace("<b>", "**")
        .replace("</b>", "**")
}
