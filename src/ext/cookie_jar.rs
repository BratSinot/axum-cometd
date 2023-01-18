use crate::types::{CookieId, BAYEUX_BROWSER};
use axum_extra::extract::{cookie::Cookie, CookieJar};

pub(crate) trait CookieJarExt {
    fn get_cookie_id(&self) -> Option<CookieId>;
}

impl CookieJarExt for CookieJar {
    fn get_cookie_id(&self) -> Option<CookieId> {
        self.get(BAYEUX_BROWSER)
            .map(Cookie::value)
            .map(CookieId::parse)
            .and_then(Result::ok)
    }
}
