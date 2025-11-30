use std::{cell::RefCell, rc::Rc};

use futures::channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, PositionError, PositionOptions};

use crate::models::mosque::Center;

async fn get_user_location() -> Result<Center, String> {
    let window = window().ok_or_else(|| String::from("no global `window` exists"))?;
    let navigator = window.navigator();
    let geolocation = navigator
        .geolocation()
        .map_err(|err| format!("Geolocation not supported: {:?}", err))
        .map_err(|err| String::from(err))?;

    let (sender, receiver) = oneshot::channel();
    let sender = Rc::new(RefCell::new(Some(sender)));

    let success_callback = {
        let sender = sender.clone();
        Closure::wrap(Box::new(move |position: web_sys::Position| {
            let coords = position.coords();
            let lat = coords.latitude();
            let lon = coords.longitude();

            if let Some(sender) = sender.borrow_mut().take() {
                sender.send(Ok(Center { lat, lon })).ok();
            }
        }) as Box<dyn FnMut(web_sys::Position)>)
    };

    let error_callback = {
        let sender = sender.clone();
        Closure::wrap(Box::new(move |err: PositionError| {
            let msg = err.message();
            if let Some(sender) = sender.borrow_mut().take() {
                sender.send(Err(format!("Location error: {}", msg))).ok();
            }
        }) as Box<dyn FnMut(PositionError)>)
    };

    let options = PositionOptions::new();
    options.set_enable_high_accuracy(true);
    options.set_timeout(10000);

    geolocation
        .get_current_position_with_error_callback_and_options(
            success_callback.as_ref().unchecked_ref(),
            Some(error_callback.as_ref().unchecked_ref()),
            &options,
        )
        .map_err(|err| format!("Failed to request location: {:?}", err))?;

    success_callback.forget();
    error_callback.forget();

    receiver
        .await
        .map_err(|_| String::from("Location request cancelled"))?
}
