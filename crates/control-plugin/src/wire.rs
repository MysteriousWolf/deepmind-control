//! A parameter edit as the control changes that carry it.

use deepmind_midi::param::{NrpnEdit, ParamId};

/// Returns the four control changes that set `parameter` to `value`, as the
/// controller number and the level a host is handed for each.
///
/// The library says what the four are (the NRPN number's two halves and then the
/// value's); this only changes how the value is written down. A host hands a
/// plugin a control change as a level between nought and one, and takes it back
/// as `level * 127` rounded, so a seven-bit value divided by 127 comes out as
/// itself.
///
/// Returns `None` for a value the parameter does not accept, which is a value
/// the library would not send either.
#[must_use]
pub fn controls(parameter: ParamId, value: u8) -> Option<[(u8, f32); 4]> {
    let edit = NrpnEdit::new(parameter, u16::from(value)).ok()?;
    Some(
        edit.controls()
            .map(|(controller, data)| (controller, level(data))),
    )
}

/// A seven-bit data byte as the level a host carries it in.
fn level(data: u8) -> f32 {
    f32::from(data) / 127.0
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use deepmind_midi::wire::Channel;

    use super::*;

    /// What a host does to a level on the way out.
    fn byte(level: f32) -> u8 {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to a seven-bit range first, as the host does"
        )]
        let byte = (level * 127.0).round().clamp(0.0, 127.0) as u8;
        byte
    }

    #[test]
    fn every_seven_bit_value_survives_the_host() {
        for data in 0..=127_u8 {
            assert_eq!(byte(level(data)), data);
        }
    }

    #[test]
    fn it_is_the_nrpn_the_library_encodes() {
        let parameter = ParamId::ALL.last().copied().expect("there are parameters");
        let value = u8::try_from(parameter.min()).unwrap_or(0);
        let controls = controls(parameter, value).expect("the minimum is accepted");

        let mut wire = [0_u8; NrpnEdit::LEN];
        let edit = NrpnEdit::new(parameter, u16::from(value)).expect("the minimum is accepted");
        edit.encode_into(Channel::ONE, &mut wire).expect("room");

        for (message, (controller, level)) in wire.as_chunks::<3>().0.iter().zip(controls) {
            assert_eq!(*message, [0xB0, controller, byte(level)]);
        }
    }

    #[test]
    fn a_value_out_of_range_is_not_sent() {
        let parameter = ParamId::ALL.first().copied().expect("there are parameters");
        let past = u8::try_from(parameter.max() + 1).unwrap_or(u8::MAX);
        if u16::from(past) > parameter.max() {
            assert_eq!(controls(parameter, past), None);
        }
    }
}
