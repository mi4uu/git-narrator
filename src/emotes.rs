//! Commit message categorization and emote mapping functionality
//! 
//! This module provides functionality to analyze commit messages and automatically
//! categorize them, then add appropriate UTF8 emotes to enhance visual appeal.

use std::collections::HashMap;

/// Represents different categories of commits with their associated emotes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommitCategory {
    Fix,        // 🐛 Bug fixes
    Feat,       // ✨ New features
    Chore,      // 🧹 Maintenance tasks
    Docs,       // 📚 Documentation
    Style,      // 💄 Formatting/styling
    Refactor,   // ♻️ Code restructuring
    Test,       // 🧪 Testing
    Perf,       // ⚡ Performance improvements
    Build,      // 🔧 Build system
    Ci,         // 👷 Continuous integration
    Deploy,     // 🚀 Deployment
    Security,   // 🔒 Security fixes
    Deps,       // 📦 Dependency updates
    Revert,     // ⏪ Reverting changes
    Config,     // ⚙️ Configuration changes
    Init,       // 🎉 Initial commit
    Wip,        // 🚧 Work in progress
    Hotfix,     // 🚨 Critical hotfix
    Release,    // 🏷️ Release/version tags
    Merge,      // 🔀 Merge commits
    Unknown,    // ❓ Uncategorized
}

impl CommitCategory {
    /// Get the UTF8 emote for this commit category
    pub fn emote(&self) -> &'static str {
        match self {
            CommitCategory::Fix => "🐛",
            CommitCategory::Feat => "✨",
            CommitCategory::Chore => "🧹",
            CommitCategory::Docs => "📚",
            CommitCategory::Style => "💄",
            CommitCategory::Refactor => "♻️",
            CommitCategory::Test => "🧪",
            CommitCategory::Perf => "⚡",
            CommitCategory::Build => "🔧",
            CommitCategory::Ci => "👷",
            CommitCategory::Deploy => "🚀",
            CommitCategory::Security => "🔒",
            CommitCategory::Deps => "📦",
            CommitCategory::Revert => "⏪",
            CommitCategory::Config => "⚙️",
            CommitCategory::Init => "🎉",
            CommitCategory::Wip => "🚧",
            CommitCategory::Hotfix => "🚨",
            CommitCategory::Release => "🏷️",
            CommitCategory::Merge => "🔀",
            CommitCategory::Unknown => "❓",
        }
    }

    /// Get a human-readable description of this category
    pub fn description(&self) -> &'static str {
        match self {
            CommitCategory::Fix => "Bug fixes",
            CommitCategory::Feat => "New features",
            CommitCategory::Chore => "Maintenance tasks",
            CommitCategory::Docs => "Documentation",
            CommitCategory::Style => "Formatting/styling",
            CommitCategory::Refactor => "Code restructuring",
            CommitCategory::Test => "Testing",
            CommitCategory::Perf => "Performance improvements",
            CommitCategory::Build => "Build system",
            CommitCategory::Ci => "Continuous integration",
            CommitCategory::Deploy => "Deployment",
            CommitCategory::Security => "Security fixes",
            CommitCategory::Deps => "Dependency updates",
            CommitCategory::Revert => "Reverting changes",
            CommitCategory::Config => "Configuration changes",
            CommitCategory::Init => "Initial commit",
            CommitCategory::Wip => "Work in progress",
            CommitCategory::Hotfix => "Critical hotfix",
            CommitCategory::Release => "Release/version tags",
            CommitCategory::Merge => "Merge commits",
            CommitCategory::Unknown => "Uncategorized",
        }
    }
}

