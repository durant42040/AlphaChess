use crate::components::row::Row;
use crate::utils;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ChessBoardProps {
    pub board: Vec<Vec<Option<utils::Piece>>>,
    pub game: char,
    pub position_from: Option<[u8; 2]>,
    pub on_click: Callback<[u8; 2]>,
    pub on_drag_start: Callback<[u8; 2]>,
    pub on_drop: Callback<[u8; 2]>,
}

#[function_component(ChessBoard)]
pub fn chessboard(props: &ChessBoardProps) -> Html {
    let order: Vec<usize> = if props.game == 'w' {
        (0..8).collect()
    } else {
        (0..8).rev().collect()
    };

    html! {
        <div>
            { for order.into_iter().map(|i| {
                let rank = i as u8;
                let row_positions = props.board[i].clone();
                let game = props.game;
                let position_from = props.position_from;
                let on_click = props.on_click.clone();
                let on_drag_start = props.on_drag_start.clone();
                let on_drop = props.on_drop.clone();
                html! {
                    <Row
                        key={i}
                        {row_positions}
                        {rank}
                        {game}
                        {position_from}
                        {on_click}
                        {on_drag_start}
                        {on_drop}
                    />
                }
            })}
        </div>
    }
}
