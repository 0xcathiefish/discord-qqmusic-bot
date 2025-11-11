use crate::error::*;
use crate::structs::*;

use reqwest::header::{HeaderMap, COOKIE, REFERER};
use reqwest::Client;
use serde_json::json;
use serenity::json::Value;

use prettytable::{Table, Row, Cell, row, format};

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

    pub async fn get_search_list(&self, keyword: &str) -> Result<String,QQMusicError> {

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

                let tabel_display = Self::_format_display(&playlist).await;

                Ok(tabel_display)
            }

            None => {

                error!("QQmusic: Failed to search list");
                return Err(QQMusicError::QQMusicPlaylistError);
            }
        }
    }

    async fn _format_display(playlist: &Vec<MusicPlayList>) -> String {

        if playlist.is_empty() {
            return "List is empty".to_string();
        }
    
        // 1. 定义你想要的固定宽度
        const ID_WIDTH: usize = 25;
        const NAME_WIDTH: usize = 30;
        const PLAYER_WIDTH: usize = 30;
    
        let mut table = Table::new();
    
        // 关键：我们不再设置表格的格式，而是直接创建无边框的表格
        // 后面手动绘制边框
        table.set_format(*format::consts::FORMAT_CLEAN);
    
    
        // 2. 创建被空格填充过的表头，以强制设定列宽
        // 使用 format! 宏来左对齐文本并填充空格
        let id_header = format!("{:<width$}", "ID", width = ID_WIDTH);
        let name_header = format!("{:<width$}", "Name", width = NAME_WIDTH);
        let player_header = format!("{:<width$}", "Player", width = PLAYER_WIDTH);
        
        table.add_row(row![b->id_header, b->name_header, b->player_header]);
    
        // 3. 遍历数据并添加行
        // 内容会自动被填充到与表头相同的宽度
        for item in playlist {
            // 为了防止内容过长撑破表格，可以手动截断
            let name = if item.name.len() > NAME_WIDTH {
                format!("{}...", &item.name[..NAME_WIDTH - 3])
            } else {
                item.name.clone()
            };
    
            let player = if item.player.len() > PLAYER_WIDTH {
                format!("{}...", &item.player[..PLAYER_WIDTH - 3])
            } else {
                item.player.clone()
            };
    
    
            table.add_row(Row::new(vec![
                Cell::new(&item.id),
                Cell::new(&name),
                Cell::new(&player),
            ]));
        }
        
        // 4. 返回被代码块包裹的字符串
        format!("```\n{}```", table.to_string())
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


    #[tokio::test] 
    pub async fn test_format_display() {

        dotenvy::dotenv().ok();
        env_logger::init();

        let playlist = vec![
            MusicPlayList {
                id: "C400000HnvQU05eTgI".to_string(),
                name: "晴天".to_string(),
                player: "周杰伦".to_string(),
            },
            MusicPlayList {
                id: "C400003lghpv0iXmD6".to_string(),
                name: "以父之名".to_string(),
                player: "周杰伦".to_string(),
            },
            MusicPlayList {
                id: "C400001aBvJ41eRkL".to_string(),
                name: "十年".to_string(),
                player: "陈奕迅".to_string(),
            },
        ];

        let table_string = QQMusic::_format_display(&playlist).await;
        info!("{}", table_string);
    }
}