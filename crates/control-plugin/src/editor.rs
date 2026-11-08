//! The window: the desktop's front panel, in a plugin's.

use std::sync::{Arc, Mutex};

use control_ui::{Band, Confidence, Livery, Mapper, Patch, Screen, Size};
use deepmind_midi::ids::ProtocolVersion;
use deepmind_midi::param::{DEFAULT_FIRMWARE, Group, ParamId};
use deepmind_midi::program::Program;
use nice_plug::editor::dpi::LogicalSize;
use nice_plug_iced::iced::widget::{button, column, container, row, scrollable, space, text};
use nice_plug_iced::iced::{Center, Element, Fill, Subscription, Task};
use nice_plug_iced::{
    IcedEditor, IcedEditorState, IcedNiceContext, IcedNiceSettings, PersistentState, application,
    iced::PollSubNotifier,
};

use crate::outbox::Outbox;
use crate::plugin::ControlParams;

/// How much window there is around everything in it.
const GROUND: f32 = 16.0;

/// How wide the bar is that the panel scrolls on, which is the toolkit's own.
const BAR: f32 = 10.0;

/// How much room the bar has beside the panel, so that it is cut into the
/// window rather than laid over the last plate of every row.
const BESIDE: f32 = 6.0;

/// How deep the window opens: deep enough for the front panel with little to
/// scroll, and short enough for a laptop.
const DEEP: f32 = 880.0;

/// What the window and the plugin hold in common.
#[derive(Debug)]
pub struct Shared {
    /// What the host keeps for the plugin.
    pub params: Arc<ControlParams>,
    /// Where an edit goes to become MIDI.
    pub outbox: Arc<Mutex<Outbox>>,
}

/// Builds the editor.
pub fn create(shared: Shared) -> Option<IcedEditor> {
    let width = control_ui::panel_width().max(control_ui::effects_width() + control_ui::margins())
        + GROUND * 2.0
        + BAR
        + BESIDE;
    nice_plug_iced::create_iced_editor(
        IcedEditorState::from_size(LogicalSize::new(width, DEEP), 1.0),
        shared,
        PollSubNotifier::new(),
        IcedNiceSettings::new().with_tile("DeepMind Control"),
        |shared: PersistentState<Shared>, context: IcedNiceContext| {
            Ok(
                application(shared, context, Window::open, Window::update, Window::view)
                    .default_font(control_ui::printed())
                    .theme(control_ui::deepmind())
                    .subscription(|_: &Window| Subscription::none())
                    .run(),
            )
        },
    )
}

/// What can happen in the window.
#[derive(Debug, Clone, Copy)]
enum Message {
    /// Something on the panel, which says what it means in its own words.
    Ui(control_ui::Message),
    /// Say the whole sound again.
    Resend,
}

/// The window's state.
struct Window {
    shared: PersistentState<Shared>,
    patch: Patch,
    mapper: Mapper,
    editing: Option<Group>,
    pointed: Option<ParamId>,
    hinted: Option<String>,
}

