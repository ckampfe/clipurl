use anyhow::{anyhow, Context, Result};
use clap::Parser;
use copypasta::ClipboardProvider;
use std::path::PathBuf;
#[cfg(not(target_os = "macos"))]
use tracing::info;
#[cfg(target_os = "macos")]
use tracing::{debug, info};

#[cfg(target_os = "macos")]
const MACOS_PASTEBOARD_NULL_ERROR: &str = "NSPasteboard#stringForType returned null";
#[cfg(target_os = "macos")]
const MACOS_PASTEBOARD_NON_STRING_ERROR: &str =
    "NSPasteboard#types doesn't contain NSPasteboardTypeString";

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, name = "clipurl")]
struct Options {
    #[clap(short, long)]
    links_db_file: PathBuf,

    #[clap(short, long, default_value = "5000")]
    poll_interval_milliseconds: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("started, logging initialized");

    let options = Options::parse();

    info!("got options: {:?}", &options);

    let mut conn = rusqlite::Connection::open(&options.links_db_file)
        .context("Could not open link database file")?;

    info!("Connected to database: {:?}", &options.links_db_file);

    initialize_db(&mut conn)?;

    info!("Initialized database: {:?}", &options.links_db_file);

    let mut clipboard =
        copypasta::ClipboardContext::new().map_err(|_e| anyhow!("Could not set up clipboard"))?;

    info!("Initialized clipboard context");

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(
        options.poll_interval_milliseconds,
    ));

    info!("Set clipboard poll interval: {:?}", interval);

    enter_poll_loop(&mut clipboard, &conn, &mut interval).await?;

    Ok(())
}

#[tracing::instrument(err, skip_all)]
async fn enter_poll_loop(
    clipboard: &mut copypasta::ClipboardContext,
    conn: &rusqlite::Connection,
    interval: &mut tokio::time::Interval,
) -> Result<()> {
    let mut previous_clipboard_contents = String::new();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let clipboard_contents = clipboard
                    .get_contents();

                let clipboard_contents = match clipboard_contents {
                    Ok(s) => s,
                    #[cfg(target_os = "macos")]
                    Err(e) if e.to_string() == MACOS_PASTEBOARD_NULL_ERROR => {
                        debug!("This is the error the copypasta library raises when the pasteboard is empty: {}", e.to_string());
                        continue;
                    },
                    #[cfg(target_os = "macos")]
                    Err(e) if e.to_string() == MACOS_PASTEBOARD_NON_STRING_ERROR => {
                        debug!("This is the error the copypasta library raises when the pasteboard contains content that is not a string (e.g., an image): {}", e.to_string());
                        continue;
                    }
                    Err(e) => {
                        return Err(anyhow!(e)).context("Error when attempting to get clipboard contents")
                    }
                };

                if clipboard_contents == previous_clipboard_contents {
                    continue;
                }

                previous_clipboard_contents = clipboard_contents;

                match url::Url::parse(&previous_clipboard_contents) {
                    Ok(url) => {
                        write_link_to_db(conn, url).context("Could not write link to database")?;
                    }
                    Err(_e) => {
                        continue
                    },
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received SIGINT, shutting down");
                break
            }
        }
    }

    Ok(())
}

#[tracing::instrument]
fn initialize_db(conn: &mut rusqlite::Connection) -> Result<()> {
    let tx = conn.transaction()?;

    tx.execute(
        "CREATE TABLE IF NOT EXISTS links (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        link TEXT,
        inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
        [],
    )
    .context("Could not create table 'links'")?;

    tx.execute("CREATE INDEX IF NOT EXISTS link_index ON links (link)", [])
        .context("Could not create link index on table 'links'")?;

    tx.commit()?;

    Ok(())
}

#[tracing::instrument]
fn write_link_to_db(conn: &rusqlite::Connection, link: url::Url) -> Result<usize, rusqlite::Error> {
    let link_id = conn.query_row::<usize, _, _>(
        "INSERT INTO links (link)
        VALUES (?1)
        RETURNING id",
        [link.as_ref()],
        |r| r.get(0),
    )?;

    Ok(link_id)
}
