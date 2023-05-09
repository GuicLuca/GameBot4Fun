use rusqlite::params;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use crate::database::SharedConnection;
use crate::utils::{display_full_tip, get_optional_param_from_options, get_required_param_from_options};


/**
 * This method is the execution of the command /tips_create.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return string, the response message
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> String {
    // 1 - check if optional values are present
    let tags: String = get_optional_param_from_options(options, 2);
    let tags_clone = tags.clone();

    // 2 - Get required param (title and content)
    let title = match get_required_param_from_options(options, 0, "title"){
        Ok(title) => title,
        Err(err) => return err,
    };
    let title_clone = title.clone();

    let content = match get_required_param_from_options(options, 1, "content"){
        Ok(content) => content,
        Err(err) => return err,
    };
    let content_clone = content.clone();

    // 3 - Insert the new tip in the database and return a response message
    return match conn.lock().await.call(move |conn| {
        let query = &*format!("INSERT INTO tips (title, content, tags) VALUES (?1,?2,?3)");
        conn.execute(query, params![title_clone, content_clone, tags_clone])?;
        Ok(())
    }).await {
        Ok(_) => {
            display_full_tip(title, content, Some(tags))
        }
        Err(err) => {
            format!("Failed to create the new tip. Error:\n{}", err)
        }
    };
}

/**
 * This method is the signature of the command /tips_create.
 * THis is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_create").description("Create a new tip.")
        .create_option(|option| {
        option
            .name("title")
            .description("The title of the tip. It will be shown in tips list and on the top of the daily tips.")
            .kind(CommandOptionType::String)
            .required(true)
        })
        .create_option(|option| {
        option
            .name("content")
            .description("The body of the tip. This is here you must put the tip's message.")
            .kind(CommandOptionType::String)
            .required(true)
        })
        .create_option(|option| {
        option
            .name("tags")
            .description("Tags are used to sort tips. Format: lowercase csv with no spaces around coma. tag1,tag2,tag3,...")
            .kind(CommandOptionType::String)
            .required(false)
        })
}