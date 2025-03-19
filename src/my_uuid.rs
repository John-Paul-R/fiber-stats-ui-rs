use std::sync::Arc;

use leptos_router::params::{IntoParam, ParamsError};
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct MyUuid(Uuid);

impl MyUuid {
    pub fn to_pretty_string(&self) -> String {
        self.0.hyphenated().to_string()
    }
}

impl IntoParam for MyUuid {
    fn into_param(
        value: Option<&str>,
        name: &str,
    ) -> Result<Self, ParamsError> {
        value
            .map(Uuid::try_parse)
            .map(|uuid| uuid.map(MyUuid))
            .map(|res| res.map_err(|err| ParamsError::Params(Arc::new(err)))) // format!("Failed to parse '{}'", name)
            .unwrap_or_else(|| Err(ParamsError::MissingParam(name.to_string())))
    }
}

// impl IntoView for MyUuid {
//     fn into_view(self) -> View<Self> {
//         view! {
//             <>{self.0.hyphenated().to_string()}</>
//         }
//     }
// }
