use std::sync::Arc;
use std::time::Duration;
use chrono::{Local, Timelike};
use log::error;
use rand::{Rng, thread_rng};
use rusqlite::{Error};
use rusqlite::Error::InvalidParameterCount;
use serenity::builder::{CreateApplicationCommand,CreateEmbed};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::model::mention::Mention;
use serenity::model::mention::Mention::Channel;
use serenity::model::prelude::command::{CommandOptionType};
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Color;
use tokio::time::sleep;
use crate::commands::tips_scheduler::config::{CONFIG_ID, SchedulerConfig};
use crate::database::SharedConnection;
use crate::SharedJoinHandle;
use crate::utils::{display_full_tip_in_embed, get_required_string_param_from_options, make_error_embed};


/*
This structure is used to group fetched data
from the database and then iterate over vec<Tip>
 */
#[derive(Clone)]
struct Tip {
    title: String,
    content: String,
    tags: String
}

/**
 * This method is the execution of the command /tips_create.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 * @param scheduler_status: SharedJoinHandle, the joinHandle of the scheduler to perform action on it.
 * @param http: &Arc<Http>, Http element used to send message on the discord server.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection, scheduler_status: SharedJoinHandle, http: &Arc<Http>) -> CreateEmbed {
    // 1 - get action value to chose the procedure to execute :
    let action = &*match get_required_string_param_from_options(options, 0, "action") {
        Ok(val) => {val}
        Err(err) => {
            return make_error_embed("scheduler::run", err.to_string());
        }
    };

    // Return the embed resulting of the procedure executed
    return match action {
        "start" => {
            start(conn, scheduler_status,http.clone()).await
        },
        "stop" => {
            stop(conn, scheduler_status).await
        },
        "info" => {
            info(conn, scheduler_status).await
        },
        _ => {
            // Action invalid or not implemented
            CreateEmbed::default()
                .title(format!("Action  `{}`  not implemented :(", action))
                .colour(Color::from_rgb(255, 0, 0))
                .description("Please retry later. If you think it's an error contact the administrator of the server.")
                .timestamp(Timestamp::now())
                .to_owned()
        }
    };
}

/**
 * Action START : start the tips scheduler with the current configuration.
 *
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 * @param scheduler_status: SharedJoinHandle, the joinHandle of the scheduler to perform action on it.
 * @param http: &Arc<Http>, Http element used to send message on the discord server.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn start(conn: SharedConnection, scheduler_status: SharedJoinHandle, http: Arc<Http>) -> CreateEmbed
{
    return match conn.lock().await.call(move |conn| {
        // Get the config object to pass it to the async task:
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

        // return the element found or an rusqlite::Error
        Ok::<_, Error>(row_data)
    }).await {
        Ok(config) => {
            // Successfully found a configuration :
            // Spawn a tips_scheduler async task
            {
                let mut scheduler_mut =scheduler_status.write().await;
                let task_conn = conn.clone();
                let handler = tokio::spawn(async move {
                    // While task not aborted or crashed:
                    loop {
                        // Check if time is equal to the config time
                        let now = Local::now();
                        println!("now : {}:{}    config: {}:{}", now.hour(), now.minute(), config.hour, config.minute);
                        if now.hour() == config.hour && now.minute() == config.minute {
                            // It's time to send a tips !!
                            // Get all tips from the database
                            match task_conn.lock().await.call(|conn|{
                                let mut stmt = conn.prepare("SELECT title, content, tags FROM tips")?;
                                let rows_data = stmt.query_map([], |row|
                                    Ok(
                                        Tip{
                                            title: row.get(0)?,
                                            content: row.get(1)?,
                                            tags: row.get(2)?,
                                        }
                                    )
                                )?
                                    .collect::<Result<Vec<Tip>, Error>>()?;

                                // return avery rows found in a Vec<Tip>
                                Ok::<_, Error>(rows_data)
                            }).await{
                                Ok(rows_data) => {
                                    // List of tip successfully fetched :
                                    // Select a random one to display.
                                    let tip = rows_data[thread_rng().gen_range(0..rows_data.len())].clone();
                                    // Send the message
                                    if let Err(why) = ChannelId::from(config.channel).send_message(&http, |m| {
                                        m.set_embed(
                                            display_full_tip_in_embed(tip.title, tip.content, Some(tip.tags))
                                        )
                                    }).await {
                                        error!("Failed to send embed message. Error:\n{}", why);
                                    }
                                }
                                Err(err) => {
                                    // Failed to fetch tips from database
                                    let msg = ChannelId::from(config.channel).send_message(&http, |m| {
                                        m.set_embed(
                                            make_error_embed(
                                                "scheduler::run",
                                                format!("Failed to get the list of tips title. Error:\n{}", err),
                                            )
                                        )
                                    }).await;

                                    if let Err(why) = msg {
                                        error!("Failed to send embed message. Error:\n{}", why);
                                    }
                                }
                            }
                        }

                        // Delay for a minute before checking the time again
                        sleep(Duration::from_secs(60)).await;
                    }
                });
                // Set the scheduler JoinHandle to keep control on it even after the end of this command
                *scheduler_mut = Some(handler);
            } // End spawn task

            // return the response embed with the current config and the scheduler status
            let channel: Mention = Channel(ChannelId::from(config.channel));
            display_full_tip_in_embed(
                format!("Scheduler is now running:"),
                format!("- Channel : {}\n- Hour:{:02}H{:02}", channel, config.hour, config.minute),
                None
            )
        }
        Err(err) => {
            // fail to get the config
            if let tokio_rusqlite::Error::Rusqlite(stmt_err) = &err {
                if let InvalidParameterCount(_,_) = stmt_err {
                    return CreateEmbed::default()
                        .title("Config not initialised !")
                        .description("Use the command  `/scheduler_config`  and fulfill all parameters before running the scheduler.")
                        .timestamp(Timestamp::now())
                        .color(Color::from_rgb(255, 0, 0)).to_owned();
                }
            }

            make_error_embed("scheduler::run", err.to_string())
        }
    };
}

/**
 * Action STOP : stop the tips scheduler.
 *
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 * @param scheduler_status: SharedJoinHandle, the joinHandle of the scheduler to perform action on it.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn stop(conn: SharedConnection, scheduler_status: SharedJoinHandle) -> CreateEmbed
{
    // Stop the task and drop the joinHandle
    {
        let mut scheduler_write = scheduler_status.write().await;
        if  scheduler_write.is_some() {
            scheduler_write.as_ref().unwrap().abort();
            *scheduler_write = None;
        }
    }
    // Return the current info of the scheduler but change the title.
    info(conn, scheduler_status).await.title("Scheduler is now stopped").to_owned()
}

/**
 * Action INFO : Show every information about the tips scheduler.
 *
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 * @param scheduler_status: SharedJoinHandle, the joinHandle of the scheduler to perform action on it.
 *
 * @return CreateEmbed, the embed message to say in response
 */
