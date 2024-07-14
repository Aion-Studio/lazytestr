use ignore::WalkBuilder;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_for_tests<P: AsRef<Path>>(
    dir: P,
) -> Result<Vec<(PathBuf, Vec<String>)>, Box<dyn Error>> {
    let test_regex =
        Regex::new(r"(?m)#\[(cfg\(test\)|test|(tokio::)?test)\][\s\n]*(async\s+)?fn\s+(\w+)")?;
    let mut test_info = Vec::new();

    let walker = WalkBuilder::new(dir).hidden(false).git_ignore(true).build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            let content = fs::read_to_string(path)?;
            let tests: Vec<String> = test_regex
                .captures_iter(&content)
                .filter_map(|cap| cap.get(4).map(|m| m.as_str().to_string()))
                .collect();

            if !tests.is_empty() {
                test_info.push((path.to_path_buf(), tests));
            }
        }
    }

    Ok(test_info)
}
