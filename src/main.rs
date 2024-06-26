use std::env;
use std::path::{Path, PathBuf};

use async_process::Command;
use clap::Parser;
use colored::Colorize;
use futures::{FutureExt, TryFutureExt};
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

async fn is_in_git_repository() -> bool {
    let output_res = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .await;

    if let Ok(output) = output_res {
        if output.status.success() {
            String::from_utf8(output.stdout).map_or(false, |mut x| {
                x.retain(|c| !c.is_whitespace());
                x == "true"
            })
        } else {
            return false
        }
    } else {
        return false
    }
}

async fn get_best_git_name() -> String {
    let branch_future = get_git_branch();
    let commit_future = get_git_commit();
    let tag_future = get_git_tag();

    let (branch, commit, tag) = futures::join!(branch_future, commit_future, tag_future);

    branch.unwrap_or(commit.unwrap_or("".to_owned())) + &tag.as_ref().map(|t| " [".to_string() + t + "]").unwrap_or("".to_string())
}

async fn get_git_tag() -> Option<String> {
    let output_res = Command::new("git")
        .arg("tag")
        .arg("--points-at")
        .arg("HEAD")
        .output()
        .await;

    if let Ok(output) = output_res {
        if output.status.success() {
            String::from_utf8(output.stdout).map_or(None, |mut x| {
                x.retain(|c| !c.is_whitespace());
                if x.len() == 0 {
                    None
                } else {
                    Some(x)
                }
            })
        } else {
            return None
        }
    } else {
        return None
    }
}

async fn get_git_branch() -> Option<String> {
    let output_res = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .output()
        .await;

    if let Ok(output) = output_res {
        if output.status.success() {
            String::from_utf8(output.stdout).map_or(None, |mut x| {
                x.retain(|c| !c.is_whitespace());
                if x.len() == 0 {
                    None
                } else {
                    Some(x)
                }
            })
        } else {
            return None
        }
    } else {
        return None
    }
}

async fn get_git_commit() -> Option<String> {
    let output_res = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .await;

    if let Ok(output) = output_res {
        if output.status.success() {
            String::from_utf8(output.stdout).map_or(None, |mut x| {
                x.retain(|c| !c.is_whitespace());
                if x.len() == 0 {
                    None
                } else {
                    Some(x)
                }
            })
        } else {
            return None
        }
    } else {
        return None
    }
}

enum UnstagedChanges {
    None,
    FilesChanged,
    FilesNotAdded
}

