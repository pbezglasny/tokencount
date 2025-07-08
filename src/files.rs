use glob;
use glob::Pattern;
use std::path::{Path, PathBuf};

pub struct FileContent {
    file_name: String,
    content: String,
}

pub struct FileMatchConfig {
    recursive: bool,
    include_symlinks: bool,
    include: Option<String>,
    exclude: Option<String>,
    exclude_dir: Option<String>,
}

impl FileMatchConfig {
    pub fn new(
        recursive: bool,
        include_symlinks: bool,
        include: Option<String>,
        exclude: Option<String>,
        exclude_dir: Option<String>,
    ) -> Self {
        FileMatchConfig {
            recursive,
            include_symlinks,
            include,
            exclude,
            exclude_dir,
        }
    }
}

fn option_pattern_to_glob(pattern_option: Option<String>) -> Option<Pattern> {
    pattern_option
        .map(|pattern| Pattern::new(&pattern))
        .map(|res| {
            res.map_err(|e| format!("Incorrect format of pattern: {}", e.msg))
                .unwrap()
        })
}

fn matches(pattern_opt: &Option<Pattern>, path: &Path) -> bool {
    if pattern_opt.is_none() {
        return false;
    }
    let pattern = pattern_opt.as_ref().unwrap();
    let file_name = path.file_name().unwrap().to_str().unwrap();
    pattern.matches(file_name)
}

struct PathMatcher {
    include_pattern: Option<Pattern>,
    exclude_pattern: Option<Pattern>,
    exclude_dir_pattern: Option<Pattern>,
}

impl PathMatcher {
    pub fn new(
        include: Option<String>,
        exclude: Option<String>,
        exclude_dir: Option<String>,
    ) -> Self {
        PathMatcher {
            include_pattern: option_pattern_to_glob(include),
            exclude_pattern: option_pattern_to_glob(exclude),
            exclude_dir_pattern: option_pattern_to_glob(exclude_dir),
        }
    }

    /// Checks if the file should be included based on the patterns.
    /// Returns true if there are no patterns or the file matches the include pattern and
    /// does not match the exclude pattern.
    fn should_file_be_included(&self, path: &Path) -> bool {
        if self.include_pattern.is_none() && self.exclude_pattern.is_none() {
            if path.is_file() {
                true // No patterns, include all files
            } else {
                !matches(&self.exclude_dir_pattern, path)
            }
        } else if self.include_pattern.is_some() && path.is_file() {
            matches(&self.include_pattern, path)
        } else {
            if path.is_file() {
                !matches(&self.exclude_pattern, path)
            } else {
                !matches(&self.exclude_pattern, path) && !matches(&self.exclude_dir_pattern, path)
            }
        }
    }
}

fn get_folder_content(path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(path)
        .into_iter()
        .flat_map(|dir| {
            dir.filter(|entry| entry.is_ok())
                .map(|entry| entry.unwrap().path())
        })
        .collect();
    files
}

pub fn get_matched_files(
    initial_files: Vec<String>,
    file_match_config: FileMatchConfig,
) -> Vec<PathBuf> {
    let path_matcher = PathMatcher::new(
        file_match_config.include,
        file_match_config.exclude,
        file_match_config.exclude_dir,
    );
    let mut result: Vec<PathBuf> = initial_files
        .into_iter()
        .flat_map(|file| {
            if !std::fs::exists(&file).unwrap_or(false) {
                panic!("Path {} does not exists", &file)
            }
            let path = Path::new(&file);
            if path.is_file() {
                return if path_matcher.should_file_be_included(path) {
                    vec![path.to_path_buf()]
                } else {
                    Vec::new()
                };
            }
            if path.is_dir() && !file_match_config.recursive {
                panic!("Path {} is directory and recursive flag is false", file);
            }
            let mut folder_stack: Vec<PathBuf> = Vec::new();
            folder_stack.push(path.to_path_buf());
            let mut found_files = Vec::new();

            while !folder_stack.is_empty() {
                let top_folder = folder_stack.pop().unwrap();
                if !path_matcher.should_file_be_included(&top_folder) {
                    continue;
                }
                let folder_content = get_folder_content(&top_folder);
                for entry in folder_content {
                    if entry.is_symlink() && !file_match_config.include_symlinks {
                        continue;
                    }
                    if entry.is_dir() {
                        if path_matcher.should_file_be_included(&entry) {
                            folder_stack.push(entry);
                        }
                    } else {
                        if path_matcher.should_file_be_included(&entry) {
                            found_files.push(entry);
                        }
                    }
                }
            }
            found_files
        })
        .collect();
    result.sort_by(|path1, path2| {
        let component_num1 = path1.components().count();
        let component_num2 = path2.components().count();
        if component_num1 == component_num2 {
            path1.cmp(path2)
        } else {
            component_num1.cmp(&component_num2)
        }
    });
    result
}

