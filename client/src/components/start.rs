use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct StartPageProps {
    pub on_choose: Callback<char>,
}

#[function_component(StartPage)]
pub fn start_page(props: &StartPageProps) -> Html {
    let on_choose = props.on_choose.clone();
    let on_white = Callback::from(move |_| on_choose.emit('w'));
    let on_choose_black = props.on_choose.clone();
    let on_black = Callback::from(move |_| on_choose_black.emit('b'));
    let on_choose_random = props.on_choose.clone();
    let on_random = Callback::from(move |_| {
        let c = if js_sys::Math::random() < 0.5 {
            'w'
        } else {
            'b'
        };
        on_choose_random.emit(c);
    });

    html! {
        <div class="start">
            <h1>{ "AlphaChess" }</h1>
            <div>{ "Choose Side" }</div>
            <div>
                <button class="button" onclick={on_white}>{ "White" }</button>
                <button class="button" onclick={on_black}>{ "Black" }</button>
                <button class="button" onclick={on_random}>{ "Random" }</button>
            </div>
        </div>
    }
}
