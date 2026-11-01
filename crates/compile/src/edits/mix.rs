use opsis_types::edits::{BusPatch, MasterPatch, MixEdit, StripPatch};
use opsis_types::mix::{Bus, Mix, Strip};

use crate::edits::EditError;
use crate::ids::Ids;

pub fn patch_strip(strip: &mut Strip, patch: &StripPatch) {
    if let Some(trim) = patch.trim {
        strip.trim = trim;
    }
    if let Some(highpass) = &patch.highpass {
        strip.highpass = highpass.clone();
    }
    if let Some(eq) = &patch.eq {
        strip.eq = eq.clone();
    }
    if let Some(comp) = &patch.comp {
        strip.comp = comp.clone();
    }
    if let Some(fader) = patch.fader {
        strip.fader = fader;
    }
    if let Some(pan) = patch.pan {
        strip.pan = pan;
    }
    if let Some(mute) = patch.mute {
        strip.mute = mute;
    }
    if let Some(solo) = patch.solo {
        strip.solo = solo;
    }
    if let Some(bus) = &patch.bus {
        strip.bus = Some(bus.clone()).filter(|b| !b.is_empty());
    }
}

pub fn reset_strip(strip: &mut Strip) {
    *strip = Strip {
        mute: strip.mute,
        solo: strip.solo,
        bus: strip.bus.take(),
        ..Strip::default()
    };
}

pub fn patch_bus(bus: &mut Bus, patch: &BusPatch) {
    if let Some(name) = &patch.name {
        bus.name.clone_from(name);
    }
    if let Some(highpass) = &patch.highpass {
        bus.highpass = highpass.clone();
    }
    if let Some(eq) = &patch.eq {
        bus.eq = eq.clone();
    }
    if let Some(comp) = &patch.comp {
        bus.comp = comp.clone();
    }
    if let Some(fader) = patch.fader {
        bus.fader = fader;
    }
    if let Some(mute) = patch.mute {
        bus.mute = mute;
    }
    if let Some(solo) = patch.solo {
        bus.solo = solo;
    }
    if let Some(duck) = &patch.duck {
        bus.duck = duck.clone();
    }
}

pub fn reset_bus(bus: &mut Bus) {
    let fresh = Bus::new(&bus.id, &bus.name);
    *bus = Bus {
        mute: bus.mute,
        solo: bus.solo,
        duck: std::mem::take(&mut bus.duck),
        ..fresh
    };
}

fn patch_master(mix: &mut Mix, patch: MasterPatch) {
    let master = &mut mix.master;
    if let Some(fader) = patch.fader {
        master.fader = fader;
    }
    if let Some(eq) = patch.eq {
        master.eq = eq;
    }
    if let Some(comp) = patch.comp {
        master.comp = comp;
    }
    if let Some(limiter) = patch.limiter {
        master.limiter = limiter;
    }
    if let Some(loudness) = patch.loudness {
        master.loudness = loudness;
    }
}

fn bus_mut<'a>(mix: &'a mut Mix, id: &str) -> Result<&'a mut Bus, EditError> {
    mix.buses
        .iter_mut()
        .find(|b| b.id == id)
        .ok_or_else(|| EditError::NoBus(id.to_owned()))
}

fn channels(mix: &mut Mix, ids: &[String], change: impl Fn(&mut Strip)) {
    for id in ids {
        change(mix.channels.entry(id.clone()).or_default());
    }
}

pub fn add_bus(mix: &mut Mix, id: String) -> String {
    let name = format!("Bus {}", mix.buses.len() + 1);
    mix.buses.push(Bus::new(&id, &name));
    id
}

pub fn remove_bus(mix: &mut Mix, id: &str) {
    mix.buses.retain(|b| b.id != id);
    for bus in &mut mix.buses {
        bus.duck.keys.retain(|k| k != id);
    }
    for strip in mix.channels.values_mut() {
        if strip.bus.as_deref() == Some(id) {
            strip.bus = None;
        }
    }
}

pub fn apply_mix_edit(
    mix: &mut Mix,
    edit: MixEdit,
    ids: &mut dyn Ids,
) -> Result<Option<String>, EditError> {
    match edit {
        MixEdit::Channels { ids, patch } => channels(mix, &ids, |s| patch_strip(s, &patch)),
        MixEdit::ResetChannels { ids } => channels(mix, &ids, reset_strip),
        MixEdit::Bus { id, patch } => patch_bus(bus_mut(mix, &id)?, &patch),
        MixEdit::ResetBus { id } => reset_bus(bus_mut(mix, &id)?),
        MixEdit::AddBus => return Ok(Some(add_bus(mix, ids.id("bus")))),
        MixEdit::RemoveBus { id } => {
            bus_mut(mix, &id)?;
            remove_bus(mix, &id);
        }
        MixEdit::Master { patch } => patch_master(mix, patch),
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use opsis_types::mix::{Bus, Mix, Strip};

    use super::*;
    use crate::ids::SaltedIds;

    #[test]
    fn resets_a_strip_but_keeps_its_routing() {
        let mut strip = Strip {
            trim: 3.0,
            fader: -6.0,
            mute: true,
            bus: Some("music".into()),
            ..Strip::default()
        };
        reset_strip(&mut strip);
        assert_eq!(
            strip,
            Strip {
                mute: true,
                bus: Some("music".into()),
                ..Strip::default()
            }
        );
    }

    #[test]
    fn applies_the_bus_part_of_a_patch_and_resets_it() {
        let mut bus = Bus {
            mute: true,
            ..Bus::new("b", "Voices")
        };
        let before = bus.clone();
        patch_bus(
            &mut bus,
            &BusPatch {
                fader: Some(-3.0),
                ..BusPatch::default()
            },
        );
        assert_eq!(bus.fader, -3.0);
        assert_eq!(bus.name, "Voices");
        reset_bus(&mut bus);
        assert_eq!(bus, before);
    }

    #[test]
    fn adds_and_removes_a_bus_everywhere_it_is_used() {
        let mut mix = Mix::default();
        let mut ids = SaltedIds::new("t");
        let id = apply_mix_edit(&mut mix, MixEdit::AddBus, &mut ids)
            .unwrap()
            .unwrap();
        assert_eq!(mix.buses.last().map(|b| b.name.as_str()), Some("Bus 4"));
        mix.buses[0].duck.keys = vec![id.clone()];
        apply_mix_edit(
            &mut mix,
            MixEdit::Channels {
                ids: vec!["a".into()],
                patch: StripPatch {
                    bus: Some(id.clone()),
                    ..StripPatch::default()
                },
            },
            &mut ids,
        )
        .unwrap();
        apply_mix_edit(&mut mix, MixEdit::RemoveBus { id }, &mut ids).unwrap();
        let buses: Vec<&str> = mix.buses.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(buses, ["music", "effects", "clips"]);
        assert_eq!(mix.buses[0].duck.keys, [] as [String; 0]);
        assert_eq!(mix.channels["a"].bus, None);
    }
}
