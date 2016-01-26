extern crate walkdir;

use std::path::PathBuf;
use std::fs;
use std::env;
use file_operations::FileOperations;


pub struct Shell<'a> {
    pub name: String,
    pub root_path: &'a PathBuf,
}

impl<'a> Shell<'a> {
    pub fn new<S>(name: S, root_path: &'a PathBuf) -> Shell<'a>
        where S: Into<String>
    {
        Shell {
            name: name.into(),
            root_path: root_path,
        }
    }

    pub fn root_path(&self) -> PathBuf {
        self.root_path.join("shells").join(&self.name)
    }

    pub fn path_for(&self, filename: &str) -> PathBuf {
        self.root_path().join(filename)
    }

    pub fn create_links(&self) -> FileOperations {
        let paths = fs::read_dir(self.root_path()).unwrap();
        let base_length = self.root_path().to_str().unwrap().len();
        let mut file_ops = FileOperations::rooted_at(self.root_path());
        // skip the root directory
        for entry in walkdir::WalkDir::new(self.root_path()).into_iter().skip(1) {
            let entry = entry.unwrap();
            let path = entry.path().to_str().unwrap();
            let rel_to_shell = &path[base_length+1..path.len()];
            let new_path = env::home_dir().unwrap().join(rel_to_shell);
            println!("Linking {} to {}", env::current_dir().unwrap().join(path).display(), env::current_dir().unwrap().join(&new_path).display());
            file_ops.link(path, new_path);
        }
        return file_ops;
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use super::Shell;
    use std::env;
    use file_operations::FileOperations;
    use std::fs;

    use test_helpers::filesystem::{set_up, clean_up};

    fn root_path(path_str: &str) -> PathBuf {
        PathBuf::from(path_str)
    }

    #[test]
    fn has_a_name() {
        let root_path = root_path("/");
        let s = Shell::new("my_shell", &root_path);
        assert_eq!(s.name, "my_shell");
    }

    #[test]
    fn has_a_string_name() {
        let root_path = root_path("/");
        let s = Shell::new(String::from("my_shell"), &root_path);
        assert_eq!(s.name, "my_shell");
    }

    #[test]
    fn can_resolve_its_path() {
        let root_path = root_path("/Users/geoff/.config/hermit");
        let s = Shell::new("default", &root_path);

        let expected_path = root_path.join("shells")
                                     .join("default");
        assert_eq!(s.root_path(), expected_path);
    }

    #[test]
    fn resolves_empty_string_to_root() {
        let root_path = root_path("/Users/geoff/.config/hermit");
        let s = Shell::new("default", &root_path);

        let expected_path = root_path.join("shells")
                                     .join("default");
        assert_eq!(s.path_for(""), expected_path);
    }

    #[test]
    fn can_resolve_paths() {
        let root_path = root_path("/Users/geoff/.config/hermit");
        let s = Shell::new("default", &root_path);

        let expected_path = root_path.join("shells")
                                     .join("default")
                                     .join(".bashrc");
        assert_eq!(s.path_for(".bashrc"), expected_path);
    }

    #[test]
    fn can_link_files() {
        let test_root = set_up("shell");
    
        // save original home directory
        let og_home = env::home_dir();
        env::set_var("HOME", &test_root);
        fs::create_dir(test_root.join(".config"));
        fs::create_dir(test_root.join(".config/hermit"));
        fs::create_dir(test_root.join(".config/hermit/shells"));
        fs::create_dir(test_root.join(".config/hermit/shells/default"));

        let shell_root = test_root.join(".config/hermit/shells/default");
        fs::File::create(shell_root.join(".bashrc"));
        fs::File::create(shell_root.join(".gitconfig"));
        fs::create_dir(shell_root.join("foo"));
        fs::File::create(shell_root.join("foo/bar.txt"));
        println!("Created {}", shell_root.join("foo/bar.txt").display());

        let hermit_root = test_root.join(".config/hermit");
        let s = Shell::new("default", &hermit_root);
        let file_ops = s.create_links();
        let results = file_ops.commit();
        println!("{:?}", results);
        println!("{}", results.len());
    }
}
