use crate::error::*;
use crate::structs::*;

use reqwest::header::{HeaderMap, COOKIE, REFERER};
use reqwest::Client;
use serde_json::json;
use serenity::json::Value;

use std::env;
use log::{info, warn, error, debug};

pub struct QQMusic {

    pub client: Client,
}

impl QQMusic {

    // Intialize the client

    pub async fn new() -> Result<Self,QQMusicError> {

        let user_cookie = env::var("COOKIE").unwrap();
        let base_url = "https://y.qq.com/";

        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, user_cookie.parse().unwrap());
        headers.insert(REFERER, base_url.parse().unwrap());

        match Client::builder().default_headers(headers).build() {

            Ok(client) => {

                info!("QQmusic: Success to initialize the client");
                Ok(   QQMusic {client }  )
            },

            Err(_) => {

                error!("QQmusic: Failed to initialize the client");
                Err(QQMusicError::QQMusicClientError)
            }
        }
    }


    // Get the music record url

    pub async fn get_qqmusic_play_url(&self, songmid: &str) -> Result<String,QQMusicError> {

        let url = "https://u.y.qq.com/cgi-bin/musicu.fcg";

        let payload = json!({
            "req_1": {
                "module": "vkey.GetVkeyServer",
                "method": "CgiGetVkey",
                "param": {
                    "guid": "10000",
                    "songmid": [songmid],
                    "uin": "0",
                    "platform": "20"
                }
            }
        });

        let res = self.client.post(url).json(&payload).send().await.unwrap();

        let api_response: ApiResponse = res.json().await.unwrap();

        if let Some(sip) = api_response.req_1.data.sip.get(0) {

            if let Some(midurlinfo) = api_response.req_1.data.midurlinfo.get(0) {

                if !midurlinfo.purl.is_empty() {

                    let play_url = format!("{}{}", sip, midurlinfo.purl);

                    info!("QQmusic: Success to get music play url");
                    info!("{:?}",play_url);
                    return Ok(play_url);
                }
            }
        }

        error!("QQmusic: Failed to get music play url");

        Err(QQMusicError::QQMusicPlayError)
    }


    // Get search result of song's name

    pub async fn get_search_list(&self, keyword: &str) -> Result<Vec<MusicPlayList>,QQMusicError> {

        let mut playlist: Vec<MusicPlayList> = vec![];

        let url = "https://u.y.qq.com/cgi-bin/musicu.fcg";

        let payload = json!({
            "comm": {"ct": 24, "cv": 0},
            "req_1": {
                "module": "music.search.SearchCgiService",
                "method": "DoSearchForQQMusicDesktop",
                "param": {
                    "query": keyword,
                    "num_per_page": 10,
                    "page_num": 1,
                    "search_type": SearchType::Song as i32,
                    "remoteplace": "txt.yqq.top",
                },
            },
        });

        let res = self.client.post(url).json(&payload).send().await.unwrap();

        let json_response: Value = res.json().await.unwrap();

        let base_data = &json_response["req_1"]["data"]["body"];

        match base_data["song"]["list"].as_array() {

            Some(songs_list) => {

                for item in songs_list {

                    let singers: Vec<String> = item["singer"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|s| s["name"].as_str().unwrap_or("").to_string())
                    .collect();

                    playlist.push(MusicPlayList {
                        id: item["mid"].as_str().unwrap_or("").to_string(),
                        name: item["name"].as_str().unwrap_or("").to_string(),
                        player: singers.join(" / "),
                    });
                }

                info!("Found Play list");

                let mut count = 1;

                for song in &playlist {

                    debug!("{} {:?} {:?} {:?}",count,&song.name,&song.id,&song.player);
                    count = count + 1;
                }

                Ok(playlist)
            }

            None => {

                error!("QQmusic: Failed to search list");
                return Err(QQMusicError::QQMusicPlaylistError);
            }
        }
    }
}


#[cfg(test)]

mod tests {

    use super::*;

    #[tokio::test]
    pub async fn test_get_qqmusic_play_url() {

        dotenvy::dotenv().ok();
        env_logger::init();

        let app = QQMusic::new().await.unwrap();

        let songmid = "002GwAma2DGN2x";

        let _ = app.get_qqmusic_play_url(songmid).await;
    }


    #[tokio::test]
    pub async fn test_get_search_list() {

        dotenvy::dotenv().ok();
        env_logger::init();

        let app = QQMusic::new().await.unwrap();

        let keyword = "永不失联的爱";

        app.get_search_list(keyword).await.unwrap();
    }
}