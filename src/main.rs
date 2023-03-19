mod deals_source;
mod settings;

use async_recursion::async_recursion;
use env_logger;
use log::{debug, error, info, log_enabled, Level};
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
#[commands(deals, test)]
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
    env_logger::init();

    error!("Starting bot server...");

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

    match client.start().await {
        Ok(_) => println!("Bot started"),
        Err(e) => println!("Bot encountered an error on startup - {}", e),
    }
}

async fn get_deals_posts() -> Result<serde_json::Value, reqwest::Error> {
    let url = "https://www.reddit.com/user/Nurseresidences.json";

    let res = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res)
}

fn get_post(all_posts: serde_json::Value, today: String) -> Option<PostData> {
    let data = all_posts.get("data").unwrap().get("children").unwrap();

    let children = data.as_array().unwrap();

    let mut deals_posts: Vec<PostData> = Vec::new();

    for child in children {
        let title = child.get("data").unwrap().get("title");

        if title.is_some() {
            let post: PostData =
                serde_json::from_value(child.get("data").unwrap().to_owned()).unwrap();
            deals_posts.push(post);
        }
    }

    let item = deals_posts.into_iter().find(|x| {
        let todays_post_title = format!("Daily Golf Deals {} (NurseResidences)", today);
        x.title.contains(&todays_post_title)
    });

    item
}

#[async_recursion]
async fn get_post_body(attempts: u64) -> String {
    if attempts > 4 {
        return String::from("");
    }

    let mut text = String::from("");
    let today = deals_source::get_title_date(attempts);

    let posts = get_deals_posts().await.unwrap();
    let result = get_post(posts, today);

    if let None = result {
        text = get_post_body(attempts + 1).await;
    } else {
        text = result.unwrap().selftext;
    }

    text
}

#[command]
async fn deals(ctx: &Context, msg: &Message) -> CommandResult {
    let result = get_post_body(0).await;

    let parts: Vec<&str> = result.split_inclusive("\n").collect();

    let bad_parts = [
        "in case you missed it",
        "Non-clubs Request",
        "Clubs Request",
        "[Sign-up here]",
        "Fill out the Google form below",
    ];

    for part in parts {
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

#[command]
async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    let posts = get_deals_posts().await.unwrap();

    // find_posts(posts);

    msg.reply(ctx, "test").await?;
    Ok(())
}
