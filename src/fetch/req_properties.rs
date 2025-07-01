use web_sys::{ReferrerPolicy, RequestMode};

use crate::fetch::fetch_api::{L8RequestObject, Mode};

// Ref: <https://developer.mozilla.org/en-US/docs/Web/API/Request>
pub fn add_properties_to_request(
    req_wrapper: &mut L8RequestObject,
    options: &web_sys::RequestInit,
) {
    // body used
    req_wrapper.body_used = false; // default value

    // cache
    req_wrapper.cache = js_sys::Reflect::get(&options, &"cache".into())
        .ok()
        .and_then(|val| val.as_string())
        .unwrap_or_else(|| "default".to_string()); // "default" — The browser looks for a matching request in its HTTP cache.

    // credentials
    req_wrapper.credentials = js_sys::Reflect::get(&options, &"credentials".into())
        .ok()
        .and_then(|val| val.as_string())
        .unwrap_or_else(|| "same-origin".to_string()); // "same-origin" — The browser includes credentials in the request if the URL is on the same origin as the calling script.

    // destination
    req_wrapper.destination = js_sys::Reflect::get(&options, &"destination".into())
        .ok()
        .and_then(|val| val.as_string())
        .unwrap_or_else(|| "".to_string()); // "" — The request does not have a specific destination.

    // integrity
    req_wrapper.integrity = js_sys::Reflect::get(&options, &"integrity".into())
        .ok()
        .and_then(|val| val.as_string())
        .unwrap_or_else(|| "".to_string()); // "" — The request does not have an integrity value.

    // is_history_navigation
    req_wrapper.is_history_navigation =
        js_sys::Reflect::get(&options, &"isHistoryNavigation".into())
            .ok()
            .and_then(|val| val.as_bool())
            .unwrap_or(false); // false — The request is not a history navigation.

    // keepalive
    js_sys::Reflect::get(&options, &"keepalive".into())
        .ok()
        .and_then(|val| val.as_bool())
        .map(|keep_alive| req_wrapper.keep_alive = Some(keep_alive));

    // mode
    req_wrapper.mode = match options.get_mode() {
        Some(RequestMode::SameOrigin) => Some(Mode::SameOrigin),
        Some(RequestMode::NoCors) => Some(Mode::NoCors),
        Some(RequestMode::Cors) => Some(Mode::Cors),
        Some(RequestMode::Navigate) => Some(Mode::Navigate),
        _ => Some(Mode::Cors),
    };

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

    // signal
    req_wrapper.signal = options.get_signal();
}
