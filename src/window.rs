use bevy::prelude::{ClearColor, Color, Msaa, NonSend, Plugin, WindowDescriptor};
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use std::io::Cursor;
use winit::window::Icon;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(set_window_icon)
            .insert_resource(Msaa { samples: 1 })
            .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
            .insert_resource(WindowDescriptor {
                width: 680.,
                height: 680.,
                title: "Ascii Snake".to_string(), // ToDo
                canvas: Some("#bevy".to_owned()),
                fit_canvas_to_parent: true,
                ..Default::default()
            });
    }
}

// Sets the icon on windows and X11
fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!("../assets/icon.png"));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
