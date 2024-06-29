use eframe::NativeOptions;
use egui::{CentralPanel, Color32, Frame, ScrollArea, Ui};
use egui_inbox::{UiInbox, UiInboxSender};
use egui_router::{EguiRouter, OwnedRequest, Route};

type AppState = UiInboxSender<RouterMessage>;

enum RouterMessage {
    Navigate(String),
    Back,
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut router: Option<EguiRouter<AppState>> = None;

    let mut inbox = UiInbox::new();
    let mut sender = inbox.sender();

    eframe::run_simple_native(
        "Router Example",
        NativeOptions::default(),
        move |ctx, frame| {
            let router = router.get_or_insert_with(|| {
                EguiRouter::builder()
                    .route("/", home)
                    .async_route("/post/{id}", post)
                    .default_path("/")
                    .build(&mut sender)
            });

            inbox.read(ctx).for_each(|msg| match msg {
                RouterMessage::Navigate(route) => {
                    router.navigate(&mut sender, route).ok();
                }
                RouterMessage::Back => {
                    router.back().ok();
                }
            });

            CentralPanel::default().show(ctx, |ui| {
                router.ui(ui, &mut sender);
            });
        },
    )
}

fn home() -> impl Route<AppState> {
    |ui: &mut Ui, inbox: &mut UiInboxSender<RouterMessage>| {
        background(ui, ui.style().visuals.faint_bg_color, |ui| {
            ui.heading("Home!");

            ui.label("Navigate to post:");

            if ui.link("Post 1").clicked() {
                inbox
                    .send(RouterMessage::Navigate("/post/1".to_string()))
                    .ok();
            }

            if ui.link("Post 2").clicked() {
                inbox
                    .send(RouterMessage::Navigate("/post/2".to_string()))
                    .ok();
            }

            if ui.link("Invalid Post").clicked() {
                inbox
                    .send(RouterMessage::Navigate("/post/".to_string()))
                    .ok();
            }
        });
    }
}

async fn post(request: OwnedRequest<AppState>) -> impl Route<AppState> {
    let id = request.params.get("id").map(ToOwned::to_owned);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    move |ui: &mut Ui, sender: &mut AppState| {
        background(ui, ui.style().visuals.extreme_bg_color, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                if let Some(id) = &id {
                    ui.label(format!("Post: {}", id));

                    if ui.button("back").clicked() {
                        sender.send(RouterMessage::Back).ok();
                    }

                    ui.label(include_str!("../../../README.md"));
                } else {
                    ui.label("Post not found");
                    if ui.button("back").clicked() {
                        sender.send(RouterMessage::Back).ok();
                    }
                }
            });
        });
    }
}

fn background(ui: &mut Ui, color: Color32, content: impl FnOnce(&mut Ui)) {
    Frame::none().fill(color).inner_margin(16.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        content(ui);
    });
}
