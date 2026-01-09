use std::{collections::HashSet, fs::File, io::BufWriter, time::Duration};

use anyhow::Result;
use clap::Parser;
use jiff::civil::Date;
use octocrab::Octocrab;
use tokio::time::sleep;

use pr_history::PullRequests;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let token = args.token.or_else(|| std::env::var("GITHUB_TOKEN").ok());

    let octocrab = if let Some(token) = token {
        Octocrab::builder().personal_token(token).build()?
    } else {
        Octocrab::builder().build()?
    };

    let start = Date::new(args.year, 1, 1).unwrap();
    let end = Date::new(args.year, 12, 31).unwrap();
    let query = format!("type:pr author:{} created:{start}..{end}", args.user);
    let mut authored = HashSet::default();
    search(&query, &mut authored, &octocrab).await?;

    let mut reviewed = HashSet::default();
    for i in 1..=12 {
        let start = Date::new(args.year, i, 1).unwrap();
        let end = start.last_of_month();
        let query = format!("type:pr reviewed-by:{} created:{start}..{end}", args.user);
        search(&query, &mut reviewed, &octocrab).await?;
    }

    let file = File::create("data.json")?;
    serde_json::to_writer_pretty(
        BufWriter::new(file),
        &PullRequests {
            authored: authored.into_iter().collect(),
            reviewed: reviewed.into_iter().collect(),
        },
    )?;

    Ok(())
}

async fn search(query: &str, urls: &mut HashSet<String>, client: &Octocrab) -> anyhow::Result<()> {
    let (mut page, mut total) = (1u32, None);
    loop {
        println!("{query} -- page {page}, total {total:?}");
        let response = client
            .search()
            .issues_and_pull_requests(query)
            .per_page(100)
            .page(page)
            .send()
            .await?;

        if let Some(count) = response.total_count {
            total = Some(count);
        }

        for item in &response.items {
            urls.insert(item.html_url.to_string());
        }

        if response.next.is_none() {
            break;
        }

        page += 1;
        sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}



#[derive(Parser, Debug)]
struct Args {
    /// GitHub username to analyze
    #[arg(short, long)]
    user: String,

    #[arg(long)]
    year: i16,

    /// GitHub personal access token
    #[arg(short, long)]
    token: Option<String>,
}
