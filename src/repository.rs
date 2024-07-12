use git2::{Blob, Commit, ObjectType, Oid, Repository, TreeWalkMode, TreeWalkResult};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const MAX_FILE_SIZE: usize = 16 * 1024; // 16KB

pub struct RepositoryInfo {
    repository: Repository,
    oid: Oid,
}

impl RepositoryInfo {
    pub fn new() -> anyhow::Result<Self> {
        let repo_path = std::env::current_dir()?;
        let repository = Repository::discover(repo_path)?;
        let oid = repository.head()?.target().unwrap();
        Ok(Self { repository, oid })
    }

    pub fn current_commit(&mut self) -> anyhow::Result<(String, String)> {
        let commit = self.repository.find_commit(self.oid)?;
        let commit_message = commit.message().unwrap_or("No commit message");
        Ok((self.oid.to_string(), commit_message.to_owned()))
    }

    pub fn set_parent_commit(&mut self) {
        let commit = self.repository.find_commit(self.oid).unwrap();
        if commit.parent_count() > 0 {
            self.oid = commit.parent(0).unwrap().id();
        }
    }

    pub fn set_next_commit(&mut self) -> anyhow::Result<(String, String)> {
        let next_commit_id = {
            let next_commit = self.find_next_commit()?;
            if let Some(next_commit) = next_commit {
                Some(next_commit.id())
            } else {
                None
            }
        };

        if let Some(next_commit_id) = next_commit_id {
            self.oid = next_commit_id;
        }
        self.current_commit()
    }

    fn find_next_commit(&mut self) -> anyhow::Result<Option<Commit>> {
        let commit = self.repository.find_commit(self.oid)?;
        let mut revwalk = self.repository.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::REVERSE)?;

        let mut next_commit_found = false;
        for oid_result in revwalk {
            let oid = oid_result?;
            let rev_commit = self.repository.find_commit(oid)?;

            if next_commit_found {
                return Ok(Some(rev_commit));
            }

            if rev_commit.id() == commit.id() {
                next_commit_found = true;
            }
        }

        Ok(None)
    }

    pub fn get_content(&mut self, filename: String) -> anyhow::Result<String> {
        if filename == "not found".to_owned() {
            return Ok("".to_owned());
        }
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
        let head = self.repository.find_commit(self.oid)?;
        let tree = head.tree()?;
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
