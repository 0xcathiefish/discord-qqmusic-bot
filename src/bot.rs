use crate::error::BotError;
use crate::structs::BotCommand;
use crate::qqmusic::QQMusic;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

use songbird::SerenityInit;
use songbird::input::HttpRequest;
use songbird::input::Input;
use songbird::Call;
use songbird::input::cached::{Compressed, Memory};
use symphonia::*;


use tokio::sync::mpsc::{self, Sender, Receiver};
use log::{info, error,debug};
use std::env;
use dotenvy::dotenv;


pub struct Bot {

    pub client: Client,
}

impl Bot {

    pub async fn new(tx:Sender<BotCommand>) -> Result<Self, BotError> {

        let token = env::var("DISCORD_TOKEN").unwrap();
        let bot_id_str = env::var("DISCORD_BOT_ID").unwrap();
        let bot_id_u64 = bot_id_str.parse::<u64>().unwrap();
        let bot_id = UserId::new(bot_id_u64);

        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_VOICE_STATES
            | GatewayIntents::GUILD_MODERATION;
        
        let handler = Handler { bot_id,tx };

        match Client::builder(&token, intents).event_handler(handler).register_songbird().await {

            Ok(client) => {

                info!("Bot: Success to initialize the client");
                Ok(Bot { client })
            }

            Err(e) => {

                error!("Bot: Failed to initialize the client: {:?}",e);
                Err(BotError::BotClientError)
            }
        }
    }


    pub async fn play_music(ctx: &Context, msg: &Message, record_url: &str) -> Result<(), BotError> {
        

        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => return Err(BotError::BotAudioChannelError),
        };


        let manager = match songbird::get(ctx).await {
            Some(manager) => manager.clone(),
            None => return Err(BotError::BotPlayerError),
        };


        let channel_id = ctx.cache
            .guild(guild_id)
            .and_then(|guild| guild.voice_states.get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id));


        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                return Err(BotError::BotUserNotJoinChannelError);
            }
        };


        let handle_lock = match manager.join(guild_id, connect_to).await {
            Ok(handle) => handle,
            Err(e) => {
                return Err(BotError::BotJoinChannelError);
            }
        };


        debug!("Downloading the music: {}", record_url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0")
            .build()
            .map_err(|e| format!("Failed to build the downloader client: {}", e)).unwrap();

        let response = client.get(record_url)
            .send()
            .await
            .map_err(|e| format!("Failed to download with error: {}", e)).unwrap();

        if !response.status().is_success() {
            return Err(BotError::BotDownloadMusicError);
        }

        let bytes = response.bytes()
            .await
            .map_err(|e| format!("Failed to load data from memory with error: {}", e)).unwrap();

        debug!("Suucess to download the music: {} KB", bytes.len() / 1024);


        let source: Input = bytes.into();

        let mut handle = handle_lock.lock().await;

        debug!("Playing with: {}", record_url);

        let track = handle.play_input(source.into());


        let info = track.get_info().await
            .map_err(|e| format!("Failed to get record info: {:?}", e)).unwrap();

        debug!("Status: {:?}", info.playing);

        Ok(())
    }


}


struct Handler {
    bot_id: UserId,
    tx: Sender<BotCommand>,
}

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        
        let bot_id = self.bot_id;
        let mentioned_me = msg.mentions.iter().any(|user| user.id == bot_id);


        if msg.author.bot {
            return;
        }
        if !mentioned_me {
            return; 
        }


        let mention_string_normal = format!("<@{}>", bot_id.get());
        let mention_string_nick = format!("<@!{}>", bot_id.get());


        let mut content = msg.content.clone();
        content = content.replace(&mention_string_normal, "");
        content = content.replace(&mention_string_nick, "");


        let command_text = content.trim();

        let (command, args) = command_text.split_once(' ').unwrap_or((command_text, ""));

        let cmd_to_send = match command {

            "/cancel" => {

                Some(BotCommand::Cancel { 
                    ctx: ctx.clone(), 
                    msg: msg.clone() 
                })
            }


            "/search" => {

                let query = args.trim();

                if query.is_empty() {

                    let _ = msg.reply(&ctx, "Error! eg. @me /search 永不失联的爱").await;
                    None
                } 

                else {

                    Some(BotCommand::Search { 
                        ctx: ctx.clone(), 
                        msg: msg.clone(), 
                        Name: query.to_string() 
                    })
                }
            }


            "/play" => {

                let query = args.trim();

                if query.is_empty() {

                    let _ = msg.reply(&ctx, "Error! eg. @me /play 002GwAma2DGN2x").await;
                    None
                } 

                else {

                    Some(BotCommand::Play { 
                        ctx: ctx.clone(), 
                        msg: msg.clone(), 
                        ID: query.to_string() 
                    })
                }
            }

            _ => {

                let _ = msg.reply(&ctx, "Error: Unkown Command").await;
                None 
            }
        };


        if let Some(cmd) = cmd_to_send {

            if let Err(e) = self.tx.send(cmd).await {

                error!("Send Command Error: {:?}", e);
            }
        }

    }

    async fn ready(&self, _: Context, ready: Ready) {

        info!("{} Connected", ready.user.name);
    }
}



#[cfg(test)]
mod tests {

    use super::*;
    use tokio::sync::mpsc::{self, Sender, Receiver};

    #[tokio::test]
    async fn test_bot() {

        dotenv().ok();
        env_logger::init();

        let (tx, mut rx) = mpsc::channel(100);


        tokio::spawn(async move {

            let mut app = Bot::new(tx).await.unwrap();
            app.client.start().await.unwrap();
        });


        loop {

            let command = rx.recv().await.unwrap();

            info!("Result = {:?}",command);

            tokio::spawn(async move {

                let (ctx, msg, response_content) : (_,_,String) = match command {
                    
                    // Command Cancel match
                    BotCommand::Cancel { ctx, msg } => {

                        let result = "Sir, I can't cancle this shit music".to_string();
                        (ctx, msg, result)
                    }

                    // Command Search match
                    BotCommand::Search { ctx, msg, Name } => {

                        let result = "Got Command Search";

                        let playlist_table = QQMusic::new().await.unwrap().get_search_list(&Name).await.unwrap();

                        (ctx, msg, playlist_table)
                    }

                    // Command Play match
                    BotCommand::Play { ctx, msg, ID } => {

                        let result = "Got it! I'm playing this music".to_string();

                        let id = String::from("002GwAma2DGN2x");

                        let url = QQMusic::new().await.unwrap().get_qqmusic_play_url(&ID).await.unwrap();

                        Bot::play_music(&ctx,&msg,&url).await.unwrap();

                        (ctx, msg, result)
                    }
                };

                msg.reply(&ctx, response_content).await.unwrap();
            });
        }

    }

}