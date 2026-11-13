//! The payload this executable carries: read once at startup from its own
//! file, or a built-in sample for `--preview` on a bare runtime.

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use bach_pack::container::{self, Layout};
use bach_pack::manifest::{self, FileEntry, LogoKind, Manifest};

/// Everything setup knows about the app it installs or removes.
pub struct Package {
    pub manifest: Manifest,
    /// The file the payload was read from (this executable).
    pub exe: PathBuf,
    /// None for the built-in sample, which has nothing to unpack.
    pub layout: Option<Layout>,
    pub logo: Option<Logo>,
}

pub enum Logo {
    /// Served to GPUI as an asset and drawn in the theme's ink.
    Svg(Arc<[u8]>),
    /// Drawn as is.
    Png(Arc<gpui::Image>),
}

impl Package {
    /// The payload of the running executable; Ok(None) when it's a bare
    /// runtime.
    pub fn load() -> Result<Option<Self>> {
        let exe = std::env::current_exe().context("couldn't find this setup file")?;
        let mut file = File::open(&exe).context("couldn't open this setup file")?;
        let Some(layout) = container::locate(&mut file)? else {
            return Ok(None);
        };
        let manifest = container::read_manifest(&mut file, &layout)?;
        let bytes = container::read_logo(&mut file, &layout)?;
        let logo = match manifest.look.logo {
            _ if bytes.is_empty() => None,
            Some(LogoKind::Svg) => Some(Logo::Svg(bytes.into())),
            Some(LogoKind::Png) => Some(Logo::Png(Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, bytes)))),
            None => None,
        };
        Ok(Some(Self { manifest, exe, layout: Some(layout), logo }))
    }

    /// A made-up app for previewing the window on a bare runtime.
    pub fn sample() -> Self {
        let manifest = Manifest {
            format: bach_pack::FORMAT_VERSION,
            builder: bach_pack::VERSION.into(),
            app: manifest::App {
                id: "Sample".into(),
                name: "Sample".into(),
                version: "1.0.0".into(),
                publisher: "Sample Inc.".into(),
                description: Some("A preview of the installer window.".into()),
                website: None,
                executable: "sample.exe".into(),
            },
            install: manifest::Install {
                folder: "Sample".into(),
                desktop_shortcut: manifest::Choice::On,
                start_menu_shortcut: manifest::Choice::Always,
                start_with_windows: manifest::Choice::Never,
                add_to_path: manifest::Choice::Never,
                path_folder: None,
                change_folder: true,
                open_button: true,
            },
            uninstall: manifest::Uninstall { data: vec![r"%APPDATA%\Sample".into()], data_description: None },
            look: manifest::Look::default(),
            files: vec![FileEntry::new("sample.exe", 48 << 20)],
            components: Vec::new(),
            protocols: Vec::new(),
            file_types: Vec::new(),
            locale: manifest::Locale::default(),
            chosen: None,
        };
        Self { manifest, exe: PathBuf::new(), layout: None, logo: None }
    }

    /// Whether this is an installer (it carries files) rather than an
    /// uninstaller.
    pub fn has_files(&self) -> bool {
        self.layout.is_some_and(|layout| layout.has_archive())
    }

    pub fn version(&self) -> &str {
        &self.manifest.app.version
    }

    /// The accent colour as `0xrrggbb`.
    pub fn accent(&self) -> Option<u32> {
        self.manifest.look.accent.as_deref().and_then(bach_pack::config::parse_hex)
    }
}
