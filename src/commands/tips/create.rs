use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};

pub fn run(options: &[CommandDataOption]) -> String {
    match options.get(0) {
        None => {

        }
        Some(_) => {}
    }
    "".to_string()
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("tips_create").description("Create a new tips.")
        .create_option(|option| {
        option
            .name("tag")
            .description("The tag you want to search in tips list")
            .kind(CommandOptionType::String)
            .required(false)
    })
}