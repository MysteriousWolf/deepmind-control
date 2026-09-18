//! The rubber cap a `DeepMind` puts under every finger.
//!
//! A button on this instrument is not a keycap and not a filled rectangle. It
//! is a piece of pale translucent rubber moulded into a slot in the panel, with
//! a lamp behind it, and a photograph of the front of a `DeepMind` is mostly
//! rows of them. Four things make one read as rubber rather than as a coloured
//! box, and this module draws all four:
//!
//! | | |
//! | --- | --- |
//! | the slot | the cap comes up through a hole, so there is a dark gap around it and never a bright rim |
//! | the crown | a moulded cap is domed, so the room's light lands in a band across its upper half |
//! | the lamp | the light is a point source *under the middle*, so a lit cap is brightest at its centre and falls away to its edges |
//! | the relief | it stands proud of the panel, and pressing it takes that away |
//!
//! # Drawn in quads, because that is what a renderer here has
//!
//! The constraint the [marks](crate::mark) and the [cases](crate::case) are
//! drawn under: this crate is generic over the renderer, and what every renderer
//! behind [`iced_core::Renderer`] can do is fill a rounded rectangle. There is
//! no radial gradient to light a cap from its middle with, so the lamp is a
//! short stack of rounded quads standing inside one another, each a little
//! smaller and a little brighter: at this size the steps are below what an eye
//! resolves and what is left is a glow with its hot point where the LED is.
//!
//! That is also why this is a widget rather than a `button` with a style. A
//! `button::Style` carries one background, one border and one shadow, which is
//! exactly enough for a rounded rectangle with a gradient in it, and the whole
//! difference between that and a rubber cap is the four things above.
//!
//! # What it does not do
//!
//! It takes a child and never gives it an event. What this window prints on a
//! cap is a [mark](crate::lcd::stencil) and a name, and neither is something
//! anybody clicks: the cap is the control, and a press anywhere on it is the
//! press.

use iced_core::layout::{self, Layout};
use iced_core::widget::{Tree, tree};
use iced_core::{
    Background, Border, Clipboard, Color, Element, Event, Length, Rectangle, Shell, Size, Theme,
    Widget, mouse, renderer, touch,
};

use crate::style::{self, materials, mix};

/// How round the corner of a cap is, as a share of how tall it stands.
///
/// A *share* and not a measurement, because the caps in this window are not all
/// one size: the ones on the panel are the instrument's own, and the ones in
/// the band of ways in are four times as wide and half again as tall. A fixed
/// radius on both is a panel button that reads as moulded rubber and a tab that
/// reads as a slab with the corners filed, which is the same drawing stretched
/// rather than the same button made bigger.
///
/// A quarter, which is what a `DeepMind`'s cap measures: turned far enough that
/// it reads as something soft pushed through the panel, and not so far that it
/// becomes a lozenge.
const MOULD: f32 = 0.25;

/// How round a cap `tall` points high is cut.
fn mould(tall: f32) -> f32 {
    tall * MOULD
}

/// How thick the bezel round a cap is.
///
/// The rubber is moulded into a rim, so there is a dark line all the way round
/// every button on the instrument. It is most of what stops a row of lit caps
/// reading as a row of stickers, and it is why nothing here draws a *bright*
/// border: a cap has no metal edge, it has a shadow.
const RIM: f32 = 2.0;

/// How far the bezel is carried from the panel towards black.
const BEZEL: f32 = 0.55;

/// What the bezel and the hair of panel inside it cost a cap across its width.
///
/// Published, because what is printed on a cap has to be laid out in the room
/// the rubber has rather than the room the slot has: a caller measuring a name
/// against the slot gets a name with its last letter under the bezel.
pub(crate) const BEZELS: f32 = (RIM + 1.0) * 2.0;

/// How far a cap stands proud of the panel.
///
/// Small, because the whole cap is only a few millimetres tall, and enough that
/// the shadow under its lower edge is visible. Pressing spends it.
const RELIEF: f32 = 1.0;

/// How many quads the diffuser under a cap is built from.
///
/// Each stands inside the one before it and carries the same weight, so the
/// light gathers towards the middle in equal steps. Six is past the point where
/// another one shows at the size a cap is drawn.
const GLOW: usize = 6;

