//! `--preview`: the steps of an install or removal, with their labels and
//! pacing, and no change to the computer. For designing an installer with
//! `bach preview`.

use std::time::Duration;

use bach_pack::i18n::Key;

use super::Report;
use crate::texts::Texts;

/// Walks through the labels a real job shows, with pauses instead of work.
pub fn pretend(texts: &Texts, removing: bool, report: Report) -> Result<(), String> {
    let copying = texts.get(Key::step_copying);
    let steps: Vec<(f32, String, u64)> = if removing {
        vec![
            (0.1, texts.get(Key::step_removing), 420),
            (0.4, texts.get(Key::step_removing_shortcuts), 420),
            (0.6, texts.get(Key::step_unregistering), 420),
            (0.9, texts.get(Key::step_cleaning), 420),
        ]
    } else {
        vec![(0.02, texts.get(Key::step_preparing), 420)]
            .into_iter()
            .chain((1..=20).map(|step| (0.04 + 0.8 * step as f32 / 20.0, copying.clone(), 90)))
            .chain([
                (0.86, texts.get(Key::step_tidying), 300),
                (0.89, texts.get(Key::step_uninstaller), 420),
                (0.92, texts.get(Key::step_shortcuts), 420),
                (0.97, texts.get(Key::step_registering), 420),
            ])
            .collect()
    };
    for (progress, label, pause) in steps {
        report(progress, &label);
        std::thread::sleep(Duration::from_millis(pause));
    }
    report(1.0, &texts.get(Key::step_done));
    Ok(())
}
