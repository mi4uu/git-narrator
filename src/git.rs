use color_eyre::eyre::{bail, eyre, Context, Result};
use colored::Colorize;
use std::process::Command;

/// Size threshold for splitting diffs (in characters)
const DIFF_SIZE_THRESHOLD: usize = 8000;
/// Maximum number of split attempts
const MAX_SPLIT_ATTEMPTS: usize = 4;

/// Represents a split diff chunk with context
#[derive(Debug, Clone)]
pub struct DiffChunk {
    pub content: String,
    pub description: String,
}

/// Result of diff splitting operation
#[derive(Debug)]
pub struct SplitDiffResult {
    pub chunks: Vec<DiffChunk>,
    pub total_size: usize,
    pub split_method: String,
}

/// Get the diff for staged changes in the git repository
pub fn get_diff() -> Result<String> {
    // Check git installation and is in a repo by `git status`
    let git_status_output = Command::new("git").arg("status").output()?;

    if !git_status_output.status.success() {
        println!(
            "{}",
            "⚠️  Make sure git is installed and you're in a git repository.".yellow()
        );
        return Ok("".to_string());
    }

    // Get the diff of staged changes
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to execute git diff command.")?;

    // Parse diff content
    let diff = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(diff)
}

/// Check if a diff needs to be split based on size threshold
pub fn needs_splitting(diff: &str) -> bool {
    diff.len() > DIFF_SIZE_THRESHOLD
}

/// Split a large diff into smaller chunks using progressive strategies
pub fn split_large_diff(diff: &str) -> Result<SplitDiffResult> {
    if !needs_splitting(diff) {
        return Ok(SplitDiffResult {
            chunks: vec![DiffChunk {
                content: diff.to_string(),
                description: "Complete diff (no splitting needed)".to_string(),
            }],
            total_size: diff.len(),
            split_method: "none".to_string(),
        });
    }

    // Try progressive splitting strategies
    for attempt in 0..MAX_SPLIT_ATTEMPTS {
        let result = match attempt {
            0 => split_by_files(diff),
            1 => split_by_hunks(diff),
            2 => split_by_character_chunks(diff, DIFF_SIZE_THRESHOLD / 2),
            3 => split_by_character_chunks(diff, DIFF_SIZE_THRESHOLD / 4),
            _ => break,
        };

        if let Ok(split_result) = result {
            // Check if all chunks are within size limit
            let all_chunks_valid = split_result.chunks.iter()
                .all(|chunk| chunk.content.len() <= DIFF_SIZE_THRESHOLD);
            
            if all_chunks_valid {
                return Ok(split_result);
            }
        }
    }

    // If all splitting attempts fail, return error
    Err(eyre!("Unable to split diff into manageable chunks after {} attempts", MAX_SPLIT_ATTEMPTS))
}

/// Split diff by individual files
fn split_by_files(diff: &str) -> Result<SplitDiffResult> {
    let mut chunks = Vec::new();
    let mut current_file_content = String::new();
    let mut current_file_name = String::new();
    let mut in_file = false;

    for line in diff.lines() {
        if line.starts_with("diff --git") {
            // Save previous file if exists
            if in_file && !current_file_content.is_empty() {
                chunks.push(DiffChunk {
                    content: current_file_content.trim().to_string(),
                    description: format!("File: {}", current_file_name),
                });
            }
            
            // Start new file
            current_file_content.clear();
            current_file_name = extract_file_name(line);
            in_file = true;
        }
        
        if in_file {
            current_file_content.push_str(line);
            current_file_content.push('\n');
        }
    }

    // Add the last file
    if in_file && !current_file_content.is_empty() {
        chunks.push(DiffChunk {
            content: current_file_content.trim().to_string(),
            description: format!("File: {}", current_file_name),
        });
    }

    if chunks.is_empty() {
        return Err(eyre!("No files found in diff"));
    }

    Ok(SplitDiffResult {
        chunks,
        total_size: diff.len(),
        split_method: "by_files".to_string(),
    })
}

/// Split diff by hunks (sections of changes within files)
fn split_by_hunks(diff: &str) -> Result<SplitDiffResult> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut current_file_header = String::new();
    let mut hunk_count = 0;

    for line in diff.lines() {
        if line.starts_with("diff --git") || line.starts_with("index ") ||
           line.starts_with("--- ") || line.starts_with("+++ ") {
            current_file_header.push_str(line);
            current_file_header.push('\n');
        } else if line.starts_with("@@") {
            // Save previous hunk if exists
            if !current_chunk.is_empty() {
                chunks.push(DiffChunk {
                    content: format!("{}{}", current_file_header, current_chunk.trim()),
                    description: format!("Hunk {}", hunk_count),
                });
                current_chunk.clear();
            }
            
            hunk_count += 1;
            current_chunk.push_str(line);
            current_chunk.push('\n');
        } else if !line.starts_with("diff --git") {
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }
    }

    // Add the last hunk
    if !current_chunk.is_empty() {
        chunks.push(DiffChunk {
            content: format!("{}{}", current_file_header, current_chunk.trim()),
            description: format!("Hunk {}", hunk_count),
        });
    }

    if chunks.is_empty() {
        return Err(eyre!("No hunks found in diff"));
    }

    Ok(SplitDiffResult {
        chunks,
        total_size: diff.len(),
        split_method: "by_hunks".to_string(),
    })
}

