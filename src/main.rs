use anyhow::{anyhow, Context, Result};
use copypasta::ClipboardProvider;
use rusqlite::params;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    links_db_file: PathBuf,

    #[structopt(short, long, default_value = "5000")]
    poll_interval_milliseconds: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let options = Options::from_args();

    let conn = rusqlite::Connection::open(&options.links_db_file)
        .context("Could not open link database file")?;

    initialize_db(&conn).await?;

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
                        write_link_to_db(&conn, url).await.context("Could not write link to database")?;
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

async fn initialize_db(conn: &rusqlite::Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        link TEXT,
        inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
        [],
    )
    .context("Could not create table 'links'")?;

    conn.execute("CREATE INDEX IF NOT EXISTS link_index ON links (link)", [])
        .context("Could not create link index on table 'links'")?;

    Ok(())
}

async fn write_link_to_db(
    conn: &rusqlite::Connection,
    link: url::Url,
) -> Result<usize, rusqlite::Error> {
    let link_id = conn.query_row::<usize, _, _>(
        "INSERT INTO links (link)
        VALUES (?1)
        RETURNING id",
        params![link.to_string()],
        |r| r.get(0),
    )?;

    Ok(link_id)
}