async fn get_unstaged_changes() -> UnstagedChanges {
    let output1_future = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .output();

    let output1_timed_future = tokio::time::timeout(std::time::Duration::from_millis(500), output1_future).unwrap_or_else(|e| Result::Err(e.into()));

    let output2_future = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .output();

    if let Ok((output1, output2)) = futures::try_join!(output1_timed_future, output2_future) {
        if output1.status.success() && output2.status.success() {
            let output3 = Command::new("git")
                .arg("ls-files")
                .arg("--other")
                .arg("--directory")
                .arg("--exclude-standard")
                .output()
                .await;

            if output3.map(|x| x.stdout.len() == 0).unwrap_or(false) {
                return UnstagedChanges::None;
            } else {
                return UnstagedChanges::FilesNotAdded;
            }
        } else {
            return UnstagedChanges::FilesChanged;
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

async fn get_unpushed_changes() -> UnpushedChanges {
    let output1 = Command::new("git")
        .arg("log")
        .arg("@{u}..")
        .output()
        .await;

    if output1.map(|x| x.stdout.len() == 0).unwrap_or(false) {
        let output2_future = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output();

        let output3_future = Command::new("git")
            .arg("rev-parse")
            .arg("@{u}")
            .output();

        let (output2, output3) = futures::join!(output2_future, output3_future);

        let head = String::from_utf8(output2.unwrap().stdout).map_or(None, |mut x| {
            x.retain(|c| !c.is_whitespace());
            Some(x)
        });

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

async fn get_k8s_context() -> Option<String> {
    let output = Command::new("kubectl")
        .arg("config")
        .arg("current-context")
        .output()
        .await;

    String::from_utf8(output.unwrap().stdout).map_or(None, |mut x| {
        x.retain(|c| !c.is_whitespace());
        Some(x)
    })
}

async fn get_k8s_namespace() -> Option<String> {
    let output_res = Command::new("kubectl")
        .arg("config")
        .arg("view")
        .arg("--minify")
        .arg("--output")
        .arg("jsonpath={..namespace}")
        .output()
        .await;

    if let Ok(output) = output_res {
        if output.status.success() {
            String::from_utf8(output.stdout).map_or(None, |mut x| {
                x.retain(|c| !c.is_whitespace());
                Some(x)
            })
        } else {
            return None;
        }
    } else {
        return None
    }
}

fn get_aws_profile() -> Option<String> {
    env::var("AWS_PROFILE").ok()
}

fn get_aws_region() -> Option<String> {
    env::var("AWS_REGION").ok().or(env::var("AWS_DEFAULT_REGION").ok()).or(env::var("AWS_PROFILE_REGION").ok())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Colored likes to follow the environment, however prompts appear like pipes and it disables
    // colour!
    colored::control::set_override(true);

    let current_dir = get_current_working_directory();

    let is_in_git_repostory = is_in_git_repository().await;

    let current_context_future = get_k8s_context().map(|c| c.unwrap_or("-".to_string()));
    let current_namespace_future = get_k8s_namespace().map(|c| c.unwrap_or("-".to_string()));

    let aws_profile = get_aws_profile().unwrap_or("-".to_string());
    let aws_region = get_aws_region().unwrap_or("-".to_string());

    let chevron_a = match args.exit_code {
        0 => "❯".green().bold(),
        _ => "❯".red().bold()
    };

    let current_context;
    let current_namespace;
    let current_branch;
    let chevron_b;
    let chevron_c;
    if is_in_git_repostory {
        let current_branch_future = get_best_git_name();

        let unstaged_changes_future = get_unstaged_changes();

        let unpushed_changes_future = get_unpushed_changes();

        let unstaged_changes;
        let unpushed_changes;
        (current_context, current_namespace, current_branch, unstaged_changes, unpushed_changes) = futures::join!(
            current_context_future,
            current_namespace_future,
            current_branch_future,
            unstaged_changes_future,
            unpushed_changes_future
        );

        chevron_b = match unstaged_changes {
            UnstagedChanges::None => "❯".green().bold(),
            UnstagedChanges::FilesChanged => "❯".yellow().bold(),
            UnstagedChanges::FilesNotAdded => "❯".blue().bold()
        };

        chevron_c = match unpushed_changes {
            UnpushedChanges::None => "❯".green().bold(),
            UnpushedChanges::UnpushedChanges => "❯".yellow().bold(),
            UnpushedChanges::UnpulledChanges => "❯".blue().bold(),
            UnpushedChanges::NoUpstreamBranch => "❯".white().bold()
        };
    } else {
        current_branch = "-".to_string();

        chevron_b = "❯".bold();
        chevron_c = "❯".bold();

        (current_context, current_namespace) = futures::join!(current_context_future, current_namespace_future);
    }

    let top_line = vec![
        format!("{}", current_dir.display()).cyan().bold(),
        args.message.unwrap_or("".to_string()).green().bold(),
        current_branch.purple().bold(),
        current_context.bright_blue().bold(),
        current_namespace.bright_blue().bold(),
        aws_profile.red().bold(),
        aws_region.red().bold(),
    ];

    println!(
        "\n{}\n{}{}{} ",
        top_line.iter().filter(|x| !x.is_empty()).map(|x| x.to_string()).collect::<Vec<_>>().join(" "),
        chevron_a,
        chevron_b,
        chevron_c
    );
}
