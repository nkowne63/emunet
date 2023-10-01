use std::error::Error;

use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

use inquire::{Select, Text};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let client = Client::new();

    let mut past_messages = vec![];

    loop {
        let user_message = Text::new("You: ").prompt()?;
        past_messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(user_message)
                .build()?,
        );
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(512u16)
            .model("gpt-4")
            .messages(past_messages.clone())
            .build()?;
        let response = client.chat().create(request).await?;
        let text = match response.choices[0].message.content.clone() {
            Some(text) => text,
            None => {
                println!("AI: No response found, try again.");
                continue;
            }
        };
        println!("AI: {:?}", text);
        past_messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(text)
                .build()?,
        );
        let is_done = Select::new("Continue?", vec!["Yes", "No"]).prompt()?;

        match is_done {
            "Yes" => continue,
            "No" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}
