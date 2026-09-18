//! The window: what it draws, and what wakes it up.

use std::thread;
use std::time::Duration;

use control_ui::{Band, Confidence, Ink, Screen, Size};
use deepmind_midi::param::{Group, ParamId};
use iced::futures::Stream;
use iced::futures::channel::mpsc;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, space, svg, text, tooltip,
};
use iced::{Center, Element, Fill, Length, Subscription, Theme, keyboard};

use crate::app::{App, Message, View};
use crate::{files, librarian};

/// How much window there is between a surface and the bar it scrolls on.
///
/// The bar takes its own room rather than floating over the last thing in the
/// row, which is what a scroll bar cut into a panel does.
const BESIDE: f32 = 6.0;

/// How wide that bar is, which is the toolkit's own width for one.
///
/// Written down here because the window opens wide enough for the panel and
/// everything beside it, and the bar is beside it.
const BAR: f32 = 10.0;

/// How much window there is around everything in it.
const GROUND: f32 = 16.0;

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
    // Whatever the desktop handed this application to open, which is the one
    // way onto the shelf that does not go through a dialog.
    let named = files::named();
    iced::application(move || App::opening(&named), App::update, view)
        .title(title)
        // The face the instrument's own legends are printed in, asked for once
        // here so that every unstyled word in the window is already in it.
        .default_font(control_ui::printed())
        .theme(theme)
        .subscription(subscription)
        .window_size(window_size())
        .run()
}

/// How big a window opens.
///
/// As wide as the widest surface measures, plus the ground it stands on and the
/// bar it scrolls on, so a window opens on the instrument's own arrangement
/// rather than on a wrapped one. Narrower than this and the panel's rows wrap,
/// which is readable and is no longer two rows and a screen, and the effects
/// page draws its chain past the edge of its own glass, which is not readable at
/// all.
///
/// Written down rather than spelled out in [`run`] because the previews open the
/// same window: a picture taken at a size the application never opens at is a
/// picture of a layout nobody sees.
pub(crate) fn window_size() -> (f32, f32) {
    let panel = control_ui::panel_width() + GROUND * 2.0 + BAR + BESIDE;
    // The widest thing the editor draws is the effects, and they are drawn on a
    // sheet rather than on the window: a sheet spends its margin, its own
    // padding and the bar it scrolls on before anything is on it, and a window
    // sized as though the rack stood on the panel is a window that draws the
    // chain past the edge of its own glass.
    let sheet = control_ui::effects_width() + control_ui::margins();
    // And wide enough for the band of ways in with every cap's name spelled
    // out. The band fits itself into whatever it is given by dropping the
    // names and keeping the marks, which is what a row of buttons on the
    // instrument is anyway — but a window that *opens* on that has thrown away
    // the one thing the row has to say about where each press goes.
    let band = control_ui::ways_width() + GROUND * 2.0;
    (panel.max(sheet).max(band), DEEP)
}

/// How deep it opens.
///
/// The one number here that no surface measures: a panel is as tall as it is and
/// a window is as tall as a screen has room for. Deep enough for the front panel
/// without a scroll and short enough for a laptop.
const DEEP: f32 = 940.0;

/// What the window is drawn in.
///
/// The instrument is a dark panel, so the window is one. Every colour in it
/// comes from this theme, which lives in `control-ui` because the plugin is
/// drawn in the same one.
pub(crate) fn theme(app: &App) -> Theme {
    if app.is_negative() {
        control_ui::negative()
    } else {
        control_ui::deepmind()
    }
}

/// What the title bar says.
pub(crate) fn title(app: &App) -> String {
    match app.patch().name() {
        Some(name) => format!("{} \u{2014} deepmind control", name.as_str().trim()),
        None => "deepmind control".to_owned(),
    }
}

/// What wakes the window up.
///
/// Two things, and a window with its port down and nothing open over it hears
/// neither: there is nothing to redraw and nothing to dismiss.
///
/// The frames are the port's. The key is the modal's, and it is here rather
/// than in `control_ui` because a key is an event before it is a press: the
/// view layer has no runtime to listen in, so the application listens and sends
/// the same message the mark on the sheet sends.
pub(crate) fn subscription(app: &App) -> Subscription<Message> {
    let frames = if app.is_connected() {
        Subscription::run(ticks)
    } else {
        Subscription::none()
    };
    let escape = if app.editing().is_some() {
        keyboard::listen().with(()).filter_map(dismissal)
    } else {
        Subscription::none()
    };
    Subscription::batch([frames, escape])
}

