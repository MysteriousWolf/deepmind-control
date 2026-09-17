//! The window: what it draws, and what wakes it up.

use std::thread;
use std::time::Duration;

use control_ui::{Band, Confidence, Ink, Screen, Size};
use deepmind_midi::param::ParamId;
use iced::futures::Stream;
use iced::futures::channel::mpsc;
use iced::widget::{button, column, container, pick_list, row, scrollable, space, text};
use iced::{Background, Center, Element, Fill, Length, Subscription, Theme, border};

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
        // As wide as the panel measures, plus the ground it stands on and the
        // bar it scrolls on, so a window opens on the instrument's own
        // arrangement rather than on a wrapped one. Narrower than this and the
        // rows wrap, which is readable and is no longer two rows and a screen.
        .window_size((
            control_ui::panel_width() + GROUND * 2.0 + BAR + BESIDE,
            940.0,
        ))
        .run()
}

/// What the window is drawn in.
///
/// The instrument is a dark panel, so the window is one. Every colour in it
/// comes from this theme, which lives in `control-ui` because the plugin is
/// drawn in the same one.
fn theme(app: &App) -> Theme {
    if app.is_negative() {
        control_ui::negative()
    } else {
        control_ui::deepmind()
    }
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
///
/// Everything stands on the panel: the gradient the mark fills its case with,
/// lit at the top where the light is, rather than the flat dark the theme's own
/// background would give. It is the one surface in the window nothing is cut
/// into, so it is drawn once, here, around all of it.
fn view(app: &App) -> Element<'_, Message> {
    container(
        column![header(app), identity(app), controls(app), surfaces(app)]
            .push(match app.view() {
                // The panel scrolls for the same reason the rack does, and it
                // did not have to before its plates carried displays: two rows
                // of plates are taller than a window somebody has made short,
                // and a front panel with its lower row below the fold is a
                // front panel with the whole voice missing.
                View::Panel => scrollable(
                    control_ui::panel(app.patch(), app.firmware(), app.aim(), |screen| {
                        paint(screen, app);
                    })
                    .map(Message::Ui),
                )
                // The bar is cut into the window beside the panel rather than
                // laid over it. The panel is drawn out to the room it is given
                // now, so a bar that floated over the end of it would be a bar
                // over the last plate of every row — and a panel that shrank
                // away from one the moment there was enough of it to scroll.
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new().spacing(BESIDE),
                ))
                .height(Fill)
                .into(),
                View::Editor => editor(app),
                View::Library => librarian::view(app),
            })
            // Along the foot, under whichever surface is showing, because a
            // control is pointed at on all three of them and a footer that
            // moved with the surface would be a different footer each time.
            .push(
                control_ui::footer(
                    app.pointed(),
                    app.patch(),
                    app.firmware(),
                    app.aim().aimed(),
                )
                .map(Message::Ui),
            )
            .spacing(12)
            .padding(GROUND),
    )
    .width(Fill)
    .height(Fill)
    .style(control_ui::ground)
    .into()
}

/// The three things this application is, and which one is in front of somebody.
///
/// The instrument's own front, one section of it, and the sounds you keep. Not
/// three windows and not three modes of one: moving between them is one press,
/// and the sound survives the move, because putting a pack down to look at a
/// filter and finding the filter gone would be the wrong lesson to teach
/// anybody about this application.
///
/// The middle one is named after the section it holds rather than "Editor",
/// because the panel's `EDIT` opened that section and the way back to it should
/// say which one it is.
fn surfaces(app: &App) -> Element<'_, Message> {
    let showing = app.view();
    let tab = |view: View, label: String| {
        let pressed = view == showing;
        button(text(label).size(13))
            .padding([5, 12])
            .style(move |theme: &Theme, _status| surface(theme, pressed))
            .on_press(Message::Show(view))
    };
    row![
        tab(View::Panel, "Panel".to_owned()),
        tab(View::Editor, app.section().name().to_owned()),
        tab(View::Library, "Library".to_owned()),
    ]
    .push(space::horizontal())
    .push(livery(app))
    .spacing(6)
    .align_y(Center)
    .into()
}

/// The press that turns the displays over.
///
/// At the far end of the row that chooses a surface, because it belongs to the
/// window rather than to any one of the three: every display in all of them
/// turns over together, the way a screen has one backlight.
///
/// It says which way up they will be rather than which way up they are, because
/// that is what a press does — and it is the one control in this window that
/// changes nothing about the sound, which is why it is a plain press and not
/// something that lights.
fn livery(app: &App) -> Element<'_, Message> {
    let label = if app.is_negative() {
        "Positive display"
    } else {
        "Negative display"
    };
    button(text(label).size(12))
        .padding([5, 10])
        .style(control_ui::chrome)
        .on_press(Message::Invert)
        .into()
}

/// What the instrument's own display would be showing.
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

