use crate::{
    imgui,
    game::recording::{
        keybinds::self,
        input_edit::InputEditWindow,
        window::{
            Window,
            Openable,
        },
    },
};

pub fn show_menu_bar(frame: &mut imgui::Frame, windows: &mut Vec<(Box<dyn Window>, bool)>, close: &mut bool) {
    if frame.begin_menu_main_bar() {
        if frame.begin_menu("File", true) {
            if frame.menu_item("Close") {
                *close = true;
                return;
            }
            frame.end_menu();
        }
        
        if frame.begin_menu("Windows", true) {
            if frame.begin_menu("Active Windows", true) {
                for (window, focus) in &mut *windows {
                    if frame.menu_item(&window.name()) {
                        *focus = true;
                    }
                }
                frame.end_menu();
            }

            macro_rules! openable {
                (@single $type:ty) => {
                    // see if a window of this type is already open
                    if let Some((_, focus)) = windows.iter_mut().find(|(win, _)| win.window_type_self() == <$type>::window_type()) {
                        // focus the window if it's already open
                        *focus = true;
                    } else {
                        // or create it
                        windows.push((Box::new(<$type>::open()), true));
                    }
                };
                (@multi $type:ty) => {{
                    // just add another window if multiple instances of this window are allowed
                    windows.push((Box::new(<$type>::open()), true));
                }};
                ($($id:ident $type:ty),* $(,)?) => {{
                    $(
                        // create the menu item for the window
                        if frame.menu_item(<$type>::window_name()) {
                            // and add the code for clicking on it, depending on whether multiple instances are allowed or not
                            openable!(@$id $type)
                        }
                    )*
                }};
            }
            
            if frame.begin_menu("Open", true) {
                openable! {
                    single keybinds::KeybindWindow,
                    single InputEditWindow,
                }
                
                frame.end_menu();
            }
            frame.end_menu();
        }
        
        frame.end_menu_main_bar();
    }
}