/// Turns the escape key into the message that puts a sheet away.
///
/// Every other key goes past. `keyboard::listen` carries only the events
/// nothing in the window took, so a name being typed into the program's own
/// field keeps its keystrokes and this never sees them.
fn dismissal(((), event): ((), keyboard::Event)) -> Option<Message> {
    matches!(
        event,
        keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }
    )
    .then_some(Message::Ui(control_ui::Message::Close))
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
///
/// Everything stands on the panel: the gradient the mark fills its case with,
/// lit at the top where the light is, rather than the flat dark the theme's own
/// background would give. It is the one surface in the window nothing is cut
/// into, so it is drawn once, here, around all of it.
///
/// Three bands down: the header, the band of ways in, and then the surface
/// itself with whatever sheet is lying over it. The footer is under all of them.
///
/// There were four. The second was a switch between the two surfaces, `Panel`
/// and `Library`, which said the shelf is a different kind of thing from the
/// sections when what it is, to a hand, is another place this window can be. It
/// is a cap in the band now, at the end of the row, and one row of presses does
/// what two did.
pub(crate) fn view(app: &App) -> Element<'_, Message> {
    // The band stands *outside* the sheet's shade, which is the whole of what
    // makes it a row of tabs: a press on it while a section is open swaps the
    // sheet, the way the second `EDIT` on the instrument does, instead of
    // landing on the shade and closing what is up.
    let window = column![
        header(app),
        control_ui::ways(app.showing()).map(Message::Ui)
    ];
    let window = container(
        window
            .push(showing(app))
            // Along the foot, under whichever surface is showing, because a
            // control is pointed at on all three of them and a footer that
            // moved with the surface would be a different footer each time.
            .push(status(app))
            .spacing(12)
            .padding(GROUND),
    )
    .width(Fill)
    .height(Fill)
    .style(control_ui::ground);
    window.into()
}

/// Whichever surface is showing, with whatever is open over it.
///
/// The sheet is laid over the surface rather than over the whole window, which
/// is a change the band of tabs asked for: a shade across everything is a shade
/// across the tabs, and a tab that shuts the sheet instead of changing it is not
/// a tab. What is left under the shade is what a sheet is the detail of — the
/// panel it was opened from — and what stays out from under it is the chrome
/// that is true whatever is open: the port, the switch between the surfaces, the
/// band, and the footer saying what is under the pointer.
///
/// The three ways out of a sheet are all still there. The margin around it is
/// still a press on the window that closes, because the shade still reaches
/// every edge of the surface.
fn showing(app: &App) -> Element<'_, Message> {
    let surface: Element<'_, Message> = match app.view() {
        // The panel scrolls for the same reason the rack does, and it
        // did not have to before its plates carried displays: two rows
        // of plates are taller than a window somebody has made short,
        // and a front panel with its lower row below the fold is a
        // front panel with the whole voice missing.
        View::Panel => scrollable(
            control_ui::panel(
                app.patch(),
                app.firmware(),
                app.livery(),
                app.mapper(),
                |screen| {
                    paint(screen, app);
                },
            )
            .map(Message::Ui),
        )
        // The bar is cut into the window beside the panel rather than
        // laid over it. The panel is drawn out to the room it is given
        // now, so a bar that floated over the end of it would be a bar
        // over the last plate of every row, and a panel that shrank away
        // from one the moment there was enough of it to scroll.
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().spacing(BESIDE),
        ))
        .height(Fill)
        .into(),
        View::Library => librarian::view(app),
    };
    // And over it, where a way in has been pressed, the section it opened. What
    // is underneath is still drawn: it is where the sheet came from and where
    // putting the sheet away goes back to.
    match app.editing() {
        Some(section) => container(control_ui::modal(
            surface,
            opened(app, section),
            Message::Ui(control_ui::Message::Close),
        ))
        .height(Fill)
        .into(),
        None => surface,
    }
}

/// The sheet one of the panel's `EDIT` presses opened.
///
/// The section's rack, on the frame `control_ui` draws every modal on: the
/// name it was printed under on the plate somebody pressed, the claim that
/// section is under, and the mark that puts it away.
///
/// It is the same [`control_ui::group`] the surface below it drew, taking the
/// room a sheet has rather than the room a rack under a tab bar had. What a
/// control is has not changed, and neither has what drew it: the arrangement is
/// what a modal is a change to.
fn opened(app: &App, section: Group) -> Element<'_, Message> {
    control_ui::sheet(
        control_ui::section_name(section),
        app.patch().claim_of(section),
        control_ui::group(app.patch(), section, app.firmware(), app.mapper()),
    )
    .map(Message::Ui)
}

