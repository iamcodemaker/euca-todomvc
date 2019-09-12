use wasm_bindgen::prelude::*;
use cfg_if::cfg_if;
use log::{debug,info};
use euca::app::*;
use euca::dom;

cfg_if! {
    if #[cfg(feature = "console_error_panic_hook")] {
        #[inline]
        fn set_panic_hook() {
            console_error_panic_hook::set_once();
            debug!("panic hook set");
        }
    }
    else {
        fn set_panic_hook() {}
    }
}

cfg_if! {
    if #[cfg(feature = "console_log")] {
        #[inline]
        fn init_log() {
            console_log::init_with_level(log::Level::Trace)
                .expect("error initializing log");
            debug!("log initialized");
        }
    }
    else {
        fn init_log() {}
    }
}

#[derive(Default)]
struct Todo {
}

#[derive(PartialEq,Clone,Debug)]
enum Message {
}

impl Update<Message> for Todo {
    fn update(&mut self, msg: Message, _cmds: &mut Commands<Message>) {
    }
}

impl Render<dom::DomVec<Message>> for Todo {
    fn render(&self) -> dom::DomVec<Message> {
        let mut vec = vec![];
        vec.into()
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    init_log();
    set_panic_hook();

    let parent = web_sys::window()
        .expect("couldn't get window handle")
        .document()
        .expect("couldn't get document handle")
        .query_selector("section.todoapp")
        .expect("error querying for element")
        .expect("expected <section class=\"todoapp\"></section>");

    AppBuilder::default()
        .attach(parent, Todo::default());

    info!("Euca • TodoMVC initialized");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
