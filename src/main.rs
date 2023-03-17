mod settings;

use html_parser;
use reqwest;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use settings::Settings;
use tl;

#[group]
#[commands(ping, deals)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    // lookup golf deals on reddit

    println!("Hello, world!");

    let settings = Settings::new().unwrap();

    println!("{}", settings.discord.token);

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let token = settings.discord.token;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

async fn get_todays_deals() {
    let url =
        "https://www.reddit.com/r/golf/comments/11trhpq/daily_golf_deals_03172023_nurseresidences/";

    let res = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let dom = tl::parse(&res, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom
        .get_elements_by_class_name("usertext-body")
        .last()
        .expect("Failed to find element")
        .get(parser)
        .unwrap();

    println!("{:?}", element);
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

#[command]
async fn deals(ctx: &Context, msg: &Message) -> CommandResult {
    get_todays_deals().await;

    Ok(())
}
