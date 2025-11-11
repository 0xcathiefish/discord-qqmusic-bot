use thiserror::Error;

#[derive(Debug,Error)]
pub enum BotError {

    #[error("DiscordBot: Failed to initialize client")]
    BotClientError,

}

#[derive(Debug,Error)]
pub enum QQMusicError{

    #[error("QQMusic: Failed to build client")]
    QQMusicClientError,

    #[error("QQMusic: Failed to get the music play url")]
    QQMusicPlayError,

    #[error("QQMusic: Failed to get playlist")]
    QQMusicPlaylistError,
}
