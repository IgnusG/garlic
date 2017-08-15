error_chain!{}

/** Pretty-prints current app status **/
macro_rules! status {
    ($msg:expr, $type:expr) => {{
        use colored::*;
        let color = match $type.as_ref() {
            "msg" => { "green" }
            "error" => { "red" }
            "warn" => { "yellow" }
            _ => { "white" }
        };
        info!("{}", $msg.color(color));
    }};
    ($msg:expr) => {{
        status!($msg, "msg");
    }}
}
/** Pretty-prints sub-status (to be nested under status)**/
macro_rules! note {
    ($msg:expr) => {{
        use colored::*;
        info!(" {}  {}", "→".dimmed() , $msg.dimmed());
    }}
}
/** Pretty-prints an error traceback **/
macro_rules! trace_error {
    ($($b:tt)*) => {
        || -> Result<()> {
            $($b)*;
            Ok(())
        }().unwrap_or_else(|e| {
            use colored::*;
            use case::CaseExt;

            let e = &e;
            warn!("{} {}", "Problem:".yellow(), format!("{}", e).to_capitalized().yellow());
            for e in e.iter().skip(1) {
                warn!(" {}: {}", "→ Caused by".bold().dimmed() , format!("{}", e).to_capitalized());
            };
        });
    }
}
/** Adds the provided error message to error chain and calls `trace_error` **/
macro_rules! trace_labeled_error {
    ($msg:expr, $($b:tt)*) => {
        trace_error! {
            || -> Result<()> {
                $($b)*;
                Ok(())
            }().chain_err(|| $msg)?;
        };
    }
}
/** Wrapper around error chain's result which panics on error **/
macro_rules! trace_panic {
    ( $($b:tt)* ) => {
        || -> Result<()> {
            $($b)*;
            Ok(())
        }().unwrap_or_else(|e| {
            panic!(e);
        });
    }
}
/** Adds the provided error message to error chain and calls `trace_panic` **/
macro_rules! trace_labeled_panic {
    ($msg:expr, $($b:tt)*) => {
        trace_panic! {
            || -> Result<()> {
                $($b)*;
                Ok(())
            }().chain_err(|| $msg)?;
        };
    }
}