/// The strip along the foot: what is under the pointer, and what can be asked
/// of the synthesizer.
///
/// A status bar, which is where a desktop application puts the things that are
/// true of the window rather than of the page in it. All three of these were on
/// their own rows under the header, which is the most expensive room on the
/// screen and was being spent on two presses somebody uses twice a session and
/// a sentence the instrument's own display was already printing.
fn status(app: &App) -> Element<'_, Message> {
    let open = app.is_connected();
    row![
        control_ui::footer(
            app.pointed(),
            app.patch(),
            app.firmware(),
            app.mapper().mapped(),
            app.hinted(),
        )
        .map(Message::Ui),
        // At the right-hand end, in the order somebody reaches for them: ask
        // the instrument what it is, ask it what it is playing, and turn the
        // glass over.
        chrome(control_ui::WHO)
            .on_press_maybe(open.then_some(Message::Identify))
            .on_hint("Ask the synthesizer what it is: device, firmware, voices and channel."),
        chrome(control_ui::READ)
            .on_press_maybe(open.then_some(Message::Read))
            .on_hint("Read the sound the synthesizer is playing into this window."),
        livery(app),
        glass(app),
    ]
    .spacing(6)
    .align_y(Center)
    .into()
}

/// The press that wears the other `DeepMind`'s front.
///
/// A `12` and a `12D` print every section name in white caps on the bare panel;
/// a `12X` knocks the same names out of filled banners, red down the signal
/// path, blue on the arpeggiator and the high-pass, white on the envelopes.
/// Both are the instrument, and which one somebody is looking at is a fact
/// about the window rather than about the sound, so it lives at the foot of it
/// beside the press that turns the glass over.
///
/// It shows the livery it is about to *give* you, which is the rule that press
/// already follows: a swatch of a red banner while the panel is plain, and the
/// panel's own outline while it is not.
fn livery(app: &App) -> Element<'_, Message> {
    let next = match app.livery() {
        control_ui::Livery::Plain => control_ui::Livery::Banners,
        control_ui::Livery::Banners => control_ui::Livery::Plain,
    };
    pressed(control_ui::livery_swatch(next).map(Message::Ui))
        .on_press(Message::Wear)
        .on_hint("Wear the other DeepMind's front: banners for a 12X, plain for a 12.")
}

/// The press that turns the displays over.
///
/// Every display in the window turns over together, the way a screen has one
/// backlight, so it belongs to the window rather than to any one surface, and
/// the foot of the window is where the things that are true of the window live.
///
/// It is a display rather than a word. `Negative display` said which way up they
/// would be, in a sentence, on a row of sentences; a screen the size of a
/// character showing itself the way it is about to be says the same thing
/// without being read, and says it in the one material this press is about.
fn glass(app: &App) -> Element<'_, Message> {
    pressed(Element::from(control_ui::swatch(!app.is_negative())).map(Message::Ui))
        .on_press(Message::Invert)
        .on_hint("Turn every display in this window over, dark glass for light.")
}

/// What the instrument's own display would be showing./// What the instrument's own display would be showing.
///
/// The panel leaves a screen-shaped hole in itself and the application fills
/// it, because what a display says is the application's business rather than
/// the view layer's: which sound is on the screen, what backs it, and what the
/// last thing to happen was. A plugin has different answers to all three.
///
/// It is written in dots, on the same glass at the same pitch as the strip over
/// every plate, because it is the same display. What it is not is a panel of
/// toolkit text in a dark box: the instrument's screen is a dot matrix, and a
/// program name set in the machine's own sans is the one thing on this window
/// that would be pretending to be something it is not.
/// Writes what the display is showing onto `screen`.
///
/// Apart from [`screen`] so that what is drawn can be looked at without a
/// window to draw it in, which for a display made of dots is the only way to
/// look at it at all.
fn paint(screen: &mut Screen, app: &App) {
    let all = screen.all();
    let small = Screen::height_of(Size::Small);

    // The heading, inverted, which is how a display with one colour of light
    // says that a line is a heading rather than a reading.
    screen.write(2, 1, "PROGRAM", Size::Small);
    if let Some(category) = category(app) {
        let from = all.width - 2 - Screen::width_of(category, Size::Small);
        screen.write(from, 1, category, Size::Small);
    }
    screen.invert(Band::new(0, 0, all.width, small + 2));

    // The sound's own name, in the one size this window writes anything twice
    // over, because it is the one thing on the screen somebody is looking for.
    let name = app.patch().name().map_or_else(
        || "\u{2014}".to_owned(),
        |name| name.as_str().trim().to_owned(),
    );
    let title = small + 8;
    screen.centre(title, &name, Size::Large);
    screen.across(4, title + small * 2 + 4, all.width - 8, Ink::Dotted);

    // What backs it, in the display's own words rather than a colour: the dots
    // are already the claim's colour, and a screen that said it only in copper
    // would be saying it only to the readers who see copper.
    let backing = match app.patch().confidence() {
        Confidence::Unknown => "nothing has been read",
        Confidence::Assumed => "this window's claim",
        Confidence::Confirmed => "as the synthesizer described it",
    };
    let mut line = title + small * 2 + 9;
    for words in fold(backing, all.width) {
        screen.centre(line, &words, Size::Small);
        line += small + 2;
    }

    // And the last thing that happened, along the bottom, where the instrument
    // prints what it is doing.
    let status = fold(app.status(), all.width);
    let mut line = all.height - (small + 2) * i32::try_from(status.len()).unwrap_or_default() - 1;
    for words in status {
        screen.write(2, line, &words, Size::Small);
        line += small + 2;
    }
}

