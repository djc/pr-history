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

        let authored_url = search_link(&repo, "author", &args.user, args.start, args.end);
        let reviewed_url = search_link(&repo, "reviewed-by", &args.user, args.start, args.end);

        if args.mode == Mode::Rest {
            println!("   * - `{repo} <https://github.com/{repo}>`__");
            println!("     - `{authored} <{authored_url}>`__");
            println!("     - `{review} <{reviewed_url}>`__");
        } else {
            println!("{repo}:");
            println!("  {authored:02} ({authored_url})");
            println!("  {review:02} ({reviewed_url})");
            println!();
        }
    }

    println!("PROJECTS: {}", totals.0);
    println!("TOTAL AUTHORED: {}", totals.1);
    println!("TOTAL REVIEWED: {}", totals.2);

    Ok(())
}

fn search_link(repo: &str, r#type: &str, user: &str, start: Date, end: Date) -> String {
    let mut url = "https://github.com/search?q=is%3Apr".to_owned();
    url.write_fmt(format_args!("+{type}%3A{user}")).unwrap();
    let (org, repo) = repo.split_once('/').unwrap();
    url.write_fmt(format_args!("+repo%3A{org}%2F{repo}"))
        .unwrap();
    url.write_fmt(format_args!("+created%3A{start}..{end}"))
        .unwrap();
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
    input: PathBuf,
    #[clap(short, long)]
    user: String,
    #[clap(long, default_value = "plain")]
    mode: Mode,
    #[arg(long)]
    start: Date,
    #[arg(long)]
    end: Date,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Mode {
    Rest,
    #[default]
    Plain,
}

impl FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rest" => Ok(Self::Rest),
            "plain" => Ok(Self::Plain),
            _ => Err(format!("invalid mode: {s}")),
        }
    }
}
