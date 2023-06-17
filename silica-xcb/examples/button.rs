use silica::{model::IntModel, signal, taffy::prelude::*, widget::*};

fn main() -> silica_xcb::Result<()> {
    let root = silica::Gui::new();
    root.set_layout(Style {
        flex_direction: FlexDirection::Row,
        align_items: Some(AlignItems::Start),
        size: Size {
            width: points(640.0),
            height: points(480.0),
        },
        padding: Rect::points(64.0),
        gap: Size::points(8.0),
        ..Default::default()
    });

    let times_clicked = IntModel::new(0);

    let button = Button::with_label(root.gui(), "Click Me!".to_string());
    button.connect_activate({
        let times_clicked = times_clicked.clone();
        move |_, signal::Activate| {
            times_clicked.increment();
        }
    });
    root.add_child(button);

    let label = Label::new(root.gui());
    label.set_layout(Style {
        min_size: Size {
            width: Dimension::Points(256.),
            height: Dimension::Points(32.),
        },
        ..Default::default()
    });
    times_clicked.connect_change({
        let label = label.clone();
        move |times_clicked, signal::Change| {
            label.set_text(format!("Times Clicked: {}", times_clicked.get()));
        }
    });
    root.add_child(label);

    let window = silica_xcb::Window::new(root);
    window.set_title("Button Example");
    window.run_event_loop()
}
