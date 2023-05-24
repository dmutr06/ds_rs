use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use songbird::input::Restartable;
use youtube_dl::{SearchOptions, Playlist};

use crate::check_msg;

#[command]
#[aliases("p")]
#[only_in(guilds)]
pub async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut url = String::from(args.rest());

    if !url.starts_with("https://") && !url.starts_with("http://") {
        let playlist = match youtube_dl::YoutubeDl::search_for(&SearchOptions::youtube(url)).run_async().await {
            Ok(search) => search.into_playlist(),
            Err(why) => {
                println!("Error searching for vids:\n{}", why);

                check_msg(msg.reply(&ctx.http, "Something went wrong").await);

                return Ok(());
            }
        };

        if let Some(Playlist { entries, .. }) = playlist {
            match entries {
                Some(entries) => {
                    if entries.is_empty() {
                        check_msg(msg.reply(&ctx.http, "No video was found").await);

                        return Ok(());
                    }
                    url = entries[0].clone().webpage_url.unwrap();
                },
                None => {
                    check_msg(msg.reply(&ctx.http, "No video was found").await);

                    return Ok(());
                }
            }
        } else {
            check_msg(msg.reply(&ctx.http, "No video was found").await);

            return Ok(());
        }
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel_id = match channel_id {
        Some(channel_id) => channel_id,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let call = manager.get(guild_id);

    if call.is_none() {
        let (_, success) = manager.join(guild_id, channel_id).await;

        if let Ok(_channel) = success {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, &format!("Joined {}", channel_id.mention()))
                    .await,
            );
        } else {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx.http,
                        format!("Error joining the channel {}", channel_id.mention()),
                    )
                    .await,
            );

            return Ok(());
        }
    }

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        // Here, we use lazy restartable sources to make sure that we don't pay
        // for decoding, playback on tracks which aren't actually live yet.
        let source = match Restartable::ytdl(url, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                check_msg(msg.reply(&ctx.http, "Provide valid url").await);

                return Ok(());
            }
        };

        handler.enqueue_source(source.into());

        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Added song to queue: position {}", handler.queue().len()),
                )
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}
