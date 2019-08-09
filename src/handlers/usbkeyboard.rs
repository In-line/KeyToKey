use crate::handlers::ProcessKeys;
use crate::key_codes::{KeyCode, UNICODE_BELOW_256};
use crate::key_stream::{iter_unhandled_mut, Event, EventStatus};
use crate::Modifier::*;
use crate::USBKeyOut;
use core::convert::TryInto;
use no_std_compat::prelude::v1::*;
use smallbitvec::sbvec;

/// The default bottom layer
///
/// this simulates a bog standard regular USB
/// Keyboard.
/// Just map your keys to the usb keycodes.
///
/// key repeat is whatever usb does...
pub struct USBKeyboard {}
impl USBKeyboard {
    pub fn new() -> USBKeyboard {
        USBKeyboard {}
    }
}
fn is_usb_keycode(kc: u32) -> bool {
    return UNICODE_BELOW_256 <= kc && kc <= UNICODE_BELOW_256 + 0xE7; //RGui
}
impl<T: USBKeyOut> ProcessKeys<T> for USBKeyboard {
    fn process_keys(&mut self, events: &mut Vec<(Event, EventStatus)>, output: &mut T) {
        //step 0: on key release, remove all prior key presses.
        let mut codes_to_delete: Vec<u32> = Vec::new();
        let mut modifiers_sent = sbvec![false; 4];
        for (e, status) in iter_unhandled_mut(events).rev() {
            //note that we're doing this in reverse, ie. releases happen before presses.
            match e {
                Event::KeyRelease(kc) => {
                    if is_usb_keycode(kc.keycode) {
                        if !codes_to_delete.contains(&kc.original_keycode) {
                            codes_to_delete.push(kc.original_keycode);
                        }
                        *status = EventStatus::Handled;
                    }
                    if kc.keycode == KeyCode::LShift.into() || kc.keycode == KeyCode::RShift.into()
                    {
                        output.state().set_modifier(Shift, false);
                    } else if kc.keycode == KeyCode::LCtrl.into()
                        || kc.keycode == KeyCode::RCtrl.into()
                    {
                        output.state().set_modifier(Ctrl, false);
                    } else if kc.keycode == KeyCode::LAlt.into()
                        || kc.keycode == KeyCode::RAlt.into()
                    {
                        output.state().set_modifier(Alt, false);
                    } else if kc.keycode == KeyCode::LGui.into()
                        || kc.keycode == KeyCode::RGui.into()
                    {
                        output.state().set_modifier(Gui, false);
                    }
                }
                Event::KeyPress(kc) => {
                    let mut send = false;
                    if codes_to_delete.contains(&kc.original_keycode) {
                        *status = EventStatus::Handled;
                        if kc.flag & 0x1 == 0 {
                            //we have never send this before
                            send = true;
                        }
                    } else {
                        send = true;
                        if kc.keycode == KeyCode::LShift.into()
                            || kc.keycode == KeyCode::RShift.into()
                        {
                            output.state().set_modifier(Shift, true);
                            modifiers_sent.set(0, true);
                        } else if kc.keycode == KeyCode::LCtrl.into()
                            || kc.keycode == KeyCode::RCtrl.into()
                        {
                            output.state().set_modifier(Ctrl, true);
                            modifiers_sent.set(1, true);
                        } else if kc.keycode == KeyCode::LAlt.into()
                            || kc.keycode == KeyCode::RAlt.into()
                        {
                            output.state().set_modifier(Alt, true);
                            modifiers_sent.set(2, true);
                        } else if kc.keycode == KeyCode::LGui.into()
                            || kc.keycode == KeyCode::RGui.into()
                        {
                            output.state().set_modifier(Gui, true);
                            modifiers_sent.set(3, true);
                        }
                    }
                    if is_usb_keycode(kc.keycode) {
                        let oc: Result<KeyCode, String> = (kc.keycode).try_into();
                        match oc {
                            Ok(x) => {
                                if send {
                                    output.register_key(x);
                                }
                                if *status != EventStatus::Handled {
                                    *status = EventStatus::Ignored; //so we may resend it...
                                }
                            }
                            Err(_) => *status = EventStatus::Handled, //throw it away, will ya?
                        };
                        kc.flag |= 1;
                    }
                }
                Event::TimeOut(_) => {}
            }
        }
        if output.state().modifier(Shift) && !modifiers_sent[0] {
            output.register_key(KeyCode::LShift);
        }
        if output.state().modifier(Ctrl) && !modifiers_sent[1] {
            output.register_key(KeyCode::LCtrl);
        }
        if output.state().modifier(Alt) && !modifiers_sent[2] {
            output.register_key(KeyCode::LAlt);
        }
        if output.state().modifier(Gui) && !modifiers_sent[3] {
            output.register_key(KeyCode::LGui);
        }
        output.send_registered();
    }
}
#[cfg(test)]
//#[macro_use]
//extern crate std;
mod tests {
    use crate::handlers::USBKeyboard;
    #[allow(unused_imports)]
    use crate::key_codes::KeyCode;
    #[allow(unused_imports)]
    use crate::test_helpers::{check_output, KeyOutCatcher};
    use crate::Modifier::*;
    #[allow(unused_imports)]
    use crate::{
        Event, EventStatus, Keyboard, KeyboardState, ProcessKeys, USBKeyOut, UnicodeSendMode,
    };
    #[allow(unused_imports)]
    use no_std_compat::prelude::v1::*;
    #[test]
    fn test_usbkeyboard_single_key() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(KeyCode::A, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::A]]);
        assert!(!keyboard.events.is_empty());
        keyboard.add_keyrelease(KeyCode::A, 20);
        assert!(keyboard.events.len() == 2);
        keyboard.output.clear();
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[]]);
        assert!(keyboard.events.is_empty());
    }
    #[test]
    fn test_usbkeyboard_multiple_key() {
        use crate::key_codes::KeyCode::*;
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(A, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[A]]);
        assert!(!keyboard.events.is_empty());
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::X, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[A, X]]);
        assert!(!keyboard.events.is_empty());
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::A, 20);
        assert!(keyboard.events.len() == 3);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[X]]);
        assert!(!keyboard.events.is_empty());
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::X, 20);
        assert!(keyboard.events.len() == 2);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[]]);
        assert!(keyboard.events.is_empty());
    }
    #[test]
    fn test_panic_on_unhandled() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(0xF0000u32, 0);
        assert!(keyboard.handle_keys().is_err());
    }
    #[test]
    fn test_modifiers_add_left_keycodes() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::Kb1]]);
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[]]);
        keyboard.output.clear();
        keyboard.output.state().set_modifier(Shift, true);
        keyboard.add_keypress(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::Kb1, KeyCode::LShift]]);
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift]]);
        keyboard.output.clear();
        keyboard.output.state().set_modifier(Shift, false);
        keyboard.output.state().set_modifier(Ctrl, true);
        keyboard.add_keypress(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::Kb1, KeyCode::LCtrl]]);
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LCtrl]]);
        keyboard.output.clear();
        keyboard.output.state().set_modifier(Ctrl, false);
        keyboard.output.state().set_modifier(Alt, true);
        keyboard.add_keypress(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::Kb1, KeyCode::LAlt]]);
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LAlt]]);
        keyboard.output.clear();
        keyboard.output.state().set_modifier(Alt, false);
        keyboard.output.state().set_modifier(Gui, true);
        keyboard.add_keypress(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::Kb1, KeyCode::LGui]]);
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::Kb1, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LGui]]);
    }
    #[test]
    fn test_modifiers_set_by_keycodes() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::LAlt, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift, KeyCode::LAlt]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::LCtrl, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[KeyCode::LShift, KeyCode::LAlt, KeyCode::LCtrl]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::LGui, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[
                KeyCode::LShift,
                KeyCode::LAlt,
                KeyCode::LCtrl,
                KeyCode::LGui,
            ]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::LGui, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[KeyCode::LShift, KeyCode::LAlt, KeyCode::LCtrl]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::LCtrl, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift, KeyCode::LAlt]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::LAlt, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[]]);
        assert!(!keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::RShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::RShift]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::RAlt, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::RShift, KeyCode::RAlt]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::RCtrl, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[KeyCode::RShift, KeyCode::RAlt, KeyCode::RCtrl]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::RGui, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[
                KeyCode::RShift,
                KeyCode::RAlt,
                KeyCode::RCtrl,
                KeyCode::RGui,
            ]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::RGui, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[KeyCode::RShift, KeyCode::RAlt, KeyCode::RCtrl]],
        );
        assert!(keyboard.output.state().modifier(Shift));
        assert!(keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::RCtrl, 0);
        keyboard.handle_keys().unwrap();
        dbg!(&keyboard.output.reports);
        check_output(&keyboard, &[&[KeyCode::RShift, KeyCode::RAlt]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::RAlt, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::RShift]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keyrelease(KeyCode::RShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[]]);
        assert!(!keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
        keyboard.add_keypress(KeyCode::LShift, 0);
        keyboard.add_keypress(KeyCode::RShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift, KeyCode::RShift]]);
        assert!(keyboard.output.state().modifier(Shift));
        assert!(!keyboard.output.state().modifier(Ctrl));
        assert!(!keyboard.output.state().modifier(Alt));
        assert!(!keyboard.output.state().modifier(Gui));
        keyboard.output.clear();
    }
    #[test]
    fn test_unshift() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift]]);
        keyboard.add_keypress(KeyCode::A, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[&[KeyCode::LShift], &[KeyCode::LShift, KeyCode::A]],
        );
        keyboard.add_keyrelease(KeyCode::A, 0);
        keyboard.handle_keys().unwrap();
        check_output(
            &keyboard,
            &[
                &[KeyCode::LShift],
                &[KeyCode::LShift, KeyCode::A],
                &[KeyCode::LShift],
            ],
        );
        keyboard.add_keyrelease(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        assert!(keyboard.output.state().modifier(Shift) == false);
        check_output(
            &keyboard,
            &[
                &[KeyCode::LShift],
                &[KeyCode::LShift, KeyCode::A],
                &[KeyCode::LShift],
                &[],
            ],
        );
        &keyboard.output.clear();
        keyboard.add_keypress(KeyCode::A, 0);
        keyboard.handle_keys().unwrap();
        dbg!(&keyboard.output.reports);
        check_output(&keyboard, &[&[KeyCode::A]]);
    }
    #[test]
    fn test_unshift2() {
        let mut keyboard = Keyboard::new(KeyOutCatcher::new());
        keyboard.add_handler(Box::new(USBKeyboard::new()));
        keyboard.add_keypress(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift]]);
        keyboard.add_keyrelease(KeyCode::LShift, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift], &[]]);
        keyboard.add_keypress(KeyCode::A, 0);
        keyboard.handle_keys().unwrap();
        check_output(&keyboard, &[&[KeyCode::LShift], &[], &[KeyCode::A]]);
    }
}
