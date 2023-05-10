use rusqlite::params;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use crate::database::SharedConnection;
use crate::utils::{display_full_tip_in_embed, get_optional_string_param_from_options, get_required_number_param_from_options, make_error_embed};


/**
 * This method is the execution of the command /tips_create.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> CreateEmbed {
    let mut updated_columns:Vec<&str> = Vec::with_capacity(3); // we will add column name updated for the query creation
    let mut updated_values:Vec<String> = Vec::with_capacity(3); // we will add new values


    // 1 - check if optional values are present
    let title: String = String::from("");
    let content: String = String::from("");
    let tags: String = String::from("");


    for option in options {
        match option.name.as_str() {
            "tags" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(tags) => {
                            println!("Tags option value: {}", tags);
                            // Handle the tags value
                        }
                        _ => {
                            println!("Unsupported option value type for tags");
                            // Handle unsupported option value types for tags
                        }
                    }
                } else {
                    println!("Missing tags option value");
                    // Handle missing tags option value
                }
            }
            "content" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(content) => {
                            println!("Content option value: {}", content);
                            // Handle the content value
                        }
                        _ => {
                            println!("Unsupported option value type for content");
                            // Handle unsupported option value types for content
                        }
                    }
                } else {
                    println!("Missing content option value");
                    // Handle missing content option value
                }
            }
            _ => {
                println!("Unknown option name");
                // Handle unknown option names
            }
        }
    }

    /*let title: String = get_optional_string_param_from_options(options, 1);
    if title != "" {
        updated_columns.push("title");
        updated_values.push(title);
    }
    let content: String = get_optional_string_param_from_options(options, 2);
    if content != "" {
        updated_columns.push("content");
        updated_values.push(content);
    }
    let tags: String = get_optional_string_param_from_options(options, 3);
    if tags != "" {
        updated_columns.push("tags");
        updated_values.push(tags);
    }

    let tip_id = match get_required_number_param_from_options(options, 0, "id"){
        Ok(title) => title,
        Err(err) => return make_error_embed("tips_create::run", err),
    };
    let tip_id_clone = tip_id.clone();

     */

    let mut set_clause_tmp: Vec<String> = Vec::with_capacity(3);
    for id in 0..updated_columns.len()-1 {
        set_clause_tmp.push(format!("{}='{}'",updated_columns.get(id).unwrap(), updated_values.get(id).unwrap()))
    }
    let set_clause = set_clause_tmp.join(", ");


    // 3 - Insert the new tip in the database and return a response message
    return match conn.lock().await.call(move |conn| {
        let query = &*format!("UPDATE tips SET {} WHERE id = {}", set_clause, tip_id_clone);
        println!("{}", query);
        conn.execute(query, params![])?;
        Ok(())
    }).await {
        Ok(_) => {
            display_full_tip_in_embed(
                format!("Tip n°{} successfully updated", tip_id),
                format!("Enter `/tips_read {}` to see the new version of the tip", tip_id),
                None
            )
        }
        Err(err) => {
            make_error_embed("tips_create::run", err.to_string())
        }
    };
}

/**
 * This method is the signature of the command /tips_update.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_update").description("Update an already created tips.  /!\\ Warning: new values will override old ones !")
        .create_option(|option| {
        option
            .name("id")
            .description("The id of the tip you want to update.")
            .kind(CommandOptionType::Number)
            .required(true)
        })
        .create_option(|option| {
        option
            .name("title")
            .description("The title of the tip. It will be shown in tips list and on the top of the daily tips.")
            .kind(CommandOptionType::String)
            .required(false)
        })
        .create_option(|option| {
        option
            .name("content")
            .description("The body of the tip. This is here you must put the tip's message.")
            .kind(CommandOptionType::String)
            .required(false)
        })
        .create_option(|option| {
        option
            .name("tags")
            .description("Tags are used to sort tips. Format: lowercase csv with no spaces around coma. tag1,tag2,tag3,...")
            .kind(CommandOptionType::String)
            .required(false)
        })
}