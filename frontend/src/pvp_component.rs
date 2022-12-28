use crate::chat_bus::{ChatBus, Request};
use crate::routes::Route;
use crate::ws::ws_connect;
use regex::Regex;

use futures::{SinkExt, StreamExt};
use gloo_net::websocket::WebSocketError;
use gloo_net::websocket::{futures::WebSocket, Message};
use std::rc::Rc;

use web_sys::window;
use web_sys::{Element, MouseEvent};
use yew::platform::pinned::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yew::platform::spawn_local;
use yew_agent::{Bridge, Bridged, Dispatched};
use yew_router::prelude::*;

use yew::{html, Callback, Component, Html, NodeRef};

pub enum Msg {
    Roll,
    HandleMsg(String),
}

pub struct PvPComponent {
    node_ref: NodeRef,
    tx: UnboundedSender<Message>,
    feed: Vec<String>,
    _producer: Box<dyn Bridge<ChatBus>>,
    start_roll: String,
    status_msg: String,
    player: String,
}

impl PvPComponent {
    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();

            chat_main.set_scroll_top(chat_main.scroll_height());
        })
    }
}

impl Component for PvPComponent {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &yew::Context<Self>) -> Self {
        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        let url_split: Vec<&str> = url.split('/').collect();

        let start = "ws://".to_owned();
        let host = url_split[2];
        let ws = "/ws/";
        let game_id = url_split[3];
        let roll_amount = url_split[4];

        let full_url = start + host + ws + game_id;

        let ws = ws_connect(full_url);
        match ws {
            Ok(game_tx) => {
                game_tx
                    .send_now(Message::Text(String::from(roll_amount)))
                    .unwrap();

                let cb = {
                    let link = ctx.link().clone();
                    move |msg| link.send_message(Msg::HandleMsg(msg))
                };
                let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";

                Self {
                    node_ref: NodeRef::default(),
                    tx: game_tx,
                    feed: Vec::new(),
                    _producer: ChatBus::bridge(Rc::new(cb)),
                    start_roll: roll_amount.to_string(),
                    status_msg: "connected".to_string(),
                    player: p1.to_string(),
                }
            }
            Err(_) => {
                let cb = {
                    let link = ctx.link().clone();
                    move |msg| link.send_message(Msg::HandleMsg(msg))
                };
                let p1 = "\u{1F9D9}\u{200D}\u{2642}\u{FE0F}";

                Self {
                    node_ref: NodeRef::default(),
                    tx: ws.unwrap(),
                    feed: Vec::new(),
                    _producer: ChatBus::bridge(Rc::new(cb)),
                    start_roll: roll_amount.to_string(),
                    status_msg: "disconnected".to_string(),
                    player: p1.to_string(),
                }
            }
        }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));

        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

        let on_click = ctx.link().callback(move |_: MouseEvent| Msg::Roll);

        let window = window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();

        html! {
          <body>
          <div class="app-body">
            <header class="header">
              <div>
                <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
                <br/>
                <br/>
                {"To invite someone to play, give this URL: "}
                <br/>
                <br/>
                {url}
              </div>
            </header>
            <br/>
            <div class="msger">
              <main class="msger-chat" id="chat-main" ref={self.node_ref.clone()}>
                <div class="dets">
                 {"start roll: "}{&self.start_roll}
                  {
                    self.feed.clone().into_iter().map(|name| {
                      html!{

                        <div class="msg" >
                          {" "}{name}
                        </div>
                      }
                    }).collect::<Html>()
                  }
                </div>
              </main>
            </div>
            <div>
              <button onclick={on_click} class="roll-button">{&self.player}{roll_emoji}
              <div>{&self.status_msg}</div></button>
            </div>
          </div>
        </body>
              }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Roll => {
                let roll = "rolling".to_string();
                self.tx.send_now(Message::Text(String::from(roll))).unwrap();

                self.scroll_top();

                true
            }
            Msg::HandleMsg(result) => {
                self.scroll_top();
                let p2 = "\u{1F9DF}";
                let result_clone = result.clone();
                //log::debug!("result {:?}", result);
                let re = Regex::new(r"\d").unwrap();

                let contains_number = re.is_match(&result);

                if contains_number == true {
                    //sends message to gamechat vector
                    self.feed.push(result);

                    //clear status message
                    let clear_event = "";
                    self.status_msg = clear_event.to_string();
                } else {
                    //update status message
                    self.status_msg = result;
                }

                if result_clone.contains("waiting for \u{1F9D9}\u{200D}\u{2642}\u{FE0F}") {
                    self.player = p2.to_string();
                }

                true
            }
        }
    }
}
