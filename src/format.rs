#[macro_export]
macro_rules! info {
    (to: $writer:expr, from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{}{}{} {}\r",
            style("info[").bold().blue(),
            style($from).bold(),
            style("]:").bold().blue(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (to: $writer:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{} {}\r",
            style("info:").bold().blue(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {
        $crate::info!(to: std::io::stdout(), from: $from, $message $(, $( $args ),* )?)
    };

    ($message:literal $(, $($args:expr),*)? $(,)?) => {
        $crate::info!(to: std::io::stdout(), $message $(, $( $args ),* )?)
    };
}

#[macro_export]
macro_rules! success {
    (to: $writer:expr, from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{}{}{} {}\r",
            style("success[").bold().green(),
            style($from).bold(),
            style("]:").bold().green(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (to: $writer:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{} {}\r",
            style("success:").bold().green(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::success!(to: std::io::stdout(), from: $from, $message $(, $( $args ),* )?)
    }};

    ($message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::success!(to: std::io::stdout(), $message $(, $( $args ),* )?)
    }};
}

#[macro_export]
macro_rules! error {
    (to: $writer:expr, from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{}{}{} {}\r",
            style("error[").bold().red(),
            style($from).bold(),
            style("]:").bold().red(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (to: $writer:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{} {}\r",
            style("error:").bold().red(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::error!(to: std::io::stdout(), from: $from, $message $(, $( $args ),* )?)
    }};

    ($message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::error!(to: std::io::stdout(), $message $(, $( $args ),* )?)
    }};
}

#[macro_export]
macro_rules! warning {
    (to: $writer:expr, from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{}{}{} {}\r",
            style("warning[").bold().yellow(),
            style($from).bold(),
            style("]:").bold().yellow(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (to: $writer:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{} {}\r",
            style("warning:").bold().yellow(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::warning!(to: std::io::stdout(), from: $from, $message $(, $( $args ),* )?)
    }};

    ($message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::warning!(to: std::io::stdout(), $message $(, $( $args ),* )?)
    }};
}

#[macro_export]
macro_rules! debug {
    (to: $writer:expr, from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{}{}{} {}\r",
            style("debug[").bold().magenta(),
            style($from).bold(),
            style("]:").bold().magenta(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (to: $writer:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::std::io::Write;
        use ::console::style;
        writeln!(
            $writer,
            "{} {}\r",
            style("debug:").bold().magenta(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};

    (from: $from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::debug!(to: std::io::stdout(), from: $from, $message $(, $( $args ),* )?)
    }};

    ($message:literal $(, $($args:expr),*)? $(,)?) => {{
        $crate::debug!(to: std::io::stdout(), $message $(, $( $args ),* )?)
    }};
}

#[macro_export]
macro_rules! format_info {
    ($from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::console::style;
        format!(
            "{}{}{} {}\r",
            style("info[").bold().blue(),
            style($from).bold(),
            style("]:").bold().blue(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};
}

#[macro_export]
macro_rules! format_success {
    ($from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::console::style;
        format!(
            "{}{}{} {}\r",
            style("success[").bold().green(),
            style($from).bold(),
            style("]:").bold().green(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};
}

#[macro_export]
macro_rules! format_error {
    ($from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::console::style;
        format!(
            "{}{}{} {}\r",
            style("error[").bold().red(),
            style($from).bold(),
            style("]:").bold().red(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};
}

#[macro_export]
macro_rules! format_warning {
    ($from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::console::style;
        format!(
            "{}{}{} {}\r",
            style("warning[").bold().yellow(),
            style($from).bold(),
            style("]:").bold().yellow(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};
}

#[macro_export]
macro_rules! format_debug {
    ($from:expr, $message:literal $(, $($args:expr),*)? $(,)?) => {{
        use ::console::style;
        format!(
            "{}{}{} {}\r",
            style("debug[").bold().magenta(),
            style($from).bold(),
            style("]:").bold().magenta(),
            style(format_args!($message, $($($args),*)?)).bold(),
        )
    }};
}
