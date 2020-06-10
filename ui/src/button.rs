
use pango;

use pugl_sys::*;
use pugl_ui::*;
use pugl_ui::ui::*;
use pugl_ui::widget::*;

pub struct Button {
    stub: WidgetStub,
    min_size: Size,
    text: String,

    clicked: bool,

    toggle_state: Option<bool>,
    changed_toggle_state: Option<bool>
}

impl Widget for Button {
    widget_stub!();

    fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
        let (r, g, b) = if self.toggle_state.unwrap_or(false) {
            (0.7, 0.7, 0.7)
        } else {
            (0.4, 0.4, 0.4)
        };
        let size = self.size();
        let pos = self.pos();

        cr.set_source_rgb (r, g, b);
        cr.rectangle (pos.x, pos.y, size.w, size.h);
        cr.fill ();

        cr.set_source_rgb (0., 0., 0.);

        cr.save();
        cr.translate(pos.x, pos.y);

        let ctx = pangocairo::functions::create_context (&cr).unwrap();
        let lyt = pango::Layout::new (&ctx);

        let font_desc = pango::FontDescription::from_string ("Sans 24px");

        lyt.set_font_description (Some(&font_desc));
        lyt.set_text (&self.text);

        pangocairo::functions::show_layout (cr, &lyt);

        cr.restore();

        if self.has_focus() {
            cr.set_source_rgb (1., 1., 1.);
            cr.rectangle(pos.x, pos.y, size.w, size.h);
            cr.stroke();
        }
    }
    fn event (&mut self, ev: Event) -> Option<Event> {
        match ev.data {
            EventType::MouseMove (_mm) => {
                event_processed!()
            }
            EventType::MouseButtonRelease (btn) => {
                self.clicked = true;
                self.changed_toggle_state = self.toggle_state.and_then(|ts| {
                    Some(!ts)
                });
                println!("Some click {:?}", self.toggle_state);
                event_processed!()
            },
            EventType::KeyRelease (ke) => {
                ke.try_char().and_then(|c| {
                    match c {
                        ' ' => {
                            event_processed!()
                        },
                        _ => event_not_processed!()
                    }
                }).or (event_not_processed!())
            },
            _ => event_not_processed!()
        }.and_then (|es| es.pass_event (ev))
    }
    fn min_size(&self) -> Size { self.min_size }

    fn takes_focus(&self) -> bool { true }
}

impl Button {
    pub fn new_toggle_button(text: &str, toggle_state: bool) -> Box<Button> {
        let mut btn = Self::new(text);
        btn.toggle_state = Some(toggle_state);
        btn
    }

    pub fn new (text: &str) -> Box<Button> {
        let sf = cairo::ImageSurface::create (cairo::Format::ARgb32, 8, 8).unwrap();
        let cr = cairo::Context::new (&sf);

        let ctx = pangocairo::functions::create_context (&cr).unwrap();
        let lyt = pango::Layout::new (&ctx);

        let font_desc = pango::FontDescription::from_string ("Sans 24px");

        lyt.set_font_description (Some(&font_desc));
        lyt.set_text (text);

        let (w, h) = lyt.get_pixel_size();
        let min_size: Size = Size { w: w.into(), h: h.into() };

        Box::new(Button {
            stub: WidgetStub::default(),
            text: String::from(text),
            min_size,
            clicked: false,
            toggle_state: None,
            changed_toggle_state: None
        })
    }

    pub fn changed_toggle_state(&mut self) -> Option<bool> {
        self.changed_toggle_state.take()
    }

    pub fn set_toggle_state(&mut self, new_state: bool) {
        if self.toggle_state.is_some() {
            self.toggle_state = Some(new_state)
        }
    }
}
