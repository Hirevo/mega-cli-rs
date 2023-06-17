use std::fmt;

use indicatif::style::ProgressStyle;
use indicatif::{DecimalBytes, ProgressState};
use once_cell::sync::Lazy;

pub static USER_ATTENDED: Lazy<bool> = Lazy::new(|| console::user_attended());

fn decimal_bytes_per_sec(state: &ProgressState, w: &mut dyn fmt::Write) {
    write!(w, "{0}/s", DecimalBytes(state.per_sec() as u64)).unwrap();
}

pub fn standard_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .progress_chars("▰▰▱")
        .with_key("decimal_bytes_per_sec", decimal_bytes_per_sec)
        .template(
            "{bar:30.cyan.bold/cyan/bold} {percent} % ({decimal_bytes}, {decimal_bytes_per_sec}, ETA {eta}): {msg}",
        )
        .unwrap()
}

pub fn discrete_progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .progress_chars("▰▰▱")
        .template("{bar:30.cyan.bold/cyan/bold} [{human_pos}/{human_len}] ({percent} %): {msg}")
        .unwrap()
}

pub fn spinner_style() -> ProgressStyle {
    let template = {
        let mut template = generate_template(30, 10, '▰', '▱');
        let mut reversed: Vec<_> = template.iter().rev().cloned().collect();
        template.append(&mut reversed);
        template
    };

    let template: Vec<_> = template.iter().map(|it| it.as_str()).collect();

    ProgressStyle::default_spinner()
        .tick_strings(template.as_slice())
        .template("{spinner:.cyan.bold} {msg}")
        .unwrap()
}

pub fn generate_template(width: usize, extent: usize, ch1: char, ch2: char) -> Vec<String> {
    (0..=(width + extent))
        .map(|pos| {
            let pad_start = pos.saturating_sub(extent);
            let within = extent
                .saturating_sub(extent.saturating_sub(pos))
                .saturating_sub((pos + extent).saturating_sub(width + extent));
            let pad_end = (width + extent).saturating_sub(pos + extent);

            format!(
                "{before}{extent}{after}",
                before = std::iter::repeat(ch2).take(pad_start).collect::<String>(),
                extent = std::iter::repeat(ch1).take(within).collect::<String>(),
                after = std::iter::repeat(ch2).take(pad_end).collect::<String>(),
            )
        })
        .collect()
}
