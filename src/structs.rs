use serde::Deserialize;


#[derive(Debug,Clone)]
pub struct MusicPlayList {

    pub id: String,
    pub name: String,
    pub player: String,
}


#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SearchType {
    Song = 0,
    SongList = 1,
    Album = 2,
    Singer = 3,
    Lyric = 7,
    Mv = 8,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub req_1: Req1,
}

#[derive(Debug, Deserialize)]
pub struct Req1 {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub sip: Vec<String>,
    pub midurlinfo: Vec<MidUrlInfo>,
}

#[derive(Debug, Deserialize)]
pub struct MidUrlInfo {
    pub purl: String,
}