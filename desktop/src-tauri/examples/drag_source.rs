// Test helper: a window whose button drags the given files (as text/uri-list), so an X11 test can drop real files.
// Usage: cargo run --example drag_source -- <file>...
use gtk::prelude::*;

fn main() {
    let uris: Vec<String> = std::env::args()
        .skip(1)
        .map(|path| gtk::glib::filename_to_uri(std::fs::canonicalize(&path).expect("file not found"), None).unwrap().to_string())
        .collect();
    gtk::init().expect("no display");
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("drag-source");
    window.set_default_size(200, 120);
    window.move_(0, 0);
    let button = gtk::Button::with_label("drag me");
    button.drag_source_set(
        gtk::gdk::ModifierType::BUTTON1_MASK,
        &[gtk::TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP, 0)],
        gtk::gdk::DragAction::COPY,
    );
    button.connect_drag_data_get(move |_, _, data, _, _| {
        data.set_uris(&uris.iter().map(String::as_str).collect::<Vec<_>>());
    });
    window.add(&button);
    window.connect_destroy(|_| gtk::main_quit());
    window.show_all();
    gtk::main();
}
