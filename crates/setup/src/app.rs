//! Opening the setup window: the installer or uninstaller for a payload,
//! or a short explanation when this executable can't install anything. A
//! silent run (`--silent`) never opens a window at all.

use std::sync::Arc;

use bach_pack::i18n::{self, Key};
use bach_pack::manifest::Theme as ThemeChoice;
use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, IntoElement, ParentElement, Render,
    SharedString, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};

use crate::assets::{self, Assets};
use crate::launch::{self, Args, Mode};
use crate::package::{Logo, Package};
use crate::pages::{footer, frame, page_icon, row_count, subtitle_text, PageLayout};
use crate::texts::Texts;
use crate::theme::{ActiveTheme, Theme};
use crate::ui::FormButton;
use crate::view::{SetupView, Stage};

/// The window's width and its height with up to two rows of choices.
const WINDOW_SIZE: (f32, f32) = (540., 490.);
/// Each row of choices past the second adds this much height.
const ROW_HEIGHT: f32 = 73.;
/// The window never grows past this share of the screen; the card scrolls.
const MAX_SCREEN_SHARE: f32 = 0.85;

pub fn main() {
    let args = Args::parse();
    let package = match Package::load() {
        Ok(Some(package)) => Ok(package),
        Ok(None) if args.preview => Ok(Package::sample()),
        Ok(None) => Err(None),
        Err(err) => Err(Some(format!("{err:#}"))),
    };
    if let Ok(package) = &package {
        if !args.preview {
            // An update run that starts a setup file must not share its log.
            let path = args.log.clone().unwrap_or_else(|| {
                if args.updating() {
                    crate::log::update_path(&package.manifest.app.id)
                } else {
                    crate::log::default_path(&package.manifest.app.id)
                }
            });
            let app = &package.manifest.app;
            let run: Vec<String> = std::env::args().skip(1).collect();
            crate::log::start(&path, &format!("{} {} setup, Bach {}, run with {run:?}", app.name, app.version, bach_pack::VERSION));
        }
    }
    if args.updating() && !args.preview {
        let code = match &package {
            Ok(package) => crate::updates::run(&args, package),
            Err(_) => 1,
        };
        std::process::exit(code);
    }
    if args.silent && !args.preview {
        let code = match package {
            Ok(package) => crate::silent::run(&args, package),
            Err(_) => {
                crate::system::host::attach_parent_console();
                eprintln!("This setup file is damaged or carries nothing to install.");
                1
            }
        };
        std::process::exit(code);
    }
    match package {
        Ok(package) => {
            let mode = launch::mode(&args, &package);
            let texts = Arc::new(Texts::new(&package.manifest, args.lang.as_deref()));
            let theme = args.theme.unwrap_or(package.manifest.look.theme);
            let accent = package.accent();
            let logo_svg = match &package.logo {
                Some(Logo::Svg(bytes)) => Some(bytes.clone()),
                _ => None,
            };
            let removing = matches!(mode, Mode::Remove { .. });
            let title = texts.get(if removing { Key::window_remove } else { Key::window_install });
            let rows = row_count(&package.manifest, removing);
            let tall = package.manifest.locale.license.is_some() && !removing;
            let package = Arc::new(package);
            open(Assets { logo_svg }, title, theme, accent, (rows, tall), move |window, cx| {
                let view = SetupView::new(package, texts, mode, args.preview, cx);
                window.focus(&view.focus_handle);
                view
            });
        }
        Err(problem) => {
            let theme = args.theme.unwrap_or_default();
            let language = i18n::pick(&[], &args.lang.clone().map(|lang| vec![lang]).unwrap_or_else(crate::system::host::ui_languages));
            let message = match problem {
                // The bare runtime is something only developers see.
                None => "This is the Bach setup runtime on its own. Build an installer with `bach build`, \
                         or look at this window with `bach-setup --preview`."
                    .to_string(),
                Some(error) => i18n::fill(language.strings.problem_damaged, &[("error", &error)]),
            };
            let title = language.strings.problem_title.to_string();
            open(Assets { logo_svg: None }, title.clone(), theme, None, (0, false), move |_, _| Problem {
                title: title.into(),
                message: message.into(),
                close: language.strings.close.into(),
            });
        }
    }
}

