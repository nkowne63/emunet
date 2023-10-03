use std::{collections::HashMap, error::Error};

use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequestArgs, CreateEmbeddingRequest, CreateEmbeddingRequestArgs, Role,
    },
    Client,
};

use qdrant_client::{
    client,
    prelude::{CreateCollection, QdrantClient},
    qdrant::{vectors_config::Config, Distance, PointStruct, Value, VectorParams, VectorsConfig},
};

use reedline_repl_rs::clap::{Arg, ArgMatches, Command};
use reedline_repl_rs::Repl;

#[derive(Default)]
struct ReplContext {
    messages: Vec<ChatCompletionRequestMessage>,
    id: usize,
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
    let response = match response.choices[0].message.content.clone() {
        Some(text) => text,
        None => {
            return Ok(Some("AI: No response found, try again.".to_string()));
        }
    };
    context.messages.push(
        ChatCompletionRequestMessageArgs::default()
            .role(Role::Assistant)
            .content(response.clone())
            .build()
            .unwrap(),
    );
    let text_embeddings = Client::new()
        .embeddings()
        .create(
            CreateEmbeddingRequestArgs::default()
                .model("text-embedding-ada-002")
                .input(text)
                .build()
                .unwrap(),
        )
        .await
        .unwrap()
        .data
        .get(0)
        .unwrap()
        .to_owned();
    let response_embeddings = Client::new()
        .embeddings()
        .create(
            CreateEmbeddingRequestArgs::default()
                .model("text-embedding-ada-002")
                .input(response.clone())
                .build()
                .unwrap(),
        )
        .await
        .unwrap()
        .data
        .get(0)
        .unwrap()
        .to_owned();

    let client_qdrant = QdrantClient::from_url("http://localhost:6334")
        .build()
        .unwrap();

    let collections_list = client_qdrant.list_collections().await.unwrap();
    // dbg!(collections_list);

    if !collections_list
        .collections
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .contains(&"emunet".to_string())
    {
        let create_responce = client_qdrant
            .create_collection(&CreateCollection {
                collection_name: "emunet".into(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine as i32,
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .unwrap();
        dbg!(create_responce);
    }

    // then pass embeddings to qdrant
    client_qdrant
        .upsert_points(
            "emunet",
            [(text_embeddings, text), (response_embeddings, &response)]
                .iter()
                .enumerate()
                .map(|(i, e)| PointStruct {
                    id: Some(((context.id + i) as u64).into()),
                    vectors: Some(e.0.embedding.clone().into()),
                    payload: [("text".to_string(), Value::from(e.1.clone()))]
                        .into_iter()
                        .collect(),
                })
                .collect(),
            None,
        )
        .await
        .unwrap();
    context.id += 2;

    Ok(Some(format!("AI: {:?}", response)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv()?;

    let client = QdrantClient::from_url("http://localhost:6334").build()?;

    let collections_list = client.list_collections().await?;
    // dbg!(collections_list);

    if !collections_list
        .collections
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .contains(&"emunet".to_string())
    {
        let create_responce = client
            .create_collection(&CreateCollection {
                collection_name: "emunet".into(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine as i32,
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;
        dbg!(create_responce);
    }

    let mut repl = Repl::new(ReplContext {
        messages: vec![],
        id: 0,
    })
    .with_name("emunet")
    .with_version("v0.1.0")
    .with_description("emulation of human group by llm for high level tasks")
    .with_command_async(
        Command::new("chat")
            .arg(Arg::new("text").required(true))
            .about("Greetings!"),
        |args, context| Box::pin(chat(args, context)),
    );
    // TODO: add retry command which will retry last command with new text and embedding from qdrant

    repl.run_async().await?;

    Ok(())
}
