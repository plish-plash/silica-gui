use std::{cell::Cell, rc::Rc};

use pangocairo::pango;
use silica::{
    taffy::prelude::*, widget::Container, GraphicsContext, HorizontalAlign, TextSection,
    ThemeColor, VerticalAlign,
};
use xcb::{x, Xid};

pub use xcb::Result;

xcb::atoms_struct! {
    #[derive(Debug)]
    struct Atoms {
        wm_protocols    => b"WM_PROTOCOLS",
        wm_del_window   => b"WM_DELETE_WINDOW",
    }
}

pub struct CairoContext(cairo::Context, pango::Context);

impl silica::GraphicsContext for CairoContext {
    fn save(&mut self) {
        self.0.save().unwrap();
    }
    fn restore(&mut self) {
        self.0.restore().unwrap();
    }
    fn translate(&mut self, tx: f32, ty: f32) {
        self.0.translate(tx as f64, ty as f64);
    }

    fn set_color(&mut self, color: ThemeColor) {
        let rgba = color.to_rgba();
        self.0.set_source_rgba(
            rgba[0] as f64,
            rgba[1] as f64,
            rgba[2] as f64,
            rgba[3] as f64,
        );
    }
    fn draw_rect(&mut self, size: Size<f32>) {
        let size = size.map(|v| v as f64);
        self.0.rectangle(0.0, 0.0, size.width, size.height);
        self.0.fill().unwrap();
    }
    fn draw_border(&mut self, size: Size<f32>, border: Rect<LengthPercentage>) {
        let size = size.map(|v| v as f64);
        let border = border.map(|val| match val {
            LengthPercentage::Points(points) => points as f64,
            LengthPercentage::Percent(_) => 0.0,
        });
        self.0.move_to(0.0, 0.0);
        if border.top > 0.0 {
            self.0.set_line_width(border.top);
            self.0.line_to(size.width, 0.0);
        } else {
            self.0.move_to(size.width, 0.0);
        }
        if border.right > 0.0 {
            self.0.set_line_width(border.right);
            self.0.line_to(size.width, size.height);
        } else {
            self.0.move_to(size.width, size.height);
        }
        if border.bottom > 0.0 {
            self.0.set_line_width(border.bottom);
            self.0.line_to(0.0, size.height);
        } else {
            self.0.move_to(0.0, size.height);
        }
        if border.left > 0.0 {
            self.0.set_line_width(border.left);
            self.0.line_to(0.0, 0.0);
        } else {
            self.0.move_to(0.0, 0.0);
        }
        self.0.stroke().unwrap();
    }
    fn draw_text(&mut self, size: Size<f32>, text: &TextSection) {
        pangocairo::update_context(&self.0, &self.1);
        let layout = pango::Layout::new(&self.1);
        layout.set_width((size.width * (pango::SCALE as f32)) as i32);
        layout.set_alignment(match text.h_align {
            HorizontalAlign::Left => pango::Alignment::Left,
            HorizontalAlign::Center => pango::Alignment::Center,
            HorizontalAlign::Right => pango::Alignment::Right,
        });
        let font = pango::FontDescription::from_string(&text.font); // TODO cache this
        layout.set_font_description(Some(&font));
        layout.set_text(&text.text);

        let height = size.height as f64;
        let text_height = (layout.size().1 as f64) / (pango::SCALE as f64);
        match text.v_align {
            VerticalAlign::Top => self.0.move_to(0.0, 0.0),
            VerticalAlign::Center => self.0.move_to(0.0, (height / 2.0) - (text_height / 2.0)),
            VerticalAlign::Bottom => self.0.move_to(0.0, height - text_height),
        }
        pangocairo::show_layout(&self.0, &layout);
    }
}

pub struct Window {
    xcb: xcb::Connection,
    atoms: Atoms,
    window: x::Window,
    surface: cairo::XCBSurface,
    size: Cell<Size<u16>>,
    gui: Rc<silica::Gui>,
    root: Container,
}

impl Window {
    fn make_surface(
        conn: &xcb::Connection,
        screen: &x::Screen,
        window: x::Window,
        size: Size<u16>,
    ) -> cairo::XCBSurface {
        fn lookup_visual(screen: &x::Screen, visual: x::Visualid) -> Option<*const x::Visualtype> {
            for depth in screen.allowed_depths() {
                for visual_type in depth.visuals() {
                    if visual_type.visual_id() == visual {
                        return Some(visual_type);
                    }
                }
            }
            None
        }

        let connection = unsafe { cairo::XCBConnection::from_raw_full(conn.get_raw_conn() as _) };
        let drawable = cairo::XCBDrawable(window.resource_id());
        let visual = lookup_visual(screen, screen.root_visual())
            .expect("root_visual not found in screen visuals");
        let visual = unsafe { cairo::XCBVisualType::from_raw_full(visual as _) };
        cairo::XCBSurface::create(
            &connection,
            &drawable,
            &visual,
            size.width.into(),
            size.height.into(),
        )
        .expect("failed to create cairo surface")
    }

