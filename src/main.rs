mod logger;

use std::env;
use dotenv::dotenv;
use log::{error};
use serenity::{async_trait, Client};
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use crate::logger::init;

struct Handler;

#[async_trait]
impl EventHandler for Handler{
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called
    //
    // Event handlers are dispatched through a thread-pool, and so multiple events
    // can be dispatched simultaneously
    async fn message(&self, ctx: Context, message: Message) {
        if message.content == "!tips" {
            // sending a message can fail, due to network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            if let Err(error) = message.channel_id.say(&ctx.http, "This is a random daily ").await {
                error!("Failed to send message to respond a tips command. Error:\n{}", error);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, data: Ready) {
        println!("{} is connected and ready to use !", data.user.name);
    }
}


#[tokio::main]
async fn main() {
    init().expect("Failed to init the logger.");
    dotenv().expect("Failed to load .env variables into system env");
    // Configure the client with the discord bot token in the environment token
    let token = env::var("DISCORD_TOKEN").expect("Expected an env file with the DISCORD_TOKEN entry set. {}");
    //let token = "MTA5NTQ2ODcyMzk0NDU2NjkwNA.G2jCIh.epkFFjAxVlo1eWQv0a4cfdyaKYWvLEXvSWqOrQ";

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the client, logging in as a bot.
    // This will be automatically prepend your bot token with "Bot",
    // which is a requirement by Discord for bot user.
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Error creating client.");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until it reconnects.
    if let Err(error) = client.start().await {
        error!("Client error : {:?}", error);
    }
}