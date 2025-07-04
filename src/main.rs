use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use std::fs;
use url::Url;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "gemma3:latest")]
    model: String,
}

fn get_prompt_template() -> String {
    let default_template =
        "Action: Generate a clear, single-line git commit message for the following staged files.
            Staged files:
            {}

            - Follow the conventional commit style (e.g., feat, fix, refactor, docs).
            - Describe the intent of the changes (what and why, not how).
            - Use only the information available from the file names.
            - Respond with only the commit message, no extra text.
            - Do not return any other information.

            **Example 1**
            Staged files:
            - src/user/profile.ts
            - src/components/Avatar.tsx

            Commit Message:
            feat: add user profile page and avatar component

            now generate a commit message for the following staged files:
            Staged files:
            {}

            DO NOT RETURN ANYTHING ELSE, ONLY THE COMMIT MESSAGE.";

    if let Ok(home) = std::env::var("HOME") {
        let mut config_path = std::path::PathBuf::from(home);
        config_path.push(".config/commit_generator/prompt.md");
        if let Ok(custom_prompt) = fs::read_to_string(config_path) {
            return custom_prompt;
        }
    }

    default_template.to_string()
}

fn check_ollama_installed() -> anyhow::Result<()> {
    let output = std::process::Command::new("ollama")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => {
            eprintln!("❌ Error: `ollama` command not found.");
            eprintln!(
                "Please install Ollama from https://ollama.com and make sure it's in your PATH."
            );
            anyhow::bail!("Ollama is not installed or not in PATH.")
        }
    }
}

fn get_staged_files() -> anyhow::Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--staged", "--cached"])
        .output()?;
    let s = String::from_utf8(output.stdout)?;
    Ok(s.lines().map(|l| l.to_string()).collect())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    check_ollama_installed()?;

    let args = Args::parse();

    // 1. Collect staged files
    let files = get_staged_files()?;
    if files.is_empty() {
        println!("Nothing to commit 😴");
        return Ok(());
    }

    // 2. Build prompt for LLM
    let prompt_template = get_prompt_template();
    let prompt = prompt_template.replace("{}", &files.join("\n"));
    let ollama_url_str =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let ollama = match Url::parse(&ollama_url_str) {
        Ok(url) => {
            let host = format!(
                "{}://{}",
                url.scheme(),
                url.host_str().unwrap_or("localhost")
            );
            let port = url.port().unwrap_or(11434);
            Ollama::new(host, port)
        }
        Err(_) => {
            // Fallback to default if URL is malformed
            Ollama::new("http://localhost".to_string(), 11434)
        }
    };

    // 3. Loop for suggestions until user approves or cancels
    loop {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} Generating commit message...")
                .unwrap(),
        );
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));
        let res = ollama
            .generate(GenerationRequest::new(args.model.clone(), prompt.clone()))
            .await;
        spinner.finish_and_clear();

        match res {
            Ok(res) => {
                let msg = res.response.trim();
                println!("\n💡 Commit suggestion:\n\"{}\"\n", msg);

                match dialoguer::Confirm::new()
                    .with_prompt("👍 Commit with this message?")
                    .interact()?
                {
                    true => {
                        std::process::Command::new("git")
                            .args(["commit", "-m", msg])
                            .status()?;
                        println!("✅ Commit successful!");
                        break;
                    }
                    false => {
                        if !dialoguer::Confirm::new()
                            .with_prompt("🔄 Generate another suggestion?")
                            .interact()?
                        {
                            println!("❌ Canceled.");
                            break;
                        }
                        // back to generation
                    }
                }
            }
            Err(e) => {
                if e.to_string().contains("model") && e.to_string().contains("not found") {
                    println!("\nModel '{}' not found. Downloading...", &args.model);
                    let status = std::process::Command::new("ollama")
                        .args(["pull", &args.model])
                        .status()?;

                    if status.success() {
                        println!("✅ Model downloaded successfully. Retrying...");
                        continue;
                    } else {
                        eprintln!("\n❌ Error: Failed to download model '{}'.", &args.model);
                        eprintln!(
                            "Please make sure Ollama is running and try to pull the model manually: ollama pull {}",
                            &args.model
                        );
                    }
                } else {
                    eprintln!("\nAn unexpected error occurred: {}", e);
                }
                break;
            }
        }
    }

    Ok(())
}
