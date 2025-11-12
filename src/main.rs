use discord_qqmusic_bot::bot::*;
use discord_qqmusic_bot::qqmusic::*;
use discord_qqmusic_bot::error::*;
use discord_qqmusic_bot::structs::*;

use dotenvy::dotenv;
use tokio::sync::mpsc::{self, Sender, Receiver};
use log::{info, error,debug,warn};
use std::sync::Arc;

#[tokio::main]
async fn main () {

    
    dotenv().ok();
    env_logger::init();

    let (tx, mut rx) = mpsc::channel(100);

    let qqmusic_instance = Arc::new(QQMusic::new().await.unwrap());

    tokio::spawn(async move {

        let mut app = Bot::new(tx).await.unwrap();
        app.client.start().await.unwrap();
    });


    loop {

        let command = rx.recv().await.unwrap();

        debug!("Result = {:?}",command);

        let qqmusic_clone = Arc::clone(&qqmusic_instance);

        tokio::spawn(async move {

            let (ctx, msg, response_content) : (_,_,String) = match command {
                
                // Command Cancel match
                BotCommand::Cancel { ctx, msg } => {

                    let result = match Bot::stop_music(&ctx,&msg).await {

                        Ok(_) => {

                            info!("Success to cancel shit music");

                            "Sir, I suceess to cancle this shit music".to_string()
                        }

                        Err(_) => {

                            warn!("Failed to cancel music");
                            "Sir, I failed to cancel this shit music".to_string()
                        }
                    };


                    (ctx, msg, result)
                }

                // Command Search match
                BotCommand::Search { ctx, msg, Name } => {

                    // let result = "Got Command Search";

                    let playlist_table = qqmusic_clone.get_search_list(&Name).await.unwrap();

                    info!("Success to get search result");

                    (ctx, msg, playlist_table)
                }

                // Command Play match
                BotCommand::Play { ctx, msg, ID } => {

                    let result = "Got it! I'm playing this music".to_string();

                    // let id = String::from("002GwAma2DGN2x");

                    let url = qqmusic_clone.get_qqmusic_play_url(&ID).await.unwrap();

                    Bot::play_music(&ctx,&msg,&url).await.unwrap();

                    info!("Success to add music into queue");

                    (ctx, msg, result)
                }
            };

            msg.reply(&ctx, response_content).await.unwrap();
        });
    }


}