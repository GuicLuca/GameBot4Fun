use serenity::model::application::interaction::application_command::{CommandDataOption, CommandDataOptionValue};

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
pub fn get_required_param_from_options(options: &[CommandDataOption], index: usize, name: &str) -> Result<String, String>
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
 *
 * @return String, the value of the parameter or an empty string instead
 */
pub fn get_optional_param_from_options(options: &[CommandDataOption], index: usize) -> String
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
 * @return String, the formatted message
 */
pub fn display_full_tip(title: String, content: String, tags_string: Option<String>) -> String
{
    let tags = tags_string.unwrap_or_else(|| String::from(""));
    return if tags != "" {
        format!("**{}**\n{}\n\n# {}",
                title,
                content,
                tags
        )
    }else{
        format!("**{}**\n{}",
                title,
                content
        )
    }

}
/**
 * This method return the string message needed to display
 * a tips in the chat as list style.  >" - title (tags)"
 *
 * @param title: &str,
 * @param tags_string: &str,
 *
 * @return String, the formatted message
 */
pub fn display_minimized_tip(title: String, tags_string: Option<String>) -> String
{
    let tags = tags_string.unwrap_or_else(|| String::from(""));
    return if tags != "" {
        format!("- **{}**    #{}#",
                title,
                tags
        )
    }else{
        format!("- **{}**",
                title,
        )
    }

}