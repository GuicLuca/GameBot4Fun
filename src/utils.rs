use serenity::builder::CreateEmbed;
use serenity::model::application::interaction::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::Timestamp;
use serenity::utils::Color;

/**
 * This method return the string value of the requested parameter.
 * If no value found or the index is out of bounds, a string error
 * is returned.
 *
 * @param options: &[CommandDataOption], The interaction options wrapper
 * @param index: usize, the index of the parameter in the options list
 * @param name: &str, the name of the parameter to customize error message
 *
 * @return Result<String, String>, the value of the parameter or the error message
 */
pub fn get_required_string_param_from_options(options: &[CommandDataOption], index: usize, name: &str,) -> Result<String, String>
{
    match options.get(index) {
        Some(option) => match &option.resolved {
            Some(resolved) => match resolved {
                CommandDataOptionValue::String(content) => Ok(content.to_owned()),
                _ => {
                    return Err(format!("Incorrect type for the parameter {}.", name));
                }
            },
            None => {
                return Err(format!("Missing parameter {}.", name));
            }
        },
        None => {
            return Err(format!("Missing parameter {}.", name));
        }
    }
}
/**
 * This method return the string value of the requested parameter.
 * If no value found or the index is out of bounds, a string error
 * is returned.
 *
 * @param options: &[CommandDataOption], The interaction options wrapper
 * @param index: usize, the index of the parameter in the options list
 * @param name: &str, the name of the parameter to customize error message
 *
 * @return Result<u64, String>, the value of the parameter or the error message
 */
pub fn get_required_number_param_from_options(options: &[CommandDataOption], index: usize, name: &str,) -> Result<u64, String>
{
    match options.get(index) {
        Some(option) => match &option.resolved {
            Some(resolved) => match resolved {
                CommandDataOptionValue::Number(content) => {
                    if content.fract() != 0.0 {
                        return Err(format!("Incorrect type for the parameter {}. It must be an integer >= 0", name));
                    }
                    Ok(content.to_owned() as u64)
                },
                _ => {
                    return Err(format!("Incorrect type for the parameter {}.", name));
                }
            },
            None => {
                return Err(format!("Missing parameter {}.", name));
            }
        },
        None => {
            return Err(format!("Missing parameter {}.", name));
        }
    }
}


/**
 * This method return the string value of the requested parameter.
 * If no value found or the index is out of bounds, a string error
 * is returned.
 *
 * @param options: &[CommandDataOption], The interaction options wrapper
 * @param index: usize, the index of the parameter in the options list
 *
 * @return String, the value of the parameter or an empty string instead
 */
pub fn get_optional_string_param_from_options(options: &[CommandDataOption], index: usize) -> String
{
    options.get(index)
        .map_or(String::from(""), |opt| {
            if let Some(CommandDataOptionValue::String(tmp)) = opt.resolved.to_owned() {
                return tmp;
            }
            String::from("")
        })
}

/**
 * This method return the string message needed to display properly
 * a tips in the chat.
 *
 * @param title: &str,
 * @param content: &str,
 * @param tags_string: &str,
 *
 * @return CreateEmbed, the embed containing the tip
 */
pub fn display_full_tip_in_embed(title: String, content: String, tags_opt: Option<String>) -> CreateEmbed
{
    let tags = tags_opt.unwrap_or_else(|| String::from(""));
    let mut embed = CreateEmbed::default()
        .title(title)
        .description(content)
        .timestamp(Timestamp::now())
        .color(Color::from_rgb(102, 255, 255))
        .to_owned();
    return if tags != "" {
        embed.footer(|f| {
            f.text(format!("#: {}", tags))
        })
    } else {
        &mut embed
    }.to_owned()

}
/**
 * This method return the string message needed to display
 * a tips in the chat as list style.  >" - title (tags)"
 *
 * @param id: u32,
 * @param title: &str,
 * @param tags_string: &str,
 *
 * @return String, the formatted message
 */
pub fn display_minimized_tip(id: u32, title: String, tags_string: Option<String>) -> String
{
    let tags = tags_string.unwrap_or_else(|| String::from(""));
    return if tags != "" {
        format!("*{}* - **{}**    #{}#",
            id,
            title,
            tags
        )
    }else{
        format!("*{}* - **{}**",
            id,
            title,
        )
    }

}

/**
 * This method make a generic embed for error messages.
 *
 * @param source: &str, The source of the error.
 * @param err: String, The error message.
 *
 * @return CreateEmbed, the embed displayed in the response
 */
pub fn make_error_embed(source: &str, err: String) -> CreateEmbed
{
    CreateEmbed::default()
        .title("Oups.. Something went wrong in the process :(")
        .description(
            format!("From {}, error:\n{}", source,err)
        ).timestamp(Timestamp::now())
        .color(Color::from_rgb(255, 0, 0))
        .to_owned()
}