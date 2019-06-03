use actix_web::error;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::Result;
use biscuit::jws;
use biscuit::Empty;

#[derive(Deserialize, Debug, Clone)]
pub struct Scope {
    pub scope: String,
}

#[derive(Serialize, Deserialize)]
pub struct Groups {
    #[serde(rename = "https://sso.mozilla.com/claim/groups")]
    groups: Vec<String>,
}

impl<S> FromRequest<S> for Scope {
    type Config = ();
    type Result = Result<Self, error::Error>;

    #[inline]
    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        if let Some(token) = req
            .headers()
            .get("x-auth-token")
            .and_then(|h| h.to_str().ok())
        {
            let compact: jws::Compact<biscuit::ClaimsSet<Groups>, Empty> =
                jws::Compact::new_encoded(token);
            if let Ok(claimset) = compact.unverified_payload() {
                if claimset.private.groups.contains(&String::from("team_moco"))
                    || claimset.private.groups.contains(&String::from("team_moco"))
                {
                    info!("scope → staff");
                    return Ok(Scope {
                        scope: String::from("staff"),
                    });
                }
                if claimset
                    .private
                    .groups
                    .contains(&String::from("mozilliansorg_nda"))
                {
                    return Ok(Scope {
                        scope: String::from("ndaed"),
                    });
                }
                return Ok(Scope {
                    scope: String::from("authenticated"),
                });
            }
        }
        Err(error::ErrorForbidden("no scope"))
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserId {
    pub user_id: String,
}

impl<S> FromRequest<S> for UserId {
    type Config = ();
    type Result = Result<Self, error::Error>;

    #[cfg(not(feature = "nouid"))]
    #[inline]
    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        let user_id = req
            .headers()
            .get("x-forwarded-user-subject")
            .or_else(|| req.headers().get("x-auth-subject"));
        user_id
            .and_then(|id| id.to_str().ok())
            .map(|id| UserId {
                user_id: id.to_owned(),
            })
            .ok_or_else(|| error::ErrorForbidden("no user_id"))
    }

    #[cfg(feature = "nouid")]
    #[inline]
    fn from_request(_: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        use std::env::var;
        let user_id = var("DPF_USER_ID").unwrap();
        Ok(UserId { user_id })
    }
}