//! Opening the setup window: the installer or uninstaller for a payload,
//! or a short explanation when this executable can't install anything.

use std::sync::Arc;

use bach_pack::manifest::Theme as ThemeChoice;
use gpui::{
    div, px, size, App, AppContext, Application, Bounds, Context, IntoElement, ParentElement,
    Render, SharedString, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};

use crate::assets::{self, Assets};
use crate::launch::{self, Args, Mode};
use crate::package::{Logo, Package};
use crate::pages::{footer, frame, layout, page_icon, subtitle_text};
use crate::theme::{ActiveTheme, Theme};
use crate::ui::FormButton;
use crate::view::SetupView;

/// The window's fixed size.
const WINDOW_SIZE: (f32, f32) = (540., 490.);

pub fn main() {
    let args = Args::parse();
    let package = match Package::load() {
        Ok(Some(package)) => Ok(package),
        Ok(None) if args.preview => Ok(Package::sample()),
        Ok(None) => Err("This is the Bach setup runtime on its own. Build an installer with `bach build`, \
                         or look at this window with `bach-setup --preview`."
            .to_string()),
        Err(err) => Err(format!("This setup file is damaged: {err:#}. Download it again.")),
    };
    match package {
        Ok(package) => {
            let mode = launch::mode(&args, &package);
            let theme = args.theme.unwrap_or(package.manifest.look.theme);
            let accent = package.accent();
            let logo_svg = match &package.logo {
                Some(Logo::Svg(bytes)) => Some(bytes.clone()),
                _ => None,
            };
            let title = match mode {
                Mode::Install => format!("{} Setup", package.name()),
                Mode::Remove { .. } => format!("Remove {}", package.name()),
            };
            let package = Arc::new(package);
            open(Assets { logo_svg }, title, theme, accent, move |_, _| {
                SetupView::new(package, mode, args.preview)
            });
        }
        Err(message) => {
            let theme = args.theme.unwrap_or_default();
            open(Assets { logo_svg: None }, "Bach Setup".into(), theme, None, move |_, _| {
                Problem { message: message.into() }
            });
        }
    }
}

/// Runs the app with one fixed-size window until it closes.
fn open<V: Render + 'static>(
    assets: Assets,
    title: String,
    theme: ThemeChoice,
    accent: Option<u32>,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V + 'static,
) {
    Application::new().with_assets(assets).run(move |cx: &mut App| {
        cx.set_global(Theme::light().with_accent(accent));
        if let Err(reason) = assets::load_fonts(cx) {
            eprintln!("bach-setup: Inter didn't load ({reason}); using Segoe UI");
        }
        crate::ui::init(cx);

        let bounds = Bounds::centered(None, size(px(WINDOW_SIZE.0), px(WINDOW_SIZE.1)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some(title.into()),
                ..Default::default()
            }),
            is_resizable: false,
            app_id: Some("bach-setup".into()),
            ..Default::default()
        };
        let opened = cx.open_window(options, move |window, cx| {
            follow_appearance(theme, accent, window, cx);
            cx.new(|cx| build(window, cx))
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
    window
        .observe_window_appearance(move |window, cx| apply(window, cx))
        .detach();
}

/// Shown when there's nothing to install: a bare runtime, or a damaged file.
struct Problem {
    message: SharedString,
}

impl Render for Problem {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let body = layout(
            true,
            page_icon("alert-triangle-filled", theme.destructive),
            "Nothing to install".into(),
            subtitle_text(self.message.clone(), &theme),
            None,
            footer(
                Some(format!("Bach {}", env!("CARGO_PKG_VERSION")).into()),
                vec![FormButton::new("problem-close", "Close")
                    .on_click(|_, _, cx| cx.quit())
                    .into_any_element()],
                &theme,
            ),
        );
        frame(div().flex_1().min_h_0().flex().flex_col().child(body), &theme)
    }
}