impl Window {
    /// The window as it opens: on the sound the session was saved with, or on a
    /// program of minimums when it was saved with none.
    fn open(shared: PersistentState<Shared>, _context: IcedNiceContext) -> (Self, Task<Message>) {
        let remembered = shared
            .params
            .program
            .lock()
            .ok()
            .and_then(|bytes| Program::from_bytes(ProtocolVersion::V7, &bytes).ok());
        let mut patch = Patch::new();
        patch.assume(remembered.unwrap_or_else(|| Program::new(ProtocolVersion::V7)));
        (
            Self {
                shared,
                patch,
                mapper: Mapper::new(DEFAULT_FIRMWARE),
                editing: None,
                pointed: None,
                hinted: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ui(message) => self.asked(message),
            Message::Resend => {
                if let (Some(program), Ok(mut outbox)) =
                    (self.patch.program(), self.shared.outbox.lock())
                {
                    outbox.set_all(program);
                }
                // Saying it again is also keeping it: the session should carry
                // what the instrument was just told.
                self.keep();
            }
        }
        Task::none()
    }

    fn asked(&mut self, message: control_ui::Message) {
        match message {
            control_ui::Message::Show(section) | control_ui::Message::Open(section) => {
                self.editing = Some(section);
            }
            control_ui::Message::Close
            | control_ui::Message::Front
            | control_ui::Message::Shelf => self.editing = None,
            control_ui::Message::Edit { parameter, value } => {
                self.moved(parameter, value);
                if self
                    .mapper
                    .mapped()
                    .is_some_and(|mapped| mapped.destination() == parameter)
                {
                    self.mapper.map(None);
                }
            }
            control_ui::Message::Reach {
                destination,
                at,
                depth,
                by,
            } => {
                self.moved(destination, at);
                self.moved(depth, by);
            }
            control_ui::Message::Swap { one, other } => {
                // Read both sides before either moves, or the second pair is
                // written from a parameter the first pair has already changed.
                let held: Vec<(ParamId, Option<u8>, ParamId, Option<u8>)> = one
                    .into_iter()
                    .zip(other)
                    .map(|(one, other)| {
                        (one, self.patch.value(one), other, self.patch.value(other))
                    })
                    .collect();
                for (one, was, other, is) in held {
                    if let (Some(was), Some(is)) = (was, is) {
                        self.moved(one, is);
                        self.moved(other, was);
                    }
                }
            }
            control_ui::Message::Mapper(at) => self.mapper.map(at),
            control_ui::Message::Rename(name) => {
                for (parameter, value) in self.patch.rename(name) {
                    self.send(parameter, value);
                }
                self.keep();
            }
            control_ui::Message::Pointed(parameter) => self.pointed = parameter,
            control_ui::Message::Hinted(said) => self.hinted = said.map(str::to_owned),
        }
    }

    /// Moves a parameter, and sends it where it moved.
    fn moved(&mut self, parameter: ParamId, value: u8) {
        if self.patch.edit(parameter, value) {
            self.send(parameter, value);
            self.keep();
        }
    }

    /// Hands an edit to the audio thread.
    fn send(&self, parameter: ParamId, value: u8) {
        if let Ok(mut outbox) = self.shared.outbox.lock() {
            outbox.set(parameter, value);
        }
    }

    /// Leaves the sound in the session.
    fn keep(&self) {
        let Some(program) = self.patch.program() else {
            return;
        };
        if let Ok(mut kept) = self.shared.params.program.lock() {
            kept.clear();
            kept.extend_from_slice(program.as_bytes());
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let surface: Element<'_, Message> = scrollable(
            control_ui::panel(
                &self.patch,
                DEFAULT_FIRMWARE,
                Livery::Plain,
                &self.mapper,
                |screen| paint(screen, &self.patch),
            )
            .map(Message::Ui),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().spacing(BESIDE),
        ))
        .height(Fill)
        .into();

        let surface = match self.editing {
            Some(section) => container(control_ui::modal(
                surface,
                control_ui::sheet(
                    section,
                    control_ui::group(&self.patch, section, DEFAULT_FIRMWARE, &self.mapper),
                )
                .map(Message::Ui),
                Message::Ui(control_ui::Message::Close),
            ))
            .height(Fill)
            .into(),
            None => surface,
        };

        let header = row![
            Element::from(control_ui::logo(28.0)).map(Message::Ui),
            text("DeepMind Control")
                .size(16)
                .font(control_ui::printed()),
            space::horizontal(),
            text(format!("channel {}", self.shared.params.channel.value())).size(12),
            button(text("SEND").size(12))
                .style(control_ui::chrome)
                .on_press(Message::Resend),
        ]
        .spacing(12)
        .align_y(Center);

        let footer = control_ui::footer(
            self.pointed,
            &self.patch,
            DEFAULT_FIRMWARE,
            self.mapper.mapped(),
            self.hinted.as_deref(),
        )
        .map(Message::Ui);

        container(column![header, surface, footer].spacing(12).padding(GROUND))
            .width(Fill)
            .height(Fill)
            .style(control_ui::ground)
            .into()
    }
}

/// What the panel's screen says: the sound's name, and how far to believe it.
fn paint(screen: &mut Screen, patch: &Patch) {
    let all = screen.all();
    let small = Screen::height_of(Size::Small);

    screen.write(2, 1, "PROGRAM", Size::Small);
    screen.invert(Band::new(0, 0, all.width, small + 2));

    let name = patch.name().map_or_else(
        || "\u{2014}".to_owned(),
        |name| name.as_str().trim().to_owned(),
    );
    let title = small + 8;
    screen.centre(title, &name, Size::Large);

    let backing = match patch.confidence() {
        Confidence::Unknown => "NOT READ",
        Confidence::Assumed => "ASSUMED",
        Confidence::Confirmed => "CONFIRMED",
    };
    screen.centre(title + small * 2 + 9, backing, Size::Small);
}
