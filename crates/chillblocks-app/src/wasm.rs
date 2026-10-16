use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use chillblocks_platform::web::{WebAudio, WebClock, WebPainter, WebStore};
use chillblocks_ui::geometry::FPoint;
use chillblocks_ui::input::{Command, PointerEvent, PointerKind, PointerPhase};
use chillblocks_ui::layout::Insets;

use crate::app::App;

type Shared = Rc<RefCell<App>>;

#[wasm_bindgen]
pub fn boot() -> Result<(), String> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas = document
        .get_element_by_id("game")
        .ok_or("no element with id \"game\"")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "element \"game\" is not a canvas")?;

    let app: Shared = Rc::new(RefCell::new(App::new(
        Box::new(WebPainter::new(&canvas).map_err(|e| e.to_string())?),
        Box::new(WebStore::new()),
        Box::new(WebClock),
        Box::new(WebAudio::new(false)),
    )));

    for event in ["pointerdown", "pointermove", "pointerup", "pointercancel"] {
        let app = Rc::clone(&app);
        let target = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::PointerEvent| {
            let event = map_pointer(&event, &target);
            let app = &mut *app.borrow_mut();
            app.handle_pointer(&event);
        }) as Box<dyn FnMut(web_sys::PointerEvent)>);

        {
            let function = function(&closure)?;
            canvas
                .add_event_listener_with_callback(event, function)
                .map_err(|e| format!("{e:?}"))?;
        }
        // The page lives as long as the app, so keeping these alive is intentional.
        closure.forget();
    }

    let key_app = Rc::clone(&app);
    let key_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let command = match map_key(&event) {
            Some(command) => command,
            None => return,
        };
        let app = &mut *key_app.borrow_mut();
        app.handle_command(command);
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
    {
        let function = function(&key_closure)?;
        window
            .add_event_listener_with_callback("keydown", function)
            .map_err(|e| format!("{e:?}"))?;
    }
    key_closure.forget();

    // The frame callback has to re-arm itself, so it lives in a cell it can read.
    let loop_slot: Rc<RefCell<Option<Rc<Closure<dyn FnMut(f64)>>>>> = Rc::new(RefCell::new(None));
    {
        let app = Rc::clone(&app);
        let canvas = canvas.clone();
        let window = window.clone();
        let slot = Rc::clone(&loop_slot);
        let closure = Closure::wrap(Box::new(move |_now: f64| {
            {
                let app = &mut *app.borrow_mut();
                sync_size(app, &canvas);
                app.frame();
            }
            if let Some(loop_closure) = slot.borrow().as_ref() {
                if let Ok(function) = function(loop_closure) {
                    let _ = window.request_animation_frame(function);
                }
            }
        }) as Box<dyn FnMut(f64)>);
        *loop_slot.borrow_mut() = Some(Rc::new(closure));
    }

    if let Some(loop_closure) = loop_slot.borrow().as_ref() {
        let function = function(loop_closure)?;
        window
            .request_animation_frame(function)
            .map_err(|e| format!("{e:?}"))?;
    }

    Ok(())
}

fn function<T: ?Sized>(closure: &Closure<T>) -> Result<&js_sys::Function, String> {
    closure
        .as_ref()
        .dyn_ref::<js_sys::Function>()
        .ok_or_else(|| "closure is not a function".to_string())
}

fn map_pointer(event: &web_sys::PointerEvent, canvas: &web_sys::HtmlCanvasElement) -> PointerEvent {
    // Client coordinates relative to the canvas are used rather than offsetX/offsetY,
    // which are unreliable for synthetic events and change when the page is zoomed.
    let rect = canvas.get_bounding_client_rect();
    PointerEvent {
        id: event.pointer_id() as i32,
        phase: match event.type_().as_str() {
            "pointerdown" => PointerPhase::Down,
            "pointermove" => PointerPhase::Move,
            "pointerup" => PointerPhase::Up,
            _ => PointerPhase::Cancel,
        },
        kind: match event.pointer_type().as_str() {
            "touch" => PointerKind::Touch,
            "pen" => PointerKind::Pen,
            _ => PointerKind::Mouse,
        },
        point: FPoint {
            x: (event.client_x() as f32 - rect.x() as f32),
            y: (event.client_y() as f32 - rect.y() as f32),
        },
        time_ms: now_ms(),
    }
}

fn map_key(event: &web_sys::KeyboardEvent) -> Option<Command> {
    let command = match event.key().as_str() {
        "1" => Command::SelectSlot(0),
        "2" => Command::SelectSlot(1),
        "3" => Command::SelectSlot(2),
        "ArrowLeft" => Command::MoveCursor(-1, 0),
        "ArrowRight" => Command::MoveCursor(1, 0),
        "ArrowUp" => Command::MoveCursor(0, -1),
        "ArrowDown" => Command::MoveCursor(0, 1),
        "Enter" => Command::Place,
        "r" | "R" => Command::NewGame,
        "m" | "M" => Command::ToggleSound,
        _ => return None,
    };
    Some(command)
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or(0.0)
}

fn parse_px(value: &str) -> f32 {
    value
        .trim()
        .trim_end_matches("px")
        .trim()
        .parse()
        .unwrap_or(0.0)
}

/// Safe-area insets come from CSS custom properties, so the layout respects notches and
/// home indicators without any platform-specific code.
fn read_insets() -> Insets {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return Insets::default(),
    };
    let document = match window.document() {
        Some(document) => document,
        None => return Insets::default(),
    };
    let element = match document.document_element() {
        Some(element) => element,
        None => return Insets::default(),
    };
    let style = match window.get_computed_style(&element) {
        Ok(Some(style)) => style,
        _ => return Insets::default(),
    };
    Insets {
        top: parse_px(&style.get_property_value("--safe-top").unwrap_or_default()),
        bottom: parse_px(
            &style
                .get_property_value("--safe-bottom")
                .unwrap_or_default(),
        ),
        left: parse_px(&style.get_property_value("--safe-left").unwrap_or_default()),
        right: parse_px(&style.get_property_value("--safe-right").unwrap_or_default()),
    }
}

fn sync_size(app: &mut App, canvas: &web_sys::HtmlCanvasElement) {
    let dpr = web_sys::window()
        .map(|window| window.device_pixel_ratio())
        .unwrap_or(1.0) as f32;
    let css_w = canvas.client_width() as f32;
    let css_h = canvas.client_height() as f32;
    if css_w <= 0.0 || css_h <= 0.0 {
        return;
    }
    let target_w = (css_w * dpr).round();
    let target_h = (css_h * dpr).round();
    if canvas.width() as f32 != target_w || canvas.height() as f32 != target_h {
        canvas.set_width(target_w as u32);
        canvas.set_height(target_h as u32);
    }
    app.resize(css_w, css_h, dpr, read_insets());
}
