use adw::prelude::*;

use adw::{
    ActionRow, Application, ApplicationWindow, HeaderBar, NavigationPage, NavigationSplitView,
};
use gtk4::{Box, Button, Label, ListBox, Orientation, SelectionMode};

fn main() -> glib::ExitCode {
    let application = Application::builder()
        .application_id("ca.louisbourque.webdevtoolbox")
        .build();

    application.connect_activate(|app| {
        let nav = NavigationSplitView::builder().build();

        // Create sidebar content
        let sidebar = Box::new(Orientation::Vertical, 0);
        sidebar.append(&HeaderBar::new());

        // ActionRows are only available in Adwaita
        let row = ActionRow::builder()
            .activatable(true)
            .title("Click me")
            .build();
        row.connect_activated(|_| {
            eprintln!("Clicked!");
        });

        let list = ListBox::builder()
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .selection_mode(SelectionMode::None)
            // makes the list look nicer
            .css_classes(vec![String::from("boxed-list")])
            .build();
        list.append(&row);
        sidebar.append(&list);

        // Create main content area
        let content = Box::new(Orientation::Vertical, 0);
        let content_header = HeaderBar::new();

        // Add toggle button for sidebar
        let toggle_button = Button::builder()
            .label("☰")
            .tooltip_text("Toggle Sidebar")
            .build();

        let nav_clone = nav.clone();
        toggle_button.connect_clicked(move |_| {
            nav_clone.set_collapsed(!nav_clone.is_collapsed());
            nav_clone.set_show_content(!nav_clone.shows_content());
        });

        content_header.pack_start(&toggle_button);
        content.append(&content_header);

        let hello_label = Label::builder()
            .label("Hello World")
            .margin_top(32)
            .margin_end(32)
            .margin_bottom(32)
            .margin_start(32)
            .build();
        content.append(&hello_label);

        // Create navigation pages
        let sidebar_page = NavigationPage::builder()
            .title("Sidebar")
            .child(&sidebar)
            .build();

        let content_page = NavigationPage::builder()
            .title("Content")
            .child(&content)
            .build();

        // Set up the navigation split view
        nav.set_sidebar(Some(&sidebar_page));
        nav.set_content(Some(&content_page));

        let window = ApplicationWindow::builder()
            .application(app)
            .title("WebDev Toolbox")
            .default_width(600)
            .default_height(400)
            // add navigation to window
            .content(&nav)
            .build();
        window.present();
    });

    application.run()
}
