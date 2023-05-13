use log::{debug, error};
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use std::fmt::{Write};
use serenity::model::Timestamp;
use serenity::utils::Color;
use crate::database::SharedConnection;
use crate::utils::{display_minimized_tip, get_optional_string_param_from_options};

/*
This structure is used to group fetched data
from the database and then iterate over vec<ListTip>
 */
struct ListTip {
    id: u32,
    title: String,
    tags: String
}

/**
 * Method used by the /tips_list command only used
 * to create embed form the response message given
 * by the run function.
 *
 * @param title: String,
 * @param content; String,
 *
 * @return CreateEmbed, the embed itself
 */
fn embed_from_param(title: String, content: String)-> CreateEmbed
{
    CreateEmbed::default()
        .title(title)
        .colour(Color::from_rgb(0, 200, 55))
        .description(content)
        .timestamp(Timestamp::now())

        .to_owned()
}


/**
 * This method is the execution of the command /tips_list.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> CreateEmbed
{
    let tags_string = get_optional_string_param_from_options(options, 0);

    // 1 - Check if there is tags parameter
    if tags_string != "" {
        // 2 - Clone tags string to give it to the database closer and keep the original for this function
        let tags: String = tags_string.trim().to_string();
        let tags_clone = tags.clone();

        // 3 - call database execution
        match conn.lock().await.call(move |conn|{
            // transform the tags string into a list
            let tags_list: Vec<&str> = tags_clone.trim().split(',').collect();
            // format tags to be in the query like this : SELECT ... ... LIKE '%unreal%' OR tags LIKE '%tools%'
            let tags_placeholder = tags_list.iter().map(|i| format!("'%{}%'",i)).collect::<Vec<_>>().join(" OR tags LIKE ");


            let query: &str = &*format!("SELECT id, title, tags FROM tips WHERE tags LIKE {}", tags_placeholder);
            debug!("Query executed for tips_list : {}",query);
            // run the prepared query and return the result into a Vec<String>
            let mut stmt = conn.prepare(query)?;
            let rows_data = stmt.query_map([], |row|
                Ok(
                    ListTip{
                        id: row.get(0)?,
                        title: row.get(1)?,
                        tags: row.get(2)?,
                    }
                )
            )?
                .collect::<Result<Vec<ListTip>, rusqlite::Error>>()?;

            // return every rows found in a Vec<ListTip>
            Ok::<_, rusqlite::Error>(rows_data)
        }).await {
            Ok(tips) => {
                // 4 - Create the response message ...
                let mut response: String = "".to_string();
                // ... and add all tittles found
                for tip in tips {
                    match writeln!(response, "{}", display_minimized_tip(tip.id, tip.title, Some(tip.tags))){
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write a new line in !tips_list command. Error:\n{}", err);
                        }
                    };
                }
                embed_from_param(
                    format!("List of created  `TIPS`  with tags  `{}`", tags),
                    response,
                )
            }
            Err(err) => {
                embed_from_param(
                    format!("Failed to get the list of tips title."),
                    format!("Error:\n{}", err),
                )
            }
        }
    }else{
        // There is no tag so return the full list

        // 2 - call database execution
        match conn.lock().await.call(|conn|{
            let mut stmt = conn.prepare("SELECT id, title, tags FROM tips")?;
            let rows_data = stmt.query_map([], |row|
                Ok(
                    ListTip{
                        id: row.get(0)?,
                        title: row.get(1)?,
                        tags: row.get(2)?,
                    }
                )
            )?
                .collect::<Result<Vec<ListTip>, rusqlite::Error>>()?;

            // return every row found in a Vec<ListTip>
            Ok::<_, rusqlite::Error>(rows_data)
        }).await{
            Ok(rows_data) => {
                // 4 - Create the response message ...
                let mut response = "".to_string();
                // ... and fill it with titles founds
                for tip in rows_data {
                    match writeln!(response, "{}", display_minimized_tip(tip.id, tip.title, Some(tip.tags))){
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write a new line in !tips_list command. Error:\n{}", err);
                        }
                    };
                }
                embed_from_param(
                    format!("Here is the list of created  `TIPS`"),
                    response
                )
            }
            Err(err) => {
                embed_from_param(
                    format!("Failed to get the list of tips title."),
                    format!("Error:\n{}", err),
                )
            }
        }
    }
}

/**
 * This method is the signature of the command /tips_list.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_list").description("Show the list of every tips title")
        .create_option(|option| {
        option
            .name("tags")
            .description("The tag you want to search in tips list. Format:tag1,tag2,tag3,... Don't put spaces around coma!")
            .kind(CommandOptionType::String)
            .required(false)
    })
}