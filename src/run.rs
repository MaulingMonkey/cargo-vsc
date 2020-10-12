use crate::*;

const AUTOGEN_JSON : &'static str = "// WARNING: autogenerated by cargo-vsc, may be overwritten if this comment remains!";

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

pub fn run() {
    let meta = metadata::Root::get().unwrap_or_else(|err| { eprintln!("error parsing `cargo metadata`: {}", err); exit(1) });
    let vscode = create_vscode_dir(&meta).unwrap_or_else(|err| { eprintln!("error creating .vscode directory: {}", err); exit(1) });
    let context = Context { meta, vscode, _non_exhaustive: () };

    let mut errors = false;
    create_vscode_extensions_json   (&context).unwrap_or_else(|err| { eprintln!("error creating .vscode/extensions.json: {}", err); errors = true; });
    create_vscode_settings_json     (&context).unwrap_or_else(|err| { eprintln!("error creating .vscode/settings.json: {}", err); errors = true; });
    create_vscode_tasks_json        (&context).unwrap_or_else(|err| { eprintln!("error creating .vscode/tasks.json: {}", err); errors = true; });
    create_vscode_launch_json       (&context).unwrap_or_else(|err| { eprintln!("error creating .vscode/launch.json: {}", err); errors = true; });
    if errors { exit(1) }
}

fn create_vscode_dir(meta: &metadata::Root) -> io::Result<PathBuf> {
    let vscode = meta.workspace.dir.join(".vscode");
    match std::fs::create_dir(&vscode) {
        Ok(()) => {
            std::fs::write(vscode.join(".gitignore"), "*").map_err(|err| io::Error::new(err.kind(), format!("unable to create .gitignore: {}", err)))?; // XXX: remap err for more context?
            Ok(vscode)
        },
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(vscode),
        Err(err) => Err(err),
    }
}

struct Context {
    meta:   metadata::Root,
    vscode: PathBuf,

    _non_exhaustive: ()
}



fn create_json(path: &Path) -> io::Result<File> {
    if path.exists() {
        if let Some(line) = BufReader::new(File::open(path)?).lines().next() {
            if line?.trim() != AUTOGEN_JSON {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("`{}` already exists, and doesn't start with {:?}", path.display(), AUTOGEN_JSON)));
            }
        }
    }
    let mut file = File::create(path)?;
    writeln!(file, "{}", AUTOGEN_JSON)?;
    Ok(file)
}


fn create_vscode_extensions_json(Context { meta, vscode, .. }: &Context) -> io::Result<()> {
    let path = vscode.join("extensions.json");
    let mut o = create_json(&path)?;
    writeln!(o, "{{")?;
    writeln!(o, "    \"recommendations\": [")?;
    write_ext(&mut o, "matklad.rust-analyzer")?;
    if meta.packages.iter().any(|p| p.targets.iter().any(|t| t.kind.iter().any(|kind| ["example", "bin"].contains(&&**kind)))) {
        write_ext(&mut o, "ms-vscode.cpptools")?;
    }
    writeln!(o, "    ]")?;
    writeln!(o, "}}")?;
    Ok(())
}

fn write_ext(o: &mut impl io::Write, extension: &str) -> io::Result<()> {
    writeln!(*o, "        {},", serde_json::to_string(extension).unwrap())?;
    Ok(())
}



fn create_vscode_settings_json(Context { vscode, .. }: &Context) -> io::Result<()> {
    let files_exclude = [
        "/**/.git",         // probably implicit, but be explicit anyways
        "/target/*/*/*",    // clutters up search results
    ];

    let path = vscode.join("settings.json");
    let mut o = create_json(&path)?;
    writeln!(o, "{{")?;
    writeln!(o, "    \"files.exclude\": {{")?;
    for file_exclude in files_exclude.iter().copied() {
        writeln!(o, "        {}: true,", serde_json::to_string(file_exclude).unwrap())?;
    }
    writeln!(o, "    }}")?;
    writeln!(o, "}}")?;
    Ok(())
}



