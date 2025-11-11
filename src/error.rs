use thiserror::Error;

#[derive(Debug,Error)]
pub enum BotError {

    #[error("DiscordBot: Failed to initialize client")]
    BotClientError,

    #[error("DiscordBot: Failed to load audio channel")]
    BotAudioChannelError,

    #[error("DiscordBot: Failed to load player")]
    BotPlayerError,

    #[error("DiscordBot: You need to join a channel first")]
    BotUserNotJoinChannelError,

    #[error("DiscordBot: Bot Failed to join the audio channel")]
    BotJoinChannelError,

    #[error("DiscordBot: Bot Failed to download the target music")]
    BotDownloadMusicError,
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
