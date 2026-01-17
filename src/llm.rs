use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::prompt::{build_system_prompt, build_user_prompt};
use crate::types::{PrContext, Story};

#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    input: Vec<Message>,
    text: TextFormat,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct TextFormat {
    format: JsonSchemaFormat,
}

#[derive(Debug, Serialize)]
struct JsonSchemaFormat {
    #[serde(rename = "type")]
    format_type: String,
    name: String,
    schema: serde_json::Value,
    strict: bool,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    output: Vec<OutputItem>,
}

#[derive(Debug, Deserialize)]
struct OutputItem {
    content: Vec<ContentItem>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ContentItem {
    #[serde(rename = "output_text")]
    OutputText { text: String },
    #[serde(rename = "refusal")]
    Refusal { refusal: String },
}

fn build_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["summary", "focus", "narrative", "data", "open_questions", "suggested_changes", "clarification_questions", "next_pr"],
        "properties": {
            "summary": { "type": "string" },
            "focus": {
                "type": "object",
                "additionalProperties": false,
                "required": ["key_change", "review_these", "skim_these"],
                "properties": {
                    "key_change": { "type": "string" },
                    "review_these": { "type": "array", "items": { "type": "string" } },
                    "skim_these": { "type": "array", "items": { "type": "string" } }
                }
            },
            "narrative": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["title", "why", "changes", "risks", "tests", "diff_blocks"],
                    "properties": {
                        "title": { "type": "string" },
                        "why": { "type": "string" },
                        "changes": { "type": "array", "items": { "type": "string" } },
                        "risks": { "type": "array", "items": { "type": "string" } },
                        "tests": { "type": "array", "items": { "type": "string" } },
                        "diff_blocks": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "required": ["label", "role", "significance", "context", "hunks"],
                                "properties": {
                                    "label": { "type": "string" },
                                    "role": {
                                        "type": "string",
                                        "enum": ["root", "downstream", "supporting"]
                                    },
                                    "significance": {
                                        "type": "string",
                                        "enum": ["key", "standard", "noise"]
                                    },
                                    "context": { "type": "string" },
                                    "hunks": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "additionalProperties": false,
                                            "required": ["header", "lines"],
                                            "properties": {
                                                "header": { "type": "string" },
                                                "lines": { "type": "string" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "data": {
                "type": "object",
                "additionalProperties": false,
                "required": ["files_touched", "additions", "deletions"],
                "properties": {
                    "files_touched": { "type": "number" },
                    "additions": { "type": "number" },
                    "deletions": { "type": "number" }
                }
            },
            "open_questions": { "type": "array", "items": { "type": "string" } },
            "suggested_changes": { "type": "string" },
            "clarification_questions": { "type": "string" },
            "next_pr": { "type": "string" }
        }
    })
}

pub async fn generate_story(pr: &PrContext, api_key: &str, model: &str) -> Result<Story> {
    let client = reqwest::Client::new();

    let request = OpenAiRequest {
        model: model.to_string(),
        input: vec![
            Message {
                role: "system".to_string(),
                content: build_system_prompt(),
            },
            Message {
                role: "user".to_string(),
                content: build_user_prompt(pr),
            },
        ],
        text: TextFormat {
            format: JsonSchemaFormat {
                format_type: "json_schema".to_string(),
                name: "distillery_review".to_string(),
                schema: build_json_schema(),
                strict: true,
            },
        },
    };

    let response = client
        .post("https://api.openai.com/v1/responses")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to OpenAI")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API error ({}): {}", status, body);
    }

    let api_response: OpenAiResponse = response
        .json()
        .await
        .context("Failed to parse OpenAI response")?;

    let content = api_response
        .output
        .first()
        .and_then(|o| o.content.first())
        .context("No content in OpenAI response")?;

    let text = match content {
        ContentItem::OutputText { text } => text,
        ContentItem::Refusal { refusal } => {
            anyhow::bail!("Model refused request: {}", refusal);
        }
    };

    let story: Story = serde_json::from_str(text).context("Failed to parse story JSON")?;

    Ok(story)
}
