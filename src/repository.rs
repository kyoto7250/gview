use git2::{Commit, ObjectType, Oid, Repository, TreeWalkMode, TreeWalkResult};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const MAX_FILE_SIZE: usize = 16 * 1024; // 16KB

#[derive(Debug)]
pub struct CommitRow {
    pub _author: String,
    pub commit: Oid,
    pub number: usize,
    pub line: String,
}

impl CommitRow {
    pub fn new(author: String, commit: Oid, number: usize, line: String) -> CommitRow {
        Self {
            _author: author,
            commit,
            number,
            line,
        }
    }
}

pub struct RepositoryInfo {
    repository: Repository,
    oid: Oid,
}

impl std::fmt::Debug for RepositoryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepositoryInfo")
            .field("oid", &self.oid)
            .finish()
    }
}

impl RepositoryInfo {
    pub fn new() -> anyhow::Result<Self> {
        let repo_path = std::env::current_dir()?;
        let repository = Repository::discover(repo_path)?;
        let oid = repository.head()?.target().unwrap();
        Ok(Self { repository, oid })
    }

    // NOTE: this function should only be used during testing.
    pub fn _from_parts(repository: Repository, oid: Oid) -> Self {
        Self { repository, oid }
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
            next_commit.map(|next_commit| next_commit.id())
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

    pub fn get_content(&mut self, filename: String) -> anyhow::Result<Vec<CommitRow>> {
        if filename == *"not found" {
            return Ok(vec![]);
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
        let mut content = vec![];
        for (i, line) in reader.lines().enumerate() {
            if let (Ok(line), Some(hunk)) = (line, blame.get_line(i + 1)) {
                let signature = hunk.orig_signature();
                let author = signature.name().unwrap_or("Unknown");
                let commit_id = hunk.final_commit_id();
                // let line_number = hunk.final_start_line();
                let row = CommitRow::new(author.to_owned(), commit_id, i + 1, line);
                content.push(row);
            }
        }

        Ok(content)
    }
    pub fn get_commit_history(&self) -> anyhow::Result<Vec<(String, String)>> {
        let mut revwalk = self.repository.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        for oid_result in revwalk {
            let oid = oid_result?;
            let commit = self.repository.find_commit(oid)?;
            let commit_message = commit
                .message()
                .unwrap_or("No commit message")
                .lines()
                .next()
                .unwrap_or("")
                .to_string();
            commits.push((oid.to_string(), commit_message));
        }

        Ok(commits)
    }

    pub fn get_current_commit_id(&self) -> String {
        self.oid.to_string()
    }

    pub fn set_commit_by_id(&mut self, commit_id: &str) -> anyhow::Result<()> {
        let oid = if commit_id.len() == 40 {
            // Full commit ID
            git2::Oid::from_str(commit_id)?
        } else {
            // Short commit ID - need to resolve it
            let mut revwalk = self.repository.revwalk()?;
            revwalk.push_head()?;

            let mut found_oid = None;
            let mut match_count = 0;
            for oid_result in revwalk {
                let oid = oid_result?;
                let oid_str = oid.to_string();
                if oid_str.starts_with(commit_id) {
                    found_oid = Some(oid);
                    match_count += 1;
                    if match_count > 1 {
                        return Err(anyhow::anyhow!(
                            "Ambiguous commit ID: multiple commits match '{}'",
                            commit_id
                        ));
                    }
                }
            }

            found_oid.ok_or_else(|| anyhow::anyhow!(format!("Commit '{}' not found", commit_id)))?
        };

        // Verify the commit exists before setting it
        self.repository.find_commit(oid)?;
        self.oid = oid;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::Write;

    fn setup_test_repo_with_file() -> (Repository, String) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = env::temp_dir().join(format!("gview_test_repo_{}", timestamp));
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let repo = Repository::init(&test_dir).unwrap();

        // Create a test file
        let test_file_path = test_dir.join("test.txt");
        let mut file = fs::File::create(&test_file_path).unwrap();
        file.write_all(b"line 1\nline 2\nline 3\n").unwrap();

        // Add and commit the file
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@example.com",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let _ = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add test file",
            &tree,
            &[],
        );

        drop(tree);
        (repo, "test.txt".to_string())
    }

    fn setup_empty_repo() -> Repository {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = env::temp_dir().join(format!("gview_empty_repo_{}", timestamp));
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let repo = Repository::init(&test_dir).unwrap();

        let signature = git2::Signature::new(
            "Test User",
            "test@example.com",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        let _ = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        );

        drop(tree);
        repo
    }

    #[test]
    fn test_commit_row_new() {
        let oid = Oid::from_str("0123456789abcdef0123456789abcdef01234567").unwrap();
        let row = CommitRow::new(
            "test_author".to_string(),
            oid,
            42,
            "println!(\"Hello, world!\");".to_string(),
        );

        assert_eq!(row._author, "test_author");
        assert_eq!(row.commit, oid);
        assert_eq!(row.number, 42);
        assert_eq!(row.line, "println!(\"Hello, world!\");");
    }

    #[test]
    fn test_repository_info_current_commit() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.current_commit().unwrap();
        assert_eq!(result.0.len(), 40); // SHA length
        assert_eq!(result.1, "Initial commit");
    }

