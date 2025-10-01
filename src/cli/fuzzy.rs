use std::collections::HashMap;

/// Fuzzy matching utility for CLI commands and suggestions
pub struct FuzzyMatcher {
    commands: Vec<String>,
    aliases: HashMap<String, String>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            commands: Vec::new(),
            aliases: HashMap::new(),
        };

        matcher.initialize_commands();
        matcher.initialize_aliases();
        matcher
    }

    fn initialize_commands(&mut self) {
        // Main commands
        self.commands.extend(vec![
            "run".to_string(),
            "batch".to_string(),
            "serve".to_string(),
            "models".to_string(),
            "metrics".to_string(),
            "bench".to_string(),
            "validate".to_string(),
            "config".to_string(),
            "cache".to_string(),
            "convert".to_string(),
            "marketplace".to_string(),
            "package".to_string(),
            "install".to_string(),
            "remove".to_string(),
            "search".to_string(),
            "list".to_string(),
            "repo".to_string(),
            "tui".to_string(),
        ]);

        // Package subcommands
        self.commands.extend(vec![
            "package install".to_string(),
            "package remove".to_string(),
            "package search".to_string(),
            "package list".to_string(),
            "package update".to_string(),
            "package upgrade".to_string(),
            "package autoremove".to_string(),
            "package clean".to_string(),
            "package info".to_string(),
            "package depends".to_string(),
            "package check".to_string(),
        ]);

        // Repository subcommands
        self.commands.extend(vec![
            "repo add".to_string(),
            "repo remove".to_string(),
            "repo list".to_string(),
            "repo update".to_string(),
            "repo info".to_string(),
            "repo test".to_string(),
            "repo toggle".to_string(),
            "repo priority".to_string(),
            "repo clean".to_string(),
        ]);

        // Marketplace subcommands
        self.commands.extend(vec![
            "marketplace search".to_string(),
            "marketplace download".to_string(),
            "marketplace install".to_string(),
            "marketplace list".to_string(),
            "marketplace updates".to_string(),
        ]);
    }

    fn initialize_aliases(&mut self) {
        // Common typos and alternatives
        self.aliases
            .insert("instal".to_string(), "install".to_string());
        self.aliases
            .insert("instll".to_string(), "install".to_string());
        self.aliases
            .insert("isntall".to_string(), "install".to_string());
        self.aliases
            .insert("add".to_string(), "install".to_string());
        self.aliases
            .insert("get".to_string(), "install".to_string());

        self.aliases.insert("rm".to_string(), "remove".to_string());
        self.aliases.insert("del".to_string(), "remove".to_string());
        self.aliases
            .insert("delete".to_string(), "remove".to_string());
        self.aliases
            .insert("uninstall".to_string(), "remove".to_string());

        self.aliases
            .insert("find".to_string(), "search".to_string());
        self.aliases
            .insert("query".to_string(), "search".to_string());
        self.aliases
            .insert("lookup".to_string(), "search".to_string());

        self.aliases.insert("ls".to_string(), "list".to_string());
        self.aliases.insert("show".to_string(), "list".to_string());
        self.aliases
            .insert("display".to_string(), "list".to_string());

        self.aliases
            .insert("update".to_string(), "package update".to_string());
        self.aliases
            .insert("upgrade".to_string(), "package upgrade".to_string());
        self.aliases
            .insert("autoremove".to_string(), "package autoremove".to_string());
        self.aliases
            .insert("autoclean".to_string(), "package clean".to_string());

        self.aliases
            .insert("repository".to_string(), "repo".to_string());
        self.aliases
            .insert("repositories".to_string(), "repo".to_string());
        self.aliases.insert("repos".to_string(), "repo".to_string());

        self.aliases
            .insert("market".to_string(), "marketplace".to_string());
        self.aliases
            .insert("store".to_string(), "marketplace".to_string());
        self.aliases
            .insert("registry".to_string(), "marketplace".to_string());

        self.aliases
            .insert("pkg".to_string(), "package".to_string());
        self.aliases
            .insert("packages".to_string(), "package".to_string());

        self.aliases.insert("cfg".to_string(), "config".to_string());
        self.aliases
            .insert("configuration".to_string(), "config".to_string());
        self.aliases
            .insert("settings".to_string(), "config".to_string());

        self.aliases.insert("ui".to_string(), "tui".to_string());
        self.aliases
            .insert("terminal".to_string(), "tui".to_string());
        self.aliases
            .insert("interface".to_string(), "tui".to_string());
    }

    /// Find the best command suggestion for a given input
    pub fn suggest_command(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();

        // Check exact aliases first
        if let Some(alias) = self.aliases.get(&input_lower) {
            return Some(alias.clone());
        }

        // Find best fuzzy match
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for command in &self.commands {
            let distance = levenshtein_distance(&input_lower, &command.to_lowercase());

            // Only suggest if it's a reasonable match (within 3 edits for longer commands)
            let max_distance = if command.len() > 6 { 3 } else { 2 };

            if distance <= max_distance && distance < best_distance {
                best_distance = distance;
                best_match = Some(command.clone());
            }
        }

        // Also check if input is a prefix of any command
        if best_match.is_none() {
            for command in &self.commands {
                if command.to_lowercase().starts_with(&input_lower) && input.len() >= 3 {
                    return Some(command.clone());
                }
            }
        }

        best_match
    }

    /// Get multiple suggestions for a command
    pub fn suggest_multiple(&self, input: &str, limit: usize) -> Vec<String> {
        let input_lower = input.to_lowercase();
        let mut suggestions = Vec::new();

        // Check aliases first
        if let Some(alias) = self.aliases.get(&input_lower) {
            suggestions.push(alias.clone());
        }

        // Get fuzzy matches
        let mut matches: Vec<(String, usize)> = self
            .commands
            .iter()
            .map(|cmd| {
                let distance = levenshtein_distance(&input_lower, &cmd.to_lowercase());
                (cmd.clone(), distance)
            })
            .filter(|(_, distance)| *distance <= 3)
            .collect();

        // Sort by distance
        matches.sort_by_key(|(_, distance)| *distance);

        // Add unique suggestions
        for (command, _) in matches.into_iter().take(limit) {
            if !suggestions.contains(&command) {
                suggestions.push(command);
            }
        }

        // Add prefix matches if we don't have enough
        if suggestions.len() < limit {
            for command in &self.commands {
                if command.to_lowercase().starts_with(&input_lower)
                    && input.len() >= 2
                    && !suggestions.contains(command)
                {
                    suggestions.push(command.clone());
                    if suggestions.len() >= limit {
                        break;
                    }
                }
            }
        }

        suggestions
    }

    /// Check if a command exists or can be corrected
    pub fn validate_command(&self, input: &str) -> CommandValidation {
        let input_lower = input.to_lowercase();

        // Check if command exists exactly
        if self.commands.contains(&input.to_string()) {
            return CommandValidation::Valid;
        }

        // Check aliases
        if let Some(alias) = self.aliases.get(&input_lower) {
            return CommandValidation::Alias(alias.clone());
        }

        // Get suggestion
        if let Some(suggestion) = self.suggest_command(input) {
            CommandValidation::Suggestion(suggestion)
        } else {
            CommandValidation::Invalid
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandValidation {
    Valid,
    Alias(String),
    Suggestion(String),
    Invalid,
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("cat", "cat"), 0);
        assert_eq!(levenshtein_distance("cat", "bat"), 1);
        assert_eq!(levenshtein_distance("install", "instal"), 1);
        assert_eq!(levenshtein_distance("search", "serch"), 1);
    }

    #[test]
    fn test_command_suggestions() {
        let matcher = FuzzyMatcher::new();

        assert_eq!(
            matcher.suggest_command("instal"),
            Some("install".to_string())
        );
        assert_eq!(matcher.suggest_command("serch"), Some("search".to_string()));
        assert_eq!(matcher.suggest_command("rm"), Some("remove".to_string()));
        assert_eq!(matcher.suggest_command("ls"), Some("list".to_string()));
    }

    #[test]
    fn test_command_validation() {
        let matcher = FuzzyMatcher::new();

        assert_eq!(
            matcher.validate_command("install"),
            CommandValidation::Valid
        );
        assert_eq!(
            matcher.validate_command("rm"),
            CommandValidation::Alias("remove".to_string())
        );
        assert_eq!(
            matcher.validate_command("instal"),
            CommandValidation::Suggestion("install".to_string())
        );
        assert_eq!(
            matcher.validate_command("xyz123"),
            CommandValidation::Invalid
        );
    }

    #[test]
    fn test_multiple_suggestions() {
        let matcher = FuzzyMatcher::new();

        let suggestions = matcher.suggest_multiple("pac", 3);
        assert!(suggestions.contains(&"package".to_string()));

        let suggestions = matcher.suggest_multiple("rep", 3);
        assert!(suggestions.contains(&"repo".to_string()));
    }
}