/// How far apart the steps of the diffuser stand, as a share of the cap's own
/// height.
///
/// The `step`th quad is inset by `GATHER * step` of the height on every side,
/// so the last of the six sits a third of the height in from the rim. A
/// distance and not a proportion, because that is what a falloff is: see the
/// loop that draws it.
const GATHER: f32 = 0.055;

/// How hard one step of the diffuser is laid down.
///
/// Low, because [`GLOW`] of them accumulate: what the middle of a cap ends up
/// at is this compounded six times, and what the rim keeps is the face it
/// started on.
const SPREAD: f32 = 0.17;

/// What is in the bezel.
///
/// Three states and not two, because a window that has not been told the value
/// of a switch cannot draw one. That is the same refusal a fader makes by
/// drawing its track and no cap: the shape of the control does not depend on
/// whether a sound has arrived, but what a hand takes hold of does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Face {
    /// No cap in the bezel: the hole, flat and unlit.
    ///
    /// Not an off button. A cap that is off is still a piece of rubber standing
    /// out of the panel, and the difference between the two is relief rather
    /// than colour, which is what makes it survive a greyscale print.
    Empty,
    /// A cap, with what is behind it and what it is ringed in.
    ///
    /// **Unlit is not dark.** The buttons on a `DeepMind` are a translucent
    /// off-white, and one with no lamp behind it still catches the room and
    /// reads as pale against a panel this dark. A cap drawn at the panel's own
    /// colour is a hole, and the instrument has none.
    Cap {
        /// The colour of the lamp behind it, where there is one.
        ///
        /// The instrument's own: amber on a press that changes what the display
        /// is showing, cyan on one that changes what the other controls *mean*,
        /// and white on a plain switch.
        lamp: Option<Color>,
        /// The ring just inside the bezel, where the cap is carrying a claim.
        ///
        /// A backlit cap on the instrument already has a ring: a saturated
        /// band of the lamp's own colour between the bezel and the hot middle,
        /// which is what a diffuser does at the edge of its own aperture. This
        /// window spends it. The lamp says what the press *is*, the way the
        /// hardware does, and the ring says whether the value under it came
        /// from the synthesizer or from this window — which is the one thing a
        /// `DeepMind`'s own panel has no way to show, since it lights a button
        /// the same whether it was told or has assumed.
        ///
        /// `None` on every press that is not a parameter: a way in has nothing
        /// to claim.
        ring: Option<Color>,
    },
}

impl Face {
    /// A cap with nothing behind it and nothing to claim.
    pub(crate) const RUBBER: Self = Self::Cap {
        lamp: None,
        ring: None,
    };

    /// A cap lit in `lamp`, with nothing to claim.
    pub(crate) const fn lit(lamp: Color) -> Self {
        Self::Cap {
            lamp: Some(lamp),
            ring: None,
        }
    }
}

impl Face {
    /// The cap at its rim, which is what the diffuser is laid over.
    ///
    /// The *rim* and not the average. A backlit cap is two colours, saturated
    /// at the edge and nearly white in the middle, and drawing it as the one in
    /// between is what makes a lit button read as a swatch. So this is the
    /// darker, more saturated of the two and the middle is the diffuser's
    /// business.
    fn of(self, theme: &Theme, hovered: bool) -> Color {
        let material = materials(theme);
        match self {
            // Never drawn: an empty bezel has no cap to colour.
            Self::Empty => material.recess,
            // The rubber itself, a long way up from the panel, because the
            // material is nearly white and the panel is nearly black. Coming
            // near one lifts it further, which is the whole of what hovering
            // means on a panel that has no cursor.
            Self::Cap { lamp: None, .. } => mix(
                material.panel,
                material.metal,
                if hovered { RAISED } else { MOULDING },
            ),
            // The lamp's own colour, deepened, so the hot middle has somewhere
            // to go. A face drawn at the lamp itself has no falloff, and a cap
            // with no falloff is a coloured rectangle.
            Self::Cap {
                lamp: Some(colour), ..
            } => mix(colour, Color::BLACK, if hovered { KINDLED } else { BANKED }),
        }
    }