/// The style the surface switch is drawn in, which is the section bar's.
fn surface(theme: &Theme, pressed: bool) -> button::Style {
    let material = control_ui::materials(theme);
    let palette = theme.extended_palette();
    button::Style {
        background: Some(Background::Color(if pressed {
            material.plate
        } else {
            material.panel
        })),
        text_color: palette.background.base.text,
        border: border::rounded(3).width(1.0).color(if pressed {
            material.lit
        } else {
            material.recess_edge
        }),
        ..button::Style::default()
    }
}

/// The editing surface: one section of the instrument, and the bar that chooses
/// which.
fn editor(app: &App) -> Element<'_, Message> {
    let section = app.section();
    column![
        // The bar stays put while the rack under it scrolls: it is how a panel
        // is left, and a control that scrolls away is a control that is looked
        // for.
        control_ui::section_bar(app.patch(), section).map(Message::Ui),
        scrollable(
            column![
                text(section.name()).size(18),
                control_ui::group(app.patch(), section, app.firmware(), app.aim()).map(Message::Ui),
            ]
            .spacing(12)
            .padding([0, 12]),
        )
        .height(Fill),
    ]
    .spacing(12)
    .into()
}

/// The name of the thing, the port picker, and what to do with a port.
fn header(app: &App) -> Element<'_, Message> {
    let ports = app.ports().to_vec();
    let connection = if app.is_connected() {
        chrome("Close").on_press(Message::Disconnect)
    } else {
        chrome("Open").on_press_maybe(app.chosen().map(|_| Message::Connect))
    };
    row![
        wordmark(),
        space().width(Fill),
        pick_list(ports, app.chosen().cloned(), Message::Choose)
            .placeholder("MIDI port")
            .font(control_ui::printed())
            .text_size(13)
            .padding([5, 10])
            .style(control_ui::selector)
            .menu_style(control_ui::shortlist)
            .width(Length::Fixed(260.0)),
        chrome("Rescan").on_press(Message::Rescan),
        connection,
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// The project's own name, set the way the mark sets it.
///
/// `docs/banner.svg` puts it across the panel in the metal of a fader cap, in
/// Liberation Sans Bold, with the wordmark's lines through it. A window has the
/// first two of those and not the third: a line 1.5 points thick across a
/// 22-point word is a smudge rather than a slice, and the mark is not improved
/// by being approximated. So it is the name, in the face and the metal, over
/// the panel it is printed on.
fn wordmark() -> Element<'static, Message> {
    column![
        text("deepmind control")
            .font(control_ui::wordmark())
            .size(22)
            .style(|theme: &Theme| text::Style {
                color: Some(control_ui::materials(theme).metal),
            }),
        text("editor and librarian")
            .size(11)
            .style(|theme: &Theme| text::Style {
                color: Some(control_ui::materials(theme).metal_low),
            }),
    ]
    .spacing(1)
    .into()
}

/// A button that is not a parameter, in the instrument's own materials.
fn chrome(label: &str) -> button::Button<'_, Message, Theme, iced::Renderer> {
    button(text(label).size(13))
        .padding([5, 12])
        .style(control_ui::chrome)
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
    .style(control_ui::bay)
    .into()
}

/// What can be asked of the synthesizer, and what backs what is on the screen.
fn controls(app: &App) -> Element<'_, Message> {
    let open = app.is_connected();
    let sound = match app.patch().confidence() {
        Confidence::Unknown => "Nothing has been read.".to_owned(),
        // Edited here, or loaded off the shelf: both are this window putting a
        // value somewhere, and neither is the synthesizer agreeing that it went
        // there. Reading the edit buffer back is what settles it.
        Confidence::Assumed => "This window's claim, not the synthesizer's.".to_owned(),
        Confidence::Confirmed => match app.patch().name() {
            Some(name) => format!(
                "{} \u{2014} as the synthesizer described it.",
                name.as_str().trim()
            ),
            None => "As the synthesizer described it.".to_owned(),
        },
    };
    row![
        chrome("Read the edit buffer").on_press_maybe(open.then_some(Message::Read)),
        chrome("Ask who is there").on_press_maybe(open.then_some(Message::Identify)),
        // The sound takes what is left rather than pushing what is beside it:
        // a program name is sixteen characters and the legend is the one thing
        // on this row that has to stay readable whatever the sound is called.
        text(sound).size(13).width(Fill),
    ]
    .spacing(10)
    .align_y(Center)
    .into()
}

/// What a panel admits about itself, and where it says it.
///
/// Nowhere on the panel. Two paragraphs used to stand under every rack — what
/// the effects page cannot know about a byte, and what this window does not
/// write into the synthesizer — and they were an essay printed under a panel
/// somebody was trying to read, on every section, whether or not it was the one
/// they were about.
///
/// What they were defending is still defended, by the parts of the window that
/// are already about one control at a time: the line under a slot says the two
/// ends the manual prints and says what kind of quantity the byte is where it
/// prints none, the footer describes whatever is under the pointer in the
/// library's own words, and a value nobody has read is drawn as a value nobody
/// has read. Those answer the same questions where somebody is actually asking
/// them. The rest of it — that this window writes no program into the
/// instrument, and why — belongs in `README.md` and `docs/interface.md`, which
/// is where it now is alone.
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
