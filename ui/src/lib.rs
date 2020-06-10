
extern crate lv2_sys;
extern crate lv2_atom;
extern crate lv2_urid;
extern crate lv2_core;
extern crate urid;

extern crate lv2_ui;

extern crate pugl_ui;
extern crate cairo;
extern crate pango;

mod dial;
mod button;
mod meter;

use lv2_ui::*;
use lv2_sys::*;
use lv2_atom::prelude::*;
use lv2_atom as atom;
use urid::*;
use lv2_urid::*;
use lv2_core::prelude::*;
use lv2_units as units;

//use pugl_ui::widget::*;
use pugl_ui::ui::*;
use pugl_ui::layout::*;
use pugl_ui::*;
use pugl_sys::*;

#[derive(FeatureCollection)]
struct Features<'a> {
    map: LV2Map<'a>
}


#[uri("http://lv2plug.in/ns/ext/parameters#sampleRate")]
struct SampleRate;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#uiOn")]
struct UIOn;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#uiOff")]
struct UIOff;


#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    unit: units::UnitURIDCollection,
    ui_on: URID<UIOn>,
    ui_off: URID<UIOff>,
    atom_event_transfer: URID<AtomEventTransfer>,
    sample_rate: URID<SampleRate>
}

struct MyUIPorts {
    gain: ControlPort,
    enabled: ControlPort,
    meter_in: ControlPort,
    meter_out: ControlPort,
    control: UIAtomPort,
    notify: UIAtomPort
}

impl MyUIPorts {
    fn new(urid: URID<AtomEventTransfer>) -> Self {
        MyUIPorts {
            gain: ControlPort::new(0),
            enabled: ControlPort::new(1),
            meter_in: ControlPort::new(2),
            meter_out: ControlPort::new(3),
            control: UIAtomPort::new(urid, 4),
            notify: UIAtomPort::new(urid, 5)
        }
    }
}

impl UIPortsTrait for MyUIPorts {
    fn map_control_port(&mut self, port_index: u32) -> Option<&mut ControlPort>{
        match port_index {
            0 => Some(&mut self.gain),
            1 => Some(&mut self.enabled),
            2 => Some(&mut self.meter_in),
            3 => Some(&mut self.meter_out),
            _ => None
        }
    }
    fn map_atom_port(&mut self, port_index: u32) -> Option<&mut UIAtomPort> {
        match port_index {
            4 => Some(&mut self.control),
            5 => Some(&mut self.notify),
            _ => None
        }
    }
}

#[uri("https://johannes-mueller.org/lv2/ampmeter-rs#ui")]
struct AmpUI {
    view: Box<PuglView<UI<RootWidget>>>,

    gain_dial: widget::WidgetHandle<dial::Dial>,
    enable_btn: widget::WidgetHandle<button::Button>,
    meter_in: widget::WidgetHandle<meter::Meter>,
    meter_out: widget::WidgetHandle<meter::Meter>,

    ports: MyUIPorts,
    write_handle: PluginPortWriteHandle,

    urids: URIDs
}

impl AmpUI {
    pub fn new(features: &mut Features<'static>,
               parent_window: *mut std::ffi::c_void,
               write_handle: PluginPortWriteHandle) -> Option<Self> {
        eprintln!("new");
        let mut ui = Box::new(UI::new(Box::new(RootWidget::default())));

        let h_layout = ui.new_layouter::<HorizontalLayouter>();
        let v_layout = ui.new_layouter::<VerticalLayouter>();

        let gain_dial = ui.new_widget(dial::Dial::new(-90., 24., 1.));
        let enable_btn = ui.new_widget(button::Button::new_toggle_button("enable", false));
        let meter_in = ui.new_widget(meter::Meter::new(-60., 20.));
        let meter_out = ui.new_widget(meter::Meter::new(-60., 20.));

        ui.pack_to_layout(h_layout.widget(), ui.root_layout(), StackDirection::Front);
        ui.pack_to_layout(v_layout.widget(), h_layout, StackDirection::Front);
        ui.pack_to_layout(gain_dial, v_layout, StackDirection::Front);
        ui.pack_to_layout(enable_btn, v_layout, StackDirection::Front);
        ui.pack_to_layout(meter_in, h_layout, StackDirection::Back);
        ui.pack_to_layout(meter_out, h_layout, StackDirection::Back);
        ui.do_layout();

        let view = PuglView::make_view(ui, parent_window);

        let ui = view.handle();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.set_window_title("ampmeter");
        ui.show_window();

        let urids: URIDs = features.map.populate_collection()?;

        let ports = MyUIPorts::new(urids.atom_event_transfer);

        Some(Self {
            view,
            gain_dial,
            enable_btn,
            meter_in,
            meter_out,
            ports,
            write_handle,
            urids
        })
    }

    fn ui(&self) -> &mut UI<RootWidget> {
        self.view.handle()
    }

    fn send_ui_enable(&mut self) {
        self.ports.control.init(
            self.urids.atom.object,
            ObjectHeader {
                id: None,
                otype: self.urids.ui_on.into_general()
            });
        self.write_handle.write_port(&self.ports.control);
    }

    fn send_ui_disable(&mut self) {
        self.ports.control.init(
            self.urids.atom.object,
            ObjectHeader {
                id: None,
                otype: self.urids.ui_off.into_general()
            });
        self.write_handle.write_port(&self.ports.control);
    }
}

