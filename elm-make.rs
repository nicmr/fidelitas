use std::process::Command;
use std::env;

fn main(){
    let key = "SKIP_ELM";
    match env::var_os(key) {
        Some(val) => {
            if val == "TRUE" {
                println!("{} is set to TRUE. Skipping elm file compilation.", key);
            } else {
                compile_elm();
            }
        },
        None => {
           compile_elm();
        }
    }
}

fn compile_elm() -> () {
    let output = {
        Command::new("elm")
            .arg("make")
            .arg("elm-src/Main.elm")
            .output()
            .expect("failed to compile elm files")
    };
    println!("{:#?}", output);
}