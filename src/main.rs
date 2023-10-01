use std::error::Error;

use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};

use inquire::Text;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u16)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content("You are a helpful assistant.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content("Who won the world series in 2020?")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content("The Los Angeles Dodgers won the World Series in 2020.")
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content("Where was it played?")
                .build()?,
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    println!("\nResponse:\n");
    for choice in response.choices {
        println!(
            "{}: Role: {}  Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        );
    }

    let name = Text::new("What is your name?").prompt();

    match name {
        Ok(name) => println!("Hello {}", name),
        Err(_) => println!("An error happened when asking for your name, try again later."),
    }

    let name = Text::new("What is your name?").prompt();

    match name {
        Ok(name) => println!("2 > Hello {}", name),
        Err(_) => println!("2 > An error happened when asking for your name, try again later."),
    }

    Ok(())
}
