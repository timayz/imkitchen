use std::collections::HashMap;

pub(crate) mod filters {
    #[askama::filter_fn]
    pub fn t(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
        let lang = askama::get_value::<String>(values, "lang")
            .expect("Unable to get lang from askama::get_value");

        Ok(rust_i18n::t!(value, locale = lang).to_string())
    }
}

pub struct Template {
    lang: String,
}

impl Template {
    pub fn new(lang: impl Into<String>) -> Self {
        let lang = lang.into();

        Self { lang }
    }

    fn render_with_values<T: askama::Template>(
        &self,
        template: T,
    ) -> Result<String, askama::Error> {
        let mut values: HashMap<&str, Box<dyn std::any::Any>> = HashMap::new();
        values.insert("lang", Box::new(self.lang.to_owned()));

        #[cfg(debug_assertions)]
        {
            values.insert("is_dev", Box::new(true));
        }
        #[cfg(not(debug_assertions))]
        {
            values.insert("is_dev", Box::new(false));
        }

        template.render_with_values(&values)
    }

    pub fn to_string<T: askama::Template>(&self, template: T) -> String {
        match self.render_with_values(template) {
            Ok(html) => html,
            Err(err) => format!("Failed to render template. Error: {err}"),
        }
    }
}
