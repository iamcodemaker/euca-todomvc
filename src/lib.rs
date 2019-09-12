use wasm_bindgen::prelude::*;
use cfg_if::cfg_if;
use log::{debug,info};

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

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    init_log();
    set_panic_hook();

    info!("Hello world!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
