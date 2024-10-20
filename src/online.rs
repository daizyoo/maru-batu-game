use std::fmt::Debug;

use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::*;

use crate::{input, Field, GameF, Square};

macro_rules! input {
    ($(  $print:expr => $name:ident ),*) => {
        $(
            println!($print);
            let $name = input();
        )*
    };
}

mod url {
    type Url<'a> = [&'a str; 2];
    const SERVER_URL: &str = "http://127.0.0.1:8080/";
    pub const CREATE_ROOM: Url = [SERVER_URL, "room/create"];
    pub const ENTER_ROOM: Url = [SERVER_URL, "room/enter"];
    pub const SYNC: Url = [SERVER_URL, "game/sync"];
    pub const WAIT: Url = [SERVER_URL, "game/wait"];

    pub fn to_url(url: [&str; 2]) -> String {
        url.concat()
    }
}

#[derive(Debug, Deserialize)]
struct Response<T: Debug> {
    data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OnlineGame {
    field: Field,
    turn: User,
    winner: Option<User>,
}

#[derive(Clone, Debug)]
pub struct Online {
    game: OnlineGame,
    room: Room,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    name: String,
    square: Square,
}

impl User {
    fn new(name: String, square: Square) -> User {
        User { name, square }
    }
}

// sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    name: String,
    user1: User,
    user2: User,
}

#[derive(Clone, Serialize)]
struct RoomInfo {
    name: String,
    user: User,
}

impl RoomInfo {
    fn new(room_name: &String, user_name: &String, square: Square) -> RoomInfo {
        RoomInfo {
            name: room_name.clone(),
            user: User::new(user_name.clone(), square),
        }
    }
}

async fn enter_room(room: RoomInfo) -> Result<Response<Room>> {
    let client = Client::new();
    let res = client.post(to_url(ENTER_ROOM)).json(&room).send().await?;
    res.json().await
}

async fn create_room(room: RoomInfo) -> Result<Response<Room>> {
    let client = Client::new();
    let res = client.post(to_url(CREATE_ROOM)).json(&room).send().await?;
    res.json().await
}

pub async fn online() {
    // input!(
    //     "user name input..." => user_name,
    //     "room name input..." => room_name
    // );
    let user_name = String::from("daizyoo");
    let room_name = String::from("room");

    println!("create or enter");
    let (room, my) = loop {
        match input::<String>().as_str() {
            "c" => {
                println!("wait enter...");
                if let Ok(res) =
                    create_room(RoomInfo::new(&room_name, &user_name, Square::Maru)).await
                {
                    if let Some(r) = res.data {
                        break (r.clone(), r.user1);
                    } else {
                        eprintln!("create room error: {:?}", res);
                        continue;
                    }
                }
            }
            "e" => {
                if let Ok(res) =
                    enter_room(RoomInfo::new(&room_name, &user_name, Square::Batu)).await
                {
                    if let Some(r) = res.data {
                        break (r.clone(), r.user2);
                    } else {
                        eprintln!("enetr room error: {:#?}", res);
                    }
                }
            }
            _ => (),
        }
        println!("error");
    };
    let mut online = Online {
        game: OnlineGame::new(room.clone().user1),
        room,
    };

    println!("{:#?}", online);

    online.start(my).await;
}

impl OnlineGame {
    fn new(user: User) -> OnlineGame {
        OnlineGame {
            field: [[None; 3]; 3],
            turn: user,
            winner: None,
        }
    }
}

impl Online {
    async fn sync(&self) -> Result<Response<bool>> {
        let res = Client::new()
            .post(to_url(SYNC))
            .json(&json!({
                "game": self.game,
                "room": self.room.name
            }))
            .send()
            .await?;
        res.json().await
    }

    async fn wait(&self) -> Result<Response<OnlineGame>> {
        let res = Client::new()
            .post(to_url(WAIT))
            .json(&json!({
                "name": self.room.name
            }))
            .send()
            .await?;
        res.json().await
    }

    async fn start(&mut self, my: User) {
        loop {
            self.game.draw();

            if self.game.turn != my {
                println!("wait turn...");
                if let Ok(sync) = self.wait().await {
                    self.game = sync.data.unwrap();
                    self.game.draw();
                } else {
                    panic!("wait error")
                }
            }

            if !self.game.turn(input()) {
                println!("input continue: not number");
                continue;
            }
            println!("turn");

            if self.game.check() {
                self.game.winner = Some(self.game.turn.clone());
                break;
            }

            if self.room.user1 == my {
                self.game.turn = self.room.user2.clone()
            } else {
                self.game.turn = self.room.user1.clone()
            }

            println!("sync");
            if let Ok(res) = self.sync().await {
                if !res.data.unwrap() {
                    panic!("sync error")
                }
            } else {
                panic!("sync error")
            }
        }
        self.game.draw();
    }
}

impl GameF for OnlineGame {
    fn field(&self) -> &Field {
        &self.field
    }
    fn field_mut(&mut self) -> &mut Field {
        &mut self.field
    }
    fn turn_square(&self) -> Square {
        self.turn.square
    }
    fn start(&mut self) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::test;

    const ROOM_NAME: &str = "room";

    #[test]
    async fn create() {
        tokio::spawn(async {
            create_room(RoomInfo {
                name: String::from(ROOM_NAME),
                user: User::new(String::from("daizyoo"), Square::Maru),
            })
            .await
            .unwrap()
            .data
            .unwrap();
        })
        .await
        .unwrap()
    }

    #[test]
    async fn create_and_enter() -> Result<()> {
        let room = enter_room(RoomInfo {
            name: String::from(ROOM_NAME),
            user: User::new(String::from("daizyoo2"), Square::Maru),
        })
        .await?
        .data
        .unwrap();

        let online = Online {
            game: OnlineGame::new(room.clone().user1),
            room,
        };
        let online2 = online.clone();
        tokio::spawn(async move {
            online2.clone().wait().await.unwrap();
        });

        let sync = online.sync().await.unwrap();

        // assert!(sync.data.unwrap() == true);

        Ok(())
    }
}
