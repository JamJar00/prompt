use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use colored::Colorize;
use homedir::get_my_home;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    exit_code: u8,

    #[arg(short, long)]
    message: Option<String>,
}

fn get_current_working_directory() -> PathBuf {
    let current_dir = env::current_dir().expect("No current working directory?");

    if let Ok(Some(home_dir)) = get_my_home() {
        if current_dir.starts_with(&home_dir) {
            return Path::new("~").join(current_dir.strip_prefix(&home_dir).unwrap());
        }

        return current_dir;
    }

    current_dir
}

fn is_in_git_repository() -> bool {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();

    if output.is_err() {
        return false
    }

    String::from_utf8(output.unwrap().stdout).map_or(false, |mut x| {
        x.retain(|c| !c.is_whitespace());
        x == "true"
    })
}

fn get_best_git_name() -> String {
    let branch = get_git_branch();
    let tag = get_git_tag();

    branch.unwrap_or("".to_owned()) + &tag.as_ref().map(|t| " [".to_string() + t + "]").unwrap_or("".to_string())
}

fn get_git_tag() -> Option<String> {
    let output = Command::new("git")
        .arg("tag")
        .arg("--points-at")
        .arg("HEAD")
        .output();

    if output.is_err() {
        return Option::None
    }

    String::from_utf8(output.unwrap().stdout).map_or(None, |mut x| {
        x.retain(|c| !c.is_whitespace());
        Some(x)
    })
}

fn get_git_branch() -> Option<String> {
    let output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output();

    if output.is_err() {
        return Option::None
    }

    String::from_utf8(output.unwrap().stdout).map_or(None, |mut x| {
        x.retain(|c| !c.is_whitespace());
        Some(x)
    })
}

enum UnstagedChanges {
    None,
    FilesChanged,
    FilesNotAdded
}

fn get_unstaged_changes() -> UnstagedChanges {
    let output1 = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .output();

    let output2 = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .output();

    if output1.is_ok() && output2.is_ok() {
        let output3 = Command::new("git")
            .arg("ls-files")
            .arg("--other")
            .arg("--directort")
            .arg("--exclude-standard")
            .output();

        if output3.map(|x| x.stdout.len() == 0).unwrap_or(false) {
            return UnstagedChanges::None;
        } else {
            return UnstagedChanges::FilesNotAdded;
        }
    } else {
        return UnstagedChanges::FilesChanged;
    }
}

enum UnpushedChanges {
    None,
    UnpushedChanges,
    UnpulledChanges,
    NoUpstreamBranch
}

fn get_unpushed_changes() -> UnpushedChanges {
    let output1 = Command::new("git")
        .arg("log")
        .arg("@{u}..")
        .output();

    if output1.map(|x| x.stdout.len() == 0).unwrap_or(false) {
        let output2 = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output();

        let head = String::from_utf8(output2.unwrap().stdout).map_or(None, |mut x| {
            x.retain(|c| !c.is_whitespace());
            Some(x)
        });

        let output3 = Command::new("git")
            .arg("rev-parse")
            .arg("@{u}")
            .output();

        let u = String::from_utf8(output3.unwrap().stdout).map_or(None, |mut x| {
            x.retain(|c| !c.is_whitespace());
            if x.len() == 0 {
                None
            } else {
                Some(x)
            }
        });

        if u.is_none() {
            return UnpushedChanges::NoUpstreamBranch;
        } else if head == u {
            return UnpushedChanges::None;
        } else {
            return UnpushedChanges::UnpulledChanges;
        }
    } else {
        return UnpushedChanges::UnpushedChanges;
    }
}

fn get_k8s_context() -> Option<String> {
    let output = Command::new("kubectl")
        .arg("config")
        .arg("current-context")
        .output();

    if output.is_err() {
        return Option::None
    }

    String::from_utf8(output.unwrap().stdout).map_or(None, |mut x| {
        x.retain(|c| !c.is_whitespace());
        Some(x)
    })
}

fn get_k8s_namespace() -> Option<String> {
    let output = Command::new("kubectl")
        .arg("config")
        .arg("view")
        .arg("--minify")
        .arg("--output")
        .arg("jsonpath={..namespace}")
        .output();

    if output.is_err() {
        return Option::None
    }

    String::from_utf8(output.unwrap().stdout).map_or(None, |mut x| {
        x.retain(|c| !c.is_whitespace());
        Some(x)
    })
}

fn get_aws_profile() -> Option<String> {
    env::var("AWS_PROFILE").ok()
}

fn get_aws_region() -> Option<String> {
    env::var("AWS_REGION").ok().or(env::var("AWS_DEFAULT_REGION").ok()).or(env::var("AWS_PROFILE_REGION").ok())
}

fn main() {
    let args = Args::parse();

    // Colored likes to follow the environment, however prompts appear like pipes and it disables
    // colour!
    colored::control::set_override(true);

    let current_dir = get_current_working_directory();

    let is_in_git_repostory = is_in_git_repository();

    let current_branch;
    if is_in_git_repostory {
        current_branch = get_best_git_name();
    } else {
        current_branch = "-".to_string();
    }

    let current_context = get_k8s_context().unwrap_or("-".to_string());
    let current_namespace = get_k8s_namespace().unwrap_or("-".to_string());

    let aws_profile = get_aws_profile().unwrap_or("-".to_string());
    let aws_region = get_aws_region().unwrap_or("-".to_string());

    let chevron_a = match args.exit_code {
        0 => "❯".green().bold(),
        _ => "❯".red().bold()
    };

    let chevron_b;
    let chevron_c;
    if is_in_git_repostory {
        let unstaged_changes = get_unstaged_changes();
        chevron_b = match unstaged_changes {
            UnstagedChanges::None => "❯".green().bold(),
            UnstagedChanges::FilesChanged => "❯".yellow().bold(),
            UnstagedChanges::FilesNotAdded => "❯".blue().bold()
        };

        let unpushed_changes = get_unpushed_changes();
        chevron_c = match unpushed_changes {
            UnpushedChanges::None => "❯".green().bold(),
            UnpushedChanges::UnpushedChanges => "❯".yellow().bold(),
            UnpushedChanges::UnpulledChanges => "❯".blue().bold(),
            UnpushedChanges::NoUpstreamBranch => "❯".white().bold()
        };
    } else {
        chevron_b = "❯".bold();
        chevron_c = "❯".bold();
    }

    println!(
        "\n{} {} {} {} {} {} {}\n{}{}{} ",
        format!("{}", current_dir.display()).cyan().bold(),
        args.message.unwrap_or("".to_string()).green().bold(),
        current_branch.purple().bold(),
        current_context.bright_blue().bold(),
        current_namespace.bright_blue().bold(),
        aws_profile.red().bold(),
        aws_region.red().bold(),
        chevron_a,
        chevron_b,
        chevron_c
    );
}