/// Analyzes a commit message and determines its category
pub fn categorize_commit_message(message: &str) -> CommitCategory {
    let message_lower = message.to_lowercase();
    let first_line = message.lines().next().unwrap_or("").to_lowercase();

    // Check for conventional commit format first (type: description)
    if let Some(category) = parse_conventional_commit(&first_line) {
        return category;
    }

    // Check for common keywords and patterns
    if let Some(category) = analyze_keywords(&message_lower) {
        return category;
    }

    // Check for file patterns and context clues
    if let Some(category) = analyze_context(&message_lower) {
        return category;
    }

    // Default to unknown if no pattern matches
    CommitCategory::Unknown
}

/// Parse conventional commit format (type: description or type(scope): description)
fn parse_conventional_commit(first_line: &str) -> Option<CommitCategory> {
    // Match patterns like "feat:", "fix(auth):", "docs(readme):", etc.
    let conventional_patterns = [
        ("feat", CommitCategory::Feat),
        ("feature", CommitCategory::Feat),
        ("fix", CommitCategory::Fix),
        ("bugfix", CommitCategory::Fix),
        ("bug", CommitCategory::Fix),
        ("docs", CommitCategory::Docs),
        ("doc", CommitCategory::Docs),
        ("documentation", CommitCategory::Docs),
        ("style", CommitCategory::Style),
        ("refactor", CommitCategory::Refactor),
        ("refact", CommitCategory::Refactor),
        ("test", CommitCategory::Test),
        ("tests", CommitCategory::Test),
        ("testing", CommitCategory::Test),
        ("perf", CommitCategory::Perf),
        ("performance", CommitCategory::Perf),
        ("build", CommitCategory::Build),
        ("ci", CommitCategory::Ci),
        ("chore", CommitCategory::Chore),
        ("revert", CommitCategory::Revert),
        ("deploy", CommitCategory::Deploy),
        ("deployment", CommitCategory::Deploy),
        ("security", CommitCategory::Security),
        ("sec", CommitCategory::Security),
        ("deps", CommitCategory::Deps),
        ("dependencies", CommitCategory::Deps),
        ("dependency", CommitCategory::Deps),
        ("config", CommitCategory::Config),
        ("configuration", CommitCategory::Config),
        ("init", CommitCategory::Init),
        ("initial", CommitCategory::Init),
        ("wip", CommitCategory::Wip),
        ("hotfix", CommitCategory::Hotfix),
        ("release", CommitCategory::Release),
        ("version", CommitCategory::Release),
        ("merge", CommitCategory::Merge),
    ];

    for (pattern, category) in &conventional_patterns {
        // Check for exact conventional format: "type:" or "type(scope):"
        if first_line.starts_with(&format!("{}:", pattern)) ||
           first_line.contains(&format!("{}(", pattern)) && first_line.contains("):") {
            return Some(category.clone());
        }
    }

    None
}

/// Analyze keywords in the commit message for categorization
fn analyze_keywords(message: &str) -> Option<CommitCategory> {
    let keyword_patterns = create_keyword_patterns();

    // Check for more specific patterns first (longer/more specific keywords)
    let mut matches = Vec::new();
    
    for (category, keywords) in keyword_patterns {
        for keyword in keywords {
            if message.contains(keyword) {
                matches.push((category.clone(), keyword.len()));
            }
        }
    }
    
    // Sort by keyword length (longer = more specific) and return the most specific match
    if !matches.is_empty() {
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        return Some(matches[0].0.clone());
    }

    None
}