/// What the program calls itself, where the instrument has a word for it.
///
/// The library's own table, read for the firmware that answered, like every
/// other name in this window.
fn category(app: &App) -> Option<&'static str> {
    let value = app.patch().value(ParamId::ProgramCategory)?;
    ParamId::ProgramCategory.label_for(u16::from(value), app.firmware())
}

/// Breaks `words` into the lines a screen `columns` dots wide can hold.
///
/// A display wraps or it clips, and a status that clipped would stop saying
/// what went wrong half way through the sentence. Broken between words where
/// there is one, and never more than three lines, because the fourth would be
/// over the program's name.
fn fold(words: &str, columns: i32) -> Vec<String> {
    let each = ((columns - 4) / (Screen::width_of("n", Size::Small) + 1)).max(1);
    let each = usize::try_from(each).unwrap_or(1);
    let mut lines: Vec<String> = Vec::new();
    for word in words.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= each => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.chars().take(each).collect()),
        }
    }
    lines.truncate(3);
    lines
}

/// The name of the thing, what is known about the instrument, the port picker,
/// and what to do with a port.
///
/// One line, and the name is the only thing on it that is not a control. It
/// carried the project's mark and a subtitle on a case with wooden end cheeks,
/// which is `docs/banner.svg` reproduced. The banner is the picture that
/// introduces this project to somebody who has never seen it, which is not the
/// job of the top of a window somebody has open all afternoon. The mark is the
/// application's icon, where an icon belongs.
///
/// What is left is the wordmark, sliced the way the banner slices it, standing
/// on the panel with the port beside it.
fn header(app: &App) -> Element<'_, Message> {
    let ports = app.ports().to_vec();
    // A socket with a plug in it where one is open, and an empty one where none
    // is: the two presses do the same thing to the same port and the word that
    // told them apart is in the footer now, so the mark is what says which of
    // them this is.
    let connection = if app.is_connected() {
        chrome(control_ui::PLUGGED)
            .on_press(Message::Disconnect)
            .on_hint("Put this port down.")
    } else {
        chrome(control_ui::PORT)
            .on_press_maybe(app.chosen().map(|_| Message::Connect))
            .on_hint("Open the chosen port and listen on it.")
    };
    row![
        wordmark(),
        space().width(Fill),
        // Beside the picker, because everything it says is about whatever that
        // picker has open.
        about(app),
        pick_list(ports, app.chosen().cloned(), Message::Choose)
            .placeholder("MIDI port")
            .font(control_ui::printed())
            .text_size(13)
            .padding([5, 10])
            .style(control_ui::selector)
            .menu_style(control_ui::shortlist)
            .width(Length::Fixed(220.0)),
        chrome(control_ui::RESCAN)
            .on_press(Message::Rescan)
            .on_hint("Look for MIDI ports again."),
        connection,
    ]
    .spacing(10)
    // Against the baseline the name sits on rather than against the middle of
    // the line it stands in: a thirty-four point word beside a twenty-four
    // point press, centred, is a press floating in the middle of a word.
    .align_y(iced::alignment::Vertical::Bottom)
    .into()
}