    /// The lamp behind it, where there is one.
    const fn lamp(self) -> Option<Color> {
        match self {
            Self::Empty => None,
            Self::Cap { lamp, .. } => lamp,
        }
    }

    /// The ring just inside the bezel, where it is carrying one.
    const fn ring(self) -> Option<Color> {
        match self {
            Self::Empty => None,
            Self::Cap { ring, .. } => ring,
        }
    }

    /// Whether there is anything standing in the bezel at all.
    const fn moulded(self) -> bool {
        !matches!(self, Self::Empty)
    }
}

/// How far an unlit cap is carried from the panel towards the metal.
///
/// Most of the way: the rubber a `DeepMind`'s buttons are moulded from is
/// nearly white, and the panel is nearly black. This is the one number that
/// decides whether a row of unlit buttons reads as a row of buttons or as a row
/// of holes.
const MOULDING: f32 = 0.6;

/// The same, for a cap a pointer is over.
const RAISED: f32 = 0.74;

/// How far a lit cap's rim is taken down from the lamp's own colour.
///
/// Enough to leave the middle somewhere to go. A face drawn at the lamp itself
/// has no rim, and a cap with no rim is a coloured rectangle.
const BANKED: f32 = 0.2;

/// The same, for a lit cap a pointer is over: the lamp coming up.
const KINDLED: f32 = 0.04;

/// A rubber cap, with `content` printed on it.
///
/// Built by [`cap`]; [`Cap::on_press`] is what makes it a control rather than a
/// lamp, since a `DeepMind` has both and the `EDIT` legend beside one of them
/// is the same silkscreen either way.
pub(crate) struct Cap<'a, Message, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    face: Box<dyn Fn(&Theme) -> Face + 'a>,
    width: Length,
    height: Length,
    on_press: Option<Message>,
}

/// What a cap remembers between events.
#[derive(Debug, Clone, Copy, Default)]
struct State {
    /// Whether the pointer went down on this cap and has not come up.
    held: bool,
}

/// Returns a cap with `content` printed on it, of whatever `face` says.
///
/// `face` is asked at every frame rather than given once, because what is in
/// the slot is a question about the theme: the claim a switch is carrying is a
/// colour the theme chooses, so a window somebody has themed differently lights
/// its caps in its own colours with nothing here to change.
pub(crate) fn cap<'a, Message, Renderer>(
    face: impl Fn(&Theme) -> Face + 'a,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Cap<'a, Message, Renderer>
where
    Renderer: iced_core::Renderer + 'a,
{
    Cap {
        content: content.into(),
        face: Box::new(face),
        width: Length::Fill,
        height: Length::Fill,
        on_press: None,
    }
}

impl<Message, Renderer> Cap<'_, Message, Renderer> {
    /// What pressing it publishes. A cap with nothing to say is a lamp.
    pub(crate) fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// How wide the slot is cut.
    pub(crate) fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// How tall it is cut.
    pub(crate) fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

