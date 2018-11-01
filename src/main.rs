extern crate clap;
#[macro_use]
extern crate failure;
extern crate regex;

use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::{env, fs};

use failure::Fallible;
use regex::Regex;

struct Settings {
    dry: bool,
    copy: bool,
    force: bool,
    interactive: bool,
    no_clobber: bool,
    verbose: bool,
}

struct Repl<'a> {
    expr: Regex,
    repl: &'a str,
}

fn cli() -> clap::App<'static, 'static> {
    use clap::{App, AppSettings, Arg, Shell};

    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::DisableHelpSubcommand)
        // Flags.
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose output")
        )
        .arg(
            Arg::with_name("dry")
                .short("d")
                .long("dry")
                .help("Run as a dry run, without renaming any files"),
        )
        .arg(
            Arg::with_name("copy")
                .short("c")
                .long("copy")
                .help("Copy files instead of renaming them"),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("Do not prompt before overwriting existing files")
                .conflicts_with_all(&["interactive", "no-clobber"]),
        )
        .arg(
            Arg::with_name("interactive")
                .short("i")
                .long("interactive")
                .help("Prompt before each file is renamed, the program will prompt regardless if a file would be overwritten"),
        )
        .arg(
            Arg::with_name("no-clobber")
                .short("n")
                .long("no-clobber")
                .help("Do not overwrite existing files"),
        )
        // Options.
        .arg(
            Arg::with_name("completions")
                .long("completions")
                .help("Generate completions for the shell")
                .takes_value(true)
                .possible_values(&Shell::variants())
        )
        // Args.
        .arg(
            Arg::with_name( "expression")
                .required_unless("completions")
                .value_name("EXPRESSION")
                .help("File matching expression using regex"),
        )
        .arg(
            Arg::with_name("replacement")
                .required_unless("completions")
                .value_name("REPLACEMENT")
                .help("Replacement string"),
        )
        .arg(
            Arg::with_name("file")
                .required_unless("completions")
                .min_values(1)
                .value_name("FILE")
                .help("Files to rename"),
        )
}

fn getch() -> Fallible<Option<char>> {
    let mut s = String::new();
    if let Err(_) = io::stdin().read_line(&mut s) {
        bail!("could not read from stdin");
    }
    Ok(s.chars().next())
}

fn rename_file<P: AsRef<Path>>(path: P, settings: &Settings, repl: &Repl) -> Fallible<()> {
    let path = path.as_ref();

    let expr = repl.expr;
    let repl = repl.repl;

    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(file_name) => file_name,
        None => bail!("could not get file name: {}", path.display()),
    };

    if let Some(captures) = expr.captures(file_name) {
        let mut out = String::with_capacity(file_name.len());
        captures.expand(repl, &mut out);

        let new_path = path.with_file_name(out);

        if settings.interactive || (!settings.no_clobber && !settings.force && new_path.exists()) {
            print!("{} -> {}: ", path.display(), new_path.display());
            if let Err(_) = io::stdout().flush() {
                bail!("could not flush stdout");
            }

            match getch()? {
                Some(ch) if ch == 'Y' || ch == 'y' => (),
                _ => return Ok(()),
            }
        } else if settings.dry || settings.verbose {
            println!("{} -> {}", path.display(), new_path.display());
        }

        if settings.dry || settings.no_clobber && new_path.exists() {
            return Ok(());
        }

        if settings.copy {
            if let Err(_) = fs::copy(&path, &new_path) {
                bail!(
                    "could not copy file: {} -> {}",
                    path.display(),
                    new_path.display()
                );
            }
        } else {
            if let Err(_) = fs::rename(&path, &new_path) {
                bail!(
                    "could not rename file: {} -> {}",
                    path.display(),
                    new_path.display()
                );
            }
        }
    }

    Ok(())
}

fn run() -> Fallible<()> {
    let matches = cli().get_matches();

    if let Some(shell) = matches.value_of("completions") {
        // Safe unwrap since only Shell variant can be passed to here.
        let shell = shell.parse::<clap::Shell>().unwrap();

        let bin_name = env::args().next().unwrap();

        cli().gen_completions_to(bin_name, shell, &mut io::stdout());

        return Ok(());
    }

    let mut settings = Settings {
        dry: matches.is_present("dry"),
        copy: matches.is_present("copy"),
        force: matches.is_present("force"),
        interactive: false,
        no_clobber: false,
        verbose: matches.is_present("verbose"),
    };

    if !settings.force {
        settings.interactive = matches.is_present("interactive");
        settings.no_clobber = matches.is_present("no-clobber");
    }

    let expr = matches.value_of("expression").unwrap();
    let repl = matches.value_of("replacement").unwrap();
    let files = matches.values_of("file").unwrap();

    let expr = Regex::new(expr)?;

    let repl = Repl { expr, repl };

    for file in files {
        rename_file(file, &settings, &repl)?;
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);

        for cause in err.iter_causes() {
            eprintln!("{}", cause);
        }
        process::exit(1);
    }
}
