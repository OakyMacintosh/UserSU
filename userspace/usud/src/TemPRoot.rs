use std::process::Command;

let CheckPRoot = Command::new("which proot")
    .output()
    .expect("Failed to execute command");

let PRoot = Command::new("proot")
    .arg("-r $USERSUROOT/temproot/default")
    .output()
    .expect("Failed to execute proot!")