fn create_vscode_launch_json(Context { meta, vscode, .. }: &Context) -> io::Result<()> {
    let path = vscode.join("launch.json");
    let mut o = create_json(&path)?;
    writeln!(o, "{{")?;
    writeln!(o, "    \"version\": \"0.2.0\",")?;
    writeln!(o, "    \"configurations\": [")?;

    for package in meta.packages.iter() {
        if !meta.workspace_members.contains(&package.id) {
            continue
        }

        writeln!(o, "        // {}", package.name)?;
        for target in package.targets.iter() {
            for kind in target.kind.iter() {
                let (subdir, cargo_build_debug) = match kind.as_str() {
                    "example"   => ("examples/", format!("cargo build --package {} --example {}", package.name, target.name)),
                    "bin"       => ("",          format!("cargo build --package {} --bin {}", package.name, target.name)),
                    _other      => continue // not currently launchable
                };
                let cargo_build_release = format!("{} --release", cargo_build_debug);

                for (config, build) in vec![("debug", cargo_build_debug), ("release", cargo_build_release)].into_iter() {
                    let name = if package.name == target.name && kind == "bin" {
                        format!("{} • {}", package.name, config)
                    } else {
                        format!("{} • {} • {} • {}", package.name, kind, target.name, config)
                    };

                    writeln!(o, "        {{")?;
                    writeln!(o, "            \"name\":                     {},", serde_json::to_string(&name).unwrap())?;
                    writeln!(o, "            \"type\":                     \"cppdbg\",")?;
                    writeln!(o, "            \"request\":                  \"launch\",")?;
                    writeln!(o, "            \"internalConsoleOptions\":   \"openOnSessionStart\",")?;
                    writeln!(o, "            \"preLaunchTask\":            {},", serde_json::to_string(&build).unwrap())?;
                    writeln!(o, "            \"program\":                  {},", serde_json::to_string(&format!("${{workspaceFolder}}/target/{}/{}{}", config, subdir, target.name)).unwrap())?;
                    writeln!(o, "            \"cwd\":                      \"${{workspaceFolder}}\",")?;
                    writeln!(o, "            \"environment\":              [ {{ \"name\": \"RUST_BACKTRACE\", \"value\": \"1\" }} ],")?;
                    writeln!(o, "            \"windows\": {{")?;
                    writeln!(o, "                \"type\":                     \"cppvsdbg\",")?; // despite vscode intellisense errors to the contrary, this totally works & is necessary
                    writeln!(o, "                \"program\":                  {},", serde_json::to_string(&format!("${{workspaceFolder}}/target/{}/{}{}.exe", config, subdir, target.name)).unwrap())?;
                    writeln!(o, "                \"enableDebugHeap\":          {},", config == "debug")?;
                    writeln!(o, "            }}")?;
                    writeln!(o, "        }},")?;
                }
            }
        }
    }

    writeln!(o, "    ]")?; // configurations
    writeln!(o, "}}")?;
    Ok(())
}



