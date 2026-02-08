//! Ignored smoke test for the in-process `mistralrs` provider integration.
#![allow(missing_docs)]

#[cfg(feature = "mistralrs")]
use std::sync::Arc;

#[cfg(feature = "mistralrs")]
use saorsa_agent::{AgentConfig, AgentEvent, AgentLoop, ToolRegistry, event_channel};
#[cfg(feature = "mistralrs")]
use saorsa_ai::{MistralrsConfig, MistralrsProvider};

// This is intentionally ignored: it requires a local GGUF model file and can be slow.
//
// To run:
// - Set `SAORSA_MISTRALRS_GGUF_REPO` (Hugging Face model repo id)
// - Set `SAORSA_MISTRALRS_GGUF_FILE` (GGUF filename within that repo)
// - Optional: set `HF_HOME` to control the download/cache location
// - Run: `cargo test -p saorsa-agent --features mistralrs -- --ignored --nocapture`
#[tokio::test]
#[ignore]
#[cfg(feature = "mistralrs")]
async fn agentloop_streams_with_inprocess_mistralrs() {
    let repo = match std::env::var("SAORSA_MISTRALRS_GGUF_REPO") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("SAORSA_MISTRALRS_GGUF_REPO not set; skipping ignored smoke test");
            return;
        }
    };
    let file = match std::env::var("SAORSA_MISTRALRS_GGUF_FILE") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("SAORSA_MISTRALRS_GGUF_FILE not set; skipping ignored smoke test");
            return;
        }
    };

    eprintln!(
        "mistralrs download/cache dir: {:?}",
        saorsa_ai::mistralrs::default_download_dir()
    );

    let model = match mistralrs::GgufModelBuilder::new(repo, vec![file])
        .with_force_cpu()
        .build()
        .await
    {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load GGUF model via mistralrs (repo download/cache): {e}");
            return;
        }
    };

    let provider = MistralrsProvider::new(Arc::new(model), MistralrsConfig::default());
    let config = AgentConfig::new("local")
        .system_prompt("You are a helpful assistant.")
        .max_turns(1)
        .max_tokens(128);
    let tools = ToolRegistry::new(); // must be empty for the MVP text-only provider.
    let (event_tx, mut event_rx) = event_channel(256);

    let mut agent = AgentLoop::new(Box::new(provider), config, tools, event_tx);

    let consumer = tokio::spawn(async move {
        let mut saw_text = false;
        while let Some(ev) = event_rx.recv().await {
            if matches!(ev, AgentEvent::TextDelta { .. }) {
                saw_text = true;
                break;
            }
        }
        saw_text
    });

    let result = agent.run("Say hello in one short sentence.").await;

    let saw_text = match consumer.await {
        Ok(v) => v,
        Err(e) => panic!("event consumer task failed: {e}"),
    };

    let text = result.unwrap_or_else(|e| panic!("AgentLoop failed: {e}"));
    assert!(!text.trim().is_empty());
    assert!(saw_text, "expected at least one streamed text delta event");
}
