use crate::components::piece::Piece;
use crate::utils::Piece as PieceData;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct RowProps {
    pub row_positions: Vec<Option<PieceData>>,
    pub rank: u8,
    pub game: char,
    pub position_from: Option<[u8; 2]>,
    pub on_click: Callback<[u8; 2]>,
    pub on_drag_start: Callback<[u8; 2]>,
    pub on_drop: Callback<[u8; 2]>,
}

#[function_component(Row)]
pub fn row(props: &RowProps) -> Html {
    html! {
        <div class="row">
            { for props.row_positions.iter().enumerate().map(|(file, piece)| {
                let position = [props.rank, file as u8];
                let is_selected = props.position_from.map(|p| p == position).unwrap_or(false);
                let piece = piece.clone();
                let on_click = props.on_click.clone();
                let on_drag_start = props.on_drag_start.clone();
                let on_drop = props.on_drop.clone();
                html! {
                    <div class="square" key={file}>
                        <Piece
                            {piece}
                            {position}
                            {is_selected}
                            {on_click}
                            {on_drag_start}
                            {on_drop}
                        />
                    </div>
                }
            })}
        </div>
    }
}
