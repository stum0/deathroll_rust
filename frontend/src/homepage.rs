use nanoid::nanoid;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

pub struct Home {
    new_game: bool,
}

pub enum Msg {
    ShowNewGame,
    HideNewGame,
    // Other messages go here
}

impl Component for Home {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { new_game: false }
    }
    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let roll_emoji = '\u{1F3B2}';
        let skull = '\u{1F480}';

        let navigator = ctx.link().navigator().unwrap();
        let home = Callback::from(move |_: MouseEvent| navigator.push(&Route::Home));
        let navigator = ctx.link().navigator().unwrap();
        let pve = Callback::from(move |_: MouseEvent| navigator.push(&Route::PvE));
        let navigator = ctx.link().navigator().unwrap();
        let pvp = Callback::from(move |_: MouseEvent| {
            let id = nanoid!(8);

            navigator.push(&Route::PvP { id: id })
        });

        let new_game = ctx.link().callback(move |_: MouseEvent| Msg::ShowNewGame);
        let hide_new_game = ctx.link().callback(move |_: MouseEvent| Msg::HideNewGame);

        html! {
        <div class="app-body">
           <header class="header">
           <button onclick={home} class="title-button">{"deathroll.gg "}{skull}{roll_emoji}</button>
           <button onclick={pve} class="nav-button">{ "PvE" }</button>
           <button onclick={new_game}> {"PvP" }</button>
           if self.new_game {

                <div class="popup">
                    <p>{ "to start new 2v2 game, enter roll amount" }</p>
                    <button onclick={pvp}>{ "new game" }</button>
                    <button onclick={hide_new_game}>{ "cancel" }</button>
                </div>

        } else {
            {""}
        }
           </header>
        <br/>
           <div class="text">

           {"Players take turns rolling a die. The first player rolls the die and the number they roll becomes the maximum number for the next player's roll."}
           <br/>
           <br/>
           {"For example, if the first player rolls a 4, the second player can roll any number from 1 to 4."}
           <br/>
           <br/>
           {"This continues until a player rolls a 1, at which point they lose the game."}

           </div>


           <footer class="nav-bar-bottom">


           </footer>
        </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ShowNewGame => self.new_game = true,
            Msg::HideNewGame => self.new_game = false,
        }
        true
    }
}
