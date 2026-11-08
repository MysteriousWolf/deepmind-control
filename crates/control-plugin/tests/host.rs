//! The plugin, loaded the way a DAW loads it.
//!
//! No window and no real host: this is the CLAP interface as C structs, the same
//! entry point a host calls, driven far enough to prove the one thing the plugin
//! is for. A session comes back holding a sound, and the plugin says that sound
//! to the instrument as the MIDI it takes.
#![allow(unsafe_code, reason = "a host is C calling into a library")]
#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "a failed expectation is the test failure"
)]
#![allow(
    clippy::pedantic,
    reason = "a harness that is C structs and casts between them, where the lints that help \
              the library only add noise"
)]

use std::ffi::{CStr, c_char, c_void};
use std::ptr;

use clap_sys::entry::clap_plugin_entry;
use clap_sys::events::{
    CLAP_CORE_EVENT_SPACE_ID, CLAP_EVENT_MIDI, clap_event_header, clap_event_midi,
    clap_input_events, clap_output_events,
};
use clap_sys::ext::gui::{
    CLAP_EXT_GUI, CLAP_WINDOW_API_X11, clap_host_gui, clap_plugin_gui, clap_window,
    clap_window_handle,
};
use clap_sys::ext::state::{CLAP_EXT_STATE, clap_plugin_state};
use clap_sys::factory::plugin_factory::{CLAP_PLUGIN_FACTORY_ID, clap_plugin_factory};
use clap_sys::host::clap_host;
use clap_sys::plugin::clap_plugin;
use clap_sys::process::clap_process;
use clap_sys::stream::clap_istream;
use clap_sys::version::CLAP_VERSION;
use deepmind_midi::ids::ProtocolVersion;
use deepmind_midi::param::{PARAMETER_COUNT, ParamId};
use deepmind_midi::program::Program;

// The framework wrote this symbol into the library; linking the crate is what
// puts it here.
extern crate control_plugin;
unsafe extern "C" {
    static clap_entry: clap_plugin_entry;
}

const BLOCK: u32 = 512;
const RATE: f64 = 48_000.0;

/// The one extension a host that shows a plugin's window has to answer for.
///
/// The framework asks for it before it will open an embedded editor, because a
/// window it cannot ask to be resized or tell was closed is not one it can hand
/// back. Everything here is a host with nothing to resize.
static GUI: clap_host_gui = clap_host_gui {
    resize_hints_changed: Some(nothing),
    request_resize: Some(no_resize),
    request_show: Some(accept),
    request_hide: Some(accept),
    closed: Some(closed),
};

unsafe extern "C" fn no_resize(_: *const clap_host, _: u32, _: u32) -> bool {
    false
}
unsafe extern "C" fn accept(_: *const clap_host) -> bool {
    true
}
unsafe extern "C" fn closed(_: *const clap_host, _: bool) {}

unsafe extern "C" fn extension(_: *const clap_host, id: *const c_char) -> *const c_void {
    if unsafe { CStr::from_ptr(id) } == CLAP_EXT_GUI {
        ptr::from_ref(&GUI).cast()
    } else {
        ptr::null()
    }
}
unsafe extern "C" fn nothing(_: *const clap_host) {}

unsafe extern "C" fn no_events(_: *const clap_input_events) -> u32 {
    0
}
unsafe extern "C" fn no_event(_: *const clap_input_events, _: u32) -> *const clap_event_header {
    ptr::null()
}

/// What the plugin said, with the time it said it, counted in samples from the
/// first block.
struct Heard {
    block: u64,
    said: Vec<(u64, [u8; 3])>,
}

unsafe extern "C" fn hear(
    list: *const clap_output_events,
    event: *const clap_event_header,
) -> bool {
    let heard = unsafe { &mut *(*list).ctx.cast::<Heard>() };
    let header = unsafe { &*event };
    assert_eq!(header.space_id, CLAP_CORE_EVENT_SPACE_ID);
    assert_eq!(header.type_, CLAP_EVENT_MIDI, "only MIDI leaves the plugin");
    let midi = unsafe { &*event.cast::<clap_event_midi>() };
    heard.said.push((
        heard.block * u64::from(BLOCK) + u64::from(header.time),
        midi.data,
    ));
    true
}

struct Stream {
    bytes: Vec<u8>,
    at: usize,
}

unsafe extern "C" fn read(stream: *const clap_istream, buffer: *mut c_void, size: u64) -> i64 {
    let stream = unsafe { &mut *(*stream).ctx.cast::<Stream>() };
    let left = stream.bytes.len() - stream.at;
    let n = left.min(size as usize);
    unsafe {
        ptr::copy_nonoverlapping(stream.bytes.as_ptr().add(stream.at), buffer.cast::<u8>(), n);
    }
    stream.at += n;
    n as i64
}

/// A program that is not every parameter at its minimum, so that what comes out
/// can be told from what an untouched plugin would say.
fn sound() -> Program {
    let mut program = Program::new(ProtocolVersion::V7);
    for parameter in ParamId::ALL.iter().copied() {
        let value = parameter.min() + (parameter.max() - parameter.min()) / 3;
        program
            .set(parameter, u8::try_from(value).unwrap())
            .expect("a value inside the range");
    }
    program
}

