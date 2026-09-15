//! The window: what it draws, and what wakes it up.

use std::thread;
use std::time::Duration;

use control_ui::Confidence;
use deepmind_midi::param::Group;
use iced::futures::Stream;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, container, pick_list, row, scrollable, space, text};
use iced::{Center, Element, Fill, Length, Subscription, Theme};

use crate::app::{App, Message};

/// How often the window looks at what the device thread has said.
///
/// A frame. The thread is already coalescing edits at its own rate and queueing
/// what arrives, so this is how late an answer can be on the screen rather than
/// how often anything is asked for.
const FRAME: Duration = Duration::from_millis(16);

/// Opens the window and runs until it is closed.
///
/// # Errors
///
/// Returns whatever iced could not do: a window it could not open, or a graphics
/// backend it could not start.
pub fn run() -> iced::Result {
    iced::application(App::booted, App::update, view)
        .title(title)
        .theme(theme)
        .subscription(subscription)
        .window_size((900.0, 760.0))
        .run()
}

/// What the window is drawn in.
///
/// The instrument is a dark panel, so the window is one. Every colour in it
/// comes from this theme, which lives in `control-ui` because the plugin is
/// drawn in the same one.
fn theme(_app: &App) -> Theme {
    control_ui::deepmind()
}

/// What the title bar says.
fn title(app: &App) -> String {
    match app.patch().name() {
        Some(name) => format!("{} \u{2014} deepmind control", name.as_str().trim()),
        None => "deepmind control".to_owned(),
    }
}

/// What wakes the window up.
///
/// Nothing at all until a port is open, because a window with nothing to hear
/// has nothing to redraw.
fn subscription(app: &App) -> Subscription<Message> {
    if app.is_connected() {
        Subscription::run(ticks)
    } else {
        Subscription::none()
    }
}

/// A tick every frame, from a thread and a sleep.
///
/// An async runtime would give the same thing for the price of a dependency the
/// size of tokio, and what this needs from one is `thread::sleep`.
fn ticks() -> impl Stream<Item = Message> {
    let (mut sender, receiver) = mpsc::channel(1);
    let spawned = thread::Builder::new()
        .name("deepmind-control frames".to_owned())
        .spawn(move || {
            loop {
                thread::sleep(FRAME);
                // A tick nobody took means the window has not drawn since the
                // last one, and a second one would not help it. A receiver that
                // has gone means the port has been put down.
                if sender
                    .try_send(Message::Tick)
                    .is_err_and(|refused| refused.is_disconnected())
                {
                    return;
                }
            }
        });
    debug_assert!(spawned.is_ok(), "the frame thread");
    receiver
}

/// The whole window.
fn view(app: &App) -> Element<'_, Message> {
    let section = app.section();
    column![
        header(app),
        identity(app),
        controls(app),
        // The bar stays put while the rack under it scrolls: it is how a panel
        // is left, and a control that scrolls away is a control that is looked
        // for.
        control_ui::sections(app.patch(), section).map(Message::Ui),
        scrollable(
            column![
                text(section.name()).size(18),
                control_ui::group(app.patch(), section, app.firmware()).map(Message::Ui),
            ]
            .extend(caveat(section))
            .push(remaining())
            .spacing(12)
            .padding([0, 12]),
        )
        .height(Fill),
    ]
    .spacing(12)
    .padding(16)
    .into()
}

/// The port picker, and what to do with a port.
fn header(app: &App) -> Element<'_, Message> {
    let ports = app.ports().to_vec();
    let connection = if app.is_connected() {
        button("Close").on_press(Message::Disconnect)
    } else {
        button("Open").on_press_maybe(app.chosen().map(|_| Message::Connect))
    };
    row![
        text("deepmind control").size(20),
        space().width(Fill),
        pick_list(ports, app.chosen().cloned(), Message::Choose)
            .placeholder("MIDI port")
            .width(Length::Fixed(260.0)),
        button("Rescan").on_press(Message::Rescan),
        connection,
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// Who is on the other end, and which value tables that makes true.
fn identity(app: &App) -> Element<'_, Message> {
    let who = match (app.identity(), app.channel()) {
        (Some(identity), Some(channel)) => format!(
            "{} \u{00b7} firmware {} \u{00b7} voice {} \u{00b7} channel {}",
            identity.device,
            identity.firmware,
            identity.voice,
            channel.number()
        ),
        _ if app.is_connected() => format!(
            "Nobody has answered yet \u{00b7} assuming firmware {}",
            app.firmware()
        ),
        _ => format!(
            "Nothing is open \u{00b7} assuming firmware {}",
            app.firmware()
        ),
    };
    let trouble = app.trouble().map(|trouble| text(trouble).size(13));
    container(
        column![text(who).size(13), text(app.status()).size(13)]
            .extend(trouble.map(Element::from))
            .spacing(4),
    )
    .width(Fill)
    .padding(10)
    .style(container::bordered_box)
    .into()
}

/// What can be asked of the synthesizer, and what backs what is on the screen.
fn controls(app: &App) -> Element<'_, Message> {
    let open = app.is_connected();
    let sound = match app.patch().confidence() {
        Confidence::Unknown => "Nothing has been read.".to_owned(),
        Confidence::Assumed => "Edited here since the last read.".to_owned(),
        Confidence::Confirmed => match app.patch().name() {
            Some(name) => format!(
                "{} \u{2014} as the synthesizer described it.",
                name.as_str().trim()
            ),
            None => "As the synthesizer described it.".to_owned(),
        },
    };
    row![
        button("Read the edit buffer").on_press_maybe(open.then_some(Message::Read)),
        button("Ask who is there").on_press_maybe(open.then_some(Message::Identify)),
        text(sound).size(13),
        space().width(Fill),
        control_ui::legend().map(Message::Ui),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// What a section cannot say for itself yet, where it has something to admit.
///
/// Printed under the rack rather than left for somebody to work out from a
/// panel of protocol names. A panel drawn from the library's own table is
/// complete and, in two places, ugly, and saying which two is cheaper than
/// pretending otherwise.
fn caveat(section: Group) -> Option<Element<'static, Message>> {
    let admission = match section {
        Group::Effects => {
            "Four engines, twelve slots each, under the names the protocol gives them. What a \
             slot means depends on which of the 35 algorithms is loaded, and the table that says \
             so is generated from the library's specification rather than transcribed here: the \
             readable panel arrives when the library publishes it."
        }
        Group::Program => {
            "The name is seventeen parameters, one character each, because that is how the \
             instrument stores it. The title bar reads them as a word."
        }
        _ => return None,
    };
    Some(text(admission).size(12).into())
}

/// What this window does not draw yet, said out loud.
fn remaining() -> Element<'static, Message> {
    text(
        "Every parameter the instrument has is on these fourteen panels, drawn from the \
         library's own table. Laying each panel out by hand is the rest of stage 3, the \
         librarian is stage 4, and the effect panels wait on the library publishing their \
         tables. Nothing here writes a program into the synthesizer: the manual describes no \
         message that would, so storing a sound into a slot is done at the panel with the \
         instrument's own WRITE.",
    )
    .size(12)
    .into()
}
