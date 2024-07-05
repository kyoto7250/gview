use std::{
    io::{BufRead, BufReader},
    path::Path,
};

use git2::{Blob, ObjectType, Repository, Tree, TreeWalkMode, TreeWalkResult};

const MAX_FILE_SIZE: usize = 16 * 1024; // 16KB

fn recursive_walk(repo: &Repository, tree: Tree) -> Vec<String> {
    let mut results: Vec<String> = vec![];
    let _ = tree.walk(TreeWalkMode::PreOrder, |_, entry| {
        if let Some(name) = entry.name() {
            if let Ok(obj) = entry.to_object(&repo) {
                match obj.kind() {
                    Some(ObjectType::Blob) => {
                        let blob = obj.peel_to_blob().unwrap();
                        let content = blob.content();
                        if content.len() < MAX_FILE_SIZE && content.is_ascii() {
                            println!("{} is text file", name);
                            let _ = print_blame(&repo, Path::new(name), blob);
                            results.push(name.to_owned());
                        }
                    }
                    _ => (),
                }
            }
        }
        TreeWalkResult::Ok
    });
    return results;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = std::env::current_dir()?;
    let repo = Repository::discover(repo_path)?;

    let head = repo.head();

    if head.is_err() {
        println!("Git repository does not exist. Bye!");
        return Ok(());
    }
    let tree = head?.peel_to_commit()?.tree()?;
    let results = recursive_walk(&repo, tree);

    Ok(())
}

fn print_blame(
    repo: &Repository,
    path: &Path,
    blob: Blob,
) -> Result<(), Box<dyn std::error::Error>> {
    let blame = repo.blame_file(path, None)?;
    println!("{:?}", blame.len());
    let reader = BufReader::new(blob.content());

    for (i, line) in reader.lines().enumerate() {
        if let (Ok(line), Some(hunk)) = (line, blame.get_line(i + 1)) {
            let signature = hunk.orig_signature();
            let author = signature.name().unwrap_or("Unknown");
            let commit_id = hunk.final_commit_id();
            let line_number = hunk.final_start_line();
            println!(
                "Line {} - Author: {}, Commit: {}, Line {}",
                line_number, author, commit_id, line
            );
        }
    }

    Ok(())
}
