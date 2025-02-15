use super::{operator, url};
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{Category, Example, PipelineData, Signature, Span, SyntaxShape, Value};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "url scheme"
    }

    fn signature(&self) -> Signature {
        Signature::build("url scheme")
            .rest(
                "rest",
                SyntaxShape::CellPath,
                "optionally operate by cell path",
            )
            .category(Category::Network)
    }

    fn usage(&self) -> &str {
        "gets the scheme (eg http, file) of a url"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        operator(engine_state, stack, call, input, &url::Url::scheme)
    }

    fn examples(&self) -> Vec<Example> {
        let span = Span::unknown();
        vec![
            Example {
                description: "Get scheme of a url",
                example: "echo 'http://www.example.com' | url scheme",
                result: Some(Value::String {
                    val: "http".to_string(),
                    span,
                }),
            },
            Example {
                description: "You get an empty string if there is no scheme",
                example: "echo 'test' | url scheme",
                result: Some(Value::String {
                    val: "".to_string(),
                    span,
                }),
            },
        ]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
