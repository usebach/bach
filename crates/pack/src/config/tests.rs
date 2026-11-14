use super::*;
use crate::i18n::{Key, ANY_LANGUAGE};
use crate::manifest::Choice;

fn project_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bach-config-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("build/sub")).unwrap();
    std::fs::create_dir_all(dir.join("build/samples")).unwrap();
    std::fs::write(dir.join("build/hello.exe"), b"MZ").unwrap();
    std::fs::write(dir.join("build/sub/data.txt"), b"data").unwrap();
    std::fs::write(dir.join("build/samples/a.txt"), b"sample").unwrap();
    std::fs::write(dir.join("build/hello.pdb"), b"debug").unwrap();
    std::fs::write(dir.join("LICENSE.txt"), "\u{feff}Terms.\r\n\r\nMore terms.").unwrap();
    dir
}

const MINIMAL: &str = r#"
    [app]
    name = "Hello"
    version = "1.2.3"
    publisher = "Example"
    executable = "hello.exe"

    [files]
    source = "build"
    exclude = ["*.pdb"]
"#;

#[test]
fn a_minimal_config_builds() {
    let dir = project_dir("minimal");
    let project = parse(MINIMAL, &dir).unwrap();
    let paths: Vec<_> = project.manifest.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["hello.exe", "samples/a.txt", "sub/data.txt"]);
    assert_eq!(project.manifest.app.id, "Hello");
    assert_eq!(project.manifest.install.folder, "Hello");
    assert_eq!(project.manifest.install.desktop_shortcut, Choice::On);
    assert_eq!(project.manifest.install.start_menu_shortcut, Choice::Always);
    assert_eq!(project.manifest.install.start_with_windows, Choice::Never);
    assert!(project.manifest.locale.languages.is_empty());
    assert_eq!(project.output, dir.join("dist/Hello-Setup-1.2.3.exe"));
    assert!(project.warnings.is_empty(), "{:?}", project.warnings);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn mistakes_name_their_setting() {
    let dir = project_dir("mistakes");
    let installer = |line: &str| format!("{MINIMAL}\n[installer]\n{line}");
    let cases = [
        (MINIMAL.replace("hello.exe\"", "missing.exe\""), "app.executable"),
        (MINIMAL.replace("\"build\"", "\"nowhere\""), "files.source"),
        (installer("theme = \"blue\""), "installer.theme"),
        (installer("accent = \"red\""), "installer.accent"),
        (installer("desktop_shortcut = \"maybe\""), "installer.desktop_shortcut"),
        (installer("languages = [\"xx\"]"), "installer.languages"),
        (installer("languages = []"), "installer.languages"),
        (installer("license = \"NOPE.txt\""), "installer.license"),
        (installer("path_folder = \"../bin\""), "installer.path_folder"),
        (installer("logo = \"LICENSE.txt\""), "installer.logo"),
        (format!("{MINIMAL}\n[uninstall]\ndata = ['C:\\Windows']"), "uninstall.data"),
        (MINIMAL.replace("name = \"Hello\"", "name = \"He/llo\""), "app.name"),
        (format!("{MINIMAL}\n[text]\nnope = \"x\""), "text.nope"),
        (format!("{MINIMAL}\n[text]\ninstall_title = {{ xx = \"x\" }}"), "text.install_title.xx"),
        (format!("{MINIMAL}\n[text]\ninstall_title = \"{{size}}\""), "text.install_title"),
        (format!("{MINIMAL}\n[[components]]\nid = \"Bad Id\"\ntitle = \"x\"\nfiles = [\"*\"]"), "components[0].id"),
        (format!("{MINIMAL}\n[[components]]\nid = \"path\"\ntitle = \"x\"\nfiles = [\"*\"]"), "components[0].id"),
        (format!("{MINIMAL}\n[[components]]\nid = \"x\"\ntitle = \"x\"\nfiles = [\"nothing/**\"]"), "components[0].files"),
        (format!("{MINIMAL}\n[[components]]\nid = \"x\"\ntitle = \"x\"\nfiles = [\"*.exe\"]"), "app.executable"),
        (format!("{MINIMAL}\n[[protocols]]\nscheme = \"https\""), "protocols[0].scheme"),
        (format!("{MINIMAL}\n[[file_types]]\nextension = \".exe\""), "file_types[0].extension"),
    ];
    for (text, key) in cases {
        let err = format!("{:#}", parse(&text, &dir).unwrap_err());
        assert!(err.contains(key), "expected {key} in: {err}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unknown_settings_are_typos() {
    let dir = project_dir("typo");
    let text = MINIMAL.replace("[files]", "nmae = \"x\"\n[files]");
    assert!(parse(&text, &dir).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn everything_can_be_customised() {
    let dir = project_dir("full");
    let text = format!(
        r#"{MINIMAL}
        [installer]
        languages = ["de", "en"]
        license = "LICENSE.txt"
        desktop_shortcut = false
        start_menu_shortcut = true
        start_with_windows = "always"
        add_to_path = true
        path_folder = "sub"
        change_folder = false
        open_button = false
        logo = "none"

        [[components]]
        id = "samples"
        title = {{ en = "Samples", de = "Beispiele" }}
        files = ["samples/**"]
        default = true

        [[protocols]]
        scheme = "Hello://"

        [[file_types]]
        extension = "HELLO"
        description = "Hello note"

        [text]
        install_title = {{ en = "Get {{name}}", de = "Hol dir {{name}}" }}
        footer = {{ en = "Made with care", de = "Mit Liebe gemacht" }}

        [uninstall]
        data = ['%APPDATA%\Hello']
        data_description = {{ en = "Notes.", de = "Notizen." }}
        "#
    );
    let project = parse(&text, &dir).unwrap();
    let manifest = &project.manifest;
    assert_eq!(manifest.locale.languages, ["de", "en"]);
    assert_eq!(manifest.license("de"), Some("Terms.\n\nMore terms."));
    assert_eq!(manifest.install.desktop_shortcut, Choice::Off);
    assert_eq!(manifest.install.start_with_windows, Choice::Always);
    assert_eq!(manifest.install.path_folder.as_deref(), Some("sub"));
    assert!(!manifest.install.change_folder && !manifest.install.open_button);
    assert!(manifest.look.hide_logo);
    assert_eq!(manifest.components[0].id, "samples");
    let sample = manifest.files.iter().find(|file| file.path == "samples/a.txt").unwrap();
    assert_eq!(sample.component.as_deref(), Some("samples"));
    assert_eq!(manifest.protocols[0].scheme, "hello");
    assert_eq!(manifest.file_types[0].extension, ".hello");
    assert_eq!(manifest.custom_text(Key::install_title, "de"), Some("Hol dir {name}"));
    assert_eq!(manifest.custom_text(Key::delete_data_description, "en"), Some("Notes."));
    assert!(project.warnings.is_empty(), "{:?}", project.warnings);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn untranslated_texts_are_pointed_out() {
    let dir = project_dir("untranslated");
    let text = format!("{MINIMAL}\n[text]\ninstall_button = \"Go\"");
    let project = parse(&text, &dir).unwrap();
    assert_eq!(project.manifest.locale.text["install_button"][ANY_LANGUAGE], "Go");
    assert!(project.warnings[0].contains("text.install_button"), "{:?}", project.warnings);

    // One language: nothing to translate.
    let text = format!("{MINIMAL}\n[installer]\nlanguages = [\"de\"]\n[text]\ninstall_button = \"Los\"");
    assert!(parse(&text, &dir).unwrap().warnings.is_empty());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn output_names_are_filled_in() {
    let dir = project_dir("template");
    let app = parse(MINIMAL, &dir).unwrap().manifest.app;
    assert_eq!(fill_template("{name}_{version}_{id}.exe", &app), "Hello_1.2.3_Hello.exe");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_byte_order_mark_is_fine() {
    let dir = project_dir("bom");
    assert!(parse(&format!("\u{feff}{MINIMAL}"), &dir).is_ok());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn hex_colours() {
    assert_eq!(parse_hex("#2563eb"), Some(0x2563eb));
    assert_eq!(parse_hex("FFFFFF"), Some(0xffffff));
    assert_eq!(parse_hex("#fff"), None);
}

#[test]
fn requirements_shortcuts_and_updates() {
    let dir = project_dir("extras");
    let app = MINIMAL.replace("[files]", "website = \"https://hello.example\"\n[files]");
    let text = format!(
        r#"{app}
        [installer]
        start_menu_folder = "Example"
        website_button = true
        logo_height = 72

        [[requirements]]
        kind = "webview2"

        [[requirements]]
        kind = "dotnet"
        version = "8.0"
        desktop = true

        [[shortcuts]]
        name = "Hello Docs"
        target = "sub/data.txt"
        arguments = "--docs"

        [updates]
        feed = "https://hello.example/latest.json"
        download_url = "https://hello.example/{{file}}"

        [uninstall]
        feedback_url = "https://hello.example/why"
        "#
    );
    let project = parse(&text, &dir).unwrap();
    let manifest = &project.manifest;
    assert_eq!(manifest.requirements.len(), 2);
    assert_eq!(manifest.requirements[0].name, "Microsoft Edge WebView2");
    assert_eq!(manifest.shortcuts[0].target, "sub/data.txt");
    assert_eq!(manifest.install.start_menu_folder.as_deref(), Some("Example"));
    assert!(manifest.install.website_button);
    assert_eq!(manifest.look.logo_height, Some(72));
    assert_eq!(manifest.updates.as_ref().unwrap().feed, "https://hello.example/latest.json");
    assert_eq!(project.update_download_url.as_deref(), Some("https://hello.example/{file}"));
    assert_eq!(manifest.uninstall.feedback_url.as_deref(), Some("https://hello.example/why"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn extras_name_their_setting() {
    let dir = project_dir("extras-mistakes");
    let cases = [
        (r#"[[requirements]]
kind = "java""#, "requirements[0].kind"),
        (r#"[[requirements]]
kind = "dotnet"
version = "8""#, "requirements[0].version"),
        (r#"[[requirements]]
kind = "file"
path = 'C:\x.dll'"#, "requirements[0].name"),
        (r#"[[requirements]]
kind = "registry"
key = 'HKCR\x'
name = "x"
url = "https://x""#, "requirements[0].key"),
        (r#"[[shortcuts]]
name = "Docs"
target = "nope.exe""#, "shortcuts[0].target"),
        (r#"[[shortcuts]]
name = "a/b"
target = "hello.exe""#, "shortcuts[0].name"),
        (r#"[updates]
feed = "http://insecure""#, "updates.feed"),
        ("[installer]\nlogo_height = 200", "installer.logo_height"),
        ("[installer]\nwebsite_button = true", "installer.website_button"),
        ("[uninstall]\nfeedback_url = \"ftp://x\"", "uninstall.feedback_url"),
    ];
    for (extra, key) in cases {
        let err = format!("{:#}", parse(&format!("{MINIMAL}\n{extra}"), &dir).unwrap_err());
        assert!(err.contains(key), "expected {key} in: {err}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The editor schema (schema/bach.schema.json) knows every text and
/// language, so autocompletion never misses one.
#[test]
fn the_schema_knows_every_text() {
    let schema: serde_json::Value = serde_json::from_str(include_str!("../../../../schema/bach.schema.json")).unwrap();
    let texts = &schema["properties"]["text"]["properties"];
    for key in Key::ALL {
        assert!(texts.get(key.name()).is_some(), "schema has no text.{}", key.name());
    }
    let languages = schema["properties"]["installer"]["properties"]["languages"]["items"]["enum"].as_array().unwrap();
    assert_eq!(languages.len(), crate::i18n::LANGUAGES.len());
}
