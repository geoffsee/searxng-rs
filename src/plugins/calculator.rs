//! Calculator plugin for evaluating mathematical expressions

use super::traits::{Plugin, PluginInfo};
use crate::results::Answer;

/// Calculator plugin that evaluates mathematical expressions
pub struct CalculatorPlugin;

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self
    }

    /// Simple expression evaluator
    fn evaluate(&self, expr: &str) -> Option<f64> {
        // Clean up the expression
        let expr = expr
            .trim()
            .replace(" ", "")
            .replace("×", "*")
            .replace("÷", "/")
            .replace("−", "-")
            .replace(",", "");

        // Try to parse as a simple calculation
        self.parse_expression(&expr)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn parse_expression(&self, expr: &str) -> Option<f64> {
        // Handle parentheses first
        if let Some(start) = expr.rfind('(') {
            if let Some(end) = expr[start..].find(')') {
                let inner = &expr[start + 1..start + end];
                if let Some(result) = self.parse_expression(inner) {
                    let new_expr =
                        format!("{}{}{}", &expr[..start], result, &expr[start + end + 1..]);
                    return self.parse_expression(&new_expr);
                }
            }
            return None;
        }

        // Handle addition and subtraction (lowest precedence)
        if let Some(pos) = expr.rfind(|c| c == '+' || (c == '-' && pos_is_operator(expr, c))) {
            let left = &expr[..pos];
            let right = &expr[pos + 1..];
            let op = expr.chars().nth(pos)?;

            if !left.is_empty() {
                let left_val = self.parse_expression(left)?;
                let right_val = self.parse_expression(right)?;
                return Some(if op == '+' {
                    left_val + right_val
                } else {
                    left_val - right_val
                });
            }
        }

        // Handle multiplication and division
        if let Some(pos) = expr.rfind(['*', '/']) {
            let left = &expr[..pos];
            let right = &expr[pos + 1..];
            let op = expr.chars().nth(pos)?;

            let left_val = self.parse_expression(left)?;
            let right_val = self.parse_expression(right)?;

            return Some(if op == '*' {
                left_val * right_val
            } else {
                if right_val == 0.0 {
                    return None;
                }
                left_val / right_val
            });
        }

        // Handle power
        if let Some(pos) = expr.rfind('^') {
            let left = &expr[..pos];
            let right = &expr[pos + 1..];

            let left_val = self.parse_expression(left)?;
            let right_val = self.parse_expression(right)?;

            return Some(left_val.powf(right_val));
        }

        // Handle constants
        match expr.to_lowercase().as_str() {
            "pi" => return Some(std::f64::consts::PI),
            "e" => return Some(std::f64::consts::E),
            _ => {}
        }

        // Handle functions
        if expr.starts_with("sqrt") {
            let inner = expr.strip_prefix("sqrt")?;
            let val = self.parse_expression(inner)?;
            return Some(val.sqrt());
        }
        if expr.starts_with("sin") {
            let inner = expr.strip_prefix("sin")?;
            let val = self.parse_expression(inner)?;
            return Some(val.sin());
        }
        if expr.starts_with("cos") {
            let inner = expr.strip_prefix("cos")?;
            let val = self.parse_expression(inner)?;
            return Some(val.cos());
        }
        if expr.starts_with("tan") {
            let inner = expr.strip_prefix("tan")?;
            let val = self.parse_expression(inner)?;
            return Some(val.tan());
        }
        if expr.starts_with("log") {
            let inner = expr.strip_prefix("log")?;
            let val = self.parse_expression(inner)?;
            return Some(val.log10());
        }
        if expr.starts_with("ln") {
            let inner = expr.strip_prefix("ln")?;
            let val = self.parse_expression(inner)?;
            return Some(val.ln());
        }

        // Parse as number
        expr.parse().ok()
    }
}

fn pos_is_operator(expr: &str, c: char) -> bool {
    // Check if the character is actually an operator and not a negative sign
    if c != '-' {
        return true;
    }
    // Find position of the character
    if let Some(pos) = expr.rfind('-') {
        if pos == 0 {
            return false; // Negative number at start
        }
        // Check if previous char is an operator
        let prev = expr.chars().nth(pos - 1);
        !matches!(
            prev,
            Some('+') | Some('-') | Some('*') | Some('/') | Some('^') | Some('(')
        )
    } else {
        false
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CalculatorPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            id: "calculator".to_string(),
            name: "Calculator".to_string(),
            description: "Evaluate mathematical expressions".to_string(),
            default_on: true,
        }
    }

    fn keywords(&self) -> Vec<&str> {
        vec!["=", "calc", "calculate"]
    }

    fn matches_query(&self, query: &str) -> bool {
        let q = query.trim();
        // Match expressions that look like calculations
        q.starts_with('=')
            || q.starts_with("calc ")
            || q.starts_with("calculate ")
            || (q.chars().all(|c| {
                c.is_numeric()
                    || c == '+'
                    || c == '-'
                    || c == '*'
                    || c == '/'
                    || c == '^'
                    || c == '('
                    || c == ')'
                    || c == '.'
                    || c == ' '
            }) && q.contains(['+', '-', '*', '/', '^']))
    }

    fn process(&self, query: &str) -> Option<Answer> {
        let expr = query
            .trim()
            .trim_start_matches('=')
            .trim_start_matches("calc ")
            .trim_start_matches("calculate ")
            .trim();

        self.evaluate(expr).map(|result| {
            let formatted = if result.fract() == 0.0 {
                format!("{} = {}", expr, result as i64)
            } else {
                format!("{} = {:.6}", expr, result)
            };
            Answer::new(formatted, "calculator".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let calc = CalculatorPlugin::new();
        assert_eq!(calc.evaluate("2+2"), Some(4.0));
        assert_eq!(calc.evaluate("10-3"), Some(7.0));
        assert_eq!(calc.evaluate("5*4"), Some(20.0));
        assert_eq!(calc.evaluate("15/3"), Some(5.0));
    }

    #[test]
    fn test_complex_expressions() {
        let calc = CalculatorPlugin::new();
        assert_eq!(calc.evaluate("2+3*4"), Some(14.0));
        assert_eq!(calc.evaluate("2^3"), Some(8.0));
    }
}
