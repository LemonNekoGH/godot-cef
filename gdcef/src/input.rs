use cef::{ImplBrowserHost, MouseButtonType, MouseEvent};
use cef::sys::cef_event_flags_t;
use godot::classes::{InputEventMouseButton, InputEventMouseMotion};
use godot::global::{MouseButton, MouseButtonMask};
use godot::prelude::*;

/// Creates a CEF mouse event from Godot position and DPI scale
pub fn create_mouse_event(position: Vector2, dpi: f32, modifiers: u32) -> MouseEvent {
    let x = (position.x * dpi) as i32;
    let y = (position.y * dpi) as i32;

    MouseEvent {
        x,
        y,
        modifiers,
    }
}

/// Handles mouse button events and sends them to CEF browser host
pub fn handle_mouse_button(
    host: &impl ImplBrowserHost,
    event: &Gd<InputEventMouseButton>,
    dpi: f32,
) {
    let position = event.get_position();
    let mouse_event = create_mouse_event(position, dpi, get_modifiers_from_button_event(event));

    match event.get_button_index() {
        MouseButton::LEFT | MouseButton::MIDDLE | MouseButton::RIGHT => {
            let button_type = match event.get_button_index() {
                MouseButton::LEFT => MouseButtonType::LEFT,
                MouseButton::MIDDLE => MouseButtonType::MIDDLE,
                MouseButton::RIGHT => MouseButtonType::RIGHT,
                _ => unreachable!(),
            };
            let mouse_up = !event.is_pressed();
            let click_count = if event.is_double_click() { 2 } else { 1 };
            host.send_mouse_click_event(Some(&mouse_event), button_type, mouse_up as i32, click_count);
        }
        MouseButton::WHEEL_UP => {
            let delta = (120.0 * event.get_factor()) as i32;
            host.send_mouse_wheel_event(Some(&mouse_event), 0, delta);
        }
        MouseButton::WHEEL_DOWN => {
            let delta = (120.0 * event.get_factor()) as i32;
            host.send_mouse_wheel_event(Some(&mouse_event), 0, -delta);
        }
        MouseButton::WHEEL_LEFT => {
            let delta = (120.0 * event.get_factor()) as i32;
            host.send_mouse_wheel_event(Some(&mouse_event), -delta, 0);
        }
        MouseButton::WHEEL_RIGHT => {
            let delta = (120.0 * event.get_factor()) as i32;
            host.send_mouse_wheel_event(Some(&mouse_event), delta, 0);
        }
        _ => {}
    }
}

/// Handles mouse motion events and sends them to CEF browser host
pub fn handle_mouse_motion(
    host: &impl ImplBrowserHost,
    event: &Gd<InputEventMouseMotion>,
    dpi: f32,
) {
    let position = event.get_position();
    let mouse_event = create_mouse_event(position, dpi, get_modifiers_from_motion_event(event));
    host.send_mouse_move_event(Some(&mouse_event), false as i32);
}

/// Extracts modifier flags from a mouse button event
pub fn get_modifiers_from_button_event(event: &Gd<InputEventMouseButton>) -> u32 {
    let mut modifiers = cef_event_flags_t::EVENTFLAG_NONE.0;

    if event.is_shift_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0;
    }
    if event.is_ctrl_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0;
    }
    if event.is_alt_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_ALT_DOWN.0;
    }
    if event.is_meta_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0;
    }

    let button_mask = event.get_button_mask();
    if button_mask.is_set(MouseButtonMask::LEFT) {
        modifiers |= cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON.0;
    }
    if button_mask.is_set(MouseButtonMask::MIDDLE) {
        modifiers |= cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0;
    }
    if button_mask.is_set(MouseButtonMask::RIGHT) {
        modifiers |= cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON.0;
    }

    modifiers
}

/// Extracts modifier flags from a mouse motion event
pub fn get_modifiers_from_motion_event(event: &Gd<InputEventMouseMotion>) -> u32 {
    let mut modifiers = cef_event_flags_t::EVENTFLAG_NONE.0;

    if event.is_shift_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_SHIFT_DOWN.0;
    }
    if event.is_ctrl_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_CONTROL_DOWN.0;
    }
    if event.is_alt_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_ALT_DOWN.0;
    }
    if event.is_meta_pressed() {
        modifiers |= cef_event_flags_t::EVENTFLAG_COMMAND_DOWN.0;
    }

    let button_mask = event.get_button_mask();
    if button_mask.is_set(MouseButtonMask::LEFT) {
        modifiers |= cef_event_flags_t::EVENTFLAG_LEFT_MOUSE_BUTTON.0;
    }
    if button_mask.is_set(MouseButtonMask::MIDDLE) {
        modifiers |= cef_event_flags_t::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0;
    }
    if button_mask.is_set(MouseButtonMask::RIGHT) {
        modifiers |= cef_event_flags_t::EVENTFLAG_RIGHT_MOUSE_BUTTON.0;
    }

    modifiers
}