/// What is known about the instrument on the other end of the port, under a
/// press that opens when the pointer is over it.
///
/// Two rows of sentences used to stand under the header saying this: who
/// answered, what the last thing to happen was, and whatever went wrong, on
/// every page, whether or not anybody was asking. It is four facts about a
/// cable that change perhaps twice a session, and it was being given the most
/// expensive room on the screen.
///
/// So it is one press, and what it says is laid out as the rows it always was
/// rather than as a paragraph: a name, a number, a number and a number line up
/// down a column, which is what somebody comparing them against the back of an
/// instrument is doing.
fn about(app: &App) -> Element<'_, Message> {
    let known: Vec<(&'static str, String)> = match (app.identity(), app.channel()) {
        (Some(identity), Some(channel)) => vec![
            ("Device", identity.device.to_string()),
            ("Firmware", identity.firmware.to_string()),
            ("Voice", identity.voice.to_string()),
            ("Channel", channel.number().to_string()),
        ],
        // What this window is assuming meanwhile, said as an assumption. The
        // firmware decides which value tables every list in the window is drawn
        // from, so it is never not an answer: it is either the instrument's or
        // this window's, and which of those it is is the whole distinction.
        _ => vec![
            ("Device", "nobody has answered".to_owned()),
            ("Firmware", format!("{} (assumed)", app.firmware())),
        ],
    };
    let rows = known.into_iter().map(|(name, said)| {
        row![
            text(name)
                .size(11)
                .width(Length::Fixed(62.0))
                .style(|theme: &Theme| text::Style {
                    color: Some(control_ui::materials(theme).metal_low),
                }),
            text(said).size(11).font(control_ui::reading()),
        ]
        .spacing(8)
        .into()
    });
    let said = column(rows)
        .push(text(app.status()).size(11).width(Length::Fixed(240.0)))
        .extend(app.trouble().map(|trouble| {
            text(trouble)
                .size(11)
                .width(Length::Fixed(240.0))
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.extended_palette().danger.base.color),
                })
                .into()
        }))
        .spacing(4);
    tooltip(
        pressed(badge(control_ui::ABOUT)),
        container(said).padding(10).style(control_ui::bay),
        tooltip::Position::Bottom,
    )
    .gap(6)
    .into()
}

/// The project's own name, as `docs/wordmark.svg` sets it.
///
/// A file rather than a drawing. The name is the banner's name, outlined from
/// the mark's own face, tracked the way the banner tracks it, and cut by the
/// five slices that *are* the mark. What this window did instead was set the
/// word in a font and lay five rectangles over it, each of them
/// under two points tall and each rounded to whatever the pointer's device
/// gives it. Five lines from a sixtieth of an em to a twentieth, rounded
/// independently, are five bands of grey at five weights the mark does not
/// have: what somebody sees is stripes rather than a slice.
///
/// As vector it is the same geometry at any size, which is the whole of what
/// the mark asks for, and it is the *same file's* geometry as the banner, so
/// the two cannot drift.
fn wordmark() -> Element<'static, Message> {
    svg(svg::Handle::from_memory(WORDMARK))
        .width(Length::Shrink)
        .height(Length::Fixed(NAME))
        .into()
}

/// The name, as the bytes of the file that draws it.
///
/// Carried in the binary rather than read from disk: it is the application's
/// own name, not something a user chooses, and a window whose title depended on
/// a file beside the executable is a window that can be shipped without one.
const WORDMARK: &[u8] = include_bytes!("../../../docs/wordmark.svg");

/// How tall the name is drawn.
///
/// The width follows from it, because the file knows its own proportions. It is
/// the height of the ink and the slices together, the word plus what hangs
/// below it, so a header built around it has the room the mark actually needs.
const NAME: f32 = 54.0;

/// A press whose whole face is a mark, and what it says about itself.
///
/// It was a mark and a word inside a rounded rectangle with a metal rim. Three
/// of those along the header and two along the foot are five rims on a panel
/// whose own controls have none, and the rim was drawn to the height of the
/// *words*, so a nine-dot mark stood in the middle of it with four points of
/// panel above and below. A mark scaled to a box built for type is a mark that
/// is never the size it was drawn at.
///
/// So the word goes to the footer, where this window already says what is under
/// the pointer, and what is left on the panel is the mark: stencilled on it the
/// way the numbers on a rack unit's case are stencilled on the case, with the
/// panel lifting under the pointer and dipping while it is held. What it does
/// is said by [`on_hint`](Hinted::on_hint), which every one of them carries.
fn chrome(mark: control_ui::Badge) -> button::Button<'static, Message, Theme, iced::Renderer> {
    pressed(badge(mark))
}

