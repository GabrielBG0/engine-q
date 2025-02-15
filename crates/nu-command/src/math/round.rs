use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "math round"
    }

    fn signature(&self) -> Signature {
        Signature::build("math round")
            .named(
                "precision",
                SyntaxShape::Number,
                "digits of precision",
                Some('p'),
            )
            .category(Category::Math)
    }

    fn usage(&self) -> &str {
        "Applies the round function to a list of numbers"
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let precision_param: Option<i64> = call.get_flag(engine_state, stack, "precision")?;
        let head = call.head;
        input.map(
            move |value| operate(value, head, precision_param),
            engine_state.ctrlc.clone(),
        )
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Apply the round function to a list of numbers",
                example: "[1.5 2.3 -3.1] | math round",
                result: Some(Value::List {
                    vals: vec![Value::test_int(2), Value::test_int(2), Value::test_int(-3)],
                    span: Span::unknown(),
                }),
            },
            Example {
                description: "Apply the round function with precision specified",
                example: "[1.555 2.333 -3.111] | math round -p 2",
                result: Some(Value::List {
                    vals: vec![
                        Value::Float {
                            val: 1.56,
                            span: Span::unknown(),
                        },
                        Value::Float {
                            val: 2.33,
                            span: Span::unknown(),
                        },
                        Value::Float {
                            val: -3.11,
                            span: Span::unknown(),
                        },
                    ],
                    span: Span::unknown(),
                }),
            },
        ]
    }
}

fn operate(value: Value, head: Span, precision: Option<i64>) -> Value {
    match value {
        Value::Float { val, span } => match precision {
            Some(precision_number) => Value::Float {
                val: ((val * ((10_f64).powf(precision_number as f64))).round()
                    / (10_f64).powf(precision_number as f64)),
                span,
            },
            None => Value::Int {
                val: val.round() as i64,
                span,
            },
        },
        Value::Int { .. } => value,
        other => Value::Error {
            error: ShellError::UnsupportedInput(
                String::from("Only numerical values are supported"),
                other.span().unwrap_or(head),
            ),
        },
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
