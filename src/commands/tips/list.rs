use log::{debug, error};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};
use std::fmt::{Write};
use crate::database::SharedConnection;


/**
 * This method is the execution of the command /tips_list.
 * THis is here that all the workflow occure.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return string, the response message
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> String {
    let mut response: String;

    // 1 - Check if there is tags parameter
    if let Some(option) = options.get(0){
        // 2 - resolve the value to ensure the type is valid
        match &option.resolved {
            Some(CommandDataOptionValue::String(tags_brut)) => {
                // 3 - Clone tags string to give it to the database closer and keep the original for this function
                let tags: String = tags_brut.trim().to_string();
                let tags_clone = tags.clone();

                // 4 - call database execution
                match conn.lock().await.call(move |conn|{
                    // transform the tags string into a list
                    let tags_list: Vec<&str> = tags_clone.trim().split(',').collect();
                    // format tags to be in the query like this : SELECT ... ... LIKE '%unreal%' OR tags LIKE '%tools%'
                    let tags_placeholder = tags_list.iter().map(|i| format!("'%{}%'",i)).collect::<Vec<_>>().join(" OR tags LIKE ");


                    let query: &str = &*format!("SELECT title FROM tips WHERE tags LIKE {}", tags_placeholder);
                    debug!("Query executed for tips_list : {}",query);
                    // run the prepared query and return the result into a Vec<String>
                    let mut stmt = conn.prepare(query)?;
                    let titles = stmt.query_map([], |row| row.get(0))? // first element of the query title => "Select title From ..."
                        .collect::<Result<Vec<String>, rusqlite::Error>>()?;

                    Ok::<_, rusqlite::Error>(titles)
                }).await {
                    Ok(titles) => {
                        // 5 - Create the response message ...
                        response = format!("Here is the list of created  `TIPS`  with tags  `{}`:\n", tags);
                        // ... and add all tittles found
                        for title in titles {
                            match writeln!(response, "- {}", title){
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
            None => {
                format!("Parameter tags: Expected coma separated values with no spaces like this: tag1,tag2,tag3,...")
            }
            _ => {
                format!("Parameter tags: Expected coma separated values with no spaces like this: tag1,tag2,tag3,...")
            }
        }
    }else{
        // There is no tag so return the full list

        // 2 - call database execution
        match conn.lock().await.call(|conn|{
            let mut stmt = conn.prepare("SELECT title FROM tips")?;
            let titles = stmt.query_map([], |row| row.get(0))? // first element (title)
                .collect::<Result<Vec<String>, rusqlite::Error>>()?;

            // 3 - return avery row found in a Vec<String>
            Ok::<_, rusqlite::Error>(titles)
        }).await{
            Ok(titles) => {
                // 4 - Create the response message ...
                response = format!("Here is the list of created  `TIPS`:\n");
                // ... and fill it with titles founds
                for title in titles {
                    match writeln!(response, "- {}", title){
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