/// Create keyword patterns for different commit categories
fn create_keyword_patterns() -> HashMap<CommitCategory, Vec<&'static str>> {
    let mut patterns = HashMap::new();

    patterns.insert(CommitCategory::Fix, vec![
        "fix", "bug", "error", "issue", "problem", "resolve", "correct",
        "patch", "repair", "debug", "crash", "exception", "broken",
        "regression", "hotfix", "critical", "urgent"
    ]);

    patterns.insert(CommitCategory::Feat, vec![
        "add", "new", "feature", "implement", "create", "introduce",
        "support", "enable", "allow", "enhance", "extend", "expand"
    ]);

    patterns.insert(CommitCategory::Docs, vec![
        "update readme", "readme", "documentation", "docs", "comment", "comments",
        "guide", "tutorial", "example", "examples", "changelog",
        "license", "contributing", "api doc", "docstring"
    ]);

    patterns.insert(CommitCategory::Style, vec![
        "format", "formatting", "style", "styling", "indent", "whitespace",
        "lint", "linting", "prettier", "eslint", "code style", "cleanup",
        "cosmetic", "appearance"
    ]);

    patterns.insert(CommitCategory::Refactor, vec![
        "refactor", "restructure", "reorganize", "simplify", "clean up",
        "extract", "rename", "move", "split", "combine", "optimize structure"
    ]);

    patterns.insert(CommitCategory::Test, vec![
        "test", "tests", "testing", "spec", "specs", "unit test",
        "integration test", "e2e", "coverage", "mock", "fixture"
    ]);

    patterns.insert(CommitCategory::Perf, vec![
        "performance", "optimize", "speed", "faster", "efficiency",
        "cache", "caching", "memory", "cpu", "benchmark", "profiling"
    ]);

    patterns.insert(CommitCategory::Build, vec![
        "build", "compile", "webpack", "rollup", "babel", "typescript",
        "makefile", "cmake", "gradle", "maven", "npm script", "yarn"
    ]);

    patterns.insert(CommitCategory::Ci, vec![
        "ci", "continuous integration", "github actions", "travis",
        "jenkins", "pipeline", "workflow", "automation", "deploy script"
    ]);

    patterns.insert(CommitCategory::Chore, vec![
        "chore", "maintenance", "upgrade", "bump", "cleanup",
        "housekeeping", "misc", "miscellaneous", "routine"
    ]);

    patterns.insert(CommitCategory::Security, vec![
        "security vulnerability", "security fix", "security", "vulnerability", "exploit", "xss", "csrf", "injection",
        "authorization", "permission", "sanitize", "escape"
    ]);

    patterns.insert(CommitCategory::Deps, vec![
        "package dependencies", "update dependencies", "dependency", "dependencies",
        "package", "packages", "npm", "yarn", "pip", "cargo", "gem", "composer",
        "requirements", "lock file", "bump version", "upgrade packages"
    ]);

    patterns.insert(CommitCategory::Config, vec![
        "config", "configuration", "settings", "environment", "env",
        "dotenv", "properties", "yaml", "json", "toml", "ini"
    ]);

    patterns.insert(CommitCategory::Deploy, vec![
        "deploy", "deployment", "release", "publish", "production",
        "staging", "docker", "kubernetes", "helm", "terraform"
    ]);

    patterns.insert(CommitCategory::Init, vec![
        "initial commit", "first commit", "initialize", "setup", "scaffold",
        "bootstrap", "project setup", "initial"
    ]);

    patterns.insert(CommitCategory::Wip, vec![
        "wip", "work in progress", "todo", "fixme", "temporary", "draft"
    ]);

    patterns.insert(CommitCategory::Merge, vec![
        "merge", "pull request", "pr", "branch", "conflict resolution"
    ]);

    patterns.insert(CommitCategory::Revert, vec![
        "revert", "rollback", "undo", "back out", "reverse"
    ]);

    patterns
}

/// Analyze context clues in the commit message
fn analyze_context(message: &str) -> Option<CommitCategory> {
    // Check for file extensions and paths that might indicate category
    if message.contains(".md") || message.contains("readme") || message.contains("doc/") {
        return Some(CommitCategory::Docs);
    }

    if message.contains("package.json") || message.contains("cargo.toml") || 
       message.contains("requirements.txt") || message.contains("gemfile") {
        return Some(CommitCategory::Deps);
    }

    if message.contains(".yml") || message.contains(".yaml") || 
       message.contains("config") || message.contains(".env") {
        return Some(CommitCategory::Config);
    }

    if message.contains("dockerfile") || message.contains("docker-compose") ||
       message.contains(".github/workflows") || message.contains("ci/") {
        return Some(CommitCategory::Ci);
    }

    if message.contains("test/") || message.contains("spec/") || 
       message.contains("__tests__") || message.contains(".test.") {
        return Some(CommitCategory::Test);
    }

    None
}

