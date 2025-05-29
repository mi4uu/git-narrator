# git-narrator: AI Commit Message Generator

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/git-narrator)](https://crates.io/crates/git-narrator)
[![codecov](https://codecov.io/gh/shenxiangzhuang/git-narrator/graph/badge.svg?token=Ekvrf0TzJa)](https://codecov.io/gh/shenxiangzhuang/git-narrator)

A CLI tool that uses AI to generate meaningful commit messages by analyzing your staged Git changes.

## Features

- 🤖 **AI-Powered**: Automatically generates detailed and context-aware commit messages
- ✏️ **Interactive Mode**: Review and edit generated messages before committing
- 🔌 **Multiple AI Providers**: Works with OpenAI and compatible APIs
- 🌟 **Project-level Config**: Use `.git-narrator.toml` for repository-specific settings
- ⚙️ **Customizable**: Configure prompts, models, and API endpoints
- 💭 **Thinking models**: Support for thinking models
- 🪓 **Split long diff**: Spliting long diff so it will allways fit the context size

## Installation
```
  cargo install --git https://github.com/mi4uu/git-narrator
```

or

```bash
cargo install git-narrator
```

## Quick Start

1. Configure your API settings:

```bash
# For OpenAI
git-narrator config setup --api-token your_openai_token --api-base-url https://api.openai.com --model gpt-3.5-turbo

# for Ollama
ollama pull qwen3:4b
git-narrator config setup --api-token anything-will-do --api-base-url http://127.0.0.1:11434 --model qwen3:4b  

# For DeepSeek
git-narrator config setup --api-token your_deepseek_token --api-base-url https://api.deepseek.com --model deepseek-chat
```


Output:

```bash
⚙️  Updating configuration...
✓ Set api_token to: your•••••
✓ Set api_base_url to: https://api.openai.com
✓ Set model to: gpt-3.5-turbo
🎉 Configuration updated successfully!
```

2. Verify your configuration:

```bash
git-narrator config list
```

Output:

```
⚙️  Current Configuration:
┌───────────────┬──────────────────────────────────────┐
│ api_token     │ your•••••                            │
│ api_base_url  │ https://api.openai.com               │
│ model         │ gpt-3.5-turbo                        │
│ system_prompt │ You are an expert at writing...      │
│ user_prompt   │ Here is the git diff of the staged...│
└───────────────┴──────────────────────────────────────┘

📁 Configuration file location:
   /home/user/.config/git-narrator/config.toml
```

3. Test your API connection:

```bash
git-narrator ping
```

Output:

```
🔍 Testing API connection...
🌐 API Base URL: https://api.openai.com
🤖 Model: gpt-3.5-turbo
✅ API connection successful!
✨ Configuration is working correctly.
```

4. Generate commit messages:

```bash
# Stage changes and generate commit message
git-narrator -a

# Generate and commit automatically
git-narrator -ac

# Generate, commit, and push automatically
git-narrator -acp

# Generate commit message (with staged changes)
git-narrator
```

Example output:
```
╭─────────────────────────────────────╮
│     AI Commit Message Generator     │
╰─────────────────────────────────────╯
📦 Staging all changes...
🔍 Analyzing staged changes...
🤖 Using model: gpt-3.5-turbo
✨ Generating commit message...
📋 Commit command:
git commit -m "feat: add new feature X"

Execute this commit? [Y/m/n]:
```

## Usage

### Basic Commands

```bash
# Generate commit message (requires staged changes)
git-narrator

# Stage all changes and generate commit message
git-narrator -a

# Generate and commit automatically
git-narrator -c

# Stage all changes and commit automatically
git-narrator -ac

# Generate commit message and push after committing
git-narrator -p

# Stage all changes, generate commit message and push after committing
git-narrator -ap

# Generate, commit, and push automatically
git-narrator -cp

# Stage all changes, commit, and push automatically 
git-narrator -acp

# Test API connection
git-narrator ping
```

> **Note**: The `-a` flag will stage ALL changes in your working directory with `git add .`. The `-c` flag will commit directly without confirmation. The `-p` flag will push changes to remote after a successful commit (either automatic or manual). Use these flags with caution, especially in repositories with multiple changes.

### Configuration Management

```bash
# Quick setup
git-narrator config setup --api-token <TOKEN> --api-base-url https://api.openai.com --model gpt-4-turbo

# View current settings
git-narrator config list

# View active configuration (global + project)
git-narrator config show

# Get specific setting
git-narrator config get api_token

# Update setting
git-narrator config set model gpt-4-turbo
git-narrator config set default_prompt "Write detailed commit messages"
```

You can also create a project-specific `.git-narrator.toml` file in your repository root. See [Project-level Configuration](#project-level-configuration) for details.

### Configuration Files

#### Global Configuration

The global configuration is stored in TOML format at:
- Linux/macOS: `~/.config/git-narrator/config.toml`
- Windows: `%APPDATA%\git-narrator\config.toml`

Example `config.toml`:

```toml
api_token = "your_api_token_here"
api_base_url = "https://api.openai.com"
model = "gpt-3.5-turbo"
system_prompt = """You are an expert at writing clear and concise commit messages. 
Follow these rules strictly:

1. Start with a type: feat, fix, docs, style, refactor, perf, test, build, ci, chore, or revert
2. Add a scope in parentheses when the change affects a specific component/module
3. Write a brief description in imperative mood (e.g., 'add' not 'added')
4. Keep the first line under 72 characters
5. For simple changes (single file, small modifications), use only the subject line
6. For complex changes (multiple files, new features, breaking changes):
   - Add a body explaining what and why
   - Use numbered points (1., 2., 3., etc.) to list distinct changes
   - Organize points in order of importance"""
user_prompt = """Generate a commit message for the following changes. First analyze the complexity of the diff.

For simple changes, provide only a subject line.

For complex changes, include a body with numbered points (1., 2., 3.) that clearly outline
each distinct modification or feature. Organize these points by importance.

Look for patterns like new features, bug fixes, or configuration changes to determine
the appropriate type and scope:

```diff
{}
```"""
```

### Configuration Options

- `api_token`: Your API authentication token
- `api_base_url`: API endpoint (default: OpenAI)
- `model`: AI model to use (default: gpt-3.5-turbo)
- `system_prompt`: System prompt that defines the AI's role and commit message format
- `user_prompt`: User prompt that provides context about the git changes

### Project-level Configuration

In addition to global settings, you can create a project-specific configuration file:

```bash
# Check current active configuration (global + project)
git-narrator config show
```

1. Create a `.git-narrator.toml` file in your Git repository root 
2. Project settings will override global settings when running `git-narrator` in that repository
3. The search for project config will stop at the Git repository root (directory with `.git` folder)

Example `.git-narrator.toml`:

```toml
# Project-specific configuration (.git-narrator.toml)
# All fields are optional - only specify what you want to override

# API settings
api_token = "your_api_token_here"  # Only add if different from global config
api_base_url = "https://api.openai.com"
model = "gpt-4-turbo"  # Use a different model for this project

# Customized prompts for project-specific commit conventions
system_prompt = """You are a commit message expert for our project.
Use our project conventions:
1. feat: for new features
2. fix: for bug fixes 
3. docs: for documentation
4. refactor: for code changes that neither fix bugs nor add features
5. style: for changes that do not affect the meaning of the code
6. test: for adding or modifying tests
7. chore: for routine tasks, dependency updates, etc.

Always include the scope in parentheses when possible.
Example: feat(auth): implement OAuth login

For complex changes, use bullet points to describe the details."""

user_prompt = """Generate a commit message following our project conventions.
Analyze the complexity of the diff and provide appropriate detail:

```diff
{}
```"""
```

You can view the active configuration and which files are being used with:

```bash
git-narrator config show
```

Output example:

```
📋 Active Configuration:

🔍 Configuration Sources:
   Global config: /home/user/.config/git-narrator/config.toml
   Project config: /path/to/your/project/.git-narrator.toml
   ℹ️ Project settings override global settings
...
```

### Environment Variables

- `EDITOR`: Preferred editor for modifying commit messages
  - Falls back to: vim → vi → nano

## Examples

### Basic Usage

```bash
# Stage changes and generate commit message
git add .
git-narrator

# Stage and commit automatically
git-narrator -ac

# Stage changes and push after manual commit
git-narrator -ap

# Stage, commit and push automatically (all-in-one)
git-narrator -acp

# Commit and push changes that are already staged
git-narrator -cp
```

### Configuration Examples

```bash
# Set up OpenAI
git-narrator config setup --api-token sk-... --model gpt-4-turbo

# Set up DeepSeek
git-narrator config setup --api-token ds-... --api-base-url https://api.deepseek.com --model deepseek-chat

# Customize commit message style
git-narrator config set system_prompt "You are an expert at writing clear and concise commit messages..."
git-narrator config set user_prompt "Here is the git diff of the staged changes. Generate a commit message..."
```

## Troubleshooting

1. **No Changes Detected**
   - Ensure changes are staged with `git add`
   - Check if you're in a git repository

2. **API Errors**
   - Verify your API token is set correctly
   - Check API endpoint accessibility
   - Confirm you have sufficient API credits

3. **Editor Issues**
   - Set your preferred editor: `export EDITOR=vim`
   - Ensure the editor is installed and accessible

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## References

- [The Lost Art of Commit Messages](https://www.seyhan.me/blog/post/lost-art-of-commit-messages)
- [Conventional Commits 1.0.0](https://www.conventionalcommits.org/)
