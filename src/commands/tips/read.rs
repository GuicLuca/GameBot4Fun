use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Color;
use crate::database::SharedConnection;
use crate::utils::{display_full_tip_in_embed, get_required_number_param_from_options, make_error_embed};


/*
This structure is used to group fetched data
from the database and then compute it
 */
struct ReadTip {
    title: String,
    content: String,
    tags: String
}

/**
 * This method is the execution of the command /tips_read.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> CreateEmbed {
    // 1 - check if optional values are present
    let tip_id: u64 = match get_required_number_param_from_options(options, 0, "Id"){
        Ok(val) => val,
        Err(err) => {
            return make_error_embed("tips_read::run", err.to_string())
        }
    };

    // 3 - Insert the new tip in the database and return a response message
    return match conn.lock().await.call(move |conn| {
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
        Ok(val) => {
            display_full_tip_in_embed(val.title, val.content, Some(val.tags))
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
            make_error_embed("tips_read::run", err.to_string())
        }
    };
}

/**
 * This method is the signature of the command /tips_read.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_read").description("Display a tip.")
        .create_option(|option| {
            option
                .name("id")
                .description("The tip id you want to see.")
                .kind(CommandOptionType::Number)
                .required(true)
        })
}