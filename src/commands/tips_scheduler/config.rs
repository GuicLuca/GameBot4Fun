use std::sync::Arc;
use rusqlite::{Error, OptionalExtension, params};
use rusqlite::Error::InvalidParameterCount;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::http::Http;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::channel::PartialChannel;
use serenity::model::id::ChannelId;
use serenity::model::mention::Mention;
use serenity::model::mention::Mention::Channel;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Color;
use crate::commands::tips_scheduler::scheduler::{start, stop};
use crate::database::SharedConnection;
use crate::SharedJoinHandle;
use crate::utils::{display_full_tip_in_embed, make_error_embed};


pub static CONFIG_ID: usize = 1; // Static configuration id

/*
This struct is used to access fetched data
from the database and give it to sub-functions.
 */
#[derive(Clone)]
pub struct SchedulerConfig {
    pub channel: u64,
    pub hour: u32,
    pub minute: u32,
}

/**
 * This method is the execution of the command /scheduler_config.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 * @param scheduler_status: SharedJoinHandle, the joinHandle of the scheduler to perform action on it.
 * @param http: &Arc<Http>, Http element used to send message on the discord server.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection, scheduler: SharedJoinHandle, http: &Arc<Http>) -> CreateEmbed {
    let mut updated_columns:Vec<&str> = Vec::with_capacity(3); // we will add column name updated for the query creation
    let mut updated_values:Vec<usize> = Vec::with_capacity(3); // we will add new values

    // 1 - check if optional values are present
    let mut message_channel: Option<PartialChannel> = None;
    let mut message_channel_id = 0;
    let mut hour: Option<usize> = None;
    let mut min: Option<usize> = None;

    for option in options {
        match option.name.as_str() {
            "message_chanel" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::Channel(param) => {
                            message_channel = Some(param.to_owned());
                        }
                        _ => {
                            return make_error_embed("scheduler_config::run", format!("The parameter message_chanel given has a bad format.\nIt must be an integer."));
                        }
                    }
                } else {
                    return make_error_embed("scheduler_config::run", format!("The parameter message_chanel is empty.\nExpected an integer."));
                }
            }
            "hour" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(param) => {
                            let vals: Vec<&str> = param.split(":").collect();
                            if vals.len() != 2 {
                                return make_error_embed("scheduler_config::run", format!("The parameter hour given has a bad format.\nExpected a string. with the following format : HH:mm"));
                            }
                            hour = match vals.get(0).unwrap().parse::<usize>() {
                                Ok(val) =>  Some(val),
                                Err(err) => {
                                    return make_error_embed("scheduler_config::run", format!("The parameter hour given has a bad format.\nExpected a string with the following format: HH:mm. {}", err));
                                }
                            };
                            min = match vals.get(1).unwrap().parse::<usize>() {
                                Ok(val) =>  Some(val),
                                Err(err) => {
                                    return make_error_embed("scheduler_config::run", format!("The parameter hour given has a bad format.\nExpected a string with the following format: HH:mm. {}", err));
                                }
                            };

                        }
                        _ => {
                            return make_error_embed("scheduler_config::run", format!("The parameter hour given has a bad format.\nExpected a string."));
                        }
                    }
                } else {
                    return make_error_embed("scheduler_config::run", format!("The parameter hour is empty."));
                }
            }
            _ => {
                println!("Unknown option name.\n{:?}", option);
                // Handle unknown option names
            }
        }
    }

    // if there is value, add the column name and the value to prepare the sql query
    if message_channel.is_some() {
        message_channel_id = message_channel.unwrap().id.0;
        updated_columns.push("channel");
        updated_values.push(message_channel_id as usize);
    }

    if hour.is_some() {
        updated_columns.push("hour");
        updated_values.push(hour.unwrap());
    }

    if min.is_some() {
        updated_columns.push("minute");
        updated_values.push(min.unwrap());
    }

    // 2 - Prepare the sql query
    let mut set_clause_tmp: Vec<String> = Vec::with_capacity(3);
    for id in 0..updated_columns.len() {
        set_clause_tmp.push(format!("{}='{}'",updated_columns.get(id).unwrap(), updated_values.get(id).unwrap()));
    }
    let set_clause = set_clause_tmp.join(", ");

    // 3 - Insert the new tip in the database and return a response message
    let mut respons_embed = match conn.lock().await.call(move |conn| {
        // Check if the config object exist in db :
        let config_opt: Option<usize> = conn.query_row("SELECT id FROM scheduler_config WHERE id = ?1", params![CONFIG_ID], |row|{
            row.get(0)
        }).optional()?;

        // Config exist : update it
        if let Some(_) = config_opt
        {
            let query = &format!("UPDATE scheduler_config SET {} WHERE id = {}", set_clause, CONFIG_ID);
            conn.execute(
                query,
                [],
            )?;
        }else{
            // check if all args are present
            if updated_columns.len() != 3 {
                return Err(InvalidParameterCount(updated_columns.len(),3));
            }

            let query = &*format!("INSERT INTO scheduler_config (id, channel, hour, minute) VALUES (?1,?2,?3,?4)");
            conn.execute(query, params![CONFIG_ID.to_string(), message_channel_id, hour, min])?;
        }


        // Get the final config object:
        let mut stmt = conn.prepare("SELECT channel, hour, minute FROM scheduler_config WHERE id = ?1")?;
        let row_data = stmt.query_row([CONFIG_ID.to_string()], |row|
            Ok(
                SchedulerConfig{
                    channel: row.get(0)?,
                    hour: row.get(1)?,
                    minute: row.get(2)?,
                }
            )
        )?;

        // Return the config found or an rusqlite::Error
        Ok::<_, Error>(row_data)
    }).await {
        Ok(config) => {
            // Display the config in the response embed
            let channel: Mention = Channel(ChannelId::from(config.channel)); // transform the channel id in a channel mention "#channel_name"
            display_full_tip_in_embed(
                format!("Here is the new config of the tips scheduler :"),
                format!("- Channel : {}\n- Hour:{:02}H{:02}", channel, config.hour, config.minute),
                None
            )
        }
        Err(err) => {
            if let tokio_rusqlite::Error::Rusqlite(stmt_err) = &err {
                if let InvalidParameterCount(_,_) = stmt_err {
                    return CreateEmbed::default()
                        .title("Config not initialised !")
                        .description("For the first time you set the config, you need to provide every arguments (channel and hour).")
                        .timestamp(Timestamp::now())
                        .color(Color::from_rgb(255, 0, 0)).to_owned();
                }
            }

            make_error_embed("scheduler_config::run", err.to_string())
        }
    };

    // 4 - restart scheduler if he was running
    let is_scheduler_running = {
        let scheduler_write = scheduler.read().await;
        let mut finished: bool = true;
        if  scheduler_write.is_some() {
            finished = scheduler_write.as_ref().unwrap().is_finished();
        }
        !finished // scheduler is running if the task is not complete
    };

    if is_scheduler_running {
        stop(conn.clone(), scheduler.clone()).await;
        start(conn, scheduler, http.clone()).await;
        // custom the title embed if the scheduler has restart
        return respons_embed.author(|a|{
            a.name("Scheduler has been restarted to sync with the new configuration")
        }).to_owned()
    }
    // return the default embed instead
    respons_embed
}

/**
 * This method is the signature of the command /scheduler_config.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("scheduler_config").description("Set a new configuration for the tips scheduler.")
        .create_option(|option| {
            option
                .name("message_chanel")
                .description("The chanel where the bot should say the tips every day.")
                .kind(CommandOptionType::Channel)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("hour")
                .description("The hour and minutes when the message should be sent every day. format (24h): HH:mm")
                .kind(CommandOptionType::String)
                .required(false)
        })
}