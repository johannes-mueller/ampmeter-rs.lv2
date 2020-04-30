use lv2::prelude::*;

// The input and output ports are defined by a struct which implements the `PortCollection` trait.
// In this case, there is an input control port for the gain of the amplification, an input audio
// port and an output audio port.
#[derive(PortCollection)]
struct Ports {
    gain: InputPort<Control>,
    enabled: InputPort<Control>,
    input: InputPort<Audio>,
    output: OutputPort<Audio>,
}
/*
struct MyPortsPointerCache {
    gain: *mut std::ffi::c_void,
    input: *mut std::ffi::c_void,
    output: *mut std::ffi::c_void
}

impl Default for MyPortsPointerCache {
    fn default() -> Self {
	Self {
	    gain: std::ptr::null_mut(),
	    input: std::ptr::null_mut(),
	    output: std::ptr::null_mut()
	}
    }
}

impl PortPointerCache for MyPortsPointerCache {
    fn connect(&mut self, index: u32, pointer: *mut std::ffi::c_void) {
	match index {
	    0 => { self.gain = pointer },
	    1 => { self.input = pointer },
	    2 => { self.output = pointer }
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
*/

// The plugin struct. In this case, we don't need any data and therefore, this struct is empty.
//
// LV2 uses URIs to identify types. This association is expressed via the `UriBound` trait,
// which tells the framework that the type `Amp` is identified by the given URI. The usual
// way to implement this trait is to use the `uri` attribute.
#[uri("https://johannes-mueller.org/lv2/ampmeter-rs#lv2")]
struct Amp;

// The implementation of the `Plugin` trait, which turns `Amp` into a plugin.
impl Plugin for Amp {
    // Tell the framework which ports this plugin has.
    type Ports = Ports;

    // We don't need any special host features; We can leave them out.
    type InitFeatures = ();
    type AudioFeatures = ();

    // Create a new instance of the plugin; Trivial in this case.
    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
        Some(Self)
    }

    // Process a chunk of audio. The audio ports are dereferenced to slices, which the plugin
    // iterates over.
    fn run(&mut self, ports: &mut Ports, _features: &mut ()) {
        let coef = if *(ports.enabled) < 0.5 {
	    1.0
	} else if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
            *out_frame = in_frame * coef;
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
