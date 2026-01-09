use std::{
    collections::{HashMap, HashSet},
    fs::File,
    hash::RandomState,
    io::BufReader,
    path::PathBuf,
};

use clap::Parser;

use pr_history::PullRequests;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = File::open(args.input)?;
    let data = serde_json::from_reader::<_, PullRequests>(BufReader::new(file))?;

    let authored = HashSet::<_, RandomState>::from_iter(data.authored.into_iter());
    let mut reviewed = HashSet::<_, RandomState>::from_iter(data.reviewed.into_iter());
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

    let mut repos = repos.into_iter().collect::<Vec<_>>();
    repos.sort_by_key(|(_, (authored, review))| (usize::MAX - (authored * 2), usize::MAX - review));
    for (repo, (authored, review)) in repos {
        println!("{repo:>50}: {authored:>5} | {review:>5}");
    }

    Ok(())
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
}
