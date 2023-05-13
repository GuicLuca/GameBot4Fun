use rusqlite::params;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
};
use serenity::model::Timestamp;
use serenity::utils::Color;
use crate::database::SharedConnection;
use crate::utils::{get_required_integer_param_from_options, make_error_embed};

/**
 * This method is the execution of the command /tips_delete.
 * This is here that all the workflow occur.
 *
 * @param options: &[CommandDataOption], A slice of command option found in the interaction
 * @param conn: SharedConnection, the database access to run queries on the sqlite database.
 *
 * @return CreateEmbed, the embed message to say in response
 */
pub async fn run(options: &[CommandDataOption], conn: SharedConnection) -> CreateEmbed {
    // 1 - get parm values
    let tip_id: u64 = match get_required_integer_param_from_options(options, 0, "Id"){
        Ok(val) => val,
        Err(err) => {
            return make_error_embed("tips_delete::run", err.to_string())
        }
    };
    let conf_tip_id: u64 = match get_required_integer_param_from_options(options, 1, "confirm_id"){
        Ok(val) => val,
        Err(err) => {
            return make_error_embed("tips_delete::run", err.to_string())
        }
    };

    // 2 - Ensure val and confirmation are equal
    if tip_id != conf_tip_id {
        return CreateEmbed::default()
            .title("Tip id and confirmation are different !")
            .colour(Color::from_rgb(255, 204, 0))
            .description("Please confirm the id of the tip you want to delete.\nIf you think it's an error contact the administrator of the server.")
            .timestamp(Timestamp::now())
            .to_owned();
    }

    // 3 - Delete the tip from the database and return a response message
    return match conn.lock().await.call(move |conn| {
        let affected_row = conn.execute("DELETE FROM tips WHERE id = ?1", params![tip_id])?;

        // 3 - return avery row found in a Vec<String>
        Ok(affected_row)
    }).await {
        Ok(row) => {
            if row == 1 {
                CreateEmbed::default()
                    .title("Tip deleted successfully :)")
                    .colour(Color::from_rgb(102, 255, 51))
                    .description("Nothing to say so here is a smiley `◖ᵔᴥᵔ◗ ♪ ♫`")
                    .timestamp(Timestamp::now())
                    .to_owned()
            }else{
                CreateEmbed::default()
                    .title("Tip id unknown")
                    .colour(Color::from_rgb(255, 102, 51))
                    .description("The id requested is not valid. If you think this is an error, please contact server administrator")
                    .timestamp(Timestamp::now())
                    .to_owned()
            }
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
 * This method is the signature of the command /tips_delete.
 * This is here that we describe the name, the options, all
 * descriptions and hints of the method.
 *
 * @param command: &mut CreateApplicationCommand, The command object that handle the creation of new application commands.
 *
 * @return &mut CreateApplicationCommand, used to chain operations
 */
pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_delete").description("Delete.")
        .create_option(|option| {
            option
                .name("id")
                .description("The tip id you want to delete.")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("confirm_id")
                .description("The confirmationn of tip id you want to delete.")
                .kind(CommandOptionType::Integer)
                .required(true)
        })

}