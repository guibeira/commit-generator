# Commit Generator

A command-line tool to automatically generate Git commit messages for your staged files using local Large Language Models (LLMs) via Ollama.
## Demo
![Demo cli running](https://github.com/guibeira/commit-generator/blob/main/demo.gif "Demo cli running")
## Features

-   **AI-Powered Commits**: Generates clear and conventional commit messages based on your staged file changes.
-   **Local First**: Uses your own local LLMs through [Ollama](https://ollama.com), ensuring your code stays private.
-   **Interactive Workflow**: Suggests a commit message and lets you approve, regenerate, or cancel.
-   **Customizable Models**: Easily switch between different Ollama models using a command-line flag.
-   **Customizable Prompts**: Tailor the prompt to fit your specific commit message style or language.
-   **Automatic Model Downloading**: If a specified model isn't available locally, the tool will attempt to download it for you.

## Prerequisites

Before you begin, ensure you have the following installed:

-   [Rust](https://www.rust-lang.org/tools/install) (to build the project)
-   [Ollama](https://ollama.com) (to run the local LLM)
-   [Git](https://git-scm.com/)

## Installation

1.  Clone the repository:
    ```bash
    git clone <repository-url>
    cd <repository-directory>
    ```

2.  Build the project in release mode:
    ```bash
    cargo build --release
    ```

3.  The executable will be located at `target/release/commit_generator`. For easier access, you can move it to a directory in your system's `PATH`:
    ```bash
    sudo mv target/release/commit_generator /usr/local/bin/
    ```

## Usage

1.  Stage the files you want to commit:
    ```bash
    git add src/main.rs Cargo.toml
    ```

2.  Run the tool:
    ```bash
    commit_generator
    ```

3.  The tool will generate a suggestion. You will be prompted to:
    -   **Approve**: Press `y` to commit with the suggested message.
    -   **Regenerate**: Press `n` then `y` to get a new suggestion.
    -   **Cancel**: Press `n` then `n` to exit without committing.

## Command-Line Options

You can specify which Ollama model to use with the `--model` (or `-m`) flag.

```bash
commit_generator --model <model_name>
```

**Example:**

```bash
# Use the llama3 model
commit_generator --model llama3:latest

# Use the default model (gemma3:latest)
commit_generator
```

## Customization

### Custom Prompt

You can override the default prompt by creating a custom prompt file.

1.  Create the configuration directory:
    ```bash
    mkdir -p ~/.config/commit_generator
    ```

2.  Create and edit the prompt file:
    ```bash
    touch ~/.config/commit_generator/prompt.md
    ```

The tool will replace the `{}` placeholder in your prompt with the list of staged files. Your custom prompt **must** include `{}` for it to work correctly.

**Example `prompt.md`:**

```markdown
Please generate a concise, one-line commit message in the conventional commit format for the following staged files.

Staged files:
{}

The message should be in past tense and describe the changes made.
```