/// Where the rubber is, inside the slot it came up through.
///
/// Inset by the gap the hole is cut with, and lifted by the relief it stands
/// at, so a cap that is not held down leaves its shadow along the bottom of its
/// own slot.
fn rubber(bounds: Rectangle, held: bool) -> Rectangle {
    Rectangle {
        x: bounds.x + RIM,
        y: bounds.y + RIM - if held { 0.0 } else { RELIEF },
        width: (bounds.width - RIM * 2.0).max(0.0),
        height: (bounds.height - RIM * 2.0).max(0.0),
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Cap<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_core::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(core::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Some(child) = tree.children.first_mut() else {
            return layout::atomic(limits, self.width, self.height);
        };
        // The child is laid out in the rubber and not in the slot, so nothing
        // printed on a cap runs into the hole it came up through.
        layout::padded(
            &limits.width(self.width).height(self.height),
            self.width,
            self.height,
            RIM + 1.0,
            |limits| self.content.as_widget_mut().layout(child, renderer, limits),
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(message) = self.on_press.clone() else {
            return;
        };
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(bounds) {
                    state.held = true;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. }) => {
                // A press that wandered off the cap before it was let go is a
                // press somebody changed their mind about, which is what every
                // button anywhere does.
                if core::mem::take(&mut state.held) && cursor.is_over(bounds) {
                    shell.publish(message);
                    shell.capture_event();
                }
            }
            Event::Touch(touch::Event::FingerLost { .. }) => state.held = false,
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.on_press.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let material = materials(theme);
        let over = cursor.is_over(bounds);
        let held = tree.state.downcast_ref::<State>().held && over;
        let hovered = over && self.on_press.is_some();
        let face = (self.face)(theme);

        // The bezel: the moulded rim the cap is held in, which in a photograph
        // of the instrument is a dark line all the way round every button and
        // is the first thing that says *rubber* rather than *rectangle*. It is
        // darker than the panel, because it is the one part of the front that
        // no light reaches.
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::default().rounded(mould(bounds.height) + RIM),
                ..renderer::Quad::default()
            },
            Background::Color(style::mix(material.panel, Color::BLACK, BEZEL)),
        );
        if !face.moulded() {
            // Nothing read is the bezel with nothing in it. No cap, no lamp and
            // nothing printed: there is no cap to print on.
            return;
        }

        let rubber = rubber(bounds, held);
        let colour = face.of(theme, hovered);

        // The cap itself, domed down its own height: the light lands on the
        // crown and the foot sits in its own shadow. Held, the same face is
        // seen from the other side of the press, so the two ends swap and
        // nothing needs a second colour to say it is down.
        renderer.fill_quad(
            renderer::Quad {
                bounds: rubber,
                border: Border::default().rounded(mould(rubber.height)),
                ..renderer::Quad::default()
            },
            style::moulded(colour, held),
        );

        // The diffuser. What a `DeepMind`'s cap is, optically, is a piece of
        // translucent rubber with a single point of light under the middle of
        // it, so the face is brightest at its centre and keeps its own colour
        // at the rim. That is a radial falloff, and there is no radial gradient
        // in a renderer that can only fill rounded rectangles: it is a stack of
        // quads standing inside one another, each carrying the same weight, so
        // the middle collects all of them and the edge collects none.
        //
        // Every cap gets one, lit or not. An unlit button on the instrument is
        // a white diffuser with the room's light in it rather than an LED's,
        // which is why it reads as the same object switched off instead of as a
        // different object.
        let core = match face.lamp() {
            Some(lamp) => mix(lamp, Color::WHITE, CORE),
            None => mix(material.metal, Color::WHITE, GLARE),
        };
        for step in 1..=GLOW {
            #[expect(
                clippy::cast_precision_loss,
                reason = "GLOW is six, and six converts exactly"
            )]
            // Inset by a distance rather than scaled by a fraction, and the
            // distance is read off the cap's *height*. Which is the whole
            // difference between a bigger button and the same button stretched:
            // light falls away from the rim of a diffuser over the width of the
            // rubber above it, which is a thickness and not a proportion, so a
            // cap four times as wide has the same falloff at its edges and a
            // longer even middle — exactly what a wide lens over a lamp looks
            // like. Scaling it instead gives a cap whose glow is a magnified
            // photograph of a small one.
            let inset = rubber.height * GATHER * step as f32;
            let width = (rubber.width - inset * 2.0).max(1.0);
            let height = (rubber.height - inset * 2.0).max(1.0);
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: rubber.x + (rubber.width - width) / 2.0,
                        // The hot point sits a little above the middle, because
                        // the light is behind the crown and the crown is the
                        // part of a domed cap nearest the eye.
                        y: rubber.y + (rubber.height - height) / 2.0 - inset * OFFSET,
                        width,
                        height,
                    },
                    border: Border::default().rounded(mould(height)),
                    ..renderer::Quad::default()
                },
                Background::Color(Color {
                    a: if held { SPREAD / 2.0 } else { SPREAD },
                    ..core
                }),
            );
        }

        // The ring, just inside the bezel: what the claim under this press is,
        // in the band a backlit cap already has between its bezel and its hot
        // middle. Drawn last of the cap's own layers so the diffuser does not
        // wash it out at the corners, where the rounding brings the two within
        // a point of each other.
        if let Some(ring) = face.ring() {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: rubber,
                    border: Border {
                        color: Color {
                            a: if held { RINGING / 2.0 } else { RINGING },
                            ..ring
                        },
                        width: BAND,
                        radius: mould(rubber.height).into(),
                    },
                    ..renderer::Quad::default()
                },
                Background::Color(Color::TRANSPARENT),
            );
        }

        if let Some(child) = tree.children.first() {
            self.content.as_widget().draw(
                child,
                renderer,
                theme,
                style,
                layout.children().next().unwrap_or(layout),
                cursor,
                viewport,
            );
        }
    }
}

