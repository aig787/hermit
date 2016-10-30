use std::io;
use std::rc::Rc;

use config::Config;
use file_operations::FileOperations;
use message;
use shell::Shell;

#[derive(Copy,Clone)]
#[derive(PartialEq,Eq)]
#[derive(Debug)]
pub enum Error {
    ShellDoesNotExist,
}

impl From<io::Error> for Error {
    fn from(_err: io::Error) -> Error {
        Error::ShellDoesNotExist
    }
}

pub struct Hermit<T: Config> {
    config: Rc<T>,
}

impl<T: Config> Hermit<T> {
    pub fn new(config: T) -> Hermit<T> {
        Hermit { config: Rc::new(config) }
    }

    pub fn current_shell(&self) -> Option<Shell<T>> {
        self.config
            .current_shell_name()
            .map(|shell_name| Shell::new(shell_name, self.config.clone()))
    }

    pub fn set_current_shell(&mut self, name: &str) -> Result<(), Error> {
        if self.config.does_shell_exist(name) {
            match Rc::get_mut(&mut self.config) {
                Some(config) => config.set_current_shell_name(name).map_err(From::from),
                None => {
                    unreachable!(message::error("attempted to modify config while it was being \
                                                 used."))
                }
            }
        } else {
            Err(Error::ShellDoesNotExist)
        }
    }

    pub fn init_shell(&self, file_ops: &mut FileOperations, name: &str) {
        let new_shell = Shell::new(name, self.config.clone());
        let path = new_shell.root_path();
        println!("creating {}", path.to_str().unwrap());
        file_ops.create_git_repo(path);
    }

    pub fn nuke_shell(&self, file_ops: &mut FileOperations, name: &str) -> Result<(), Error>  {
        println!("========={}===========", self.config.shell_root_path().join(name).to_str().unwrap());
        if (!self.config.does_shell_exist(name)) {
            return Err(Error::ShellDoesNotExist);
        }
        let shell_path = self.config.shell_root_path().join(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::rc::Rc;

    use config::mock::MockConfig;
    use file_operations::FileOperations;
    use file_operations::Op;

    use super::{Error, Hermit};

    fn hermit(config: &MockConfig) -> Hermit<MockConfig> {
        Hermit::new(config.clone())
    }

    fn mock_config() -> MockConfig {
        MockConfig {
            root_path: PathBuf::from("/"),
            allowed_shell_names: vec!["default".to_owned()],
            current_shell: "default".to_owned(),
        }
    }

    #[test]
    fn returns_the_current_shell() {
        let config = mock_config();
        let hermit = hermit(&config);

        let shell = hermit.current_shell().unwrap();
        assert_eq!(shell.name, "default");
        assert_eq!(shell.config, Rc::new(config));
    }

    #[test]
    fn can_set_the_current_shell() {
        let mut config = mock_config();
        config.current_shell = "current".to_owned();
        let mut hermit = hermit(&config);

        assert_eq!(hermit.current_shell().unwrap().name, "current");
        assert!(hermit.set_current_shell("default").is_ok());
        assert_eq!(hermit.current_shell().unwrap().name, "default");
    }

    #[test]
    fn cant_set_the_current_shell_to_a_nonexistent_shell() {
        let config = mock_config();
        let mut hermit = hermit(&config);

        assert_eq!(hermit.current_shell().unwrap().name, "default");
        let res = hermit.set_current_shell("non-existent");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), Error::ShellDoesNotExist);
    }

    #[test]
    fn can_initialize_a_new_shell() {
        let config = MockConfig {
            root_path: PathBuf::from(".hermit-config"),
            allowed_shell_names: vec!["default".to_owned()],
            current_shell: "default".to_owned(),
        };
        let hermit = hermit(&config);
        let mut file_ops = FileOperations::rooted_at("/home/geoff");

        hermit.init_shell(&mut file_ops, "new-one");
        let first_op = &file_ops.operations[0];
        assert_eq!(*first_op,
                   Op::GitInit(PathBuf::from("/home/geoff/.hermit-config/shells/new-one")));
    }

    #[test]
    fn does_not_delete_nonexistent_shell() {
        let config = mock_config();
        let mut hermit = hermit(&config);
        let mut file_ops = FileOperations::rooted_at("/home/agriffin");

        let res = hermit.nuke_shell(&mut file_ops, "non-existent");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), Error::ShellDoesNotExist);
    }

    #[test]
    fn can_delete_shell() {
        let config = MockConfig {
            root_path: PathBuf::from(".hermit-config"),
            allowed_shell_names: vec!["default".to_owned()],
            current_shell: "default".to_owned(),
        };
        let mut hermit = hermit(&config);
        let test_root = set_up("git-re-init");
        let mut file_ops_init = FileOperations::rooted_at(&test_root);

        let mut file_ops_init = FileOperations::rooted_at("/home/agriffin");

        hermit.init_shell(&mut file_ops_init, "new-one");
        let init_res = file_ops_init.commit();
        println!("OUTPUT");

        let mut file_ops = FileOperations::rooted_at("hom");
        let res = hermit.nuke_shell(&mut file_ops, "new-one");
        println!("error {:?}", res.err());
        assert!(!res.is_err())
    }
}
