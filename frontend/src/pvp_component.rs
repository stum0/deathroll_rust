use crate::chat_bus::ChatBus;
use crate::routes::Route;
use crate::ws::WebsocketService;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::time::Duration;
use yew::platform::time::sleep;

use web_sys::window;
use web_sys::{Element, MouseEvent};

use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged};
use yew_router::prelude::*;

use yew::{html, Component, Html, NodeRef};

pub enum Msg {
    Roll,
    HandleMsg(String),
    Home,
}

pub enum WsMsg {
    Ping(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
struct GameMsg {
    roll_msg: Vec<String>,
}

pub struct PvPComponent {
    node_ref: NodeRef,
    ws: WebsocketService,
    feed: Vec<String>,
    _producer: Box<dyn Bridge<ChatBus>>,
    start_roll: String,
    status_msg: String,
    player_icon: String,
    spectator: bool,
    game_start: bool,
    reconnecting: String,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        if self.game_start {
            spawn_local(async move {
                let chat_main = node_ref.cast::<Element>().unwrap();

                chat_main.set_scroll_top(chat_main.scroll_height());
            })
        }
    }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &yew::Context<Self>) -> Self {
        let location = web_sys::window().unwrap().location();
        let url = location.href().unwrap();
        let url_split: Vec<&str> = url.split('/').collect();

        let roll_amount = url_split[4];

        let cb = {
            let link = ctx.link().clone();
            move |msg| link.send_message(Msg::HandleMsg(msg))
        };

        let mut game_tx: WebsocketService = WebsocketService::ws_connect();

        game_tx.tx.try_send(roll_amount.to_string()).unwrap();

        Self {
            node_ref: NodeRef::default(),
            ws: game_tx,
            feed: Vec::new(),
            _producer: ChatBus::bridge(Rc::new(cb)),
            start_roll: roll_amount.to_string(),
            status_msg: "".to_string(),
            player_icon: "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}".to_string(),
            spectator: false,
            game_start: false,
            reconnecting: "\u{1F7E2}".to_string(),
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let home = ctx.link().callback(move |_: MouseEvent| Msg::Home);

        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';
        let swords = "\u{2694}\u{FE0F} ";

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
        if !self.spectator && !self.game_start {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                    if !self.game_start {
                    <h3>{"1v1 "}{&self.reconnecting}</h3>
                    <h3>{"To invite someone to play, give this URL: "}</h3>
                    <h3>{url}</h3>
                    {"waiting for player 2 to join..."}
                  }
                  </div>
                </header>

                </div>
            </body>
                  }
        } else if !self.spectator && self.game_start {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                  </div>
                  <h3>{"1v1 "}{&self.reconnecting}</h3>
                  </header>
                <div>
                  <main class="msger-chat" ref={self.node_ref.clone()}>
                    <div class="dets-pvp">
                     {swords}{&self.start_roll}
                     if self.player_icon == "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}" {
                     <br/>
                     <div>{"you \u{1F9D9}\u{200D}\u{2642}\u{FE0F} joined the game"}
                     </div>
                     } else {
                      <br/>
                      <div>
                      {"player 1 \u{1F9D9}\u{200D}\u{2642}\u{FE0F} joined the game"}
                      </div>
                     }
                     if self.player_icon == "\u{1F9DF}" {
                      <div>
                      {"you \u{1F9DF} joined the game"}
                      </div>
                  } else {
                      <div>
                      {"player 2 \u{1F9DF} joined the game"}
                      </div>
                  }

                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div>
                              {" "}{name}
                            </div>
                          }
                        }).collect::<Html>()
                      }
                    </div>
                  </main>
                </div>
                <div>

                  <button onclick={on_click} class="roll-button">
                  {&self.player_icon}{"\u{1F3B2} "}{&self.status_msg}</button>
                </div>
              </div>
            </body>
                  }
        } else {
            html! {
              <body>
              <div>
                <header>
                  <div>
                    <button onclick={home}>{"deathroll.gg "}{skull}{roll_emoji}</button>
                    <h3>{"1v1 "}{&self.reconnecting}</h3>
                    <h3>{"The arena is full, you are spectating \u{1F50E}"}</h3>
                  </div>
                </header>
                <br/>
                <div>
                  <main class="msger-chat" ref={self.node_ref.clone()}>
                    <div>
                     {swords}{&self.start_roll}
                      {
                        self.feed.clone().into_iter().map(|name| {
                          html!{

                            <div>
                              {" "}{name}
                            </div>
                          }
                        }).collect::<Html>()
                      }
                    </div>
                  </main>
                </div>
              </div>
            </body>
                  }
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let roll = "rolling".to_string();
                self.ws.tx.try_send(roll).unwrap();

                self.scroll_top();

                true
            }
            Msg::HandleMsg(result) => {
                self.scroll_top();

                if result.contains("player_two_icon") {
                    self.player_icon = "\u{1F9DF}".to_string();
                } else if result.contains("spec") {
                    self.spectator = true;
                } else if result.contains("disconnect") {
                    let game_tx: WebsocketService = WebsocketService::ws_connect();
                    spawn_local(async move {
                        sleep(Duration::from_secs(2)).await;
                    });

                    self.reconnecting = "\u{1f534} reconnecting...".to_string();
                    self.ws = game_tx;
                } else if result.contains("reconn") {
                    self.reconnecting = "\u{1F7E2}".to_string();
                } else if result.contains("...") {
                    self.status_msg = result.to_string();
                } else if result.contains("!!!") {
                    self.status_msg = result.to_string();
                } else if result.contains("\u{1F480}") {
                    let feed: GameMsg = serde_json::from_str(&result).unwrap();
                    //sends message to gamechat vector but doesnt clear the status message
                    self.feed = feed.roll_msg;
                    self.game_start = true;
                } else if result.contains("start the game") {
                    //self.game_start = true;
                    self.status_msg = result.to_string();
                } else {
                    let feed: GameMsg = serde_json::from_str(&result).unwrap();
                    //sends message to gamechat vector
                    self.feed = feed.roll_msg;
                    self.game_start = true;
                    //clear status message
                    self.status_msg = "".to_string();
                }

                true
            }
            Msg::Home => {
                let navigator = ctx.link().navigator().unwrap();

                navigator.push(&Route::Home);

                true
            }
        }
    }
    fn destroy(&mut self, _ctx: &yew::Context<Self>) {
        let mut ws = self.ws.clone();
        spawn_local(async move {
            ws.close().await;
        });
        self.ws.tx.try_send("close".to_string()).unwrap();
    }
}
