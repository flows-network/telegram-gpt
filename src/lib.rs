use tg_flows::{listen_to_update, Telegram, Update, UpdateKind};
use openai_flows::{
    chat::{ChatModel, ChatOptions},
    OpenAIFlows,
};
use flowsnet_platform_sdk::logger;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let placeholder_text = std::env::var("placeholder").unwrap_or("Typing ...".to_string());

    listen_to_update(&telegram_token, |update| {
        let tele = Telegram::new(telegram_token.to_string());
        handler(tele, &placeholder_text, update)
    }).await;

    Ok(())
}

async fn handler(tele: Telegram, placeholder_text: &str, update: Update) {
    if let UpdateKind::Message(msg) = update.kind {
        let chat_id = msg.chat.id;
        log::info!("Received message from {}", chat_id);
        let placeholder = tele
            .send_message(chat_id, placeholder_text)
            .expect("Error occurs when sending Message to Telegram");

        let text = msg.text().unwrap_or("");
        let system = "I want you to act as an English translator, spelling corrector and improver. I will speak to you in any language and you will detect the language, translate it and answer in the corrected and improved version of my text, in English. I want you to replace my simplified A0-level words and sentences with more beautiful and elegant, upper level English words and sentences. Keep the meaning same, but make them more literary. I want you to only reply the correction, the improvements and nothing else, do not write explanations.";
        let mut openai = OpenAIFlows::new();
        openai.set_retry_times(3);
        let co = ChatOptions {
            // model: ChatModel::GPT4,
            model: ChatModel::GPT35Turbo,
            restart: true,
            system_prompt: Some(system),
        };
        match openai.chat_completion(&chat_id.to_string(), &text, &co).await {
            Ok(r) => {
                _ = tele.edit_message_text(chat_id, placeholder.id, r.choice);
            }
            Err(e) => {
                log::error!("OpenAI returns error: {}", e);
            }
        }
    }
}
