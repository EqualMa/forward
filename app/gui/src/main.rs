#![windows_subsystem = "windows"]

//! Demonstrates a mutable application state manipulated over a number of UIs
//! Using the UIGrid function for a prettier interface.

extern crate iui;
use forward::auth::Authentication;
use forward::server::{ForwardServer, ForwardServerConfig};
use forward::target_addr::TargetAddr;
use forward::target_addr::ToTargetAddr;
use forward::tokio;
use forward::tokio::sync::broadcast;
use iui::controls::{
    Button, Entry, GridAlignment, GridExpand, Group, Label, LayoutGrid, VerticalBox,
};
use iui::prelude::*;
use std::net::SocketAddr;

fn entry_with_label(ui: &UI, label_text: &str, entry_init_value: Option<&str>) -> (Label, Entry) {
    let mut entry = Entry::new(&ui);

    if let Some(v) = entry_init_value {
        entry.set_value(&ui, v);
    }

    // Create a new label. Note that labels don't auto-wrap!
    let label = Label::new(&ui, label_text);

    (label, entry)
}

#[tokio::main]
async fn main() {
    // Initialize the UI library
    let ui = UI::init().expect("Couldn't initialize UI library");
    // Create a window into which controls can be placed
    let mut win = Window::new(&ui, "Forward", 200, 200, WindowType::NoMenubar);

    // Create a vertical layout to hold the controls
    let mut vbox = VerticalBox::new(&ui);
    vbox.set_padded(&ui, true);

    let mut grid = LayoutGrid::new(&ui);
    grid.set_padded(&ui, true);

    let mut group_vbox = VerticalBox::new(&ui);
    let mut group = Group::new(&ui, "Start New Server");

    let bind = entry_with_label(&ui, "Local Bind Address", Some("127.0.0.1:25565"));
    let proxy = entry_with_label(&ui, "Through Proxy (Socks5)", Some("127.0.0.1:1080"));
    let target = entry_with_label(&ui, "Target", Some("mc.hypixel.net"));

    let mut value_bind: Option<SocketAddr> = bind.1.value(&ui).parse().ok();
    let mut value_proxy: Option<TargetAddr> = proxy.1.value(&ui).as_str().to_target_addr().ok();
    let mut value_target: Option<TargetAddr> = target.1.value(&ui).as_str().to_target_addr().ok();

    let mut start_button = Button::new(&ui, "Start");
    start_button.on_clicked(&ui, {
        let ui = ui.clone();
        let bind = bind.1.clone();
        let proxy = proxy.1.clone();
        let target = target.1.clone();

        let (tx, mut _rx1) = broadcast::channel(1);
        let mut clicked: u8 = 0; // stopped

        move |btn| {
            if clicked > 1 {
                println!("[ui] Server Stopping or Starting");
                return;
            }

            if clicked == 1 {
                println!("[ui] Clicked -- Stop Server");
                tx.send(()).unwrap();
                btn.set_text(&ui, "Start");
                clicked = 0; // stopping
                return;
            }

            // clicked == 0

            clicked = 1; // started

            value_bind = bind.value(&ui).parse().ok();
            value_proxy = proxy.value(&ui).as_str().to_target_addr().ok();
            value_target = target.value(&ui).as_str().to_target_addr().ok();

            if let (Some(value_bind), Some(value_proxy), Some(value_target)) =
                (value_bind, value_proxy.clone(), value_target.clone())
            {
                let mut rx = tx.subscribe();

                tokio::spawn(async move {
                    let proxy_auth = Authentication::None;

                    let mut server = ForwardServer::new(ForwardServerConfig {
                        bind_addr: value_bind,
                        proxy: value_proxy,
                        proxy_auth: proxy_auth,
                        target: value_target,
                    });

                    server
                        .start(Some(tokio::spawn(async move {
                            rx.recv().await.expect("Try Stopping Error!");
                        })))
                        .await
                        .expect("Server exit with error");

                    clicked = 0;

                    println!("Server stopped~");
                });

                btn.set_text(&ui, "Stop");
            }
        }
    });

    grid.append(
        &ui,
        bind.0,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        0,
        0,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    grid.append(
        &ui,
        bind.1,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        1,
        0,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    grid.append(
        &ui,
        proxy.0,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        0,
        1,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    grid.append(
        &ui,
        proxy.1,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        1,
        1,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    grid.append(
        &ui,
        target.0,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        0,
        2,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    grid.append(
        &ui,
        target.1,
        // This is position (by slot) and size, expansion, and alignment.
        // In this case, row 0, col 0, 1 by 1, compress as much as possible,
        // and align to the fill.
        1,
        2,
        1,
        1,
        GridExpand::Neither,
        GridAlignment::Fill,
        GridAlignment::Fill,
    );

    group_vbox.append(&ui, grid, LayoutStrategy::Stretchy);
    group_vbox.append(&ui, start_button, LayoutStrategy::Compact);

    group.set_child(&ui, group_vbox);

    vbox.append(&ui, group, LayoutStrategy::Compact);

    // Actually put the button in the window
    win.set_child(&ui, vbox);
    // Show the window
    win.show(&ui);
    // Run the application
    ui.main();
}
