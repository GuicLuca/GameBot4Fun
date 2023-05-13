extern crate core;

mod logger;
mod commands;
mod database;
mod utils;

use std::env;
use std::sync::{Arc};
use dotenv::dotenv;
use log::{error, warn};
use serenity::{async_trait, Client};
use serenity::builder::CreateEmbed;
use serenity::client::{Context, EventHandler};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::Timestamp;
use serenity::prelude::GatewayIntents;
use serenity::utils::Color;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_rusqlite::Connection;
use crate::database::{run_migrations, SharedConnection};
use crate::logger::init;

pub type SharedJoinHandle = Arc<RwLock<Option<JoinHandle<()>>>>;

struct Bot{
    database: SharedConnection, // Shared connection to the database to run sql request from everywhere
    tips_scheduler: SharedJoinHandle, // Handler of the scheduler to stop it
}


#[async_trait]
impl EventHandler for Bot{
    // The message handler will check incoming message and check command prefix
    // to execute corresponding commands.
    async fn message(&self, ctx: Context, msg: Message) {
        let _user_id = msg.author.id.0 as i64;
        if msg.content == "!ping"{

            let msg = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("I'm alive ;)")
            }).await;


            if let Err(why) = msg {
                error!("Failed to send embed message. Error:\n{}", why);
            }
        }
    }

    // The interaction handler will handle every /commands
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let embed = match command.data.name.as_str() {
                "tips_list" => {
                    commands::tips::list::run(&command.data.options, self.database.clone()).await
                },
                "tips_create" => {
                    commands::tips::create::run(&command.data.options, self.database.clone()).await
                },
                "tips_read" => {
                    commands::tips::read::run(&command.data.options, self.database.clone()).await
                },
                "tips_update" => {
                    commands::tips::update::run(&command.data.options, self.database.clone()).await
                },
                "tips_delete" => {
                    commands::tips::delete::run(&command.data.options, self.database.clone()).await
                },
                "scheduler_config" => {
                    commands::tips_scheduler::config::run(&command.data.options, self.database.clone(), self.tips_scheduler.clone(), &ctx.http).await
                },
                "scheduler" => {
                    commands::tips_scheduler::scheduler::run(&command.data.options, self.database.clone(), self.tips_scheduler.clone(), &ctx.http).await
                },
                _ => {
                    CreateEmbed::default()
                        .title("Not implemented :(")
                        .colour(Color::from_rgb(255, 204, 0))
                        .description("Please retry later. If you think it's an error contact the administrator of the server.")
                        .timestamp(Timestamp::now())
                        .to_owned()
                },
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| {
                            m.set_embed(embed)
                        })
                })
                .await
            {
                warn!("Cannot respond to slash command: {}", why);
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, data: Ready) {
        let guild_id = data.guilds.first().unwrap().id;

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                // tips
                .create_application_command(|command| commands::tips::list::register(command))
                .create_application_command(|command| commands::tips::create::register(command))
                .create_application_command(|command| commands::tips::read::register(command))
                .create_application_command(|command| commands::tips::update::register(command))
                .create_application_command(|command| commands::tips::delete::register(command))
                // scheduler
                .create_application_command(|command| commands::tips_scheduler::config::register(command))
                .create_application_command(|command| commands::tips_scheduler::scheduler::register(command))
        })
            .await;

        println!("I now have the following guild slash commands: {:#?}", commands);
        println!("{} is connected and ready to use !", data.user.name);
    }
}


#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env variables into system env");
    // init the logger : see logger.rs
    init().expect("Failed to init the logger.");
    // Configure the client with the discord bot token in the environment token
    let token = env::var("DISCORD_TOKEN").expect("Expected an env file with the DISCORD_TOKEN entry set.");

    // Initiate a connection to the database file, creating the file if required.
    let database: SharedConnection = Arc::from(Mutex::from(Connection::open("database.sqlite").await
        .expect("Couldn't connect to database")));

    // Run migrations, which updates the database's schema to the latest version.
    {
        run_migrations(database.clone()).await.expect("Failed to run migrations. Error");
    }

    let bot = Bot{
        database,
        tips_scheduler: Arc::from(RwLock::from(None)), // there is no scheduler running
    };


    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the client, logging in as a bot.
    // This will be automatically prepend your bot token with "Bot",
    // which is a requirement by Discord for bot user.
    let mut client = Client::builder(&token, intents).event_handler(bot).await.expect("Error creating client.");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until it reconnects.
    if let Err(error) = client.start().await {
        error!("Client error : {:?}", error);
    }
}