/// How far the lamp's own colour is carried towards white at the hot point.
///
/// Most of the way. An LED under a translucent cap is nearly white where it is
/// brightest and its own colour everywhere else, which is what a photograph of
/// a lit `DeepMind` button shows: a yellow-white middle inside an orange rim.
const CORE: f32 = 0.62;

/// The same, for a cap with no lamp behind it.
///
/// Further still, because the diffuser is white and what is in it is the room.
const GLARE: f32 = 0.72;

/// How far above the middle the hot point of the diffuser sits, as a share of
/// how far in each step of it stands.
const OFFSET: f32 = 0.5;

/// How thick the ring inside the bezel is.
///
/// Two points, which is the bezel's own thickness: the band between a backlit
/// cap's rim and its hot middle is about that wide on the instrument, so a ring
/// drawn there is the cap's own anatomy carrying something rather than a
/// decoration laid over it.
const BAND: f32 = 2.0;

/// How hard the ring is laid down.
///
/// Not quite solid. It is a colour *in* the rubber and not printed on top of
/// it, so the cap's own dome still shows through the ring the way it shows
/// through everything else on the face.
const RINGING: f32 = 0.85;

impl<'a, Message, Renderer> From<Cap<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced_core::Renderer + 'a,
{
    fn from(cap: Cap<'a, Message, Renderer>) -> Self {
        Self::new(cap)
    }
}

#[cfg(test)]
mod tests {
    use iced_core::Rectangle;

    use super::{Face, MOULDING, RELIEF, RIM, rubber};
    use crate::style::{contrast, materials};

    /// A cap the size the front panel cuts its slots at.
    const SLOT_SIZE: Rectangle = Rectangle {
        x: 0.0,
        y: 0.0,
        width: 44.0,
        height: 28.0,
    };

    #[test]
    fn an_unlit_cap_is_pale_against_the_panel() {
        // The point of the whole module. A `DeepMind`'s buttons are moulded
        // from a translucent off-white, and one with no lamp behind it is still
        // plainly a button: if this ever falls back towards the panel, the
        // panel has gone back to being a row of holes.
        let theme = crate::style::deepmind();
        let panel = materials(&theme).panel;
        let rubber = Face::RUBBER.of(&theme, false);

        assert!(
            contrast(rubber, panel) > 2.0,
            "an unlit cap at {MOULDING} of the way to the metal does not show on the panel"
        );
    }

    #[test]
    fn a_lit_cap_keeps_some_of_the_rubber_it_shines_through() {
        // A lamp behind a translucent cap is not the lamp. If a lit face ever
        // becomes the colour itself, the cap has stopped being a material.
        let theme = crate::style::deepmind();
        let lamp = crate::style::WAY_IN;
        let lit = Face::lit(lamp).of(&theme, false);

        assert!(
            (lit.r - lamp.r).abs() + (lit.g - lamp.g).abs() + (lit.b - lamp.b).abs() > 0.02,
            "a lit cap is the bare lamp colour"
        );
    }

    #[test]
    fn a_cap_stands_inside_its_own_slot() {
        // The dark line all the way round a button on the instrument. A cap
        // drawn to the edge of its slot is a cap with no hole to come up
        // through, which is the second half of why a filled rectangle does not
        // read as rubber.
        let standing = rubber(SLOT_SIZE, false);
        let pressed = rubber(SLOT_SIZE, true);

        assert!(standing.width < SLOT_SIZE.width, "no slot around the cap");
        assert!(standing.height < SLOT_SIZE.height, "no slot around the cap");
        assert!(
            standing.y < pressed.y,
            "a cap that is not held down stands {RELIEF} proud of one that is"
        );
        assert!(
            standing.x - SLOT_SIZE.x >= RIM,
            "the bezel is thinner than the rubber it holds"
        );
    }
}