/// The state a host would hand back: the program the session was saved with and
/// the channel it was saved on.
fn session(program: &Program, channel: i32) -> Vec<u8> {
    let bytes: Vec<String> = program.as_bytes().iter().map(u8::to_string).collect();
    let field = format!("[{}]", bytes.join(","));
    let json = format!(
        r#"{{"version":"0","params":{{"channel":{{"i32":{channel}}}}},"fields":{{"program":{}}}}}"#,
        serde_json_string(&field),
    );
    // The framework's own framing: CLAP cannot say how long a stream is, so the
    // length goes in front of the JSON.
    let mut state = (json.len() as u64).to_le_bytes().to_vec();
    state.extend_from_slice(json.as_bytes());
    state
}

/// A string as JSON writes it. The field is itself JSON, so its quotes are the
/// only thing that needs escaping.
fn serde_json_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// A host that has nothing to offer: no extension, and nothing to call back.
fn host() -> &'static clap_host {
    Box::leak(Box::new(clap_host {
        clap_version: CLAP_VERSION,
        host_data: ptr::null_mut(),
        name: c"test host".as_ptr(),
        vendor: c"nobody".as_ptr(),
        url: c"".as_ptr(),
        version: c"0".as_ptr(),
        get_extension: Some(extension),
        request_restart: Some(nothing),
        request_process: Some(nothing),
        request_callback: Some(nothing),
    }))
}

/// Loads the library, makes the plugin, and restores `state` into it when there
/// is one: everything a host does before it asks for sound or a window.
fn create(state: Option<Vec<u8>>) -> &'static clap_plugin {
    let entry = unsafe { &clap_entry };
    unsafe {
        assert!(entry.init.expect("init")(c"".as_ptr()));
        let factory = entry.get_factory.expect("get_factory")(CLAP_PLUGIN_FACTORY_ID.as_ptr())
            .cast::<clap_plugin_factory>();
        assert!(!factory.is_null(), "the library has a plugin factory");
        let factory = &*factory;
        assert_eq!(factory.get_plugin_count.expect("count")(factory), 1);

        let descriptor = &*factory.get_plugin_descriptor.expect("descriptor")(factory, 0);
        let id = CStr::from_ptr(descriptor.id).to_str().unwrap();
        assert_eq!(id, "io.github.mysteriouswolf.deepmind-control");
        let features: Vec<&str> = (0..)
            .map(|at| *descriptor.features.add(at))
            .take_while(|feature| !feature.is_null())
            .map(|feature| CStr::from_ptr(feature).to_str().unwrap())
            .collect();
        assert!(features.contains(&"note-effect"), "{features:?}");

        let plugin = factory.create_plugin.expect("create")(factory, host(), descriptor.id);
        assert!(!plugin.is_null());
        let plugin: &clap_plugin = &*plugin;
        assert!(plugin.init.expect("init")(plugin));

        if let Some(bytes) = state {
            let stateful =
                &*plugin.get_extension.expect("extension")(plugin, CLAP_EXT_STATE.as_ptr())
                    .cast::<clap_plugin_state>();
            let mut stream = Stream { bytes, at: 0 };
            let stream = clap_istream {
                ctx: ptr::from_mut(&mut stream).cast(),
                read: Some(read),
            };
            assert!(
                stateful.load.expect("load")(plugin, &stream),
                "the state loads"
            );
        }
        plugin
    }
}

/// Lets the plugin go, and the library with it.
fn finish(plugin: &clap_plugin) {
    unsafe {
        plugin.destroy.expect("destroy")(plugin);
        clap_entry.deinit.expect("deinit")();
    }
}

/// Runs a plugin that was restored from `state` for long enough to say all of
/// it, and returns what it said.
fn run(state: Option<Vec<u8>>, blocks: u64) -> Vec<(u64, [u8; 3])> {
    let plugin = create(state);
    let mut heard = Heard {
        block: 0,
        said: Vec::new(),
    };
    unsafe {
        assert!(plugin.activate.expect("activate")(plugin, RATE, 16, BLOCK));
        assert!(plugin.start_processing.expect("start")(plugin));

        let in_events = clap_input_events {
            ctx: ptr::null_mut(),
            size: Some(no_events),
            get: Some(no_event),
        };
        let out_events = clap_output_events {
            ctx: ptr::from_mut(&mut heard).cast(),
            try_push: Some(hear),
        };
        for block in 0..blocks {
            heard.block = block;
            let process = clap_process {
                steady_time: i64::try_from(block * u64::from(BLOCK)).unwrap(),
                frames_count: BLOCK,
                transport: ptr::null(),
                audio_inputs: ptr::null(),
                audio_outputs: ptr::null_mut(),
                audio_inputs_count: 0,
                audio_outputs_count: 0,
                in_events: &in_events,
                out_events: &out_events,
            };
            let status = plugin.process.expect("process")(plugin, &process);
            assert_ne!(status, clap_sys::process::CLAP_PROCESS_ERROR);
        }

        plugin.stop_processing.expect("stop")(plugin);
        plugin.deactivate.expect("deactivate")(plugin);
    }
    finish(plugin);
    heard.said
}

