//! `--preview`: the steps of an install or removal, with their labels and
//! pacing, and no change to the computer. For designing an installer with
//! `bach preview`.

use std::time::Duration;

use super::Report;

/// Walks through the labels a real job shows, with pauses instead of work.
pub fn pretend(name: &str, removing: bool, report: Report) -> Result<(), String> {
    let steps: Vec<(f32, String)> = if removing {
        vec![
            (0.1, format!("Removing {name}")),
            (0.45, "Removing shortcuts".into()),
            (0.6, format!("Removing {name} from your apps")),
            (0.9, "Cleaning up".into()),
        ]
    } else {
        vec![(0.02, "Getting ready".into())]
            .into_iter()
            .chain((1..=20).map(|step| (0.04 + 0.82 * step as f32 / 20.0, format!("Copying {name}"))))
            .chain([
                (0.9, "Adding the uninstaller".into()),
                (0.93, "Adding shortcuts".into()),
                (0.97, format!("Adding {name} to your apps")),
            ])
            .collect()
    };
    for (progress, label) in steps {
        report(progress, &label);
        std::thread::sleep(Duration::from_millis(if label.starts_with("Copying") { 90 } else { 420 }));
    }
    report(1.0, "Done");
    Ok(())
}
