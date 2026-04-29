use bitcode::{Decode, Encode};
use serde::Deserialize;

#[derive(Default, Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct PaymentDetails {
    pub plan: String,
    pub price: u32,
    pub tax: u32,
    pub tax_rate: Option<f64>,
}

#[derive(Default, Encode, Decode, Clone, Deserialize, Debug, PartialEq)]
pub struct Address {
    /// City, district, suburb, town, or village.
    pub city: Option<String>,
    /// Two-letter country code ([ISO 3166-1 alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)).
    pub country: Option<String>,
    /// Address line 1, such as the street, PO Box, or company name.
    pub line1: Option<String>,
    /// Address line 2, such as the apartment, suite, unit, or building.
    pub line2: Option<String>,
    /// ZIP or postal code.
    pub postal_code: Option<String>,
    /// State, county, province, or region ([ISO 3166-2](https://en.wikipedia.org/wiki/ISO_3166-2)).
    pub state: Option<String>,
}

impl From<stripe_shared::Address> for Address {
    fn from(value: stripe_shared::Address) -> Self {
        Self {
            city: value.city,
            country: value.country,
            line1: value.line1,
            line2: value.line2,
            postal_code: value.postal_code,
            state: value.state,
        }
    }
}

impl Address {
    /// Formats the address as it should appear on an invoice/envelope,
    /// following the recipient country's conventions.
    pub fn format_for_invoice(&self) -> Vec<String> {
        let country = self.country.as_deref().unwrap_or("").to_uppercase();
        let mut lines: Vec<String> = Vec::new();

        match country.as_str() {
            // France: number before street, POSTAL_CODE CITY (city uppercase), no state
            "FR" => {
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                } // apt/building first
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                } // street
                let city_line = match (&self.postal_code, &self.city) {
                    (Some(p), Some(c)) => format!("{} {}", p, c.to_uppercase()),
                    (Some(p), None) => p.clone(),
                    (None, Some(c)) => c.to_uppercase(),
                    _ => String::new(),
                };
                if !city_line.is_empty() {
                    lines.push(city_line);
                }
                lines.push("FRANCE".to_string());
            }

            // United States: "City, ST ZIP", country optional domestically
            "US" => {
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                }
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                }
                let csz = match (&self.city, &self.state, &self.postal_code) {
                    (Some(c), Some(s), Some(z)) => format!("{}, {} {}", c, s, z),
                    (Some(c), Some(s), None) => format!("{}, {}", c, s),
                    (Some(c), None, Some(z)) => format!("{} {}", c, z),
                    (Some(c), _, _) => c.clone(),
                    _ => String::new(),
                };
                if !csz.is_empty() {
                    lines.push(csz);
                }
                lines.push("UNITED STATES".to_string());
            }

            // United Kingdom: city on its own line, postcode on its own line, both uppercase
            "GB" | "UK" => {
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                }
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                }
                if let Some(c) = &self.city {
                    lines.push(c.to_uppercase());
                }
                if let Some(p) = &self.postal_code {
                    lines.push(p.to_uppercase());
                }
                lines.push("UNITED KINGDOM".to_string());
            }

            // Germany / Austria / Switzerland / Netherlands / Belgium / Spain / Italy:
            // street, then "POSTAL CITY", no state
            "DE" | "AT" | "CH" | "NL" | "BE" | "ES" | "IT" | "PT" | "PL" | "SE" | "DK" | "NO"
            | "FI" => {
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                }
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                }
                let city_line = match (&self.postal_code, &self.city) {
                    (Some(p), Some(c)) => format!("{} {}", p, c),
                    (Some(p), None) => p.clone(),
                    (None, Some(c)) => c.clone(),
                    _ => String::new(),
                };
                if !city_line.is_empty() {
                    lines.push(city_line);
                }
                lines.push(country_name(&country).to_uppercase());
            }

            // Canada / Australia: "City ST PostalCode"
            "CA" | "AU" => {
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                }
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                }
                let csz = match (&self.city, &self.state, &self.postal_code) {
                    (Some(c), Some(s), Some(z)) => format!("{} {} {}", c, s, z),
                    (Some(c), None, Some(z)) => format!("{} {}", c, z),
                    (Some(c), Some(s), None) => format!("{} {}", c, s),
                    (Some(c), _, _) => c.clone(),
                    _ => String::new(),
                };
                if !csz.is_empty() {
                    lines.push(csz);
                }
                lines.push(country_name(&country).to_uppercase());
            }

            // Fallback: generic international layout
            _ => {
                if let Some(l1) = &self.line1 {
                    lines.push(l1.clone());
                }
                if let Some(l2) = &self.line2 {
                    lines.push(l2.clone());
                }
                let city_line = match (&self.postal_code, &self.city, &self.state) {
                    (Some(p), Some(c), Some(s)) => format!("{} {}, {}", p, c, s),
                    (Some(p), Some(c), None) => format!("{} {}", p, c),
                    (None, Some(c), Some(s)) => format!("{}, {}", c, s),
                    (None, Some(c), None) => c.clone(),
                    (Some(p), None, _) => p.clone(),
                    _ => String::new(),
                };
                if !city_line.is_empty() {
                    lines.push(city_line);
                }
                if !country.is_empty() {
                    lines.push(country_name(&country).to_uppercase());
                }
            }
        }

        lines.retain(|l| !l.trim().is_empty());

        lines
    }
}

fn country_name(code: &str) -> &'static str {
    match code {
        "FR" => "France",
        "US" => "United States",
        "GB" | "UK" => "United Kingdom",
        "DE" => "Germany",
        "AT" => "Austria",
        "CH" => "Switzerland",
        "NL" => "Netherlands",
        "BE" => "Belgium",
        "ES" => "Spain",
        "IT" => "Italy",
        "PT" => "Portugal",
        "PL" => "Poland",
        "SE" => "Sweden",
        "DK" => "Denmark",
        "NO" => "Norway",
        "FI" => "Finland",
        "CA" => "Canada",
        "AU" => "Australia",
        _ => "",
    }
}

#[evento::aggregator]
pub enum Subscription {
    LifePremiumToggled {
        expire_at: u64,
    },
    StripeCustomerCreated {
        id: String,
    },
    StripePaymentIntentCreated {
        id: String,
        email: String,
        details: PaymentDetails,
    },
    StripeSetupIntentCreated {
        id: String,
    },
    StripePaymentIntentSucceeded {
        id: String,
        payment_method_id: String,
        name: String,
        email: String,
        address: Address,
        paid_at: u64,
        expire_at: u64,
        details: PaymentDetails,
    },
    StripeSetupIntentSucceeded {
        id: String,
        payment_method_id: String,
        name: Option<String>,
        address: Option<Address>,
    },
    Cancelled,
}
