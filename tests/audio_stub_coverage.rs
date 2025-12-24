#![cfg(not(feature = "audio"))]

use mdhavers::audio::register_audio_functions;
use mdhavers::value::{Environment, NativeFunction, Value};
use std::cell::RefCell;
use std::rc::Rc;

fn get_native(env: &Rc<RefCell<Environment>>, name: &str) -> Rc<NativeFunction> {
    let value = env.borrow().get(name).unwrap_or_else(|| panic!("missing stub {name}"));
    match value {
        Value::NativeFunction(func) => func,
        other => panic!("expected native function for {name}, got {other:?}"),
    }
}

#[test]
fn audio_stubs_are_registered_and_return_expected_error_when_audio_feature_disabled() {
    let env = Rc::new(RefCell::new(Environment::new()));
    register_audio_functions(&env);

    let expected = "Soond isnae available - build wi' --features audio";

    let stubs: &[(&str, usize)] = &[
        ("soond_stairt", 0),
        ("soond_steek", 0),
        ("soond_wheesht", 1),
        ("soond_luid", 1),
        ("soond_hou_luid", 0),
        ("soond_haud_gang", 0),
        ("soond_lade", 1),
        ("soond_spiel", 1),
        ("soond_haud", 1),
        ("soond_gae_on", 1),
        ("soond_stap", 1),
        ("soond_unlade", 1),
        ("soond_is_spielin", 1),
        ("soond_pit_luid", 2),
        ("soond_pit_pan", 2),
        ("soond_pit_tune", 2),
        ("soond_pit_rin_roond", 2),
        ("soond_ready", 1),
        ("muisic_lade", 1),
        ("muisic_spiel", 1),
        ("muisic_haud", 1),
        ("muisic_gae_on", 1),
        ("muisic_stap", 1),
        ("muisic_unlade", 1),
        ("muisic_is_spielin", 1),
        ("muisic_loup", 2),
        ("muisic_hou_lang", 1),
        ("muisic_whaur", 1),
        ("muisic_pit_luid", 2),
        ("muisic_pit_pan", 2),
        ("muisic_pit_tune", 2),
        ("muisic_pit_rin_roond", 2),
        ("midi_lade", 2),
        ("midi_spiel", 1),
        ("midi_haud", 1),
        ("midi_gae_on", 1),
        ("midi_stap", 1),
        ("midi_unlade", 1),
        ("midi_is_spielin", 1),
        ("midi_loup", 2),
        ("midi_hou_lang", 1),
        ("midi_whaur", 1),
        ("midi_pit_luid", 2),
        ("midi_pit_pan", 2),
        ("midi_pit_rin_roond", 2),
    ];

    for &(name, arity) in stubs {
        let func = get_native(&env, name);
        assert_eq!(func.arity, arity);

        let err = (func.func)(vec![Value::Nil; arity]).unwrap_err();
        assert_eq!(err, expected, "stub {name} returned unexpected error");
    }
}