/// Add emote to a commit message
pub fn add_emote_to_commit_message(message: &str, category: CommitCategory) -> String {
    let emote = category.emote();
    let trimmed_message = message.trim();
    
    // Check if the message already starts with an emote
    if starts_with_emote(trimmed_message) {
        return trimmed_message.to_string();
    }
    
    // Add emote at the beginning
    format!("{} {}", emote, trimmed_message)
}

/// Check if a message already starts with an emote
fn starts_with_emote(message: &str) -> bool {
    if message.is_empty() {
        return false;
    }
    
    // Get the first character and check if it's an emote
    let first_char = message.chars().next().unwrap();
    
    // Common emote ranges in Unicode
    // This is a simplified check - emotes are scattered across multiple Unicode blocks
    matches!(first_char,
        '\u{1F300}'..='\u{1F9FF}' |  // Miscellaneous Symbols and Pictographs, Emoticons, etc.
        '\u{2600}'..='\u{26FF}' |    // Miscellaneous Symbols
        '\u{2700}'..='\u{27BF}' |    // Dingbats
        '\u{1F1E0}'..='\u{1F1FF}'    // Regional Indicator Symbols
    )
}

/// Process a commit message by categorizing it and adding appropriate emote
pub fn process_commit_message(message: &str) -> String {
    let category = categorize_commit_message(message);
    add_emote_to_commit_message(message, category)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conventional_commit_parsing() {
        assert_eq!(categorize_commit_message("feat: add new user authentication"), CommitCategory::Feat);
        assert_eq!(categorize_commit_message("fix: resolve login bug"), CommitCategory::Fix);
        assert_eq!(categorize_commit_message("docs: update README"), CommitCategory::Docs);
        assert_eq!(categorize_commit_message("style: format code"), CommitCategory::Style);
        assert_eq!(categorize_commit_message("refactor: simplify auth logic"), CommitCategory::Refactor);
        assert_eq!(categorize_commit_message("test: add unit tests"), CommitCategory::Test);
        assert_eq!(categorize_commit_message("perf: optimize database queries"), CommitCategory::Perf);
        assert_eq!(categorize_commit_message("build: update webpack config"), CommitCategory::Build);
        assert_eq!(categorize_commit_message("ci: add GitHub Actions"), CommitCategory::Ci);
        assert_eq!(categorize_commit_message("chore: update dependencies"), CommitCategory::Chore);
    }

    #[test]
    fn test_conventional_commit_with_scope() {
        assert_eq!(categorize_commit_message("feat(auth): add OAuth support"), CommitCategory::Feat);
        assert_eq!(categorize_commit_message("fix(ui): resolve button styling"), CommitCategory::Fix);
        assert_eq!(categorize_commit_message("docs(api): update endpoint documentation"), CommitCategory::Docs);
    }

    #[test]
    fn test_keyword_analysis() {
        assert_eq!(categorize_commit_message("Add new feature for user management"), CommitCategory::Feat);
        assert_eq!(categorize_commit_message("Fix bug in authentication system"), CommitCategory::Fix);
        assert_eq!(categorize_commit_message("Update documentation for API"), CommitCategory::Docs);
        assert_eq!(categorize_commit_message("Optimize performance of database queries"), CommitCategory::Perf);
        assert_eq!(categorize_commit_message("Refactor user service"), CommitCategory::Refactor);
    }

    #[test]
    fn test_context_analysis() {
        assert_eq!(categorize_commit_message("Update README.md with installation instructions"), CommitCategory::Docs);
        assert_eq!(categorize_commit_message("Bump package.json version"), CommitCategory::Deps);
        assert_eq!(categorize_commit_message("Add .github/workflows/ci.yml"), CommitCategory::Ci);
        assert_eq!(categorize_commit_message("Update config.yaml settings"), CommitCategory::Config);
        assert_eq!(categorize_commit_message("Add test/user.spec.js"), CommitCategory::Test);
    }

    #[test]
    fn test_emote_addition() {
        assert_eq!(add_emote_to_commit_message("feat: add new feature", CommitCategory::Feat), "✨ feat: add new feature");
        assert_eq!(add_emote_to_commit_message("fix: resolve bug", CommitCategory::Fix), "🐛 fix: resolve bug");
        assert_eq!(add_emote_to_commit_message("docs: update README", CommitCategory::Docs), "📚 docs: update README");
    }

    #[test]
    fn test_emote_already_present() {
        // Should not add emote if one is already present
        assert_eq!(add_emote_to_commit_message("✨ feat: add new feature", CommitCategory::Feat), "✨ feat: add new feature");
        assert_eq!(add_emote_to_commit_message("🐛 fix: resolve bug", CommitCategory::Fix), "🐛 fix: resolve bug");
    }

    #[test]
    fn test_process_commit_message() {
        assert_eq!(process_commit_message("feat: add user authentication"), "✨ feat: add user authentication");
        assert_eq!(process_commit_message("Fix critical security vulnerability"), "🔒 Fix critical security vulnerability");
        assert_eq!(process_commit_message("Update package dependencies"), "📦 Update package dependencies");
    }

    #[test]
    fn test_unknown_category() {
        assert_eq!(categorize_commit_message("Random commit message"), CommitCategory::Unknown);
        assert_eq!(process_commit_message("Random commit message"), "❓ Random commit message");
    }

    #[test]
    fn test_emote_detection() {
        assert!(starts_with_emote("✨ some message"));
        assert!(starts_with_emote("🐛 bug fix"));
        assert!(starts_with_emote("📚 documentation"));
        assert!(!starts_with_emote("feat: add feature"));
        assert!(!starts_with_emote("regular message"));
        assert!(!starts_with_emote(""));
    }

    #[test]
    fn test_category_emotes() {
        assert_eq!(CommitCategory::Fix.emote(), "🐛");
        assert_eq!(CommitCategory::Feat.emote(), "✨");
        assert_eq!(CommitCategory::Docs.emote(), "📚");
        assert_eq!(CommitCategory::Style.emote(), "💄");
        assert_eq!(CommitCategory::Refactor.emote(), "♻️");
        assert_eq!(CommitCategory::Test.emote(), "🧪");
        assert_eq!(CommitCategory::Perf.emote(), "⚡");
        assert_eq!(CommitCategory::Build.emote(), "🔧");
        assert_eq!(CommitCategory::Ci.emote(), "👷");
        assert_eq!(CommitCategory::Chore.emote(), "🧹");
        assert_eq!(CommitCategory::Deploy.emote(), "🚀");
        assert_eq!(CommitCategory::Security.emote(), "🔒");
        assert_eq!(CommitCategory::Deps.emote(), "📦");
        assert_eq!(CommitCategory::Unknown.emote(), "❓");
    }

    #[test]
    fn test_multiline_commit_messages() {
        let multiline_message = "feat: add user authentication\n\nImplement OAuth2 support with Google and GitHub providers.\nAdd secure token storage and refresh mechanism.";
        assert_eq!(categorize_commit_message(multiline_message), CommitCategory::Feat);
        
        let processed = process_commit_message(multiline_message);
        assert!(processed.starts_with("✨ feat: add user authentication"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        assert_eq!(categorize_commit_message("FEAT: ADD NEW FEATURE"), CommitCategory::Feat);
        assert_eq!(categorize_commit_message("Fix: Resolve Bug"), CommitCategory::Fix);
        assert_eq!(categorize_commit_message("Add New Feature"), CommitCategory::Feat);
    }
}