impl PluginUI for AmpUI {

    type InitFeatures = Features<'static>;
    type UIPorts = MyUIPorts;

    fn new(plugin_ui_info: &PluginUIInfo,
           features: &mut Self::InitFeatures,
           parent_window: *mut std::ffi::c_void,
           write_handle: PluginPortWriteHandle
    ) -> Option<Self> {
        eprintln!("AmpUI::new()");
        let mut ui = Self::new(features, parent_window, write_handle)?;
        ui.send_ui_enable();
        Some(ui)
    }

    fn cleanup(&mut self) {
        eprintln!("cleanup called");
        self.send_ui_disable();
    }

    fn ports(&mut self) -> &mut MyUIPorts {
        &mut self.ports
    }

    fn widget(&self) -> LV2UI_Widget {
        eprintln!("AmpUI::widget() {:?}", self.view.native_window() as *const std::ffi::c_void);

        self.view.native_window() as LV2UI_Widget
    }

    fn idle(&mut self) -> i32 {
        let ui = self.ui();
        ui.next_event(0.0);

        if ui.close_request_issued() {
            return 1;
        }

        if ui.root_widget().focus_next() {
                ui.focus_next_widget();
        }

        if let Some(v) = self.ui().widget(self.gain_dial).changed_value() {
            self.ports.gain.set_value(v as f32);
            self.write_handle.write_port(&self.ports.gain);
        }

        if let Some(v) = self.ui().widget(self.enable_btn).changed_toggle_state() {
            self.ports.enabled.set_value( if v { 1.0 } else { 0.0 } );
            self.write_handle.write_port(&self.ports.enabled);
        }

        0
    }

    fn update(&mut self) {
        if let Some(v) = self.ports.gain.changed_value() {
            self.ui().widget(self.gain_dial).set_value(v as f64);
        }
        if let Some(v) = self.ports.enabled.changed_value() {
            self.ui().widget(self.enable_btn).set_toggle_state(v > 0.5);
        }
        if let Some(v) = self.ports.meter_in.changed_value() {
            self.ui().widget(self.meter_in).set_value(v);
        }
        if let Some(v) = self.ports.meter_out.changed_value() {
            self.ui().widget(self.meter_out).set_value(v);
        }

        let notification_sequence = self.ports.notify.read(
            self.urids.atom.sequence,
            self.urids.unit.beat
        );

        if let Some(seq) = notification_sequence {
            println!("recieved sequence");
            for (_, msg) in seq {
                if let Some((_, object_reader)) = msg.read(self.urids.atom.object, ()) {
                    for (header, atom) in object_reader {
                        if header.key == self.urids.sample_rate {
                            let sample_rate = match atom.read(self.urids.atom.float, ()) {
                                Some(float) => float,
                                None => {
                                    println!("expected float for sample rate, got something different");
                                    continue
                                }
                            };
                            println!("sample rate is {}", sample_rate);
                        }
                    }
                }
            }
        }

        if let Some((_, object_reader)) = self.ports.notify.read(self.urids.atom.object, ()) {
            for (header, atom) in object_reader {
                if header.key == self.urids.sample_rate {
                    let sample_rate = match atom.read(self.urids.atom.float, ()) {
                        Some(float) => float,
                        None => {
                            println!("expected float for sample rate, got something different");
                            continue
                        }
                    };
                    println!("sample rate is {}", sample_rate);
                }
            }
        }
    }
}


unsafe impl PluginUIInstanceDescriptor for AmpUI {
    const DESCRIPTOR: LV2UI_Descriptor = LV2UI_Descriptor {
        URI: Self::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
        instantiate: Some(PluginUIInstance::<Self>::instantiate),
        cleanup: Some(PluginUIInstance::<Self>::cleanup),
        port_event: Some(PluginUIInstance::<Self>::port_event),
        extension_data: Some(PluginUIInstance::<Self>::extension_data)
    };
}

#[no_mangle]
pub unsafe extern "C" fn lv2ui_descriptor(index: u32) -> *const LV2UI_Descriptor {
    eprintln!("my ui descriptor called {}", index);
    match index {
        0 => &<AmpUI as PluginUIInstanceDescriptor>::DESCRIPTOR,
        _ => std::ptr::null()
    }
}

#[derive(Default)]
struct RootWidget {
    stub: widget::WidgetStub,
    focus_next: bool
}

impl widget::Widget for RootWidget {
    widget_stub!();
    fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
        cr.set_source_rgb (0.2, 0.2, 0.2);
        let size = self.size();
        cr.rectangle (0., 0., size.w, size.h);
        cr.fill ();
    }
    fn event(&mut self, ev: Event) -> Option<Event> {
        ev.try_keypress()
            .and_then(|kp| kp.try_char())
            .and_then(|c| {
                match c {
                    '\t' => {
                        self.focus_next = true;
                        event_processed!()
                    },
                    _ => event_not_processed!()
                }
            })
            .or(event_not_processed!()).and_then (|p| p.pass_event (ev))
    }
}

impl RootWidget {
    pub fn focus_next(&mut self) -> bool {
        let f = self.focus_next;
        self.focus_next = false;
        f
    }
}
