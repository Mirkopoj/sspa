use std::env;
use std::path::Path;
use std::fs;
use std::process::Command;

extern crate unicode_segmentation;
use unicode_segmentation::UnicodeSegmentation;

const CARGOPATH: &str = "/opt/sspa";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut verbose = false;
    let mut quiet = false;
    let mut log = false;
    let mut quit = false;
    let mut log_path = None;

    let mut arg = args.iter();
    arg.next();
    while let Some(option) = arg.next() {
        match option.as_str() {
            "-u" | "--update" => {
                actualizar();
                quit = true;
                break;
            },
            "-h" | "--help" => {
                imprimir_ayuda();
                quit = true;
                break;
            },
            "-v" | "--verbose" => {
                verbose = true;
            },
            "-l" | "--logfile" => {
                match arg.next() {
                    Some(path) => {
                        if !(path.contains("-u")||
                             path.contains("--update")||
                             path.contains("-h")||
                             path.contains("--help")||
                             path.contains("-v")||
                             path.contains("--verbose")||
                             path.contains("-l")||
                             path.contains("--logfile")||
                             path.contains("-q")||
                             path.contains("--quiet")||
                             path.contains("-V")||
                             path.contains("--version")
                            ) {
                            log = true;
                            log_path = Some(path);
                            if Path::new(path).exists() { continue; }
                            let dir = (*path)
                                .graphemes(true)
                                .rev()
                                .skip_while(|x| *x != "/")
                                .collect::<String>()
                                .graphemes(true)
                                .rev()
                                .collect::<String>();
                            fs::create_dir_all(dir).expect("Could not create dirctory");
                            fs::File::create(path).expect("Could not create logfile");
                            continue;
                        } 
                    },
                    None => { },
                }
                println!("Missing [FILE]");
                println!("Try using:");
                print!("\tsspa ");
                for v in args
                         .iter()
                         .skip(1)
                         .take_while(|x| (x.as_str() != "-l") && (x.as_str() != "--logfile") )
                         .collect::<Vec<&String>>() 
                {
                             print!("{} ", v);
                }
                print!("{} <path/to/file> ", option);
                for v in args
                         .iter()
                         .skip_while(|x| (x.as_str() != "-l") && (x.as_str() != "--logfile") )
                         .skip(1)
                         .collect::<Vec<&String>>()
                {
                             print!("{} ", v);
                }
                quit = true;
                break;
            },
            "-q" | "--quiet" => {
                quiet = true;
            },
            "-V" | "--version" => {
                println!("0.0.0");
                quit = true;
                break;
            },
            _ => {
                println!("Invalid Argument: {}", option);
                quit = true;
                break;
            },
        }
    }
    if !quit{
        run(verbose, quiet, log, log_path);
    }
}

fn imprimir_ayuda(){
    println!("Automatic board tester");
    println!();
    println!("USAGE:");
    println!("\tsspa");
    println!("\tsspa [OPTION]...");
    println!("\tsspa [OPTION]... [FILE]...");
    println!();
    println!("OPTIONS:");
    println!("\t-h --help\t\tPrints this page and exit");
    println!("\t-u --update\t\tUpdates binaries and exit");
    println!("\t-v --verbose\t\tExplain what is being done");
    println!("\t-q --quiet\t\tDo no log to stdout, will overwrite --verbose");
    println!("\t-l --logfile <FILE>\tLogs output to specified file, not afected by --quiet");
    println!("\t-V --version\t\tPrints version information and exit");
    println!();
    println!("NOTE: you can uninstall the program at any time running:");
    println!("\tsspa_uninstall.sh");
    println!();
}

fn actualizar(){
    let mut child = Command::new("git")
            .arg("pull")
            .current_dir(CARGOPATH)
            .spawn()
            .expect("failed to execute git pull");

    child.wait().expect("Failed to wait on git pull");

    let mut child = Command::new("cargo")
            .arg("update")
            .current_dir(CARGOPATH)
            .spawn()
            .expect("failed to execute cargo update");

    child.wait().expect("Failed to wait on cargo update");

    let mut child = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(CARGOPATH)
            .spawn()
            .expect("failed to execute cargo build");

    child.wait().expect("Failed to wait on cargo build");

    let mut child = Command::new("sudo")
            .arg("cp")
            .arg(CARGOPATH.to_string()+"/target/release/sspa")
            .arg("/bin/sspa")
            .spawn()
            .expect("failed to add sspa to path");

    child.wait().expect("Failed to wait on cp sspa");
}

fn run(_verbose: bool, _quiet: bool, _log: bool, _log_path: Option<&String>){}
