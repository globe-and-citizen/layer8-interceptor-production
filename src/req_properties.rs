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
    req_wrapper.mode = match options.get_mode() {
        Some(RequestMode::SameOrigin) => Some(Mode::SameOrigin),
        Some(RequestMode::NoCors) => Some(Mode::NoCors),
        Some(RequestMode::Cors) => Some(Mode::Cors),
        Some(RequestMode::Navigate) => Some(Mode::Navigate),
        _ => Some(Mode::Cors),
    };

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
    let mut referrer_policy = "";
    if let Some(referrer_policy_) = options.get_referrer_policy() {
        referrer_policy = match referrer_policy_ {
            ReferrerPolicy::NoReferrer => "no-referrer",
            ReferrerPolicy::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            ReferrerPolicy::Origin => "origin",
            ReferrerPolicy::OriginWhenCrossOrigin => "origin-when-cross-origin",
            ReferrerPolicy::UnsafeUrl => "unsafe-url",
            ReferrerPolicy::SameOrigin => "same-origin",
            ReferrerPolicy::StrictOrigin => "strict-origin",
            ReferrerPolicy::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            _ => "",
        };
    }

    if !referrer_policy.is_empty() {
        req_wrapper
            .headers
            .insert("Referrer-Policy".to_string(), referrer_policy.to_string());
    }

    // referrer
    if referrer_policy != "no-referrer" {
        // If the referrer policy is not "no-referrer", we can set the referrer header.
        if let Some(referrer) = options.get_referrer() {
            req_wrapper
                .headers
                .insert("Referrer".to_string(), referrer.to_string());
        }
    }

    None
}
