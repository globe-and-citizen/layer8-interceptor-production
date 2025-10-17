use serde::{Deserialize, Serialize};
use web_sys::{ReferrerPolicy, RequestMode};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum L8RequestMode {
    // Disallows cross-origin requests. If a request is made to another origin with this mode set, the result is an error.
    SameOrigin = 0,
    // Disables CORS for cross-origin requests. The response is opaque, meaning that its headers and body are not available to JavaScript.
    NoCors = 1,
    // If the request is cross-origin then it will use the Cross-Origin Resource Sharing (CORS) mechanism.
    // Using the Request() constructor, the value of the mode property for that Request is set to cors.
    Cors = 2,
    // A mode for supporting navigation. The navigate value is intended to be used only by HTML navigation.
    // A navigate request is created only while navigating between documents.
    Navigate = 3,
}

impl L8RequestMode {
    pub fn from_request_options(options: &web_sys::RequestInit) -> Option<Self> {
        match options.get_mode() {
            Some(RequestMode::SameOrigin) => Some(L8RequestMode::SameOrigin),
            Some(RequestMode::NoCors) => Some(L8RequestMode::NoCors),
            Some(RequestMode::Cors) => Some(L8RequestMode::Cors),
            Some(RequestMode::Navigate) => Some(L8RequestMode::Navigate),
            _ => Some(L8RequestMode::Cors),
        }
    }
}

pub fn get_request_referer_policy(options: &web_sys::RequestInit) -> &str {
    if let Some(referrer_policy) = options.get_referrer_policy() {
        return match referrer_policy {
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
    return "";
}
