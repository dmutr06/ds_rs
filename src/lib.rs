use serenity::model::prelude::Message;

pub mod commands;

pub fn check_msg(result: serenity::Result<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