async fn info(conn: SharedConnection, scheduler_status: SharedJoinHandle) -> CreateEmbed
{
    return match conn.lock().await.call(move |conn| {
        // Get the config object:
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

        // Return the SchedulerConfig found or a rusqlite::Error instead
        Ok::<_, Error>(row_data)
    }).await {
        Ok(config) => {
            // Display the configuration fetched
            let channel: Mention = Channel(ChannelId::from(config.channel));
            let status = {
                let scheduler_read = scheduler_status.read().await;
                if scheduler_read.is_some(){
                    if !scheduler_read.as_ref().unwrap().is_finished(){
                        "RUNNING"
                    }else{
                        "STOPPED"
                    }
                }else{
                    "STOPPED"
                }
            };
            display_full_tip_in_embed(
                format!("He is the current configuration of the tips scheduler :"),
                format!("- Channel : {}\n- Hour:{:02}H{:02}\n- Scheduler :{}", channel, config.hour, config.minute, status),
                None
            )
        }
        Err(err) => {
            // Can't find any configuration
            if let tokio_rusqlite::Error::Rusqlite(stmt_err) = &err {
                if let InvalidParameterCount(_,_) = stmt_err {
                    return CreateEmbed::default()
                        .title("Config not initialised !")
                        .description("Use the command  `/scheduler_config`  and fulfill all parameters.")
                        .timestamp(Timestamp::now())
                        .color(Color::from_rgb(255, 0, 0)).to_owned();
                }
            }

            make_error_embed("scheduler::run", err.to_string())
        }
    };
}

/**
 * This method is the signature of the command /scheduler.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("scheduler").description("Set a new configuration for the tips scheduler.")
        .create_option(|option| {
            option
                .name("action")
                .description("The action you want the scheduler execute.")
                .kind(CommandOptionType::String)
                .required(true)
                .add_string_choice("Start", "start")
                .add_string_choice("Stop", "stop")
                .add_string_choice("Info", "info")
        })
}