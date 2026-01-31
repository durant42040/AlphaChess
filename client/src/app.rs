//! App component: state + message-driven update. Engine is accessed via server API.

use crate::api::{self, ActResponse, GenerateResponse};
use crate::components::{ChessBoard, StartPage};
use crate::sound;
use crate::state::{Msg, State};
use crate::utils::{move_string_with_promotion, to_board};
use gloo::render::request_animation_frame;
use gloo::timers::callback::Timeout;
use yew::events::KeyboardEvent;
use yew::prelude::*;

pub struct App {
    state: State,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        Self {
            state: State::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChooseSide(c) => {
                self.state.game = Some(c);
                sound::preload();
                sound::play("start");
                self.state.reset_board();
                let link = ctx.link().clone();
                yew::platform::spawn_local(async move {
                    let result = api::reset().await;
                    link.send_message(Msg::ResetDone(result));
                });
            }
            Msg::ResetDone(Ok(())) => {
                ctx.link().send_message(Msg::PollGame);
            }
            Msg::ResetDone(Err(e)) => {
                log::error!("reset failed: {}", e);
            }
            Msg::SquareClick(position) => {
                let b = &self.state.board;
                let pf = self.state.position_from;
                let piece = b
                    .get(position[0] as usize)
                    .and_then(|row| row.get(position[1] as usize))
                    .and_then(|p| p.as_ref());
                let game = self.state.game;
                if pf.is_none() {
                    if let Some(p) = piece {
                        if Some(p.color) == game {
                            self.state.position_from = Some(position);
                        }
                    }
                    return true;
                }
                let from = pf.unwrap();
                if crate::utils::is_equal(from, position) {
                    self.state.position_from = None;
                    return true;
                }
                if let Some(p) = piece {
                    let from_piece = b[from[0] as usize][from[1] as usize].as_ref().unwrap();
                    if p.color == from_piece.color {
                        self.state.position_from = Some(position);
                        return true;
                    }
                }
                self.state.position_to = Some(position);
                ctx.link().send_message(Msg::TryAct);
            }
            Msg::DragStart(position) => {
                let b = &self.state.board;
                if let Some(row) = b.get(position[0] as usize) {
                    if let Some(Some(p)) = row.get(position[1] as usize) {
                        if Some(p.color) == self.state.game {
                            self.state.position_from = Some(position);
                        }
                    }
                }
            }
            Msg::Drop(position) => {
                if self.state.position_from.is_some() {
                    self.state.position_to = Some(position);
                    ctx.link().send_message(Msg::TryAct);
                }
            }
            Msg::Rematch => {
                self.state.game = self.state.game.map(|g| if g == 'w' { 'b' } else { 'w' });
                self.state.reset_board();
                let link = ctx.link().clone();
                yew::platform::spawn_local(async move {
                    let result = api::reset().await;
                    link.send_message(Msg::ResetDone(result));
                });
            }
            Msg::Undo => {
                let link = ctx.link().clone();
                yew::platform::spawn_local(async move {
                    let result = api::undo().await.map(|r| (r.board, r.is_check));
                    link.send_message(Msg::UndoDone(result));
                });
            }
            Msg::UndoDone(Ok((board, is_check))) => {
                self.state.board = to_board(&board);
                self.state.side = self.state.game.unwrap_or('w');
                self.state.position_from = None;
                self.state.position_to = None;
                self.state.game_over = "No".to_string();
                if is_check {
                    sound::play("check");
                } else {
                    sound::play("move");
                }
            }
            Msg::UndoDone(Err(e)) => {
                log::error!("undo failed: {}", e);
            }
            Msg::TryAct => {
                if let (Some(from), Some(to)) = (self.state.position_from, self.state.position_to) {
                    let move_str =
                        move_string_with_promotion(&self.state.board, from, to);
                    let capture = self.state.board[to[0] as usize][to[1] as usize].is_some();
                    let link = ctx.link().clone();
                    yew::platform::spawn_local(async move {
                        let result = api::act(&move_str)
                            .await
                            .map(|r: ActResponse| (r.board, r.is_check, capture));
                        link.send_message(Msg::ActDone(result));
                    });
                }
            }
            Msg::ActDone(Ok((board, is_check, capture))) => {
                self.state.board = to_board(&board);
                self.state.side = if self.state.side == 'w' { 'b' } else { 'w' };
                self.state.position_from = None;
                self.state.position_to = None;
                let sound_name: String = if is_check {
                    "check".into()
                } else if capture {
                    "capture".into()
                } else {
                    "move".into()
                };
                let frame = request_animation_frame(move |_| sound::play(&sound_name));
                std::mem::forget(frame);
                ctx.link().send_message(Msg::PollGame);
            }
            Msg::ActDone(Err(e)) => {
                log::error!("act failed: {}", e);
                self.state.position_from = None;
                self.state.position_to = None;
            }
            Msg::PollGame => {
                let link = ctx.link().clone();
                yew::platform::spawn_local(async move {
                    let result = api::game_state().await;
                    link.send_message(Msg::GameStateDone(result));
                });
            }
            Msg::GameStateDone(Ok(game_state)) => {
                match game_state.as_str() {
                    "checkmate" => {
                        self.state.game_over = "Checkmate".to_string();
                        sound::play("checkmate");
                        return true;
                    }
                    "draw" => {
                        self.state.game_over = "Draw".to_string();
                        return true;
                    }
                    _ => {
                        self.state.game_over = "No".to_string();
                    }
                }
                let game = match self.state.game {
                    Some(g) => g,
                    None => return true,
                };
                if self.state.side == game {
                    return true;
                }
                let link = ctx.link().clone();
                Timeout::new(100, move || {
                    link.send_message(Msg::GenerateMove);
                })
                .forget();
            }
            Msg::GameStateDone(Err(e)) => {
                log::error!("game_state failed: {}", e);
            }
            Msg::GenerateMove => {
                let board_before = self.state.board.clone();
                let link = ctx.link().clone();
                yew::platform::spawn_local(async move {
                    let result = api::generate().await.map(|r: GenerateResponse| {
                        let to_sq = parse_to_square(&r.move_str);
                        let was_capture = to_sq
                            .map(|(row, col)| {
                                (row as usize) < 8
                                    && (col as usize) < 8
                                    && board_before[row as usize][col as usize].is_some()
                            })
                            .unwrap_or(false);
                        (r.board, r.is_check, was_capture)
                    });
                    link.send_message(Msg::GenerateDone(result));
                });
            }
            Msg::GenerateDone(Ok((board, is_check, capture))) => {
                self.state.board = to_board(&board);
                self.state.side = if self.state.side == 'w' { 'b' } else { 'w' };
                if is_check {
                    sound::play("check");
                } else if capture {
                    sound::play("capture");
                } else {
                    sound::play("move");
                }
                ctx.link().send_message(Msg::PollGame);
            }
            Msg::GenerateDone(Err(e)) => {
                log::error!("generate failed: {}", e);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_choose = ctx.link().callback(Msg::ChooseSide);
        let on_click = ctx.link().callback(Msg::SquareClick);
        let on_drag_start = ctx.link().callback(Msg::DragStart);
        let on_drop = ctx.link().callback(Msg::Drop);
        let rematch = ctx.link().callback(|_| Msg::Rematch);
        let handle_revert = ctx.link().callback(|_| Msg::Undo);
        let onkeydown = ctx.link().batch_callback(move |e: KeyboardEvent| {
            if e.key() == "ArrowLeft" {
                e.prevent_default();
                Some(Msg::Undo)
            } else {
                None
            }
        });

        if self.state.game.is_none() {
            return html! { <StartPage on_choose={on_choose} /> };
        }

        let game_over_visibility = if self.state.game_over == "No" {
            "hidden"
        } else {
            "visible"
        };

        html! {
            <div tabindex="0" onkeydown={onkeydown} class="app-root">
                <div class="head">
                    <h1 class="gameOver" style={format!("visibility: {};", game_over_visibility)}>
                        { self.state.game_over.clone() }
                    </h1>
                    <button class="rematch" onclick={rematch}>{ "Rematch" }</button>
                </div>
                <ChessBoard
                    board={self.state.board.clone()}
                    game={self.state.game.unwrap_or('w')}
                    position_from={self.state.position_from}
                    on_click={on_click}
                    on_drag_start={on_drag_start}
                    on_drop={on_drop}
                />
                <div class="footer">
                    <button class="rematch" onclick={handle_revert}>{ "← Undo" }</button>
                </div>
            </div>
        }
    }
}

/// Parse move string "e2e4" to destination square (row, col) in client board coords.
fn parse_to_square(move_str: &str) -> Option<(u8, u8)> {
    let bytes = move_str.as_bytes();
    if bytes.len() < 4 {
        return None;
    }
    let file = bytes[2].saturating_sub(b'a');
    let rank_char = bytes[3];
    if !rank_char.is_ascii_digit() {
        return None;
    }
    let rank_1_8 = (rank_char - b'0') as u8;
    if rank_1_8 == 0 || rank_1_8 > 8 {
        return None;
    }
    let row = 8 - rank_1_8;
    Some((row, file))
}
