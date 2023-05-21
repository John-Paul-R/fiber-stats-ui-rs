use std::num::ParseIntError;
use std::string::ParseError;
use std::sync::Arc;

use leptos::*;
use leptos_router::{IntoParam, Params, ParamsError, use_params, use_params_map};
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct MyUuid(Uuid);

impl MyUuid {
    pub fn get_uuid(self) -> Uuid {
        return self.0;
    }

    pub fn to_pretty_string(self) -> String {
        return self.0.hyphenated().to_string();
    }
}

impl IntoParam for MyUuid {
    fn into_param(value: Option<&str>, name: &str) -> Result<Self, ParamsError> {
        return value
            .map(Uuid::try_parse)
            .map(|uuid| uuid.map(MyUuid))
            .map(|res| res.map_err(|err| ParamsError::Params(Arc::new(err))))//format!("Failed to parse '{}'", name)
            .unwrap_or_else(|| Err(ParamsError::MissingParam(name.to_string())))
        ;
    }
}

impl IntoView for MyUuid {
    fn into_view(self, cx: Scope) -> View {
        (view! { cx,
            <>{self.0.hyphenated().to_string()}</>
        })
            .into_view(cx)
    }
}
