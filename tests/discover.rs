use std::fs;

use lazyinstall::install::discover;
use tempfile::TempDir;

#[test]
fn it_should_discover_an_update_script_and_derive_its_name() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("update-lazygit.sh"),
        "#!/usr/bin/env bash\n",
    )
    .unwrap();

    let targets = discover::discover(tmp.path()).unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].name(), "lazygit");
    assert!(targets[0].script().ends_with("update-lazygit.sh"));
}

#[test]
fn it_should_discover_one_target_per_update_script() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("update-lazygit.sh"), "echo lazygit\n").unwrap();
    fs::write(tmp.path().join("update-nvim.sh"), "echo nvim\n").unwrap();
    fs::write(tmp.path().join("update-fzf.sh"), "echo fzf\n").unwrap();

    let targets = discover::discover(tmp.path()).unwrap();

    let names: Vec<&str> = targets.iter().map(|t| t.name()).collect();
    assert_eq!(names, ["fzf", "lazygit", "nvim"]);
}

#[test]
fn it_should_keep_only_update_scripts_when_some_are_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("aaa-helper.sh"), "echo hi\n").unwrap();
    fs::write(tmp.path().join("update-tool.sh"), "echo update\n").unwrap();

    let targets = discover::discover(tmp.path()).unwrap();

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].name(), "tool");
    assert!(targets[0].script().ends_with("update-tool.sh"));
}

#[test]
fn it_should_fall_back_to_the_first_shell_script_and_use_the_folder_name() {
    let tmp = TempDir::new().unwrap();
    let folder = tmp.path().join("mytool");
    fs::create_dir(&folder).unwrap();
    fs::write(folder.join("install.sh"), "echo install\n").unwrap();

    let targets = discover::discover(&folder).unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].name(), "mytool");
    assert!(targets[0].script().ends_with("install.sh"));
}

#[test]
fn it_should_error_when_no_shell_script_is_present() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("README.md"), "no script here").unwrap();

    assert!(discover::discover(tmp.path()).is_err());
}

#[test]
fn it_should_error_when_the_path_is_not_a_directory() {
    let tmp = TempDir::new().unwrap();
    let file = tmp.path().join("a.sh");
    fs::write(&file, "echo x\n").unwrap();

    assert!(discover::discover(&file).is_err());
}
