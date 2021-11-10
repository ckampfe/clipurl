use anyhow::{anyhow, Result};
use copypasta::ClipboardProvider;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    log_file: PathBuf,

    #[structopt(short, long, default_value = "5000")]
    poll_interval_milliseconds: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(options.log_file)
        .await?;

    let mut ctx =
        copypasta::ClipboardContext::new().map_err(|_e| anyhow!("Could not set up clipboard"))?;

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        options.poll_interval_milliseconds,
    ));

    let mut previous_clipboard_contents = String::new();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let clipboard_contents = ctx
                    .get_contents()
                    .map_err(|_e| anyhow!("Could not get clipboard contents"))?;

                if clipboard_contents == previous_clipboard_contents {
                    continue;
                } else {
                    previous_clipboard_contents = clipboard_contents;
                }

                match url::Url::parse(&previous_clipboard_contents) {
                    Ok(url) => {
                        let mut out_string = url.to_string();
                        out_string.push('\n');
                        file.write_all(out_string.as_bytes()).await?
                    }
                    Err(_e) => {
                        continue
                    },
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break
            }
        }
    }

    Ok(())
}
