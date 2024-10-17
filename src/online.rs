use std::fmt::Debug;

use reqwest::{Client, Result};
use serde::{Deserialize, Serialize};

use crate::{input, Field, GameF, Square};

const SERVER_URL: &'static str = "http://127.0.0.1:8080";

#[derive(Debug, Deserialize)]
struct Response<T: Debug> {
    data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OnlineGame {
    field: Field,
    turn: User,
    winner: Option<User>,
}

#[derive(Debug)]
pub struct Online {
    game: OnlineGame,
    room: Room,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
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
    fn new(user_name: String, square: Square) -> RoomInfo {
        RoomInfo {
            name: input(),
            user: User::new(user_name, square),
        }
    }
}

async fn enter_room(client: Client, room: RoomInfo) -> Result<Response<Room>> {
    let res = client.post(SERVER_URL).json(&room).send().await?;
    res.json().await
}

async fn create_room(client: Client, room: RoomInfo) -> Result<Response<Room>> {
    let res = client.post(SERVER_URL).json(&room).send().await?;
    res.json().await
}

pub async fn online() {
    let client = Client::new();
    let name: String = input();

    let room = loop {
        match input::<String>().as_str() {
            "create" => {
                if let Ok(res) =
                    create_room(client.clone(), RoomInfo::new(name.clone(), Square::Maru)).await
                {
                    if let Some(r) = res.data {
                        break r;
                    } else {
                        eprintln!("create room error: {:?}", res);
                        continue;
                    }
                }
            }
            "enter" => {
                if let Ok(res) =
                    enter_room(client.clone(), RoomInfo::new(name.clone(), Square::Batu)).await
                {
                    if let Some(r) = res.data {
                        break r;
                    } else {
                        eprintln!("enetr room error: {:#?}", res);
                    }
                }
            }
            _ => continue,
        }
    };
    let mut online = Online {
        game: OnlineGame::new(room.clone().user1),
        room,
    };

    println!("{:#?}", online);

    online.game.start();
}

impl OnlineGame {
    fn new(user: User) -> OnlineGame {
        OnlineGame {
            field: [[None; 3]; 3],
            turn: user,
            winner: None,
        }
    }

    async fn sync(&self, client: Client) -> Result<Response<OnlineGame>> {
        let res = client.post(SERVER_URL).json(&self).send().await?;
        res.json().await
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

    #[tokio::main(flavor = "current_thread")]
    async fn start(&mut self) {
        loop {
            self.draw();
            if !self.turn(input()) {
                println!("input continue: not number");
                continue;
            }

            if self.check() {
                self.winner = Some(self.turn.clone());
                if let Ok(r) = self.sync(Client::new()).await {
                    println!("{:#?}", r.data)
                }
            }
            if let Ok(res) = self.sync(Client::new()).await {
                if let Some(online) = res.data {
                    if let Some(winner) = online.winner {
                        println!("winner!: {:#?}", winner);
                        break;
                    }
                }
            }
        }
        self.draw();
    }
}