mod tests {
    use super::*;

    #[test]
    fn test_get_matched_files() {
        let top_folder = std::env::current_dir().unwrap();
        let test_data = top_folder.join("resources").join("test_data");
        let files = vec![format!("{}", test_data.to_str().unwrap().to_owned())];
        let config = FileMatchConfig::new(true, false, None, None, None);
        let matched_files = get_matched_files(files, config);
        assert!(!matched_files.is_empty());
        let expected_files = vec![
            test_data.join("a.txt"),
            test_data.join("b.json"),
            test_data.join("a_folder").join("aa.txt"),
            test_data.join("a_folder").join("ab.txt"),
            test_data.join("b_folder").join("ba.txt"),
            test_data.join("b_folder").join("bb.txt"),
        ];
        assert_eq!(matched_files, expected_files);
    }

    #[test]
    fn test_get_matched_files_symlinks() {
        let top_folder = std::env::current_dir().unwrap();
        let test_data = top_folder.join("resources").join("test_data");
        let files = vec![format!("{}", test_data.to_str().unwrap().to_owned())];
        let config = FileMatchConfig::new(true, true, None, None, None);
        let matched_files = get_matched_files(files, config);
        assert!(!matched_files.is_empty());
        let expected_files = vec![
            test_data.join("a.txt"),
            test_data.join("b.json"),
            test_data.join("a_folder").join("aa.txt"),
            test_data.join("a_folder").join("ab.txt"),
            test_data.join("b_folder").join("ba.txt"),
            test_data.join("b_folder").join("bb.txt"),
            test_data.join("c_folder").join("ba.txt"),
            test_data.join("c_folder").join("bb.txt"),
        ];
        assert_eq!(matched_files, expected_files);
    }

    #[test]
    fn test_get_matched_files_include() {
        let top_folder = std::env::current_dir().unwrap();
        let test_data = top_folder.join("resources").join("test_data");
        let files = vec![format!("{}", test_data.to_str().unwrap().to_owned())];
        let config = FileMatchConfig::new(true, false, Some("*.txt".to_owned()), None, None);
        let matched_files = get_matched_files(files, config);
        assert!(!matched_files.is_empty());
        let expected_files = vec![
            test_data.join("a.txt"),
            test_data.join("a_folder").join("aa.txt"),
            test_data.join("a_folder").join("ab.txt"),
            test_data.join("b_folder").join("ba.txt"),
            test_data.join("b_folder").join("bb.txt"),
        ];
        assert_eq!(matched_files, expected_files);
    }

    #[test]
    fn test_get_matched_files_exlude() {
        let top_folder = std::env::current_dir().unwrap();
        let test_data = top_folder.join("resources").join("test_data");
        let files = vec![format!("{}", test_data.to_str().unwrap().to_owned())];
        let config = FileMatchConfig::new(true, false, None, Some("*.txt".to_string()), None);
        let matched_files = get_matched_files(files, config);
        assert!(!matched_files.is_empty());
        let expected_files = vec![test_data.join("b.json")];
        assert_eq!(matched_files, expected_files);
    }

    #[test]
    fn test_get_matched_files_eclude_dir() {
        let top_folder = std::env::current_dir().unwrap();
        let test_data = top_folder.join("resources").join("test_data");
        let files = vec![format!("{}", test_data.to_str().unwrap().to_owned())];
        let config = FileMatchConfig::new(true, false, None, None, Some("a_*".to_string()));
        let matched_files = get_matched_files(files, config);
        assert!(!matched_files.is_empty());
        let expected_files = vec![
            test_data.join("a.txt"),
            test_data.join("b.json"),
            test_data.join("b_folder").join("ba.txt"),
            test_data.join("b_folder").join("bb.txt"),
        ];
        assert_eq!(matched_files, expected_files);
    }
}
