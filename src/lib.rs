use lv2_core::prelude::*;
use lv2_atom::prelude::*;
use lv2_atom as atom;
use lv2_urid::*;
use lv2_units::*;
use urid::*;
// The input and output ports are defined by a struct which implements the `PortCollection` trait.
// In this case, there is an input control port for the gain of the amplification, an input audio
// port and an output audio port.
//#[derive(PortCollection)]
struct Ports {
    gain: InputPort<Control>,
    enabled: InputPort<Control>,
    input_level: OutputPort<Control>,
    output_level: OutputPort<Control>,
    control: InputPort<AtomPort>,
    notify: OutputPort<AtomPort>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}

struct MyPortsPointerCache {
    gain: *mut std::ffi::c_void,
    enabled: *mut std::ffi::c_void,
    input_level: *mut std::ffi::c_void,
    output_level: *mut std::ffi::c_void,
    control: *mut std::ffi::c_void,
    notify: *mut std::ffi::c_void,
    input: *mut std::ffi::c_void,
    output: *mut std::ffi::c_void
}
impl Default for MyPortsPointerCache {
    fn default() -> Self {
        Self {
            gain: std::ptr::null_mut(),
            enabled: std::ptr::null_mut(),
            input_level: std::ptr::null_mut(),
            output_level: std::ptr::null_mut(),
            control: std::ptr::null_mut(),
            notify: std::ptr::null_mut(),
            input: std::ptr::null_mut(),
            output: std::ptr::null_mut()
        }
    }
}
impl PortPointerCache for MyPortsPointerCache {
    fn connect(&mut self, index: u32, pointer: *mut std::ffi::c_void) {
        match index {
            0 => { self.gain = pointer },
            1 => { self.enabled = pointer },
            2 => { self.input_level = pointer },
            3 => { self.output_level = pointer },
            4 => { self.control = pointer },
            5 => {
                println!("notify port {:?}", pointer);
                self.notify = pointer
            },
            6 => { self.input = pointer },
            7 => { self.output = pointer }
            _ => {}
        };
    }
}
impl PortCollection for Ports {
    type Cache = MyPortsPointerCache;
    unsafe fn from_connections(connections: &<Self as PortCollection>::Cache, sample_count: u32) -> Option<Self> {
        Some(
            Self {
                gain: if let Some(conn) =  <InputPort<Control> as PortHandle>::from_raw(connections.gain, sample_count) {
                    conn
                } else {
                    return None;
                },
                enabled: if let Some(conn) =  <InputPort<Control> as PortHandle>::from_raw(connections.enabled, sample_count) {
                    conn
                } else {
                    return None;
                },
                input_level: if let Some(conn) =  <OutputPort<Control> as PortHandle>::from_raw(connections.input_level, sample_count) {
                    conn
                } else {
                    return None;
                },
                output_level: if let Some(conn) =  <OutputPort<Control> as PortHandle>::from_raw(connections.output_level, sample_count) {
                    conn
                } else {
                    return None;
                },
                control: if let Some(conn) =  <InputPort<AtomPort> as PortHandle>::from_raw(connections.control, sample_count) {
                    conn
                } else {
                    return None;
                },
                notify: if let Some(conn) =  <OutputPort<AtomPort> as PortHandle>::from_raw(connections.notify, sample_count) {
                    let buf: [u8; 64] = *(connections.notify as *const [u8; 64]);
                    /*
                    print!("{:?} [ ", connections.notify);
                    for b in buf.iter() {
                        print!("{} ", b)
                    }
                    println!("]");
                    */
                    conn
                } else {
                    return None;
                },
                input: if let Some(conn) =  <InputPort<Audio> as PortHandle>::from_raw(connections.input, sample_count) {
                    conn
                } else {
                    return None;
                },
                output: if let Some(conn) =  <OutputPort<Audio> as PortHandle>::from_raw(connections.output, sample_count) {
                    conn
                } else {
                    return None;
                }
            }
        )
    }
}

#[derive(FeatureCollection)]
struct Features<'a> {
    map: LV2Map<'a>,
}

#[uri("http://lv2plug.in/ns/ext/parameters#sampleRate")]
struct SampleRate;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#UIState")]
struct UIState;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#uiOn")]
struct UIOn;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#uiOff")]
struct UIOff;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#RawAudio")]
struct RawAudio;

#[uri("http://johannes-mueller.org/lv2/ampmeter-rs#audioData")]
struct AudioData;