/// Says what a press does in the footer while the pointer is on it.
///
/// A trait rather than a function so that it reads the way the toolkit's own
/// builders do, and so that a press built either way, as a mark or as a display
/// the size of a character, says it the same way.
trait Hinted<'a> {
    /// The sentence the footer prints while the pointer is over this.
    fn on_hint(self, said: &'static str) -> Element<'a, Message>;
}

impl<'a, Into_> Hinted<'a> for Into_
where
    Into_: Into<Element<'a, Message>>,
{
    fn on_hint(self, said: &'static str) -> Element<'a, Message> {
        iced::widget::mouse_area(self)
            .on_enter(Message::Ui(control_ui::Message::Hinted(Some(said))))
            .on_exit(Message::Ui(control_ui::Message::Hinted(None)))
            .into()
    }
}

/// Draws one of the marks a press wears.
///
/// Stencilled onto the panel rather than lit on glass: it is printing, not a
/// display, and the same call the number on an effect engine's case and the
/// number beside a matrix row already go through. The one press that is a
/// display is the one whose subject is the display.
///
/// In the metal a hand touches, carried part of the way back to the panel: it
/// is a legend on an instrument rather than a lamp, and the loudest thing on
/// this panel is a fader cap.
fn badge(mark: control_ui::Badge) -> Element<'static, Message> {
    Element::from(control_ui::stencil(mark.screen(), |theme: &Theme| {
        let material = control_ui::materials(theme);
        control_ui::mix(material.panel, material.metal, MARKED)
    }))
    .map(Message::Ui)
}

/// How far a press's mark is carried from the panel towards the metal.
const MARKED: f32 = 0.82;

/// A press with a mark or a display in it, at the size the thing in it is
/// drawn.
///
/// No rim and no fill until a hand comes near it (see
/// [`marked`](control_ui::marked)), and no height of its own: what is in it is
/// a drawing with a size, and a press built to the height of the words that are
/// no longer in it is a press built for nothing. The padding is what keeps two
/// of them from touching.
fn pressed<'a>(
    inside: impl Into<Element<'a, Message>>,
) -> button::Button<'a, Message, Theme, iced::Renderer> {
    button(inside)
        .padding(BESIDE_MARK)
        .style(control_ui::marked)
}

/// How much panel there is around the mark on a press.
///
/// Enough that the light under the pointer is a shape around the mark rather
/// than a shape the mark is touching the edges of, and little enough that the
/// mark is what somebody sees.
const BESIDE_MARK: f32 = 5.0;

/// What a panel admits about itself, and where it says it.
///
/// Nowhere on the panel. Two paragraphs used to stand under every rack, saying
/// what the effects page cannot know about a byte and what this window does not
/// write into the synthesizer, which is an essay printed under a panel somebody
/// was trying to read, on every section, whether or not it was the one they were
/// about.
///
/// What they were defending is still defended, by the parts of the window that
/// are already about one control at a time: the line under a slot says the two
/// ends the manual prints and says what kind of quantity the byte is where it
/// prints none, the footer describes whatever is under the pointer in the
/// library's own words, and a value nobody has read is drawn as a value nobody
/// has read. Those answer the same questions where somebody is actually asking
/// them. The rest of it, that this window writes no program into the instrument
/// and why, belongs in `README.md` and `docs/interface.md`, which is where it
/// now is alone.
const fn _admissions() {}

#[cfg(test)]
mod tests {
    use super::{App, Screen, Size, fold, paint};

    #[test]
    fn a_line_too_long_for_the_glass_is_broken_between_its_words() {
        // Thirty-two dots is four characters of the display's own face, so a
        // sentence in it is one word to a line and the fourth word is lost.
        let folded = fold("open a port and read the sound", 32);

        assert_eq!(folded, vec!["open", "a", "port"]);
        for line in &folded {
            assert!(Screen::width_of(line, Size::Small) <= 32);
        }
    }

    #[test]
    fn a_word_longer_than_the_glass_is_cut_rather_than_hyphenated() {
        assert_eq!(fold("unrecognisable", 20), vec!["un"]);
    }

    #[test]
    fn the_display_says_something_before_anything_has_been_read() {
        // A dark screen on a window that has just opened reads as a window that
        // has not: the heading, the dash where a name will be and what the
        // application is waiting for are all things to say before a port is
        // open.
        let mut screen = control_ui::screen();
        paint(&mut screen, &App::opening(&[]));

        assert!(!screen.is_blank());
    }
}
