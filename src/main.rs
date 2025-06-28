use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
fn get_staged_files() -> anyhow::Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()?;
    let s = String::from_utf8(output.stdout)?;
    Ok(s.lines().map(|l| l.to_string()).collect())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Coletar arquivos staged
    let files = get_staged_files()?;
    if files.is_empty() {
        println!("Nada para commitar 😴");
        return Ok(());
    }

    // 2. Construir prompt para LLM
    let prompt = format!(
        "Generate a concise git commit message based on these staged files:\n{}",
        files.join("\n")
    );

    let ollama = Ollama::default(); // conecta via http://localhost:11434 :contentReference[oaicite:3]{index=3}

    // 3. Laço de sugestões até user aprovar ou cancelar
    loop {
        let res = ollama
            .generate(GenerationRequest::new(
                "llama2:latest".to_string(),
                prompt.clone(),
            ))
            .await?;
        let msg = res.response.trim();
        println!("\n💡 Commit suggestion:\n\"{}\"\n", msg);

        match dialoguer::Confirm::new()
            .with_prompt("👍 Commitar com esta mensagem?")
            .interact()?
        {
            true => {
                std::process::Command::new("git")
                    .args(["commit", "-m", msg])
                    .status()?;
                println!("✅ Commit realizado!");
                break;
            }
            false => {
                if !dialoguer::Confirm::new()
                    .with_prompt("🔄 Gerar outra sugestão?")
                    .interact()?
                {
                    println!("❌ Cancelado.");
                    break;
                }
                // volta para geração
            }
        }
    }

    Ok(())
}
