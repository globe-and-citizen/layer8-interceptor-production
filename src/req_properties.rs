use web_sys::{AbortSignal, ReferrerPolicy, RequestMode, console};

use crate::fetch_api::{L8RequestObject, Mode};

pub fn add_properties_to_request(
    req_wrapper: &mut L8RequestObject,
    options: &web_sys::RequestInit,
) -> Option<AbortSignal> {
    // signal
    if let Some(signal) = options.get_signal() {
        // If the signal is provided, we can handle it here if needed.
        // For now, we just log it.
        console::log_1(&format!("AbortSignal: {:?}", signal).into());
        return Some(signal);
    }

    // retrieve mode if provided
    match options.get_mode() {
        Some(RequestMode::SameOrigin) => {
            req_wrapper.mode = Some(Mode::SameOrigin);
        }
        Some(RequestMode::NoCors) => {
            req_wrapper.mode = Some(Mode::NoCors);
        }
        Some(RequestMode::Cors) => {
            req_wrapper.mode = Some(Mode::Cors);
        }
        Some(RequestMode::Navigate) => {
            req_wrapper.mode = Some(Mode::Navigate);
        }
        _ => {}
    }

    // keepalive
    js_sys::Reflect::get(&options, &"keepalive".into())
        .ok()
        .and_then(|val| val.as_bool())
        .map(|keep_alive| req_wrapper.keep_alive = Some(keep_alive));

    // redirect
    js_sys::Reflect::get(&options, &"redirect".into())
        .ok()
        .map(|v| {
            let val = v.as_string().unwrap_or_else(|| "follow".to_string());
            req_wrapper.redirect = Some(val);
        });

    // referrer policy
    if let Some(referrer_policy) = options.get_referrer_policy() {
        match referrer_policy {
            ReferrerPolicy::NoReferrer => {
                req_wrapper
                    .headers
                    .insert("Referrer-Policy".to_string(), "no-referrer".to_string());
            }
            ReferrerPolicy::NoReferrerWhenDowngrade => {
                req_wrapper.headers.insert(
                    "Referrer-Policy".to_string(),
                    "no-referrer-when-downgrade".to_string(),
                );
            }
            ReferrerPolicy::Origin => {
                req_wrapper
                    .headers
                    .insert("Referrer-Policy".to_string(), "origin".to_string());
            }
            ReferrerPolicy::OriginWhenCrossOrigin => {
                req_wrapper.headers.insert(
                    "Referrer-Policy".to_string(),
                    "origin-when-cross-origin".to_string(),
                );
            }
            ReferrerPolicy::UnsafeUrl => {
                req_wrapper
                    .headers
                    .insert("Referrer-Policy".to_string(), "unsafe-url".to_string());
            }
            ReferrerPolicy::SameOrigin => {
                req_wrapper
                    .headers
                    .insert("Referrer-Policy".to_string(), "same-origin".to_string());
            }
            ReferrerPolicy::StrictOrigin => {
                req_wrapper
                    .headers
                    .insert("Referrer-Policy".to_string(), "strict-origin".to_string());
            }
            ReferrerPolicy::StrictOriginWhenCrossOrigin => {
                req_wrapper.headers.insert(
                    "Referrer-Policy".to_string(),
                    "strict-origin-when-cross-origin".to_string(),
                );
            }
            _ => {}
        }
    }

    // referrer
    options.get_referrer().map(|referrer| {
        // Update the headers with the referrer
        req_wrapper
            .headers
            .insert("Referrer".to_string(), referrer.to_string());
    });

    None
}