/// Runs the app with one fixed-size window until it closes. `rows` sizes
/// the window for that many rows of choices, or for a license (`tall`).
fn open<V: Render + 'static>(
    assets: Assets,
    title: String,
    theme: ThemeChoice,
    accent: Option<u32>,
    (rows, tall): (usize, bool),
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V + 'static,
) {
    Application::new().with_assets(assets).run(move |cx: &mut App| {
        cx.set_global(Theme::light().with_accent(accent));
        if let Err(reason) = assets::load_fonts(cx) {
            eprintln!("bach-setup: Inter didn't load ({reason}); using Segoe UI");
        }
        crate::ui::init(cx);

        let extra = rows.saturating_sub(2) as f32 * ROW_HEIGHT;
        let wanted = WINDOW_SIZE.1 + extra.max(if tall { 2. * ROW_HEIGHT } else { 0. });
        let screen = cx.primary_display().map(|display| f32::from(display.bounds().size.height));
        let height = match screen {
            Some(screen) if screen > 0. => wanted.min(screen * MAX_SCREEN_SHARE).max(WINDOW_SIZE.1.min(screen)),
            _ => wanted,
        };
        let bounds = Bounds::centered(None, size(px(WINDOW_SIZE.0), px(height)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions { title: Some(title.into()), ..Default::default() }),
            is_resizable: false,
            app_id: Some("bach-setup".into()),
            ..Default::default()
        };
        let opened = cx.open_window(options, move |window, cx| {
            follow_appearance(theme, accent, window, cx);
            let view = cx.new(|cx| build(window, cx));
            guard_close(&view, window, cx);
            view
        });
        match opened {
            Ok(_) => cx.activate(true),
            Err(err) => {
                eprintln!("bach-setup couldn't open its window: {err:#}");
                cx.quit();
            }
        }
        // Closing the window ends setup.
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
    });
}

/// Keeps the window open while files are moving: closing it then would
/// stop setup halfway, which the all-or-nothing install exists to avoid.
fn guard_close<V: 'static>(view: &gpui::Entity<V>, window: &mut Window, cx: &mut App) {
    let Ok(view) = view.clone().into_any().downcast::<SetupView>() else {
        return;
    };
    let view = view.downgrade();
    window.on_window_should_close(cx, move |_, cx| {
        view.upgrade().is_none_or(|view| !matches!(view.read(cx).stage, Stage::Working { .. }))
    });
}

/// Paints with the chosen theme, or the system's, following it live.
fn follow_appearance(choice: ThemeChoice, accent: Option<u32>, window: &mut Window, cx: &mut App) {
    let apply = move |window: &Window, cx: &mut App| {
        let theme = match choice {
            ThemeChoice::Light => Theme::light(),
            ThemeChoice::Dark => Theme::dark(),
            ThemeChoice::System => Theme::for_appearance(window.appearance()),
        };
        cx.set_global(theme.with_accent(accent));
        cx.refresh_windows();
    };
    apply(window, cx);
    window.observe_window_appearance(move |window, cx| apply(window, cx)).detach();
}

/// Shown when there's nothing to install: a bare runtime, or a damaged file.
struct Problem {
    title: SharedString,
    message: SharedString,
    close: SharedString,
}

impl Render for Problem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let body = PageLayout {
            centered: true,
            mark: Some(page_icon("alert-triangle-filled", theme.destructive)),
            subtitle: Some(subtitle_text(self.message.clone(), &theme)),
            ..PageLayout::new(
                self.title.clone(),
                footer(
                    None,
                    vec![FormButton::new("problem-close", self.close.clone())
                        .on_click(|_, _, cx| cx.quit())
                        .into_any_element()],
                    &theme,
                ),
            )
        }
        .render();
        frame(div().flex_1().min_h_0().flex().flex_col().child(body), &theme)
    }
}
