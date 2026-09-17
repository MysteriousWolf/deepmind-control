//! The interface's copy of the sound.

use deepmind_midi::param::{Group, PARAMETER_COUNT, ParamId};
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

    /// Returns what backs a whole section's worth of values.
    ///
    /// The weakest claim any parameter in it makes, because a section is only
    /// as confirmed as its least confirmed parameter: one fader moved in a
    /// panel nobody is looking at is exactly the thing a tab has to be able to
    /// say. A sound nobody has read is unknown throughout.
    #[must_use]
    pub fn claim_of(&self, group: Group) -> Confidence {
        self.claim_across(group.parameters())
    }

    /// Returns what backs a handful of values taken together.
    ///
    /// The same rule a section follows, for anything that is drawn as one thing
    /// and stored as several: the weakest claim any of them makes. The name is
    /// what needs it, because it is seventeen parameters under one display, and
    /// a display that called itself the synthesizer's because sixteen of its
    /// characters were would be the one lie this crate exists to avoid.
    #[must_use]
    pub fn claim_across(&self, parameters: impl IntoIterator<Item = ParamId>) -> Confidence {
        if self.program.is_none() {
            return Confidence::Unknown;
        }
        let mut claim = Confidence::Confirmed;
        for parameter in parameters {
            match self.claim(parameter) {
                Confidence::Unknown => return Confidence::Unknown,
                Confidence::Assumed => claim = Confidence::Assumed,
                Confidence::Confirmed => {}
            }
        }
        claim
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

    /// Names the program, because somebody in this window typed one.
    ///
    /// Returns the parameters that moved and what they moved to, which is what
    /// the application sends: a name is stored one character to a parameter, so
    /// renaming is an edit to each character that is not already what it should
    /// be, and typing a letter onto the end of a name is one NRPN rather than
    /// seventeen. [`Program::set_name`] writes the field and
    /// [`Program::changes`] is the whole of working out what that cost.
    ///
    /// Empty for a sound nobody has read, for the same reason an edit is
    /// refused there: a name is a difference, and there is nothing to take one
    /// against.
    pub fn rename(&mut self, name: ProgramName) -> Vec<(ParamId, u8)> {
        let Some(program) = self.program.as_ref() else {
            return Vec::new();
        };
        let mut named = program.clone();
        named.set_name(name);
        let changes: Vec<(ParamId, u8)> = program.changes(&named).collect();
        changes
            .into_iter()
            .filter(|&(parameter, value)| self.edit(parameter, value))
            .collect()
    }

    /// Takes a value the synthesizer reported.
    ///
    /// The instrument is the other editor: a hand on the front panel gets here,
    /// and last writer wins. `confirmed` is what the host crate said about the
    /// sound it tracks with the report applied, and an NRPN carries the whole
    /// value where a control change carries seven bits of it, so a coarse report
    /// is drawn as the assumption it is rather than quietly rounded into a fact.
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
    use deepmind_midi::program::ProgramName;

    use super::{Confidence, Group, ParamId, Patch, Program};

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
    fn a_section_is_as_confirmed_as_its_least_confirmed_parameter() {
        let mut patch = Patch::new();

        assert_eq!(patch.claim_of(Group::Vcf), Confidence::Unknown);

        patch.confirm(Program::new(ProtocolVersion::V7));
        assert_eq!(patch.claim_of(Group::Vcf), Confidence::Confirmed);

        // One fader in a panel nobody is looking at is the whole point of
        // drawing the claim on the tab.
        patch.edit(ParamId::VcfFrequency, 200);
        assert_eq!(patch.claim_of(Group::Vcf), Confidence::Assumed);
        assert_eq!(patch.claim_of(Group::Lfo1), Confidence::Confirmed);
    }

    #[test]
    fn a_name_is_the_characters_it_changed_and_nothing_else() {
        let mut patch = read();
        let name = ProgramName::new("Bass").expect("four printable characters");

        let moved = patch.rename(name);

        assert_eq!(patch.name(), Some(name));
        // Every parameter this window put a letter in says who put it there.
        for (parameter, _) in &moved {
            assert_eq!(patch.claim(*parameter), Confidence::Assumed);
        }
        // The program started with every parameter at its floor, so the four
        // letters moved and the thirteen bytes that were already zero did not.
        assert_eq!(moved.len(), name.len());
    }

    #[test]
    fn a_letter_onto_the_end_of_a_name_costs_one_parameter() {
        let mut patch = read();
        patch.rename(ProgramName::new("Bass").expect("a valid name"));

        let moved = patch.rename(ProgramName::new("Bassy").expect("a valid name"));

        assert_eq!(moved.len(), 1, "a keystroke moved more than its character");
        assert_eq!(moved.first().map(|&(_, value)| value), Some(b'y'));
    }

    #[test]
    fn renaming_a_program_to_what_it_is_called_sends_nothing() {
        let mut patch = read();
        let name = ProgramName::new("Bass").expect("a valid name");
        patch.rename(name);

        assert!(patch.rename(name).is_empty());
    }

    #[test]
    fn a_shorter_name_clears_the_characters_it_no_longer_uses() {
        let mut patch = read();
        patch.rename(ProgramName::new("Bass Sweep").expect("a valid name"));

        patch.rename(ProgramName::new("Bass").expect("a valid name"));

        assert_eq!(
            patch.name().map(|name| name.to_string()),
            Some("Bass".to_owned())
        );
    }

    #[test]
    fn a_sound_nobody_has_read_cannot_be_named() {
        let mut patch = Patch::new();

        assert!(
            patch
                .rename(ProgramName::new("Bass").expect("a valid name"))
                .is_empty()
        );
        assert_eq!(patch.name(), None);
    }

    #[test]
    fn a_name_is_as_confirmed_as_its_least_confirmed_character() {
        let mut patch = read();
        let characters = crate::name_characters().iter().copied();

        assert_eq!(
            patch.claim_across(characters.clone()),
            Confidence::Confirmed
        );

        patch.rename(ProgramName::new("B").expect("a valid name"));
        assert_eq!(patch.claim_across(characters), Confidence::Assumed);
        // One letter is not the whole program, and the rest of the sound is
        // still the synthesizer's own account of itself.
        assert_eq!(patch.claim(ParamId::VcfFrequency), Confidence::Confirmed);
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
