use log::debug;
use rusqlite::params;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Color;
use crate::commands::tips::read::ReadTip;
use crate::database::SharedConnection;
use crate::utils::{display_full_tip_in_embed, get_required_integer_param_from_options, make_error_embed};


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
    let mut title: String = String::from("");
    let mut content: String = String::from("");
    let mut tags: String = String::from("");

    for option in options {
        match option.name.as_str() {
            "tags" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(param) => {
                            tags = param.to_owned();
                        }
                        _ => {
                            return make_error_embed("tips_update::run", format!("The parameter tags given has a bad format.\nExpected lowercase csv with no spaces around coma. like this : tag,tag2,tag3,..."));
                        }
                    }
                } else {
                    return make_error_embed("tips_update::run", format!("The parameter tags is empty.\nExpected lowercase csv with no spaces around coma. like this : tag,tag2,tag3,..."));
                }
            }
            "content" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(param) => {
                            content = param.to_owned();
                        }
                        _ => {
                            return make_error_embed("tips_update::run", format!("The parameter content given has a bad format.\nExpected a string."));
                        }
                    }
                } else {
                    return make_error_embed("tips_update::run", format!("The parameter content is empty."));
                }
            }
            "title" => {
                if let Some(value) = &option.resolved {
                    match value {
                        CommandDataOptionValue::String(param) => {
                            title = param.to_owned();
                        }
                        _ => {
                            return make_error_embed("tips_update::run", format!("The parameter title given has a bad format.\nExpected a string."));
                        }
                    }
                } else {
                    return make_error_embed("tips_update::run", format!("The parameter title is empty."));
                }
            }
            _ => {
                println!("Unknown option name.\n{:?}", option);
                // Handle unknown option names
            }
        }
    }

    if title != "" {
        updated_columns.push("title");
        updated_values.push(title);
    }

    if content != "" {
        updated_columns.push("content");
        updated_values.push(content);
    }

    if tags != "" {
        updated_columns.push("tags");
        updated_values.push(tags);
    }

    // 2 - Get the id of the tip to update
    let tip_id = match get_required_integer_param_from_options(options, 0, "id"){
        Ok(title) => title,
        Err(err) => return make_error_embed("tips_create::run", err),
    };
    let tip_id_clone = tip_id.clone();

    // 3 - Prepare the sql query
    let mut set_clause_tmp: Vec<String> = Vec::with_capacity(3);
    for id in 0..updated_columns.len() {
        set_clause_tmp.push(format!("{}='{}'",updated_columns.get(id).unwrap(), updated_values.get(id).unwrap()));
    }
    let set_clause = set_clause_tmp.join(", ");


    // 4 - Insert the new tip in the database and return a response message
    return match conn.lock().await.call(move |conn| {
        if set_clause != ""{
            let query = &*format!("UPDATE tips SET {} WHERE id = {}", set_clause, tip_id_clone);
            debug!("Update query run : {}", query);
            conn.execute(query, params![])?;
        }

        let mut stmt = conn.prepare("SELECT title, content, tags FROM tips WHERE id = ?1")?;
        let row_data = stmt.query_row([tip_id], |row|
            Ok(
                ReadTip{
                    title: row.get(0)?,
                    content: row.get(1)?,
                    tags: row.get(2)?,
                }
            )
        )?;

        // 3 - return avery row found in a Vec<String>
        Ok::<_, rusqlite::Error>(row_data)
    }).await {
        Ok(tip) => {
            display_full_tip_in_embed(tip.title, tip.content, Some(tip.tags))
        }
        Err(err) => {
            if let tokio_rusqlite::Error::Rusqlite(rusqlite_err) = &err {
                if let rusqlite::Error::QueryReturnedNoRows = rusqlite_err {
                    return CreateEmbed::default()
                        .title("Tip id unknown")
                        .description("The id requested is not valid. If you think this is an error, please contact server administrator")
                        .timestamp(Timestamp::now())
                        .color(Color::from_rgb(255, 0, 0)).to_owned();
                }
            }
            make_error_embed("tips_update::run", err.to_string())
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
            .kind(CommandOptionType::Integer)
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