#[derive(URIDCollection)]
struct URIDs {
    atom: AtomURIDCollection,
    unit: UnitURIDCollection,
    ui_state: URID<UIState>,
    ui_on: URID<UIOn>,
    ui_off: URID<UIOff>,
    raw_audio: URID<RawAudio>,
    audio_data: URID<AudioData>,
    sample_rate: URID<SampleRate>
}


// The plugin struct. In this case, we don't need any data and therefore, this struct is empty.
//
// LV2 uses URIs to identify types. This association is expressed via the `UriBound` trait,
// which tells the framework that the type `Amp` is identified by the given URI. The usual
// way to implement this trait is to use the `uri` attribute.
#[uri("https://johannes-mueller.org/lv2/ampmeter-rs#lv2")]
struct Amp {
    urids: URIDs,
    ui_active: bool,
    ui_notified: bool,
    sample_rate: f64
}

// The implementation of the `Plugin` trait, which turns `Amp` into a plugin.
impl Plugin for Amp {
    // Tell the framework which ports this plugin has.
    type Ports = Ports;

    // We don't need any special host features; We can leave them out.
    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    // Create a new instance of the plugin; Trivial in this case.
    fn new(plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self> {
        Some(Self {
            urids: features.map.populate_collection()?,
            ui_active: false,
            ui_notified: false,
            sample_rate: plugin_info.sample_rate()
        })
    }

    // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
    // iterates over.
    fn run(&mut self, ports: &mut Ports, features: &mut ()) {
        let coef = if *(ports.enabled) < 0.5 {
            1.0
        } else if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        let mut out_lvl = 0.0_f32;
        let mut in_lvl = 0.0_f32;
        for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
            *out_frame = in_frame * coef;
            let abs_lvl = (*out_frame).abs();
            if abs_lvl > out_lvl {
                out_lvl = abs_lvl;
            }
            let abs_lvl = (in_frame).abs();
            if abs_lvl > in_lvl {
                in_lvl = abs_lvl;
            }
        }

        **(ports.output_level) = if out_lvl > 1e-8 {
            20.0_f32 * out_lvl.log10()
        } else {
            -160_f32
        };
        **(ports.input_level) = if in_lvl > 1e-8 {
            20.0_f32 * in_lvl.log10()
        } else {
            -160_f32
        };

        let control_sequence = ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        for (_, message) in control_sequence {
            if let Some((header, mut object_reader)) = message.read(self.urids.atom.object, ()) {
                println!("received message");

                if header.otype == self.urids.ui_on {
                    println!("UI went on");
                    self.ui_active = true;
                } else if header.otype == self.urids.ui_off {
                    println!("UI went off");
                    self.ui_active = false;
                }
            }
        }

        if self.ui_active && !self.ui_notified {
            let mut sequence_writer = ports.notify.init(
                self.urids.atom.sequence,
                TimeStampURID::Frames(self.urids.unit.frame)
            ).unwrap();

            let mut object_writer = sequence_writer.init(
                TimeStamp::Frames(0),
                self.urids.atom.object,
                ObjectHeader {
                    id: None,
                    otype: self.urids.ui_state.into_general(),
                }
            ).unwrap();

            object_writer.init(self.urids.sample_rate, None, self.urids.atom.float, self.sample_rate as f32);
            self.ui_notified = true;
            //println!("{:?}", *ports.notify);
        }

    }
}
// The `lv2_descriptors` macro creates the entry point to the plugin library. It takes structs that implement `Plugin` and exposes them. The host will load the library and call a generated function to find all the plugins defined in the library.
//lv2_descriptors!(Amp);

unsafe impl PluginInstanceDescriptor for Amp {
    const DESCRIPTOR: LV2_Descriptor = LV2_Descriptor {
        URI: Self::URI.as_ptr() as *const u8 as *const ::std::os::raw::c_char,
        instantiate: Some(PluginInstance::<Self>::instantiate),
        connect_port: Some(PluginInstance::<Self>::connect_port),
        activate: Some(PluginInstance::<Self>::activate),
        run: Some(PluginInstance::<Self>::run),
        deactivate: Some(PluginInstance::<Self>::deactivate),
        cleanup: Some(PluginInstance::<Self>::cleanup),
        extension_data: Some(PluginInstance::<Self>::extension_data)
    };
}

#[no_mangle]
pub unsafe extern "C" fn lv2_descriptor(index: u32) -> *const LV2_Descriptor {
    println!("my lv2 descriptor called {}", index);
    match index {
        0 => &<Amp as PluginInstanceDescriptor>::DESCRIPTOR,
        _ => std::ptr::null()
    }
}
