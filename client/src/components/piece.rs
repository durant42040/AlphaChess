use crate::utils::Piece as PieceData;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct PieceProps {
    pub piece: Option<PieceData>,
    pub position: [u8; 2],
    pub is_selected: bool,
    pub on_click: Callback<[u8; 2]>,
    pub on_drag_start: Callback<[u8; 2]>,
    pub on_drop: Callback<[u8; 2]>,
}

#[function_component(Piece)]
pub fn piece(props: &PieceProps) -> Html {
    let position = props.position;
    let on_click = props.on_click.clone();
    let on_drag_start = props.on_drag_start.clone();
    let on_drop = props.on_drop.clone();

    let (rank, file) = (props.position[0], props.position[1]);
    let bg = if props.is_selected {
        "#689b5f"
    } else if (rank + file) % 2 == 1 {
        "#c78443"
    } else {
        "#fde6bf"
    };

    let drag_opacity = use_state(|| false);
    let drag_opacity_setter = drag_opacity.setter();

    let onclick = Callback::from(move |_| on_click.emit(position));
    let ondragover = Callback::from(move |e: DragEvent| {
        e.prevent_default();
        if let Some(dt) = e.data_transfer() {
            let _ = dt.set_drop_effect("move");
        }
    });
    let ondrop = Callback::from(move |e: DragEvent| {
        e.prevent_default();
        on_drop.emit(position);
    });
    let drag_setter_start = drag_opacity_setter.clone();
    let ondragstart = Callback::from(move |_| {
        on_drag_start.emit(position);
        drag_setter_start.set(true);
    });
    let drag_setter_end = drag_opacity_setter.clone();
    let ondragend = Callback::from(move |_| {
        drag_setter_end.set(false);
    });

    let piece_src = props
        .piece
        .as_ref()
        .map(|p| format!("/assets/{}{}.svg", p.color, p.piece_type));
    let opacity = if *drag_opacity { "0" } else { "1" };

    html! {
        <div
            class="piece"
            style={format!("background: {};", bg)}
            onclick={onclick}
            ondragover={ondragover}
            ondrop={ondrop}
        >
            if let Some(ref src) = piece_src {
                <img
                    class="image"
                    src={src.clone()}
                    draggable="true"
                    ondragstart={ondragstart}
                    ondragend={ondragend}
                    style={format!("opacity: {};", opacity)}
                    alt="piece"
                />
            }
        </div>
    }
}
