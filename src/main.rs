use std::error::Error;

use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, Role,
    },
    Client,
};

use qdrant_client::prelude::QdrantClient;

use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;

#[derive(Default)]
struct ReplContext {
    messages: Vec<ChatCompletionRequestMessage>,
}

async fn chat(
    args: ArgMatches,
    context: &mut ReplContext,
) -> reedline_repl_rs::Result<Option<String>> {
    let text = args.get_one::<String>("text").unwrap();
    context.messages.push(
        ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content(text)
            .build()
            .unwrap(),
    );
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-4")
        .messages(context.messages.clone())
        .build()
        .unwrap();
    let response = Client::new().chat().create(request).await.unwrap();
    let text = match response.choices[0].message.content.clone() {
        Some(text) => text,
        None => {
            return Ok(Some("AI: No response found, try again.".to_string()));
        }
    };
    context.messages.push(
        ChatCompletionRequestMessageArgs::default()
            .role(Role::Assistant)
            .content(text.clone())
            .build()
            .unwrap(),
    );
    Ok(Some(format!("AI: {:?}", text)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let client = QdrantClient::from_url("http://localhost:6334").build()?;

    let collections_list = client.list_collections().await?;
    dbg!(collections_list);

    let mut repl = Repl::new(ReplContext { messages: vec![] })
        .with_name("emunet")
        .with_version("v0.1.0")
        .with_description("emulation of human group by llm for high level tasks")
        .with_command_async(
            Command::new("chat")
                .arg(Arg::new("text").required(true))
                .about("Greetings!"),
            |args, context| Box::pin(chat(args, context)),
        );

    repl.run_async().await?;

    Ok(())
}
