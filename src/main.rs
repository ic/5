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
            .max_tokens(384u16) // Enough for 5x50 words (but 1 word is not 1 token, so just a
                                // shortcut for now).
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

    let mut socrates = OneSelf {
        personality: String::from("Please always answer within 50 words."),
    };

    let mut social = OneSelf {
        personality: String::from("Please answer as an expert of intuitive Psychology, always within 50 words."),
    };

    let mut narrative = OneSelf {
        personality: String::from("Please answer with solid causal modelling, always within 50 words."),
    };

    let mut volition = OneSelf {
        personality: String::from("Please answer by your own will, always within 50 words."),
    };

    let mut physics = OneSelf {
        personality: String::from("Please answer as expert in intuitive Physics, always within 50 words."),
    };

    let mut embodied = OneSelf {
        personality: String::from("Please answer as a down-to-earth Artisan, always within 50 words."),
    };

    loop {
        sout.write_all(b"Please ask a question (just press enter to exit): ")?;
        sout.flush().ok();

        let mut buf = String::new();
        sin.read_line(&mut buf).expect("Could not read your query.\n");
        if buf.eq("\n") {
            break;
        }

        // TODO Use tokio::spawn for parallel processing. Requires removing the Box dyn
        let (social_op, narrative_op, volitional_op, physical_op, embodied_op) = tokio::join!(
            social.query(buf.clone()),
            narrative.query(buf.clone()),
            volition.query(buf.clone()),
            physics.query(buf.clone()),
            embodied.query(buf.clone()),
        );

        let mut say_to_socrates = String::new();
        say_to_socrates.push_str(
            format!("\"\"\"Question: {}\"\"\"\n", buf.clone()).as_str()
        );
        say_to_socrates.push_str(
            format!("\"\"\"Social opinion: {}\"\"\"\n", social_op.unwrap_or(Opinion {message: "No opinion".to_string()}).message).as_str()
        );
        say_to_socrates.push_str(
            format!("\"\"\"Narrative opinion: {}\"\"\"\n", narrative_op.unwrap_or(Opinion {message: "No opinion".to_string()}).message).as_str()
        );
        say_to_socrates.push_str(
            format!("\"\"\"Volitional opinion: {}\"\"\"\n", volitional_op.unwrap_or(Opinion {message: "No opinion".to_string()}).message).as_str()
        );
        say_to_socrates.push_str(
            format!("\"\"\"Physical opinion: {}\"\"\"\n", physical_op.unwrap_or(Opinion {message: "No opinion".to_string()}).message).as_str()
        );
        say_to_socrates.push_str(
            format!("\"\"\"Bodily opinion: {}\"\"\"\n", embodied_op.unwrap_or(Opinion {message: "No opinion".to_string()}).message).as_str()
        );
        say_to_socrates.push_str("What do you conclude? Be laconic.");

        match socrates.query(say_to_socrates).await {
            Ok(opinion) => {
                sout.write_all((opinion.message + "\n").as_bytes())?;
            },
            Err(msg) => eprintln!("{msg:?}"),
        };

        sout.flush().ok();
    }

    Ok(())
}
