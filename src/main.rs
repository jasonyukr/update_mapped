use clap::{App, Arg};
use std::io::{self, BufRead, Write};
use std::collections::HashMap;
use std::fs::File;
use std::process;
use std::path::Path;

use lscolors::{LsColors, Style};

#[cfg(all(
    not(feature = "nu-ansi-term"),
))]
compile_error!(
    "feature must be enabled: nu-ansi-term"
);

fn print_lscolor_path_linenum(handle: &mut dyn Write, ls_colors: &LsColors, path: &str, linenum: &str) -> io::Result<()> {
    for (component, style) in ls_colors.style_for_path_components(Path::new(path)) {
        #[cfg(any(feature = "nu-ansi-term", feature = "gnu_legacy"))]
        {
            let ansi_style = style.map(Style::to_nu_ansi_term_style).unwrap_or_default();
            write!(handle, "{}", ansi_style.paint(component.to_string_lossy()))?;
        }
    }
    if linenum.len() > 0 {
        writeln!(handle, "\t\x1b[38;2;90;90;90m{}\x1b[0m", linenum)?;
    } else {
        writeln!(handle)?;
    }
    Ok(())
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() -> io::Result<()> {
    let ls_colors = LsColors::from_env().unwrap_or_default();

    let mut stdout = io::stdout();
    let matches = App::new("update_mapped")
        .arg(
            Arg::with_name("color")
                .short("c")
                .long("color")
                .help("Use ls-colors")
        )
        .arg(
            Arg::with_name("LASTPLACE_FILE")
                .required(true)
                .help("nvim lastpalce dump file")
                .index(1),
        )
        .arg(
            Arg::with_name("TARGET_DIR")
                .help("target directory")
                .index(2),
        )
        .get_matches();

    let color = matches.is_present("color");
    let lastplace_file = matches.value_of("LASTPLACE_FILE").unwrap_or("");
    let mut target_dir = matches.value_of("TARGET_DIR").unwrap_or("").to_string();
    if target_dir.len() > 0 {
        if !target_dir.eq("/") {
            if !target_dir.ends_with("/") {
                target_dir.push('/');
            }
        }
    }

    let mut map: HashMap<String, String> = HashMap::new();

    if Path::new(lastplace_file).exists() {
        if let Ok(lines) = read_lines(lastplace_file) {
            for line in lines.flatten() {
                let ln = line.trim();
                if let Some(i) = ln.find('\t') {
                    let key = (&ln[..i]).to_string();
                    let val = (&ln[i+1..]).to_string();
                    map.insert(key, val);
                }
            }
        }
    }

    let stdin = io::stdin();
    for line_data in stdin.lock().lines() {
        if let Ok(line) = line_data {
            let ln = line.trim();

            let mut path_disp = ln;
            if target_dir.len() > 0 && !target_dir.eq("/") {
                if !ln.starts_with(&target_dir) {
                    continue;
                } else {
                    path_disp = &ln[target_dir.len()..];
                }
            }

            if !Path::new(ln).exists() {
                // print in orange for file-not-found case
                if let Err(_) = writeln!(stdout, "\x1b[38;2;255;165;0m{}\x1b[0m", path_disp) {
                    process::exit(1);
                }
                continue;
            }

            let res;
            if let Some(val) = map.get(ln) {
                if color {
                    res = print_lscolor_path_linenum(&mut stdout, &ls_colors, &path_disp, &val);
                } else {
                    res = writeln!(&mut stdout, "{}\t{}", path_disp, val);
                }
            } else {
                if color {
                    res = print_lscolor_path_linenum(&mut stdout, &ls_colors, &path_disp, "");
                } else {
                    res = writeln!(&mut stdout, "{}", path_disp);
                }
            }
            match res {
                Ok(_) => (),
                Err(_e) => { process::exit(1) },
            }
        }
    }

    Ok(())
}