/// The id of the root window of the display `DISPLAY` names, which is where a
/// host with no window of its own can put a plugin.
fn root_window() -> u64 {
    let out = std::process::Command::new("xwininfo")
        .arg("-root")
        .output()
        .expect("xwininfo is installed");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| line.split("Window id: ").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|id| u64::from_str_radix(id.trim_start_matches("0x"), 16).ok())
        .expect("xwininfo names the root window")
}

/// Opens the editor on a display, shows it for a while, and closes it.
///
/// Needs a display, so it is not part of the ordinary run:
///
/// ```text
/// xvfb-run -s "-screen 0 1700x1400x24" cargo test -p control-plugin -- --ignored
/// ```
///
/// With `WINDOW_SHOT` set to a path, the display is written there as a PNG
/// before the window goes away, so that somebody can look at what opened.
#[test]
#[ignore = "needs an X display"]
fn the_editor_opens_and_draws() {
    let plugin = create(Some(session(&sound(), 1)));
    unsafe {
        let gui = &*plugin.get_extension.expect("extension")(plugin, CLAP_EXT_GUI.as_ptr())
            .cast::<clap_plugin_gui>();
        assert!(gui.is_api_supported.expect("supported")(
            plugin,
            CLAP_WINDOW_API_X11.as_ptr(),
            false
        ));
        assert!(gui.create.expect("create")(
            plugin,
            CLAP_WINDOW_API_X11.as_ptr(),
            false
        ));
        let (mut width, mut height) = (0, 0);
        assert!(gui.get_size.expect("size")(plugin, &mut width, &mut height));
        assert!(width > 400 && height > 400, "{width}x{height}");

        let parent = clap_window {
            api: CLAP_WINDOW_API_X11.as_ptr(),
            specific: clap_window_handle { x11: root_window() },
        };
        assert!(gui.set_parent.expect("parent")(plugin, &parent));
        assert!(gui.show.expect("show")(plugin));

        // The window draws on its own thread; give it a moment and a few frames.
        std::thread::sleep(std::time::Duration::from_secs(3));

        if let Some(path) = std::env::var_os("WINDOW_SHOT") {
            let shot = std::process::Command::new("import")
                .args(["-window", "root"])
                .arg(&path)
                .status()
                .expect("import is installed");
            assert!(shot.success());
            // A window that opened and drew is not one colour.
            let colours = std::process::Command::new("convert")
                .arg(&path)
                .args(["-format", "%k", "info:"])
                .output()
                .expect("convert is installed");
            let colours: u32 = String::from_utf8_lossy(&colours.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            assert!(colours > 20, "the screen has {colours} colours");
        }

        assert!(gui.hide.expect("hide")(plugin));
        gui.destroy.expect("destroy")(plugin);
    }
    finish(plugin);
}

#[test]
fn a_plugin_nobody_has_touched_says_nothing() {
    assert_eq!(run(None, 40), Vec::new());
}

#[test]
fn a_session_comes_back_as_the_sound_it_was_saved_with() {
    let program = sound();
    // 242 edits at five milliseconds each, and a block of ten, with room to spare.
    let said = run(Some(session(&program, 3)), 200);
    assert_eq!(
        said.len(),
        PARAMETER_COUNT * 4,
        "four control changes an edit"
    );

    // Every message is a control change on channel three, which is index two.
    for (_, data) in &said {
        assert_eq!(data[0], 0xB2, "{data:?}");
    }

    // And each four are the NRPN for the parameter, with the value it was saved at.
    let mut heard = Vec::new();
    for edit in said.as_chunks::<4>().0 {
        let [number_high, number_low, value_high, value_low] = edit;
        assert_eq!(number_high.1[1], 99);
        assert_eq!(number_low.1[1], 98);
        assert_eq!(value_high.1[1], 6);
        assert_eq!(value_low.1[1], 38);
        let offset = u16::from(number_high.1[2]) << 7 | u16::from(number_low.1[2]);
        let value = u16::from(value_high.1[2]) << 7 | u16::from(value_low.1[2]);
        heard.push((edit[0].0, offset, value));
    }
    for ((_, offset, value), parameter) in heard.iter().zip(ParamId::ALL) {
        assert_eq!(*offset, u16::from(parameter.offset()));
        assert_eq!(*value, u16::from(program.get(*parameter)), "{parameter}");
    }

    // At the pace of a cable, not all at once: a synthesizer at 31250 baud
    // drops what it is sent faster than that.
    let gap = (0.005 * RATE) as u64;
    for pair in heard.windows(2) {
        assert!(
            pair[1].0 >= pair[0].0 + gap - 1,
            "{pair:?} closer than {gap}"
        );
    }
}
