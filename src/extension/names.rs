use std::{collections::HashMap, sync::OnceLock};

pub(super) fn filtertypes() -> &'static HashMap<i32, String> {
    static HASHMAP: OnceLock<HashMap<i32, String>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        HashMap::from([
            (0, String::from("Off")),
            (1, String::from("LP 12 dB")),
            (2, String::from("LP 24 dB")),
            (3, String::from("LP Legacy Ladder")),
            (10, String::from("LP Vintage Ladder")),
            (13, String::from("LP K35")),
            (15, String::from("LP Diode Ladder")),
            (11, String::from("LP OB-Xd 12 dB")),
            (12, String::from("LP OB-Xd 24 dB")),
            (16, String::from("LP Cutoff Warp")),
            (28, String::from("LP Res Warp")),
            (6, String::from("BP 12 dB")),
            (23, String::from("BP 24 dB")),
            (22, String::from("BP OB-Xd 12 dB")),
            (19, String::from("BP Cutoff Warp")),
            (31, String::from("BP Res Warp")),
            (4, String::from("HP 12 dB")),
            (5, String::from("HP 24 dB")),
            (14, String::from("HP K35")),
            (20, String::from("HP OB-Xd 12 dB")),
            (17, String::from("HP Cutoff Warp")),
            (29, String::from("HP Resonance Warp")),
            (7, String::from("Notch 12 dB")),
            (24, String::from("Notch 24 dB")),
            (21, String::from("Notch OB-Xd 12 dB")),
            (18, String::from("Notch Cutoff Warp")),
            (30, String::from("Notch Resonance Warp")),
            (33, String::from("Multi Tripole")),
            (36, String::from("FX Allpass")),
            (27, String::from("FX Cutoff Warp AP")),
            (32, String::from("FX Resonance Warp AP")),
            (8, String::from("FX Comb+")),
            (25, String::from("FX Comb-")),
            (9, String::from("FX Sample & Hold")),
        ])
    })
}

pub(super) fn fx_types() -> &'static HashMap<i32, String> {
    static HASHMAP: OnceLock<HashMap<i32, String>> = OnceLock::new();
    HASHMAP.get_or_init(|| {
        let effects = vec![
            "Off",
            "Delay",
            "Reverb 1",
            "Phaser",
            "Rotary",
            "Distortion",
            "EQ",
            "Freq Shift",
            "Conditioner",
            "Chorus",
            "Vocoder",
            "Reverb 2",
            "Flanger",
            "Ring Mod",
            "Airwindows",
            "Neuron",
            "Graphic EQ",
            "Resonator",
            "CHOW",
            "Exciter",
            "Ensemble",
            "Combulator",
            "Nimbus",
            "Tape",
            "Treemonster",
            "Waveshaper",
            "Mid-Side Tool",
            "Spring Reverb",
            "Bonsai",
            "Audio In",
        ];

        // Create a HashMap
        let effects_map: HashMap<i32, String> = effects
            .iter()
            .enumerate()
            .map(|(index, &effect)| (index as i32, effect.to_string()))
            .collect();
        effects_map
    })
}
