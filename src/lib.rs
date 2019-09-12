use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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
    pending_item: String,
    items: Vec<Item>,
    pending_edit: Option<(usize, String)>,
}

#[derive(Default)]
struct Item {
    text: String,
    is_complete: bool,
}

#[derive(PartialEq,Clone,Debug)]
enum Message {
    Noop,
    UpdatePending(String),
    AddTodo,
    RemoveTodo(usize),
    ToggleTodo(usize),
}

impl Update<Message> for Todo {
    fn update(&mut self, msg: Message, _cmds: &mut Commands<Message>) {
        use Message::*;

        match msg {
            Noop => {}
            UpdatePending(text) => {
                self.pending_item = text
            }
            AddTodo => {
                self.items.push(Item {
                    text: self.pending_item.trim().to_owned(),
                    .. Item::default()
                });
                self.pending_item.clear();
            }
            RemoveTodo(i) => {
                self.items.remove(i);
            }
            ToggleTodo(i) => {
                self.items[i].is_complete = !self.items[i].is_complete;
            }
        }
    }
}

impl Render<dom::DomVec<Message>> for Todo {
    fn render(&self) -> dom::DomVec<Message> {
        use dom::Dom;
        use dom::Handler::Event;

        let mut vec = vec![];
        vec.push(Dom::elem("header")
            .attr("class", "header")
            .push(Dom::elem("h1").push("todos"))
            .push(Dom::elem("input")
                .attr("class", "new-todo")
                .attr("placeholder", "What needs to be done?")
                .attr("autofocus", "true")
                .attr("value", self.pending_item.to_owned())
                .on("input", dom::Handler::InputValue(|s| {
                    Message::UpdatePending(s)
                }))
                .on("keyup", Event(|e| {
                    let e = e.dyn_into::<web_sys::KeyboardEvent>().expect_throw("expected web_sys::KeyboardEvent");
                    match e.key().as_ref() {
                        "Enter" => Message::AddTodo,
                        _ => Message::Noop,
                    }
                }))
            )
        );

        // render todo list if necessary
        // XXX use css visibility here?
        if !self.items.is_empty() {
            // main section
            vec.push(Dom::elem("section")
                .attr("class", "main")
                .push(Dom::elem("input")
                    .attr("id", "toggle-all")
                    .attr("class", "toggle-all")
                    .attr("type", "checkbox")
                )
                .push(Dom::elem("label")
                    .attr("for", "toggle-all")
                    .push("Mark all as complete")
                )
                .push(Dom::elem("ul")
                    .attr("class", "todo-list")
                    .extend(self.items.iter().enumerate().map(|(i, item)| {
                        match self.pending_edit {
                            Some((pending_i, ref pending_edit)) if pending_i == i => {
                                item.render(i, Some(pending_edit))
                            }
                            Some(_) | None =>  {
                                item.render(i, None)
                            }
                        }
                    }))
                )
            );

            // todo footer
            vec.push(Dom::elem("footer")
                .attr("class", "footer")
                .push(Dom::elem("span")
                    .attr("class", "todo-count")
                    .push(Dom::elem("strong")
                        .push(self.items.len().to_string())
                    )
                    .push(
                        if self.items.len() == 1 { " item left" }
                        else { " items left" }
                    )
                )
            );
        }

        vec.into()
    }
}

impl Item {
    fn render(&self, i: usize, pending_edit: Option<&str>) -> dom::Dom<Message> {
        use dom::Dom;

        let e = Dom::elem("li");

        if let Some(pending_edit) = pending_edit {
            Dom::elem("input")
                .attr("class", "edit")
                .attr("value", pending_edit)
        }
        else {
            let e = e.push(
                Dom::elem("div")
                    .attr("class", "view")
                    .push(Dom::elem("input")
                        .attr("class", "toggle")
                        .attr("type", "checkbox")
                        .attr("checked", self.is_complete.to_string())
                        .event("change", Message::ToggleTodo(i))
                    )
                    .push(Dom::elem("label")
                        .push(self.text.to_owned())
                    )
                    .push(Dom::elem("button")
                        .attr("class", "destroy")
                        .event("click", Message::RemoveTodo(i))
                    )
            );

            if self.is_complete {
                e.attr("class", "completed")
            }
            else {
                e
            }
        }
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
    use super::*;

    #[test]
    fn add_todo() {
        let mut todomvc = Todo::default();

        let mut cmds = vec![];

        todomvc.update(Message::UpdatePending("item".to_owned()), &mut cmds);
        todomvc.update(Message::AddTodo, &mut cmds);

        assert_eq!(todomvc.items.len(), 1);
        assert_eq!(todomvc.items[0].text, "item");
        assert_eq!(todomvc.items[0].is_complete, false);
    }

    #[test]
    fn remove_todo() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item::default());

        let mut cmds = vec![];

        todomvc.update(Message::RemoveTodo(0), &mut cmds);

        assert_eq!(todomvc.items.len(), 0);
    }

    #[test]
    fn toggle_todo() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item::default());

        let mut cmds = vec![];

        todomvc.update(Message::ToggleTodo(0), &mut cmds);

        assert_eq!(todomvc.items[0].is_complete, true);
    }
}
