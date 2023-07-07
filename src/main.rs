use std::error::Error;
use std::io::{stdin, stdout, Write, BufRead};

use async_openai::{
    types::{ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, Role},
    Client,
};
use futures::StreamExt;

pub struct Opinion {
    message: String,
}

pub struct OneSelf {
    personality: String,
}

impl OneSelf {
    pub async fn query(&mut self, prompt: String) -> Result<Opinion, Box<dyn Error>> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .max_tokens(256u16)
            .messages([ChatCompletionRequestMessageArgs::default()
                .content(&self.personality)
                .role(Role::System)
                .content(prompt)
                .role(Role::User)
                .build()?])
            .stream(true)
            .build()?;
        let client = Client::new();
        let mut stream = client.chat().create_stream(request).await?;
        let mut message = String::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|choice| {
                        if let Some(content) = &choice.delta.content {
                            message.push_str(content.as_str());
                        }
                    });
                }
                Err(err) => {
                    return Err(err.into())
                }
            }
        }
        Ok(Opinion {message: message})
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut sout = stdout().lock();
    let mut sin = stdin().lock();

    let mut scientist = OneSelf {
        personality: String::from("Please answer as a strictly scientific mind, everytime within 50 words."),
    };

    loop {
        sout.write_all(b"Please ask a question (just press enter to exit): ")?;
        sout.flush().ok();

        let mut buf = String::new();
        sin.read_line(&mut buf).expect("Could not read your query");
        if buf.eq("\n") {
            break;
        }
        match scientist.query(buf).await {
            Ok(opinion) => {
                sout.write_all((opinion.message + "\n").as_bytes())?;
            },
            Err(msg) => eprintln!("{msg:?}"),
        };
        sout.flush().ok();
    }

    Ok(())
}