    pub fn new(root: Container) -> Rc<Self> {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let atoms = Atoms::intern_all(&conn).unwrap();
        let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
        let window: x::Window = conn.generate_id();

        let gui = root.gui();
        gui.emit_layout(None);
        let size = root.size().map(|v| v as u16);

        conn.send_request(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: screen.root(),
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
            border_width: 10,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: &[x::Cw::EventMask(
                x::EventMask::EXPOSURE
                    | x::EventMask::STRUCTURE_NOTIFY
                    | x::EventMask::POINTER_MOTION
                    | x::EventMask::BUTTON_PRESS
                    | x::EventMask::BUTTON_RELEASE
                    | x::EventMask::KEY_PRESS,
            )],
        });

        conn.send_request(&x::MapWindow { window });

        // activate the sending of close event through `x::Event::ClientMessage`
        conn.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atoms.wm_protocols,
            r#type: x::ATOM_ATOM,
            data: &[atoms.wm_del_window],
        });

        let surface = Self::make_surface(&conn, screen, window, size);

        Rc::new(Window {
            xcb: conn,
            atoms,
            window,
            surface,
            size: Cell::new(size),
            gui,
            root,
        })
    }

    pub fn width(&self) -> u16 {
        self.size.get().width
    }
    pub fn height(&self) -> u16 {
        self.size.get().height
    }

    pub fn set_title(&self, title: &str) {
        self.xcb.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.window,
            property: x::ATOM_WM_NAME,
            r#type: x::ATOM_STRING,
            data: title.as_bytes(),
        });
    }
    pub fn set_icon_name(&self, icon_name: &str) {
        self.xcb.send_request(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.window,
            property: x::ATOM_WM_ICON_NAME,
            r#type: x::ATOM_STRING,
            data: icon_name.as_bytes(),
        });
    }

    fn queue_redraw(&self) {
        self.xcb.send_request(&x::ClearArea {
            exposures: true,
            window: self.window,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        });
    }
    pub fn run_event_loop(self: Rc<Self>) -> Result<()> {
        self.xcb.flush()?;
        loop {
            let event = match self.xcb.wait_for_event() {
                Err(xcb::Error::Connection(err)) => {
                    panic!("unexpected I/O error: {}", err);
                }
                Err(xcb::Error::Protocol(err)) => {
                    panic!("unexpected protocol error: {:#?}", err);
                }
                Ok(event) => event,
            };

            // println!("Received event {:#?}", event);
            match event {
                xcb::Event::X(x::Event::Expose(_)) => {
                    let cairo_context =
                        cairo::Context::new(&self.surface).expect("failed to create cairo context");
                    let pango_context = pangocairo::create_context(&cairo_context);
                    let mut context = CairoContext(cairo_context, pango_context);
                    context.set_color(ThemeColor::Background);
                    context.0.paint().unwrap();

                    let size = self.size.get().map(|v| v as f32);
                    self.gui.emit_layout(Some(size));
                    self.gui.draw(&mut context, self.root.clone());

                    self.surface.flush();
                }
                xcb::Event::X(x::Event::ConfigureNotify(ev)) => {
                    let size = Size {
                        width: ev.width(),
                        height: ev.height(),
                    };
                    if size != self.size.get() {
                        self.size.set(size);
                        self.surface
                            .set_size(size.width as i32, size.height as i32)
                            .expect("failed to resize surface");
                    }
                }
                xcb::Event::X(x::Event::MotionNotify(ev)) => {
                    self.gui
                        .emit_pointer_motion(ev.event_x().into(), ev.event_y().into());
                }
                xcb::Event::X(x::Event::ButtonPress(ev)) => {
                    if ev.detail() == 1 {
                        // LMB
                        self.gui
                            .emit_pointer_button(silica::signal::PointerButton::Primary(true));
                    }
                }
                xcb::Event::X(x::Event::ButtonRelease(ev)) => {
                    if ev.detail() == 1 {
                        // LMB
                        self.gui
                            .emit_pointer_button(silica::signal::PointerButton::Primary(false));
                    }
                }
                xcb::Event::X(x::Event::KeyPress(ev)) => {
                    println!("Key '{}' pressed", ev.detail());
                }
                xcb::Event::X(x::Event::ClientMessage(ev)) => {
                    if let x::ClientMessageData::Data32([atom, ..]) = ev.data() {
                        if atom == self.atoms.wm_del_window.resource_id() {
                            // window "x" button clicked by user, exit gracefully
                            break Ok(());
                        }
                    }
                }
                _ => {}
            }

            if self.gui.check_dirty() {
                self.queue_redraw();
            }
            self.xcb.flush()?;
        }
    }
}
