use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use cfg_if::cfg_if;
use log::{debug,info};
use euca::app::*;
use euca::dom;
use std::rc::Rc;
use std::cell::RefCell;

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

const TITLE: &str = "Euca • TodoMVC";

#[derive(PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::All
    }
}

#[derive(Default)]
struct Todo {
    pending_item: String,
    items: Vec<Item>,
    pending_edit: Option<(usize, String)>,
    filter: Filter,
}

#[derive(Default)]
struct Item {
    text: String,
    is_complete: bool,
}

#[derive(PartialEq,Clone,Debug)]
enum Message {
    Noop,
    FocusPending,
    UpdatePending(String),
    AddTodo,
    RemoveTodo(usize),
    ToggleTodo(usize),
    EditTodo(usize),
    FocusEdit,
    UpdateEdit(String),
    SaveEdit,
    AbortEdit,
    ClearCompleted,
    ToggleAll,
    ShowAll,
    ShowActive,
    ShowCompleted,
}

impl Update<Message> for Todo {
    fn update(&mut self, msg: Message, cmds: &mut Commands<Message>) {
        use Message::*;

        match msg {
            Noop => {}
            cmd @ FocusPending => {
                cmds.push(Command::new(cmd, focus_pending_input));
            }
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
            EditTodo(i) => {
                self.pending_edit = Some((i, self.items[i].text.clone()));
                self.update(FocusEdit, cmds);
            }
            cmd @ FocusEdit => {
                cmds.push(Command::new(cmd, focus_edit_input));
            }
            UpdateEdit(text) => {
                match self.pending_edit {
                    Some((_, ref mut pending_text)) => {
                        *pending_text = text;
                    }
                    _ => panic!("SaveEdit called with no pending edit"),
                }
            }
            SaveEdit => {
                match self.pending_edit {
                    Some((i, ref text)) => {
                        if text.trim().is_empty() {
                            self.update(RemoveTodo(i), cmds);
                        }
                        else {
                            self.items[i].text = text.trim().to_owned();
                        }
                        self.pending_edit = None;
                    }
                    _ => panic!("SaveEdit called with no pending edit"),
                }
            }
            AbortEdit => {
                self.pending_edit = None;
            }
            ClearCompleted => {
                self.items.retain(|item| !item.is_complete);
            }
            ToggleAll => {
                let all_complete = self.items.iter().all(|item| item.is_complete);

                for item in self.items.iter_mut() {
                    item.is_complete = !all_complete;
                }
            }
            ShowAll => {
                self.filter = Filter::All;
            }
            ShowActive => {
                self.filter = Filter::Active;
            }
            ShowCompleted => {
                self.filter = Filter::Completed;
            }
        }
    }
}

fn focus_pending_input(msg: Message, _: Rc<RefCell<dyn Dispatch<Message>>>) {
    match msg {
        Message::FocusPending => {
            let pending_input = web_sys::window()
                .expect("couldn't get window handle")
                .document()
                .expect("couldn't get document handle")
                .query_selector("section.todoapp header.header input.new-todo")
                .expect("error querying for element")
                .expect("expected to find an input element")
                .dyn_into::<web_sys::HtmlInputElement>()
                .expect_throw("expected web_sys::HtmlInputElement");

            pending_input.focus().expect_throw("error focusing input");
        }
        _ => unreachable!("focus_pending_input should only be called with FocusPending. Called with: {:?}", msg),
    }
}

fn focus_edit_input(msg: Message, _: Rc<RefCell<dyn Dispatch<Message>>>) {
    match msg {
        Message::FocusEdit => {
            let edit_input = web_sys::window()
                .expect("couldn't get window handle")
                .document()
                .expect("couldn't get document handle")
                .query_selector("section.todoapp section.main input.edit")
                .expect("error querying for element")
                .expect("expected to find an input element")
                .dyn_into::<web_sys::HtmlInputElement>()
                .expect_throw("expected web_sys::HtmlInputElement");

            edit_input.focus().expect_throw("error focusing input");
        }
        _ => unreachable!("focus_edit_input should only be called with FocusEdit. Called with: {:?}", msg),
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
                    .attr("checked", self.items.iter().all(|item| item.is_complete).to_string())
                    .event("change", Message::ToggleAll)
                )
                .push(Dom::elem("label")
                    .attr("for", "toggle-all")
                    .push("Mark all as complete")
                )
                .push(Dom::elem("ul")
                    .attr("class", "todo-list")
                    .extend(self.items.iter()
                        .filter(|item| {
                            match self.filter {
                                Filter::All => true,
                                Filter::Active => !item.is_complete,
                                Filter::Completed => item.is_complete,
                            }
                        })
                        .enumerate().map(|(i, item)| {
                            match self.pending_edit {
                                Some((pending_i, ref pending_edit)) if pending_i == i => {
                                    item.render(i, Some(pending_edit))
                                }
                                Some(_) | None =>  {
                                    item.render(i, None)
                                }
                            }
                        })
                    )
                )
            );

            // todo footer
            vec.push({
                let footer = Dom::elem("footer")
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
                    .push(Dom::elem("ul")
                        .attr("class", "filters")
                        .push(Dom::elem("li")
                            .push(Dom::elem("a")
                                .attr("href", "#/")
                                .attr("class",
                                    if self.filter == Filter::All { "selected" }
                                    else { "" }
                                 )
                                .push("All")
                                .on("click", Event(|e| {
                                    e.prevent_default();
                                    Message::ShowAll
                                }))
                            )
                        )
                        .push(Dom::elem("li")
                            .push(Dom::elem("a")
                                .attr("href", "#/active")
                                .attr("class",
                                    if self.filter == Filter::Active { "selected" }
                                    else { "" }
                                 )
                                .push("Active")
                                .on("click", Event(|e| {
                                    e.prevent_default();
                                    Message::ShowActive
                                }))
                            )
                        )
                        .push(Dom::elem("li")
                            .push(Dom::elem("a")
                                .attr("href", "#/completed")
                                .attr("class",
                                    if self.filter == Filter::Completed { "selected" }
                                    else { "" }
                                 )
                                .push("Completed")
                                .on("click", Event(|e| {
                                    e.prevent_default();
                                    Message::ShowCompleted
                                }))
                            )
                        )
                    )
                ;
                if self.items.iter().any(|item| item.is_complete) {
                    footer.push(Dom::elem("button")
                        .attr("class", "clear-completed")
                        .push("Clear completed")
                        .event("click", Message::ClearCompleted)
                    )
                }
                else {
                    footer
                }
            });
        }

        vec.into()
    }
}

