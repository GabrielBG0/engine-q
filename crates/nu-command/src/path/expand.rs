use std::path::Path;

use nu_engine::CallExt;
use nu_path::{canonicalize, expand_path};
use nu_protocol::{engine::Command, Example, ShellError, Signature, Span, SyntaxShape, Value};

use super::PathSubcommandArguments;

struct Arguments {
    strict: bool,
    columns: Option<Vec<String>>,
}

impl PathSubcommandArguments for Arguments {
    fn get_columns(&self) -> Option<Vec<String>> {
        self.columns.clone()
    }
}

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "path expand"
    }

    fn signature(&self) -> Signature {
        Signature::build("path expand")
            .switch(
                "strict",
                "Throw an error if the path could not be expanded",
                Some('s'),
            )
            .named(
                "columns",
                SyntaxShape::Table,
                "Optionally operate by column path",
                Some('c'),
            )
    }

    fn usage(&self) -> &str {
        "Try to expand a path to its absolute form"
    }

    fn run(
        &self,
        engine_state: &nu_protocol::engine::EngineState,
        stack: &mut nu_protocol::engine::Stack,
        call: &nu_protocol::ast::Call,
        input: nu_protocol::PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let head = call.head;
        let args = Arguments {
            strict: call.has_flag("strict"),
            columns: call.get_flag(engine_state, stack, "columns")?,
        };

        input.map(
            move |value| super::operate(&expand, &args, value, head),
            engine_state.ctrlc.clone(),
        )
    }

    #[cfg(windows)]
    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Expand an absolute path",
                example: r"'C:\Users\joe\foo\..\bar' | path expand",
                result: Some(Value::test_string(r"C:\Users\joe\bar")),
            },
            Example {
                description: "Expand a path in a column",
                example: "ls | path expand -c [ name ]",
                result: None,
            },
            Example {
                description: "Expand a relative path",
                example: r"'foo\..\bar' | path expand",
                result: Some(Value::test_string("bar")),
            },
        ]
    }

    #[cfg(not(windows))]
    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Expand an absolute path",
                example: "'/home/joe/foo/../bar' | path expand",
                result: Some(Value::test_string("/home/joe/bar")),
            },
            Example {
                description: "Expand a path in a column",
                example: "ls | path expand -c [ name ]",
                result: None,
            },
            Example {
                description: "Expand a relative path",
                example: "'foo/../bar' | path expand",
                result: Some(Value::test_string("bar")),
            },
        ]
    }
}

fn expand(path: &Path, span: Span, args: &Arguments) -> Value {
    if let Ok(p) = canonicalize(path) {
        Value::string(p.to_string_lossy(), span)
    } else if args.strict {
        Value::Error {
            error: ShellError::LabeledError(
                "Could not expand path".into(),
                "could not be expanded (path might not exist, non-final \
                    component is not a directory, or other cause)"
                    .into(),
            ),
        }
    } else {
        Value::string(expand_path(path).to_string_lossy(), span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
