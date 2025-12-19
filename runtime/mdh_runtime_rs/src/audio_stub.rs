use super::{mdh_make_string_from_rust, MdhValue};

const ERR_NO_AUDIO: &str = "Soond isnae available - build wi' --features audio";

extern "C" {
    fn __mdh_make_nil() -> MdhValue;
    fn __mdh_hurl(value: MdhValue);
}

fn hurl() -> MdhValue {
    unsafe {
        __mdh_hurl(mdh_make_string_from_rust(ERR_NO_AUDIO));
        __mdh_make_nil()
    }
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stairt() -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_steek() -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_wheesht(_value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_luid(_value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_hou_luid() -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud_gang() -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_lade(_path: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_tune(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_soond_ready(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_lade(_path: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_loup(_handle: MdhValue, _seconds: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_hou_lang(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_whaur(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_tune(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_muisic_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_lade(_path: MdhValue, _soundfont: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_spiel(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_haud(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_gae_on(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_stap(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_unlade(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_is_spielin(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_loup(_handle: MdhValue, _seconds: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_hou_lang(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_whaur(_handle: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_luid(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_pan(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}

#[no_mangle]
pub extern "C" fn __mdh_midi_pit_rin_roond(_handle: MdhValue, _value: MdhValue) -> MdhValue {
    hurl()
}