impl Item {
    fn render(&self, i: usize, pending_edit: Option<&str>) -> dom::Dom<Message> {
        use dom::Dom;
        use dom::Handler::{Event,InputValue};

        let e = Dom::elem("li");

        if let Some(pending_edit) = pending_edit {
            e.attr("class", "editing")
                .push(Dom::elem("input")
                    .attr("class", "edit")
                    .attr("value", pending_edit)
                    .on("input", InputValue(|s| {
                        Message::UpdateEdit(s)
                    }))
                    .event("blur", Message::SaveEdit)
                    .on("keyup", Event(|e| {
                        let e = e.dyn_into::<web_sys::KeyboardEvent>().expect_throw("expected web_sys::KeyboardEvent");
                        match e.key().as_ref() {
                            "Enter" => Message::SaveEdit,
                            "Escape" => Message::AbortEdit,
                            _ => Message::Noop,
                        }
                    }))
                )
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
                        .event("dblclick", Message::EditTodo(i))
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

    let app = AppBuilder::default()
        .attach(parent, Todo::default());

    App::dispatch(app, Message::FocusPending);

    info!("{} initialized", TITLE);
    
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

    #[test]
    fn save_edit_removes_empty() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item {
            text: "text".to_owned(),
            .. Item::default()
        });

        let mut cmds = vec![];

        todomvc.update(Message::EditTodo(0), &mut cmds);
        todomvc.update(Message::UpdateEdit("".to_owned()), &mut cmds);
        todomvc.update(Message::SaveEdit, &mut cmds);

        assert_eq!(todomvc.items.len(), 0);
    }

    #[test]
    fn save_edit_trims_whitespace() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item {
            text: "text".to_owned(),
            .. Item::default()
        });

        let mut cmds = vec![];

        todomvc.update(Message::EditTodo(0), &mut cmds);
        todomvc.update(Message::UpdateEdit(" edited text  ".to_owned()), &mut cmds);
        todomvc.update(Message::SaveEdit, &mut cmds);

        assert_eq!(todomvc.items.len(), 1);
        assert_eq!(todomvc.items[0].text, "edited text");
    }

    #[test]
    fn abort_edit_does_not_modify() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item {
            text: "text".to_owned(),
            .. Item::default()
        });

        let mut cmds = vec![];

        todomvc.update(Message::EditTodo(0), &mut cmds);
        todomvc.update(Message::UpdateEdit(" edited text  ".to_owned()), &mut cmds);
        todomvc.update(Message::AbortEdit, &mut cmds);

        assert_eq!(todomvc.items.len(), 1);
        assert_eq!(todomvc.items[0].text, "text");
    }

    #[test]
    fn clear_completed() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item {
            text: "text1".to_owned(),
            .. Item::default()
        });
        todomvc.items.push(Item {
            text: "text2".to_owned(),
            is_complete: true,
            .. Item::default()
        });
        todomvc.items.push(Item {
            text: "text3".to_owned(),
            .. Item::default()
        });

        let mut cmds = vec![];

        todomvc.update(Message::ClearCompleted, &mut cmds);

        assert_eq!(todomvc.items.len(), 2);
        assert_eq!(todomvc.items[0].text, "text1");
        assert_eq!(todomvc.items[1].text, "text3");
    }

    #[test]
    fn toggle_all() {
        let mut todomvc = Todo::default();
        todomvc.items.push(Item {
            text: "text1".to_owned(),
            .. Item::default()
        });
        todomvc.items.push(Item {
            text: "text2".to_owned(),
            is_complete: true,
            .. Item::default()
        });
        todomvc.items.push(Item {
            text: "text3".to_owned(),
            .. Item::default()
        });

        let mut cmds = vec![];

        todomvc.update(Message::ToggleAll, &mut cmds);
        assert!(todomvc.items.iter().all(|item| item.is_complete));

        todomvc.update(Message::ToggleAll, &mut cmds);
        assert!(todomvc.items.iter().all(|item| !item.is_complete));
    }
}
