//! Unit converter plugin

use super::traits::{Plugin, PluginInfo};
use crate::results::Answer;
use regex::Regex;

/// Plugin for converting between units
pub struct UnitConverterPlugin {
    pattern: Regex,
}

impl UnitConverterPlugin {
    pub fn new() -> Self {
        Self {
            // Pattern: "10 km to miles" or "100 usd in eur"
            pattern: Regex::new(r"(?i)^(\d+\.?\d*)\s*([a-zA-Z°]+)\s+(?:to|in|as)\s+([a-zA-Z°]+)$")
                .unwrap(),
        }
    }

    fn convert(
        &self,
        value: f64,
        from: &str,
        to: &str,
    ) -> Option<(f64, &'static str, &'static str)> {
        let from = from.to_lowercase();
        let to = to.to_lowercase();

        // Length conversions
        match (from.as_str(), to.as_str()) {
            // Kilometers <-> Miles
            ("km" | "kilometers" | "kilometer", "mi" | "miles" | "mile") => {
                Some((value * 0.621371, "km", "mi"))
            }
            ("mi" | "miles" | "mile", "km" | "kilometers" | "kilometer") => {
                Some((value * 1.60934, "mi", "km"))
            }
            // Meters <-> Feet
            ("m" | "meters" | "meter", "ft" | "feet" | "foot") => {
                Some((value * 3.28084, "m", "ft"))
            }
            ("ft" | "feet" | "foot", "m" | "meters" | "meter") => Some((value * 0.3048, "ft", "m")),
            // Centimeters <-> Inches
            ("cm" | "centimeters" | "centimeter", "in" | "inches" | "inch") => {
                Some((value * 0.393701, "cm", "in"))
            }
            ("in" | "inches" | "inch", "cm" | "centimeters" | "centimeter") => {
                Some((value * 2.54, "in", "cm"))
            }

            // Weight conversions
            // Kilograms <-> Pounds
            ("kg" | "kilograms" | "kilogram", "lb" | "lbs" | "pounds" | "pound") => {
                Some((value * 2.20462, "kg", "lb"))
            }
            ("lb" | "lbs" | "pounds" | "pound", "kg" | "kilograms" | "kilogram") => {
                Some((value * 0.453592, "lb", "kg"))
            }
            // Grams <-> Ounces
            ("g" | "grams" | "gram", "oz" | "ounces" | "ounce") => {
                Some((value * 0.035274, "g", "oz"))
            }
            ("oz" | "ounces" | "ounce", "g" | "grams" | "gram") => {
                Some((value * 28.3495, "oz", "g"))
            }

            // Temperature conversions
            ("c" | "celsius" | "°c", "f" | "fahrenheit" | "°f") => {
                Some((value * 9.0 / 5.0 + 32.0, "°C", "°F"))
            }
            ("f" | "fahrenheit" | "°f", "c" | "celsius" | "°c") => {
                Some(((value - 32.0) * 5.0 / 9.0, "°F", "°C"))
            }
            ("c" | "celsius" | "°c", "k" | "kelvin") => Some((value + 273.15, "°C", "K")),
            ("k" | "kelvin", "c" | "celsius" | "°c") => Some((value - 273.15, "K", "°C")),

            // Volume conversions
            ("l" | "liters" | "liter" | "litres" | "litre", "gal" | "gallons" | "gallon") => {
                Some((value * 0.264172, "L", "gal"))
            }
            ("gal" | "gallons" | "gallon", "l" | "liters" | "liter" | "litres" | "litre") => {
                Some((value * 3.78541, "gal", "L"))
            }
            ("ml" | "milliliters" | "milliliter", "floz" | "fl oz" | "fluid ounces") => {
                Some((value * 0.033814, "mL", "fl oz"))
            }

            // Area conversions
            ("sqm" | "m2" | "square meters", "sqft" | "ft2" | "square feet") => {
                Some((value * 10.7639, "m²", "ft²"))
            }
            ("sqft" | "ft2" | "square feet", "sqm" | "m2" | "square meters") => {
                Some((value * 0.092903, "ft²", "m²"))
            }

            // Speed conversions
            ("kph" | "km/h" | "kmh", "mph") => Some((value * 0.621371, "km/h", "mph")),
            ("mph", "kph" | "km/h" | "kmh") => Some((value * 1.60934, "mph", "km/h")),

            // Data conversions
            ("kb" | "kilobytes", "mb" | "megabytes") => Some((value / 1024.0, "KB", "MB")),
            ("mb" | "megabytes", "gb" | "gigabytes") => Some((value / 1024.0, "MB", "GB")),
            ("gb" | "gigabytes", "tb" | "terabytes") => Some((value / 1024.0, "GB", "TB")),
            ("mb" | "megabytes", "kb" | "kilobytes") => Some((value * 1024.0, "MB", "KB")),
            ("gb" | "gigabytes", "mb" | "megabytes") => Some((value * 1024.0, "GB", "MB")),
            ("tb" | "terabytes", "gb" | "gigabytes") => Some((value * 1024.0, "TB", "GB")),

            _ => None,
        }
    }
}

impl Default for UnitConverterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for UnitConverterPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "unit_converter".to_string(),
            name: "Unit Converter".to_string(),
            description: "Convert between various units of measurement".to_string(),
            default_on: true,
        }
    }

    fn keywords(&self) -> Vec<&str> {
        vec![]
    }

    fn matches_query(&self, query: &str) -> bool {
        self.pattern.is_match(query.trim())
    }

    fn process(&self, query: &str) -> Option<Answer> {
        let caps = self.pattern.captures(query.trim())?;

        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let from = caps.get(2)?.as_str();
        let to = caps.get(3)?.as_str();

        let (result, from_unit, to_unit) = self.convert(value, from, to)?;

        let answer = format!("{:.4} {} = {:.4} {}", value, from_unit, result, to_unit);

        Some(Answer::new(answer, "unit_converter".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_km_to_miles() {
        let plugin = UnitConverterPlugin::new();
        let result = plugin.process("10 km to miles");
        assert!(result.is_some());
        let answer = result.unwrap();
        assert!(answer.answer.contains("6.2137"));
    }

    #[test]
    fn test_celsius_to_fahrenheit() {
        let plugin = UnitConverterPlugin::new();
        let result = plugin.process("100 c to f");
        assert!(result.is_some());
        let answer = result.unwrap();
        assert!(answer.answer.contains("212"));
    }

    #[test]
    fn test_pattern_matching() {
        let plugin = UnitConverterPlugin::new();
        assert!(plugin.matches_query("10 km to miles"));
        assert!(plugin.matches_query("100 usd in eur"));
        assert!(!plugin.matches_query("hello world"));
    }
}
