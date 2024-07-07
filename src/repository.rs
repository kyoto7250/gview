use git2::{Blob, ObjectType, Repository, TreeWalkMode, TreeWalkResult};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const MAX_FILE_SIZE: usize = 16 * 1024; // 16KB

pub struct RepositoryInfo {
    repository: Repository,
}

impl RepositoryInfo {
    pub fn new() -> anyhow::Result<Self> {
        let repo_path = std::env::current_dir()?;
        let repository = Repository::discover(repo_path)?;
        Ok(Self {
            repository: repository,
        })
    }

    pub fn current_commit(&mut self) -> anyhow::Result<(String, String)> {
        let head = self.repository.head()?.peel_to_commit()?;
        let commit_id = head.id();
        let commit_message = head.message().unwrap_or("No commit message");

        Ok((commit_id.to_string(), commit_message.to_owned()))
    }

    pub fn get_content(&mut self, filename: String) -> anyhow::Result<String> {
        let path = Path::new(&filename);
        let blame = self.repository.blame_file(path, None)?;
        let commit = self.repository.head()?.peel_to_commit()?;
        let tree = commit.tree()?;
        let blob = tree
            .get_path(path)?
            .to_object(&self.repository)?
            .peel_to_blob()?;
        let reader = BufReader::new(blob.content());
        let mut content = String::from("");
        for (i, line) in reader.lines().enumerate() {
            if let (Ok(line), Some(hunk)) = (line, blame.get_line(i + 1)) {
                let signature = hunk.orig_signature();
                let author = signature.name().unwrap_or("Unknown");
                let commit_id = hunk.final_commit_id();
                let line_number = hunk.final_start_line();
                content += &format!("{}\n", line);
            }
        }

        Ok(content)
    }
    pub fn recursive_walk(&mut self) -> anyhow::Result<Vec<String>> {
        let head = self.repository.head();
        let tree = head?.peel_to_commit()?.tree()?;
        let mut results: Vec<String> = vec![];
        let mut path_stack: Vec<PathBuf> = vec![PathBuf::new()];
        let _ = tree.walk(TreeWalkMode::PreOrder, |root, entry| {
            if let Some(name) = entry.name() {
                let mut current_path = PathBuf::from(root);
                current_path.push(name);

                if let Ok(obj) = entry.to_object(&self.repository) {
                    match obj.kind() {
                        Some(ObjectType::Blob) => {
                            let blob = obj.peel_to_blob().unwrap();
                            let content = blob.content();
                            if content.len() < MAX_FILE_SIZE && content.is_ascii() {
                                results.push(current_path.to_string_lossy().to_string());
                                print_blame(&self.repository, &current_path, blob);
                            }
                        }
                        Some(ObjectType::Tree) => {
                            path_stack.push(current_path.clone());
                        }
                        _ => (),
                    }
                }
            }
            TreeWalkResult::Ok
        });

        Ok(results)
    }
}

fn print_blame(
    repo: &Repository,
    path: &Path,
    blob: Blob,
) -> Result<(), Box<dyn std::error::Error>> {
    let blame = repo.blame_file(path, None)?;
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
