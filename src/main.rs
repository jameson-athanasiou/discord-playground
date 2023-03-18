mod settings;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;
use settings::Settings;

#[group]
#[commands(deals)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[derive(Serialize, Deserialize, Debug)]
struct PostData {
    selftext: String,
    title: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChildrenData {
    data: PostData,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApiData {
    children: Vec<ChildrenData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RedditPost {
    data: ApiData,
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let token = settings.discord.token;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

async fn get_todays_deals() -> String {
    let url =
        "https://www.reddit.com/r/golf/comments/11trhpq/daily_golf_deals_03172023_nurseresidences.json";

    let res = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let first = res.get(0).unwrap().to_owned();

    let post: RedditPost = serde_json::from_value(first).unwrap();

    let first_child = post.data.children.first().unwrap().to_owned();

    let text = first_child.data.selftext.to_owned();

    text
}

#[command]
async fn deals(ctx: &Context, msg: &Message) -> CommandResult {
    let result = get_todays_deals().await;

    let parts: Vec<&str> = result.split_inclusive("\n").collect();

    let bad_parts = [
        "in case you missed it",
        "Non-clubs Request",
        "Clubs Request",
        "[Sign-up here]",
        "Fill out the GOogle form below",
    ];

    for part in parts {
        println!("part = {}", part);

        let mut message_to_send = Some(part);

        for baddie in bad_parts {
            if part.contains(baddie) {
                message_to_send = None;
            }
        }

        if !part.contains("https") {
            message_to_send = None;
        }

        if let Some(m) = message_to_send {
            if let Err(err) = msg.reply(ctx, m).await {
                println!("Error sending reply - {}", err);
            };
        }
    }

    Ok(())
}