/// Split diff by character count chunks as a last resort
fn split_by_character_chunks(diff: &str, chunk_size: usize) -> Result<SplitDiffResult> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut chunk_num = 1;

    while start < diff.len() {
        let end = (start + chunk_size).min(diff.len());
        let chunk_content = &diff[start..end];
        
        chunks.push(DiffChunk {
            content: chunk_content.to_string(),
            description: format!("Character chunk {} (chars {}-{})", chunk_num, start, end),
        });
        
        start = end;
        chunk_num += 1;
    }

    Ok(SplitDiffResult {
        chunks,
        total_size: diff.len(),
        split_method: "by_characters".to_string(),
    })
}

/// Extract file name from diff header line
fn extract_file_name(line: &str) -> String {
    // Parse "diff --git a/path/to/file b/path/to/file"
    if let Some(start) = line.find("a/") {
        if let Some(end) = line[start..].find(" b/") {
            return line[start + 2..start + end].to_string();
        }
    }
    
    // Fallback: try to extract any file-like pattern
    line.split_whitespace()
        .find(|part| part.contains('/') || part.contains('.'))
        .unwrap_or("unknown_file")
        .to_string()
}

/// Push committed changes to the remote repository
pub fn push_changes() -> Result<()> {
    println!("{} Running 'git push'...", "▶".green());
    let output = Command::new("git")
        .arg("push")
        .output()
        .context("Failed to execute git push command.")?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr).into_owned();
        eprintln!(
            "{}",
            format!("⚠️  Failed to push changes: {}", error_message).red()
        );
        bail!("Git push failed");
    }

    println!("{} Changes pushed successfully.", "✔".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn test_get_diff_with_staged_changes() -> Result<()> {
        // Create a temporary git repository
        let tmp_dir = Builder::new()
            .prefix("test_get_diff_with_staged_changes")
            .tempdir()
            .unwrap();
        let repo_path = tmp_dir.path();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        // Configure git user for the test
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()?;
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()?;

        // Create and stage a test file
        let test_file = repo_path.join("test.txt");
        let test_content = "Hello, World!";
        File::create(&test_file)?.write_all(test_content.as_bytes())?;

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()?;

        // Change to the test directory
        env::set_current_dir(repo_path)?;

        // Get the diff
        let diff = get_diff()?;

        // Verify the diff contains our changes
        let normalized_diff = diff.replace("\r\n", "\n");
        assert!(normalized_diff.contains("test.txt"));
        assert!(normalized_diff.contains(test_content));

        Ok(())
    }

    #[test]
    fn test_needs_splitting() {
        let small_diff = "a".repeat(1000);
        assert!(!needs_splitting(&small_diff));

        let large_diff = "a".repeat(10000);
        assert!(needs_splitting(&large_diff));
    }

    #[test]
    fn test_split_large_diff_small_input() -> Result<()> {
        let small_diff = "small diff content";
        let result = split_large_diff(small_diff)?;
        
        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.split_method, "none");
        assert_eq!(result.chunks[0].content, small_diff);
        
        Ok(())
    }

    #[test]
    fn test_split_by_files() -> Result<()> {
        let diff = r#"diff --git a/file1.rs b/file1.rs
index 1234567..89abcdef 100644
--- a/file1.rs
+++ b/file1.rs
@@ -1,3 +1,4 @@
+use std::env;
 fn main() {
-    println!("Hello, world!");
+    println!("Hello, {}!", env::var("USER").unwrap_or("world".to_string()));
 }
diff --git a/file2.rs b/file2.rs
index abcdef1..2345678 100644
--- a/file2.rs
+++ b/file2.rs
@@ -1,2 +1,3 @@
 fn test() {
+    println!("test");
 }"#;

        let result = split_by_files(diff)?;
        assert_eq!(result.chunks.len(), 2);
        assert_eq!(result.split_method, "by_files");
        assert!(result.chunks[0].description.contains("file1.rs"));
        assert!(result.chunks[1].description.contains("file2.rs"));
        
        Ok(())
    }

    #[test]
    fn test_extract_file_name() {
        let line = "diff --git a/src/main.rs b/src/main.rs";
        assert_eq!(extract_file_name(line), "src/main.rs");
        
        let line2 = "diff --git a/test.txt b/test.txt";
        assert_eq!(extract_file_name(line2), "test.txt");
    }

    #[test]
    fn test_split_by_character_chunks() -> Result<()> {
        let large_content = "a".repeat(1000);
        let result = split_by_character_chunks(&large_content, 300)?;
        
        assert_eq!(result.chunks.len(), 4); // 1000 / 300 = 3.33, so 4 chunks
        assert_eq!(result.split_method, "by_characters");
        
        // Verify total content is preserved
        let combined: String = result.chunks.iter()
            .map(|chunk| chunk.content.as_str())
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(combined, large_content);
        
        Ok(())
    }
}