    #[test]
    fn test_set_commit_by_id_full_hash() {
        let (repo, _) = setup_test_repo_with_file();
        let head_commit = repo.head().unwrap().target().unwrap();
        let head_commit_str = head_commit.to_string();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        // Test setting by full commit ID
        let result = repo_info.set_commit_by_id(&head_commit_str);
        assert!(result.is_ok());
        assert_eq!(repo_info.oid, head_commit);
    }

    #[test]
    fn test_set_commit_by_id_short_hash() {
        let (repo, _) = setup_test_repo_with_file();
        let head_commit = repo.head().unwrap().target().unwrap();
        let head_commit_str = head_commit.to_string();
        let short_commit = &head_commit_str[..7]; // Use 7 characters

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        // Test setting by short commit ID
        let result = repo_info.set_commit_by_id(short_commit);
        assert!(result.is_ok());
        assert_eq!(repo_info.oid, head_commit);
    }

    #[test]
    fn test_set_commit_by_id_invalid() {
        let (repo, _) = setup_test_repo_with_file();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        // Test setting by invalid commit ID
        let result = repo_info.set_commit_by_id("invalid123");
        assert!(result.is_err());
        assert_eq!(repo_info.oid, head_commit); // Should remain unchanged
    }

    #[test]
    fn test_repository_info_set_parent_commit() {
        let (repo, _) = setup_test_repo_with_file();
        let _head_commit = repo.head().unwrap().target().unwrap();

        // Create a second commit
        let signature = git2::Signature::new(
            "Test User",
            "test@example.com",
            &git2::Time::new(1234567890, 0),
        )
        .unwrap();
        let tree = repo
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .tree()
            .unwrap();
        let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();
        let second_commit_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Second commit",
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        drop(tree);
        drop(parent_commit);

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: second_commit_oid,
        };

        let original_oid = repo_info.oid;
        repo_info.set_parent_commit();

        // Should now be pointing to the parent commit
        assert_ne!(original_oid, repo_info.oid);
    }

    #[test]
    fn test_repository_info_set_parent_commit_no_parent() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let original_oid = repo_info.oid;
        repo_info.set_parent_commit();

        // Should remain the same as it has no parent
        assert_eq!(original_oid, repo_info.oid);
    }

    #[test]
    fn test_get_content_not_found_special_case() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.get_content("not found".to_string()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_content_with_file() {
        let (repo, filename) = setup_test_repo_with_file();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.get_content(filename).unwrap();
        assert_eq!(result.len(), 3); // 3 lines
        assert_eq!(result[0].line, "line 1");
        assert_eq!(result[1].line, "line 2");
        assert_eq!(result[2].line, "line 3");
        assert_eq!(result[0].number, 1);
        assert_eq!(result[1].number, 2);
        assert_eq!(result[2].number, 3);
    }

    #[test]
    fn test_recursive_walk_empty_repo() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.recursive_walk().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_recursive_walk_with_file() {
        let (repo, _) = setup_test_repo_with_file();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.recursive_walk().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "test.txt");
    }

    #[test]
    fn test_find_next_commit_no_next() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let result = repo_info.find_next_commit().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_set_next_commit_no_next() {
        let repo = setup_empty_repo();
        let head_commit = repo.head().unwrap().target().unwrap();

        let mut repo_info = RepositoryInfo {
            repository: repo,
            oid: head_commit,
        };

        let original_oid = repo_info.oid;
        let result = repo_info.set_next_commit().unwrap();

        // Should remain the same as there's no next commit
        assert_eq!(original_oid, repo_info.oid);
        assert_eq!(result.1, "Initial commit");
    }
}
