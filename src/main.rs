extern crate core;

mod logger;
mod commands;
mod database;
mod utils;

use std::env;
use std::sync::{Arc};
use dotenv::dotenv;
use log::{error, info, warn};
use serenity::{async_trait, Client};
use serenity::builder::CreateEmbed;
use serenity::client::{Context, EventHandler};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::GatewayIntents;
use serenity::utils::Color;
use tokio::sync::Mutex;
use tokio_rusqlite::Connection;
use crate::database::{run_migrations, SharedConnection};
use crate::logger::init;



struct Bot{
    database: SharedConnection
}


#[async_trait]
impl EventHandler for Bot{
    // The message handler will check incoming message and check command prefix
    // to execute corresponding commands.
    async fn message(&self, ctx: Context, msg: Message) {
        let _user_id = msg.author.id.0 as i64;
        if msg.content == "embed"{
            let embed = CreateEmbed::default()
                .title("Embed Title")
                .description("This is the embed description.")
                .color(Color::from_rgb(25, 78, 13))
                .author(|a| a.name("Author Name"))
                .thumbnail("https://i.imgur.com/qOYH7Mz.png")
                .image("https://i.imgur.com/vXI8C2o.png")
                .footer(|f| f.text("Footer Text")).clone();

            let msg = msg.channel_id.send_message(&ctx.http, |m| {
                m.set_embed(embed)
                    .add_file("./documentation/tips_list_example.png")
            }).await;


            if let Err(why) = msg {
                error!("Failed to send embed message. Error:\n{}", why);
            }
        }



        // !tips_category add :
        // param : <str> Name
        //
        // This command add a new tips category to the list and then, display
        // every category known.
        /*if let Some(cmd_content) = msg.content.strip_prefix("!tips_category add") {
            let category_name = cmd_content.trim();

            // 1 - insert the new type
            sqlx::query!(
                "INSERT INTO tips_category (name) VALUES (?);",
                category_name,
            )
                .execute(&self.database) // < Where the command will be executed
                .await
                .unwrap();

            let mut response = format!("Successfully added `{}` to your tips category list.\n", category_name);
            let entries = sqlx::query!("SELECT * FROM tips_category;")
                .fetch_all(&self.database)
                .await.unwrap();

            for (_, row) in entries.iter().enumerate() {
                match writeln!(response, "{} : {}", row.id, row.name){
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to write a new line in !tips_category list command. Error:\n{}", err);
                    }
                };
            }

            match msg.channel_id.say(&ctx, &response).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response, err);
                }
            };
        }
        // !tips_category list :
        // param : NONE
        //
        // This command will display every tips category name and id.
        else if let Some(_) = msg.content.strip_prefix("!tips_category list"){
            let mut response = format!("Here is the new list of  `{}`: \n", "TIPS CATEGORIES");
            let entries = sqlx::query!("SELECT * FROM tips_category;")
                .fetch_all(&self.database)
                .await.unwrap();

            for (_, row) in entries.iter().enumerate() {
                match writeln!(response, "{} : {}", row.id, row.name){
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to write a new line in !tips_category list command. Error:\n{}", err);
                    }
                };
            }

            match msg.channel_id.say(&ctx, &response).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response, err);
                }
            };
        }
        // !tips_category update :
        // param : <uint> category id
        //         <String> New category name
        //
        // This command will update the selected category name
        else if let Some(args) = msg.content.strip_prefix("!tips_category update"){
            let category_id: u8 = match args.trim().split(' ').collect::<Vec<&str>>().first(){
                Some(id) => {
                    match id.parse::<u8>() {
                        Ok(cat_id) => cat_id,
                        Err(err) => {
                            let response_msg = &format!("Failed to get category id argument.\nUsage: !tips_category update [<int> id] [<str> New name]\n\nError :\n{}\nId: \"{}\"", err, id);
                            match msg.channel_id.say(&ctx, response_msg).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response_msg, err);
                                }
                            };
                            return;
                        }
                    }
                },
                None => {
                    let response_msg = &format!("Failed to get category id argument.\nUsage: !tips_category update [<int> id] [<str> New name]");
                    match msg.channel_id.say(&ctx, response_msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response_msg, err);
                        }
                    };
                    return;
                }
            };
            let category_name: &str = &*match args.trim().split(' ').collect::<Vec<&str>>().get(1..) {
                Some(new_name) => {
                    new_name.join(" ")
                },
                None => {
                    let response_msg = &format!("Failed to get category name argument.\nUsage: !tips_category update [<int> id] [<str> New name]");
                    match msg.channel_id.say(&ctx, response_msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response_msg, err);
                        }
                    };
                    return;
                }
            };

            sqlx::query!("UPDATE tips_category SET name = ? WHERE id = ?;",
                category_name,
                category_id,
            )
            .execute(&self.database)
            .await.unwrap();

            let mut response = format!("Here is the new list of  `{}`: \n", "TIPS CATEGORIES");
            let entries = sqlx::query!("SELECT * FROM tips_category;")
                .fetch_all(&self.database)
                .await.unwrap();

            for (_, row) in entries.iter().enumerate() {
                match writeln!(response, "{} : {}", row.id, row.name){
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to write a new line in !tips_category list command. Error:\n{}", err);
                    }
                };
            }

            msg.channel_id.say(&ctx, response).await.unwrap();
        }
        // !tips_category delete :
        // param : <int> id
        //
        // This command will delete the tips_category requested
        else if let Some(args) = msg.content.strip_prefix("!tips_category delete"){
            let category_id: u8 = match args.trim().split(' ').collect::<Vec<&str>>().first(){
                Some(id) => {
                    match id.parse::<u8>() {
                        Ok(cat_id) => cat_id,
                        Err(err) => {
                            let response_msg = &format!("Failed to get category id argument.\nUsage: !tips_category delete [<int> id] \n\nError :\n{}\nId: \"{}\"", err, id);
                            match msg.channel_id.say(&ctx, response_msg).await {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response_msg, err);
                                }
                            };
                            return;
                        }
                    }
                },
                None => {
                    let response_msg = &format!("Failed to get category id argument.\nUsage: !tips_category update [<int> id]");
                    match msg.channel_id.say(&ctx, response_msg).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response_msg, err);
                        }
                    };
                    return;
                }
            };

            sqlx::query!("DELETE FROM tips_category WHERE id = ?;",
                category_id,
            )
                .execute(&self.database)
                .await.unwrap();

            let mut response = format!("Here is the new list of  `{}`: \n", "TIPS CATEGORIES");
            let entries = sqlx::query!("SELECT * FROM tips_category;")
                .fetch_all(&self.database)
                .await.unwrap();

            for (_, row) in entries.iter().enumerate() {
                match writeln!(response, "{} : {}", row.id, row.name){
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to write a new line in !tips_category list command. Error:\n{}", err);
                    }
                };
            }

            match msg.channel_id.say(&ctx, &response).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to send a message on the chanel id {}. Message:\n{}\n\nError:\n{}", msg.channel_id, response, err);
                }
            };
        }*/
    }

    // The interaction handler will handle every /commands
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            info!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "tips_list" => {
                    commands::tips::list::run(&command.data.options, self.database.clone()).await
                },
                "tips_create" => {
                    commands::tips::create::run(&command.data.options, self.database.clone()).await
                },
                _ => "Not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
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
        println!("{} is connected and ready to use !", data.user.name);

        let guild_id = data.guilds.first().unwrap().id;

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::tips::list::register(command))
                .create_application_command(|command| commands::tips::create::register(command))
        })
            .await;

        println!("I now have the following guild slash commands: {:#?}", commands);
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
        database
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