//! The interface's copy of the sound.

use deepmind_midi::param::{PARAMETER_COUNT, ParamId};
use deepmind_midi::program::{Program, ProgramName};

use crate::Confidence;

/// What the interface believes the synthesizer is making, and why it believes
/// it.
///
/// The device thread holds the truth; this is the copy built out of the events
/// it publishes, and there is no lock between them. What it adds to a
/// [`Program`] is a claim per parameter, because the library's own claim is made
/// about the whole sound at once: one dragged knob leaves 241 parameters exactly
/// as confirmed as they were, and an editor that greys all of them out because
/// one moved has thrown the information away.
///
/// Nothing here sends anything. A [`Patch`] is changed by what happened, and
/// what happens next is the application's business.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The values, once anything has told us what they are.
    program: Option<Program>,
    /// What backs the sound as a whole, which is what the library tracks.
    whole: Confidence,
    /// What backs each value, by parameter offset.
    claims: [Confidence; PARAMETER_COUNT],
}

impl Default for Patch {
    fn default() -> Self {
        Self::new()
    }
}

impl Patch {
    /// A sound nobody has read: no values, and no claims about any.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            program: None,
            whole: Confidence::Unknown,
            claims: [Confidence::Unknown; PARAMETER_COUNT],
        }
    }

    /// Returns the program, once one is known.
    #[must_use]
    pub const fn program(&self) -> Option<&Program> {
        self.program.as_ref()
    }

    /// Returns whether anything is known about the sound at all.
    ///
    /// Until this is true there is nothing to edit: an edit is a difference, and
    /// there is nothing to take a difference against.
    #[must_use]
    pub const fn is_known(&self) -> bool {
        self.program.is_some()
    }

    /// Returns what backs the sound as a whole.
    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.whole
    }

    /// Returns what backs one parameter's value.
    #[must_use]
    pub fn claim(&self, parameter: ParamId) -> Confidence {
        self.claims
            .get(usize::from(parameter.offset()))
            .copied()
            .unwrap_or_default()
    }

    /// Returns one parameter's value, once one is known.
    #[must_use]
    pub fn value(&self, parameter: ParamId) -> Option<u8> {
        self.program.as_ref().map(|program| program.get(parameter))
    }

    /// Returns the program's name, once one is known.
    #[must_use]
    pub fn name(&self) -> Option<ProgramName> {
        self.program.as_ref().map(Program::name)
    }

    /// Takes a dump the synthesizer sent.
    ///
    /// The one thing that confirms anything: every value now came from the
    /// instrument, so every claim is rewritten rather than merged. This is what
    /// "put it there, then tell me where it is" ends with.
    pub fn confirm(&mut self, program: Program) {
        self.program = Some(program);
        self.whole = Confidence::Confirmed;
        self.claims = [Confidence::Confirmed; PARAMETER_COUNT];
    }

    /// Takes a program from somewhere that is not the synthesizer.
    ///
    /// A file, or a slot the librarian is auditioning. The values are what the
    /// interface intends the sound to be and the instrument has not said a word
    /// about any of them, so nothing here is confirmed until a dump comes back.
    pub fn assume(&mut self, program: Program) {
        self.program = Some(program);
        self.whole = Confidence::Assumed;
        self.claims = [Confidence::Assumed; PARAMETER_COUNT];
    }

    /// Moves a parameter because somebody in this window moved it.
    ///
    /// Returns whether it changed anything, which is what tells the application
    /// there is something worth sending. A parameter of a sound nobody has read
    /// cannot move, because there is no sound to move it in.
    pub fn edit(&mut self, parameter: ParamId, value: u8) -> bool {
        let Some(program) = self.program.as_mut() else {
            return false;
        };
        if program.get(parameter) == value {
            return false;
        }
        if program.set(parameter, value).is_err() {
            return false;
        }
        self.claim_mut(parameter, Confidence::Assumed);
        self.whole = Confidence::Assumed;
        true
    }

    /// Takes a value the synthesizer reported.
    ///
    /// The instrument is the other editor: a hand on the front panel gets here,
    /// and last writer wins. `confirmed` is what the host crate said about the
    /// sound it tracks with the report applied — an NRPN carries the whole value
    /// and a control change carries seven bits of it — so a coarse report is
    /// drawn as the assumption it is rather than quietly rounded into a fact.
    ///
    /// A report about a sound nobody has read changes nothing: one value is not
    /// a program, and there is nowhere to put it.
    pub fn report(&mut self, parameter: ParamId, value: u8, confirmed: bool) {
        let Some(program) = self.program.as_mut() else {
            return;
        };
        if program.set(parameter, value).is_err() {
            return;
        }
        let claim = if confirmed {
            Confidence::Confirmed
        } else {
            Confidence::Assumed
        };
        self.claim_mut(parameter, claim);
        if !confirmed {
            self.whole = Confidence::Assumed;
        }
    }

    /// Forgets everything, because the port is not there any more.
    ///
    /// The next port may be another instrument, or the same one after somebody
    /// spent the afternoon at its panel. Keeping the last sound on the screen
    /// and calling it confirmed would be the one claim this crate exists to
    /// avoid making.
    pub fn forget(&mut self) {
        *self = Self::new();
    }

    /// Writes down what backs one parameter.
    fn claim_mut(&mut self, parameter: ParamId, claim: Confidence) {
        if let Some(slot) = self.claims.get_mut(usize::from(parameter.offset())) {
            *slot = claim;
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::ids::ProtocolVersion;

    use super::{Confidence, ParamId, Patch, Program};

    fn read() -> Patch {
        let mut patch = Patch::new();
        patch.confirm(Program::new(ProtocolVersion::V7));
        patch
    }

    #[test]
    fn nothing_is_known_before_a_dump() {
        let patch = Patch::new();

        assert!(!patch.is_known());
        assert_eq!(patch.confidence(), Confidence::Unknown);
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Unknown);
        assert_eq!(patch.value(ParamId::VcfFrequency), None);
    }

    #[test]
    fn a_parameter_of_an_unread_sound_cannot_move() {
        let mut patch = Patch::new();

        assert!(!patch.edit(ParamId::VcfFrequency, 200));
        patch.report(ParamId::VcfFrequency, 200, true);

        assert_eq!(patch.value(ParamId::VcfFrequency), None);
    }

    #[test]
    fn an_edit_assumes_one_parameter_and_leaves_the_rest_confirmed() {
        let mut patch = read();

        assert!(patch.edit(ParamId::VcfFrequency, 200));

        assert_eq!(patch.value(ParamId::VcfFrequency), Some(200));
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Assumed);
        assert_eq!(patch.claim(ParamId::VcfResonance), Confidence::Confirmed);
        // The sound as a whole is no longer the sound the synthesizer described.
        assert_eq!(patch.confidence(), Confidence::Assumed);
    }

    #[test]
    fn setting_a_parameter_where_it_already_is_sends_nothing() {
        let mut patch = read();
        patch.edit(ParamId::VcfFrequency, 200);

        assert!(!patch.edit(ParamId::VcfFrequency, 200));
    }

    #[test]
    fn a_dump_confirms_everything_again() {
        let mut patch = read();
        patch.edit(ParamId::VcfFrequency, 200);

        let mut dumped = Program::new(ProtocolVersion::V7);
        dumped.set(ParamId::VcfFrequency, 200).expect("in range");
        patch.confirm(dumped);

        assert_eq!(patch.confidence(), Confidence::Confirmed);
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Confirmed);
    }

    #[test]
    fn an_exact_report_confirms_the_parameter_it_names() {
        let mut patch = read();

        patch.report(ParamId::VcfFrequency, 200, true);

        assert_eq!(patch.value(ParamId::VcfFrequency), Some(200));
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Confirmed);
        assert_eq!(patch.confidence(), Confidence::Confirmed);
    }

    #[test]
    fn a_coarse_report_is_an_assumption() {
        let mut patch = read();

        // A control change carries seven bits of the value, so what arrived is
        // near where the knob is rather than where it is.
        patch.report(ParamId::VcfFrequency, 200, false);

        assert_eq!(patch.value(ParamId::VcfFrequency), Some(200));
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Assumed);
        assert_eq!(patch.confidence(), Confidence::Assumed);
    }

    #[test]
    fn a_program_from_a_file_is_assumed_throughout() {
        let mut patch = read();

        patch.assume(Program::new(ProtocolVersion::V7));

        assert_eq!(patch.confidence(), Confidence::Assumed);
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Assumed);
    }

    #[test]
    fn a_closed_port_leaves_nothing_behind() {
        let mut patch = read();

        patch.forget();

        assert!(!patch.is_known());
        assert_eq!(patch.confidence(), Confidence::Unknown);
    }
}
