use super::{mdh_error_string, MdhValue};

const ERR_NO_AUDIO: &str = "Soond isnae available - build wi' --features audio";

extern "C" {
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_make_nil__rsabi")]
    fn __mdh_make_nil() -> MdhValue;
    #[cfg_attr(target_os = "emscripten", link_name = "__mdh_hurl__rsabi")]
    fn __mdh_hurl(value: MdhValue);
}

fn hurl() -> MdhValue {
    unsafe {
        __mdh_hurl(mdh_error_string(ERR_NO_AUDIO));
        __mdh_make_nil()
    }
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_stairt__rsabi")]
pub extern "C" fn __mdh_soond_stairt() -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_steek__rsabi")]
pub extern "C" fn __mdh_soond_steek() -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_wheesht__rsabi")]
pub extern "C" fn __mdh_soond_wheesht(_value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_luid__rsabi")]
pub extern "C" fn __mdh_soond_luid(_value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_hou_luid__rsabi")]
pub extern "C" fn __mdh_soond_hou_luid() -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_haud_gang__rsabi")]
pub extern "C" fn __mdh_soond_haud_gang() -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_lade__rsabi")]
pub extern "C" fn __mdh_soond_lade(_path: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_spiel__rsabi")]
pub extern "C" fn __mdh_soond_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_haud__rsabi")]
pub extern "C" fn __mdh_soond_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_gae_on__rsabi")]
pub extern "C" fn __mdh_soond_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_stap__rsabi")]
pub extern "C" fn __mdh_soond_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_unlade__rsabi")]
pub extern "C" fn __mdh_soond_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_is_spielin__rsabi")]
pub extern "C" fn __mdh_soond_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_pit_luid__rsabi")]
pub extern "C" fn __mdh_soond_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_pit_pan__rsabi")]
pub extern "C" fn __mdh_soond_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_pit_tune__rsabi")]
pub extern "C" fn __mdh_soond_pit_tune(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_pit_rin_roond__rsabi")]
pub extern "C" fn __mdh_soond_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_soond_ready__rsabi")]
pub extern "C" fn __mdh_soond_ready(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_lade__rsabi")]
pub extern "C" fn __mdh_muisic_lade(_path: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_spiel__rsabi")]
pub extern "C" fn __mdh_muisic_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_haud__rsabi")]
pub extern "C" fn __mdh_muisic_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_gae_on__rsabi")]
pub extern "C" fn __mdh_muisic_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_stap__rsabi")]
pub extern "C" fn __mdh_muisic_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_unlade__rsabi")]
pub extern "C" fn __mdh_muisic_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_is_spielin__rsabi")]
pub extern "C" fn __mdh_muisic_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_loup__rsabi")]
pub extern "C" fn __mdh_muisic_loup(_handle: MdhValue, _seconds: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_hou_lang__rsabi")]
pub extern "C" fn __mdh_muisic_hou_lang(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_whaur__rsabi")]
pub extern "C" fn __mdh_muisic_whaur(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_pit_luid__rsabi")]
pub extern "C" fn __mdh_muisic_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_pit_pan__rsabi")]
pub extern "C" fn __mdh_muisic_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_pit_tune__rsabi")]
pub extern "C" fn __mdh_muisic_pit_tune(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_muisic_pit_rin_roond__rsabi")]
pub extern "C" fn __mdh_muisic_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_lade__rsabi")]
pub extern "C" fn __mdh_midi_lade(_path: MdhValue, _soundfont: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_spiel__rsabi")]
pub extern "C" fn __mdh_midi_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_haud__rsabi")]
pub extern "C" fn __mdh_midi_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_gae_on__rsabi")]
pub extern "C" fn __mdh_midi_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_stap__rsabi")]
pub extern "C" fn __mdh_midi_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_unlade__rsabi")]
pub extern "C" fn __mdh_midi_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_is_spielin__rsabi")]
pub extern "C" fn __mdh_midi_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_loup__rsabi")]
pub extern "C" fn __mdh_midi_loup(_handle: MdhValue, _seconds: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_hou_lang__rsabi")]
pub extern "C" fn __mdh_midi_hou_lang(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_whaur__rsabi")]
pub extern "C" fn __mdh_midi_whaur(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_pit_luid__rsabi")]
pub extern "C" fn __mdh_midi_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_pit_pan__rsabi")]
pub extern "C" fn __mdh_midi_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[cfg_attr(not(target_os = "emscripten"), no_mangle)]
#[cfg_attr(target_os = "emscripten", export_name = "__mdh_midi_pit_rin_roond__rsabi")]
pub extern "C" fn __mdh_midi_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}
