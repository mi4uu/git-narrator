use color_eyre::eyre::{eyre, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use color_eyre::Help;
use color_eyre::eyre::ContextCompat;
use crate::emotes;

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

/// Filter out unwanted lines from LLM output that are meta-commentary rather than actual commit content
fn filter_unwanted_lines(content: &str) -> String {
    let unwanted_start_patterns = [
        // Direct analysis phrases that typically start unwanted lines
        "the `diff`",
        "diff indicates",
        "here's a breakdown",
        "analysis of the changes",
        "based on the diff",
        "looking at the changes",
        "the changes show",
        "i can see",
        "from the diff",
        "the diff shows",
        "analyzing the diff",
        "examining the changes",
        "reviewing the diff",
        "summary:",
        "explanation:",
        "breakdown:",
        "analysis:",
        "here's what",
        "let me analyze",
        "looking at this",
        "from what i can see",
        "the code changes",
        "in this diff",
        "this diff shows",
        "the following changes",
        "changes made:",
        "modifications:",
        "updates:",
        "alterations:",
        "revisions:",
        // Common LLM meta-commentary starters
        "based on",
        "according to",
        "it appears",
        "it seems",
        "it looks like",
        "from the context",
        "given the",
        "considering the",
        "taking into account",
    ];

    let lines: Vec<&str> = content.lines().collect();
    let mut filtered_lines = Vec::new();
    
    for line in lines {
        let line_trimmed = line.trim().to_lowercase();
        
        // Skip empty lines at the beginning but preserve them in the middle/end
        if line_trimmed.is_empty() {
            if !filtered_lines.is_empty() {
                filtered_lines.push(line);
            }
            continue;
        }
        
        // Check if line starts with unwanted patterns (conservative approach)
        let should_filter = unwanted_start_patterns.iter().any(|pattern| {
            // Check if line starts with the pattern (case insensitive)
            line_trimmed.starts_with(pattern) ||
            // Check if pattern appears at the beginning after common prefixes
            line_trimmed.starts_with(&format!("- {}", pattern)) ||
            line_trimmed.starts_with(&format!("* {}", pattern)) ||
            line_trimmed.starts_with(&format!("• {}", pattern))
        });
        
        // Special case: filter "this commit" lines that contain meta-commentary
        let is_meta_commit_line = line_trimmed.starts_with("this commit") &&
            (line_trimmed.contains("appears") || line_trimmed.contains("seems") ||
             line_trimmed.contains("based on") || line_trimmed.contains("analysis"));
        
        if !should_filter && !is_meta_commit_line {
            filtered_lines.push(line);
        }
    }
    
    // Join the filtered lines and trim any trailing whitespace
    filtered_lines.join("\n").trim().to_string()
}

/// Generate a commit message based on the git diff
pub async fn generate_commit_message(
    diff: &str,
    system_prompt: &str,
    user_prompt: &str,
    api_token: &str,
    api_base_url: &str,
    model: &str,
) -> Result<String> {
    let client = Client::new();

    // Prepare the request to OpenAI API
    let request = OpenAIRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt.replace("{}", diff),
            },
        ],
    };

    // Construct the full API endpoint URL
    let endpoint = format!("{}/v1/chat/completions", api_base_url.trim_end_matches('/'));

    // Send the request to the API
    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context(format!("Failed to send request to API at {}", endpoint))?;

    // Parse the response
    let response_status = response.status();
    let response_text = response.text().await?;

    if !response_status.is_success() {
        return Err(eyre!(
            "API request failed ({}): {}",
            response_status,
            response_text
        ));
    }

    let response: OpenAIResponse = serde_json::from_str(&response_text).context("Failed to parse API response").note(format!("response: {}  ... {}",
        &response_text[0..100.min(response_text.len())].to_string(),
        &response_text[response_text.len() - 100.min(response_text.len())..].to_string(),
    ))?;
    // Extract the commit message
    let commit_message = response
        .choices
        .first()
        .context("No response from API")?
        .message
        .content
        .clone();
    // First, handle </think> tags (existing logic)
    let mut commit_message_o: Vec<&str> = commit_message.split("</think>").collect();
    let commit_message_t = commit_message_o.pop();
    let commit_message = match commit_message_t {
        Some(m) => m.to_string(),
        None => commit_message
    };
    
    // Apply comprehensive filtering to remove unwanted LLM meta-commentary
    let filtered_message = filter_unwanted_lines(&commit_message);
    
    Ok(filtered_message)
}