fn create_vscode_tasks_json(Context { meta, vscode, .. }: &Context) -> io::Result<()> {
    let path = vscode.join("tasks.json");
    let mut o = create_json(&path)?;

    let has_any_local_install = meta.workspace.toml.as_ref().map_or(false, |ws| ws.metadata.local_install.is_some());
    // TODO: also install for packages: meta.packages.iter().any(|p| meta.workspace_members.contains(&p.id) && p.manifest.toml.metadata.local_install.is_some());

    writeln!(o, "{{")?;
    writeln!(o, "    \"version\": \"2.0.0\",")?;
    writeln!(o, "    \"problemMatcher\": \"$rustc\",")?; // rust-analyzer
    writeln!(o, "    \"tasks\": [")?;
    writeln!(o, "        // entry points")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"default-build\",")?;
    writeln!(o, "            \"dependsOrder\":     \"sequence\",")?;
    writeln!(o, "            \"dependsOn\":        [ \"fetch\", \"check\", \"test\", \"build\" ],")?;
    writeln!(o, "            \"group\":            {{ \"kind\": \"build\", \"isDefault\": true }}")?;
    writeln!(o, "        }},")?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o, "        // fetch")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"fetch\",")?;
    writeln!(o, "            \"dependsOn\":        [")?;
    writeln!(o, "                \"cargo fetch\",")?;
    if has_any_local_install {
        writeln!(o, "                \"cargo local-install\",")?;
    }
    writeln!(o, "            ]")?;
    writeln!(o, "        }},")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"cargo fetch\",")?;
    writeln!(o, "            \"command\":          \"cargo fetch\",")?;
    writeln!(o, "            \"type\":             \"shell\",")?;
    writeln!(o, "            \"presentation\":     {{ \"clear\": true, \"group\": \"fetch\", \"reveal\": \"always\" }},")?;
    writeln!(o, "        }},")?;
    if has_any_local_install {
        writeln!(o, "        {{")?;
        writeln!(o, "            \"label\":            \"cargo local-install\",")?;
        writeln!(o, "            \"command\":          \"cargo install cargo-local-install && cargo local-install\",")?;
        writeln!(o, "            \"type\":             \"shell\",")?;
        writeln!(o, "            \"presentation\":     {{ \"group\": \"fetch\", \"reveal\": \"always\" }},")?;
        writeln!(o, "        }},")?;
    }
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o, "        // check")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"check\",")?;
    writeln!(o, "            \"command\":          \"cargo check --frozen --all-targets\",")?;
    writeln!(o, "            \"type\":             \"shell\",")?;
    writeln!(o, "            \"presentation\":     {{ \"clear\": true, \"group\": \"check\", \"reveal\": \"always\" }},")?;
    writeln!(o, "            \"problemMatcher\":   {{ \"base\": \"$rustc\", \"owner\": \"check\", \"source\": \"check\" }},")?;
    writeln!(o, "        }},")?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o, "        // test")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"test\",")?;
    writeln!(o, "            \"command\":          \"cargo test --frozen\",")?;
    writeln!(o, "            \"type\":             \"shell\",")?;
    writeln!(o, "            \"presentation\":     {{ \"clear\": true, \"group\": \"test\", \"reveal\": \"always\" }},")?;
    writeln!(o, "            \"problemMatcher\":   {{ \"base\": \"$rustc\", \"owner\": \"test\", \"source\": \"test\" }},")?;
    writeln!(o, "        }},")?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o)?;
    writeln!(o, "        // build")?;
    writeln!(o, "        {{")?;
    writeln!(o, "            \"label\":            \"build\",")?;
    writeln!(o, "            \"command\":          \"cargo build --frozen --all-targets\",")?;
    writeln!(o, "            \"type\":             \"shell\",")?;
    writeln!(o, "            \"presentation\":     {{ \"clear\": true, \"group\": \"build\", \"reveal\": \"always\" }},")?;
    writeln!(o, "            \"problemMatcher\":   {{ \"base\": \"$rustc\", \"owner\": \"build\", \"source\": \"build\" }},")?;
    writeln!(o, "        }},")?;


    for package in meta.packages.iter() {
        if !meta.workspace_members.contains(&package.id) {
            continue
        }

        writeln!(o)?;
        writeln!(o)?;
        writeln!(o)?;
        writeln!(o, "        // {}", package.name)?;
        for target in package.targets.iter() {
            for kind in target.kind.iter() {
                let cargo_build_debug = match kind.as_str() {
                    "example"   => format!("cargo build --package {} --example {}", package.name, target.name),
                    "bin"       => format!("cargo build --package {} --bin {}", package.name, target.name),
                    _other      => continue // not currently launchable
                };
                let cargo_build_release = format!("{} --release", cargo_build_debug);
                write_cmd(&mut o, &cargo_build_debug)?;
                write_cmd(&mut o, &cargo_build_release)?;

                let local_doc_flags = match kind.as_str() {
                    "lib"       => format!("--package {} --lib", package.name),
                    "example"   => format!("--package {} --bin {}", package.name, target.name),
                    "bin"       => format!("--package {} --bin {}", package.name, target.name),
                    _other      => continue // docs not supported for this kind of target
                };
                let local_doc_build = format!("cargo doc {}", local_doc_flags);
                let local_doc_open = if target.kind.len() <= 1 {
                    format!("build & open local documentation ({})", target.name)
                } else {
                    format!("build & open local documentation ({}, {})", target.name, kind)
                };
                write_cargo_doc(&mut o, &local_doc_build, &local_doc_flags)?;
                write_open_link(&mut o, &local_doc_open, &format!("${{workspaceFolder}}\\target\\doc\\{}\\index.html", package.name.replace('-', "_")), &local_doc_build)?;
            }
        }
        if let Some(repository) = package.manifest.toml.package.repository.as_ref() {
            write_open_link(&mut o, &format!("open repository ({})", package.name), &repository, "")?;
        }
        if let Some(documentation) = package.manifest.toml.package.documentation.as_ref() {
            write_open_link(&mut o, &format!("open documentation ({})", package.name), &documentation, "")?;
        }
        if let Some(homepage) = package.manifest.toml.package.homepage.as_ref() {
            write_open_link(&mut o, &format!("open homepage ({})", package.name), &homepage, "")?;
        }
    }

    writeln!(o, "    ]")?; // tasks
    writeln!(o, "}}")?;
    Ok(())
}

