use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::File,
    hash::RandomState,
    io::BufReader,
    path::PathBuf,
    str::FromStr,
};

use clap::Parser;
use jiff::civil::Date;

use pr_history::PullRequests;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = File::open(args.input)?;
    let data = serde_json::from_reader::<_, PullRequests>(BufReader::new(file))?;
    let meta = Metadata {
        user: data.user,
        start: data.start,
        end: data.end,
    };

    let authored = HashSet::<_, RandomState>::from_iter(data.authored);
    let mut reviewed = HashSet::<_, RandomState>::from_iter(data.reviewed);
    for url in &authored {
        reviewed.remove(url);
    }

    let mut repos = HashMap::<String, (usize, usize)>::default();
    for url in &authored {
        repos.entry(project(url).unwrap()).or_default().0 += 1;
    }

    for url in &reviewed {
        repos.entry(project(url).unwrap()).or_default().1 += 1;
    }

    if args.mode == Mode::Rest {
        println!(".. list-table:: Pull requests");
        println!("   :header-rows: 1");
        println!("   :widths: auto");
        println!();
        println!("   * - Repostory");
        println!("     - Auth");
        println!("     - Rev");
    }

    let mut repos = repos.into_iter().collect::<Vec<_>>();
    repos.sort_by_key(|(_, (authored, review))| (usize::MAX - (authored * 2), usize::MAX - review));
    let mut totals = (repos.len(), 0, 0);
    for (repo, (authored, review)) in repos {
        totals.1 += authored;
        totals.2 += review;

        let authored_url = search_link(&repo, "author", &meta);
        let reviewed_url = search_link(&repo, "reviewed-by", &meta);

        match args.mode {
            Mode::Markdown => {
                println!("* [{repo}](https://github.com/{repo}):");
                println!("  * [Authored]({authored_url}): {authored}");
                println!("  * [Reviewed]({reviewed_url}): {review}");
                println!();
            }
            Mode::Rest => {
                println!("   * - `{repo} <https://github.com/{repo}>`__");
                println!("     - `{authored} <{authored_url}>`__");
                println!("     - `{review} <{reviewed_url}>`__");
            }
            Mode::Plain => {
                println!("{repo}:");
                println!("  {authored:02} ({authored_url})");
                println!("  {review:02} ({reviewed_url})");
                println!();
            }
        }
    }

    println!("PROJECTS: {}", totals.0);
    println!("TOTAL AUTHORED: {}", totals.1);
    println!("TOTAL REVIEWED: {}", totals.2);

    Ok(())
}

fn search_link(repo: &str, r#type: &str, data: &Metadata) -> String {
    let mut url = "https://github.com/search?q=is%3Apr".to_owned();
    url.write_fmt(format_args!("+{type}%3A{}", data.user))
        .unwrap();
    let (org, repo) = repo.split_once('/').unwrap();
    url.write_fmt(format_args!("+repo%3A{org}%2F{repo}"))
        .unwrap();
    url.write_fmt(format_args!("+created%3A{}..{}", data.start, data.end))
        .unwrap();
    if r#type == "reviewed-by" {
        url.push_str("+-author%3Aapp%2Fdependabot");
    }
    url
}

fn project(url: &str) -> Option<String> {
    let url = url.strip_prefix("https://github.com/").unwrap();
    let mut parts = url.splitn(3, '/');
    let org = parts.next()?;
    let repo = parts.next()?;
    Some(format!("{org}/{repo}"))
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(default_value = "data.json")]
    input: PathBuf,
    #[clap(long, default_value = "markdown")]
    mode: Mode,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Mode {
    Markdown,
    Rest,
    #[default]
    Plain,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md" | "markdown" => Ok(Self::Markdown),
            "rest" => Ok(Self::Rest),
            "plain" => Ok(Self::Plain),
            _ => Err(format!("invalid mode: {s}")),
        }
    }
}

pub struct Metadata {
    pub user: String,
    pub start: Date,
    pub end: Date,
}