/// Combine multiple commit messages into a single coherent message
pub async fn combine_commit_messages(
    messages: Vec<String>,
    system_prompt: &str,
    api_token: &str,
    api_base_url: &str,
    model: &str,
) -> Result<String> {
    if messages.is_empty() {
        return Err(eyre!("No commit messages to combine"));
    }
    
    if messages.len() == 1 {
        return Ok(messages[0].clone());
    }

    let client = Client::new();
    
    // Create a prompt for combining messages
    let combined_messages = messages.iter()
        .enumerate()
        .map(|(i, msg)| format!("Message {}: {}", i + 1, msg))
        .collect::<Vec<_>>()
        .join("\n\n");
    
    let combination_prompt = format!(
        "Combine the following {} commit messages into a single, coherent commit message. \
        Remove any redundancy and create a unified message that captures all the changes. \
        Follow conventional commit format and best practices:\n\n{}",
        messages.len(),
        combined_messages
    );

    let request = OpenAIRequest {
        model: model.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: combination_prompt,
            },
        ],
    };

    let endpoint = format!("{}/v1/chat/completions", api_base_url.trim_end_matches('/'));

    let response = client
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_token))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context(format!("Failed to send combination request to API at {}", endpoint))?;

    let response_status = response.status();
    let response_text = response.text().await?;

    if !response_status.is_success() {
        return Err(eyre!(
            "API request failed ({}): {}",
            response_status,
            response_text
        ));
    }

    let response: OpenAIResponse = serde_json::from_str(&response_text)
        .context("Failed to parse API response for message combination")?;
    
    let combined_message = response
        .choices
        .first()
        .context("No response from API for message combination")?
        .message
        .content
        .clone();

    // Apply the same filtering as individual messages
    let mut combined_message_o: Vec<&str> = combined_message.split("</think>").collect();
    let combined_message_t = combined_message_o.pop();
    let combined_message = match combined_message_t {
        Some(m) => m.to_string(),
        None => combined_message
    };
    
    let filtered_message = filter_unwanted_lines(&combined_message);
    
    // Add emote to the combined message based on categorization
    let message_with_emote = emotes::process_commit_message(&filtered_message);
    
    Ok(message_with_emote)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn test_generate_commit_message() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Create a mock response
        let mock_response = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "feat: improve greeting message with username support"
                }
            }]
        });

        // Set up the mock expectation
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test_token"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        // Sample git diff
        let diff = r#"
            diff --git a/src/main.rs b/src/main.rs
            index 1234567..89abcdef 100644
            --- a/src/main.rs
            +++ b/src/main.rs
            @@ -1,3 +1,4 @@
            +use std::env;
            fn main() {
            -    println!("Hello, world!");
            +    println!("Hello, {}!", env::var("USER").unwrap_or("world".to_string()));
            }
            "#;

        let system_prompt = "You are a helpful assistant.";
        let user_prompt =
            "Here is the git diff of the staged changes. Generate a commit message that \
            follows the conventional commit format and best practices. Focus on what changed \
            and why, not how it changed:\n\n\
            ```diff\n{}\n```";
        let model = "gpt-3.5-turbo";

        // Use the mock server URL instead of the real OpenAI API
        let commit_message = generate_commit_message(
            diff,
            system_prompt,
            user_prompt,
            "test_token",
            &mock_server.uri(),
            model,
        )
        .await?;

        // Verify the response
        assert_eq!(
            commit_message,
            "feat: improve greeting message with username support"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_generate_commit_message_api_error() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Set up the mock to return an error
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&mock_server)
            .await;

        // Attempt to generate a commit message
        let result = generate_commit_message(
            "some diff",
            "system prompt",
            "user prompt",
            "invalid_token",
            &mock_server.uri(),
            "gpt-3.5-turbo",
        )
        .await;

        // Verify that we get an error
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("API request failed"));
        assert!(err.contains("401"));
        assert!(err.contains("Unauthorized"));

        Ok(())
    }

    #[test]
    fn test_filter_unwanted_lines() {
        // Test filtering of common LLM meta-commentary that starts lines
        let input = "Based on the diff, this commit adds a new feature.\nfeat: add user authentication\n\nThis commit implements login functionality.";
        let expected = "feat: add user authentication\n\nThis commit implements login functionality.";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test filtering of analysis phrases that start lines
        let input = "Here's a breakdown of the changes:\n- feat: add new component\n- Updated styling\n\nThe diff shows significant improvements.";
        let expected = "- feat: add new component\n- Updated styling";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test preservation of valid commit content
        let input = "feat: implement user dashboard\n\nAdd comprehensive user dashboard with:\n- Profile management\n- Settings panel\n- Activity history";
        let expected = input; // Should remain unchanged
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test filtering of multiple unwanted patterns
        let input = "Looking at the changes, I can see that this diff indicates:\nfix: resolve authentication bug\n\nAnalysis of the changes shows this fixes the login issue.";
        let expected = "fix: resolve authentication bug";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test handling of empty lines and whitespace
        let input = "\n\nBased on the diff:\nfeat: add new feature\n\n";
        let expected = "feat: add new feature";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test case insensitive filtering
        let input = "the diff shows that we need:\nfeat: update configuration\nFrom what I can see, this is important.";
        let expected = "feat: update configuration";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test filtering with bullet points
        let input = "- Based on the diff, this adds features\n- feat: implement search\n- The changes show improvements";
        let expected = "- feat: implement search";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test that valid content with similar words is preserved
        let input = "feat: add diff viewer component\n\nImplements a new diff viewer that shows changes clearly.";
        let expected = input; // Should remain unchanged as "diff" here is part of valid content
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test filtering of meta-commentary about commits
        let input = "This commit appears to be adding a new feature based on the analysis.\nfeat: add search functionality\n\nImplement search with filters.";
        let expected = "feat: add search functionality\n\nImplement search with filters.";
        assert_eq!(filter_unwanted_lines(input), expected);
    }

    #[test]
    fn test_filter_unwanted_lines_edge_cases() {
        // Test empty input
        assert_eq!(filter_unwanted_lines(""), "");

        // Test input with only unwanted content
        let input = "Based on the diff, here's what I can see from the analysis.";
        assert_eq!(filter_unwanted_lines(input), "");

        // Test input with mixed valid and invalid content
        let input = "The diff indicates:\nfeat: add authentication\n\nBased on the changes:\nfix: resolve bug\n\nLooking at this diff:";
        let expected = "The diff indicates:\nfeat: add authentication\n\nfix: resolve bug";
        assert_eq!(filter_unwanted_lines(input), expected);

        // Test very short lines that shouldn't be filtered
        let input = "fix: bug\nThe fix works.";
        let expected = input; // Should remain unchanged
        assert_eq!(filter_unwanted_lines(input), expected);
    }

    #[tokio::test]
    async fn test_combine_commit_messages() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Create a mock response for message combination
        let mock_response = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "feat: implement user authentication and dashboard\n\nAdd login functionality and comprehensive user dashboard with profile management."
                }
            }]
        });

        // Set up the mock expectation
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(header("Authorization", "Bearer test_token"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_response))
            .mount(&mock_server)
            .await;

        let messages = vec![
            "feat: add user authentication".to_string(),
            "feat: implement user dashboard".to_string(),
        ];

        let combined_message = combine_commit_messages(
            messages,
            "You are a helpful assistant.",
            "test_token",
            &mock_server.uri(),
            "gpt-3.5-turbo",
        )
        .await?;

        assert_eq!(
            combined_message,
            "✨ feat: implement user authentication and dashboard\n\nAdd login functionality and comprehensive user dashboard with profile management."
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_combine_commit_messages_single_message() -> Result<()> {
        let messages = vec!["feat: add new feature".to_string()];
        
        let result = combine_commit_messages(
            messages.clone(),
            "system prompt",
            "token",
            "http://example.com",
            "model",
        )
        .await?;

        assert_eq!(result, messages[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_combine_commit_messages_empty() {
        let messages = vec![];
        
        let result = combine_commit_messages(
            messages,
            "system prompt",
            "token",
            "http://example.com",
            "model",
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No commit messages to combine"));
    }
}