fn write_open_link(o: &mut impl io::Write, title: &str, url: &str, depends_on: &str) -> io::Result<()> {
    writeln!(*o, "        {{")?;
    writeln!(*o, "            \"label\":            {},", serde_json::to_string(title).unwrap())?;
    writeln!(*o, "            \"windows\":          {{ \"command\": {} }},", serde_json::to_string(&format!("start \"\"    \"{}\"", url)).unwrap())?;
    writeln!(*o, "            \"linux\":            {{ \"command\": {} }},", serde_json::to_string(&format!("xdg-open      \"{}\"", url)).unwrap())?;
    writeln!(*o, "            \"osx\":              {{ \"command\": {} }},", serde_json::to_string(&format!("open          \"{}\"", url)).unwrap())?;
    writeln!(*o, "            \"type\":             \"shell\",")?;
    writeln!(*o, "            \"presentation\":     {{ \"clear\": true, \"panel\": \"shared\", \"reveal\": \"silent\" }},")?;
    if !depends_on.is_empty() {
        writeln!(*o, "            \"dependsOn\":        [ {} ],", serde_json::to_string(depends_on).unwrap())?;
    }
    writeln!(*o, "        }},")?;
    Ok(())
}

fn write_cargo_doc(o: &mut impl io::Write, title: &str, flags: &str) -> io::Result<()> {
    writeln!(*o, "        {{")?;
    writeln!(*o, "            \"label\":            {},", serde_json::to_string(title).unwrap())?;
    writeln!(*o, "            \"command\":          {},", serde_json::to_string(&format!("cargo +nightly doc {flags} || cargo doc {flags}", flags=flags)).unwrap())?;
    writeln!(*o, "            \"type\":             \"shell\",")?;
    writeln!(*o, "            \"presentation\":     {{ \"clear\": true, \"panel\": \"shared\", \"reveal\": \"always\" }},")?;
    writeln!(*o, "        }},")?;
    Ok(())
}

fn write_cmd(o: &mut impl io::Write, cmd: &str) -> io::Result<()> {
    let cmd = serde_json::to_string(cmd).unwrap();
    writeln!(*o, "        {{")?;
    writeln!(*o, "            \"label\":            {},", cmd)?;
    writeln!(*o, "            \"command\":          {},", cmd)?;
    writeln!(*o, "            \"type\":             \"shell\",")?;
    writeln!(*o, "            \"problemMatcher\":   \"$rustc\",")?;
    writeln!(*o, "            \"presentation\":     {{ \"clear\": true, \"panel\": \"shared\", \"reveal\": \"always\" }},")?;
    writeln!(*o, "        }},")?;
    Ok(())
}
