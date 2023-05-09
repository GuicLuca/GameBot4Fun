use log::{debug, error};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use std::fmt::{Write};
use crate::database::SharedConnection;
use crate::utils::{display_minimized_tip, get_optional_param_from_options};

struct ListTip {
    title: String,
    tags: String
}


/**
 * This method is the execution of the command /tips_list.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return string, the response message
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> String {
    let mut response: String;

    let tags_string = get_optional_param_from_options(options, 0);

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


            let query: &str = &*format!("SELECT title, tags FROM tips WHERE tags LIKE {}", tags_placeholder);
            debug!("Query executed for tips_list : {}",query);
            // run the prepared query and return the result into a Vec<String>
            let mut stmt = conn.prepare(query)?;
            let rows_data = stmt.query_map([], |row|
                Ok(
                    ListTip{
                        title: row.get(0)?,
                        tags: row.get(1)?,
                    }
                )
            )?
                .collect::<Result<Vec<ListTip>, rusqlite::Error>>()?;

            // 3 - return avery row found in a Vec<String>
            Ok::<_, rusqlite::Error>(rows_data)
        }).await {
            Ok(tips) => {
                // 4 - Create the response message ...
                response = format!("Here is the list of created  `TIPS`  with tags  `{}`:\n", tags);
                // ... and add all tittles found
                for tip in tips {
                    match writeln!(response, "{}", display_minimized_tip(tip.title, Some(tip.tags))){
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write a new line in !tips_list command. Error:\n{}", err);
                        }
                    };
                }
                response
            }
            Err(err) => {
                response = format!("Failed to get the list of tips title. Error:\n{}", err);
                response
            }
        }
    }else{
        // There is no tag so return the full list

        // 2 - call database execution
        match conn.lock().await.call(|conn|{
            let mut stmt = conn.prepare("SELECT title, tags FROM tips")?;
            let rows_data = stmt.query_map([], |row|
                Ok(
                    ListTip{
                        title: row.get(0)?,
                        tags: row.get(1)?,
                    }
                )
            )?
                .collect::<Result<Vec<ListTip>, rusqlite::Error>>()?;

            // 3 - return avery row found in a Vec<String>
            Ok::<_, rusqlite::Error>(rows_data)
        }).await{
            Ok(rows_data) => {
                // 4 - Create the response message ...
                response = format!("Here is the list of created  `TIPS`:\n");
                // ... and fill it with titles founds
                for tip in rows_data {
                    match writeln!(response, "{}", display_minimized_tip(tip.title, Some(tip.tags))){
                        Ok(_) => {}
                        Err(err) => {
                            error!("Failed to write a new line in !tips_list command. Error:\n{}", err);
                        }
                    };
                }
                response
            }
            Err(err) => {
                response = format!("Failed to get the list of tips title. Error:\n{}", err);
                response
            }
        }
    }
}

/**
 * This method is the signature of the command /tips_list.
 * THis is here that we describe the name, the options, all
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