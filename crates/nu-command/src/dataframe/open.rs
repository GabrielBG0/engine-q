use super::values::NuDataFrame;
use nu_engine::CallExt;
use nu_protocol::{
    ast::Call,
    engine::{Command, EngineState, Stack},
    Category, Example, PipelineData, ShellError, Signature, Spanned, SyntaxShape,
};
use std::{fs::File, path::PathBuf};

use polars::prelude::{CsvEncoding, CsvReader, JsonReader, ParquetReader, SerReader};

#[derive(Clone)]
pub struct OpenDataFrame;

impl Command for OpenDataFrame {
    fn name(&self) -> &str {
        "dataframe open"
    }

    fn usage(&self) -> &str {
        "Opens csv, json or parquet file to create dataframe"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required(
                "file",
                SyntaxShape::Filepath,
                "file path to load values from",
            )
            .named(
                "delimiter",
                SyntaxShape::String,
                "file delimiter character. CSV file",
                Some('d'),
            )
            .switch(
                "no-header",
                "Indicates if file doesn't have header. CSV file",
                None,
            )
            .named(
                "infer-schema",
                SyntaxShape::Number,
                "Number of rows to infer the schema of the file. CSV file",
                None,
            )
            .named(
                "skip-rows",
                SyntaxShape::Number,
                "Number of rows to skip from file. CSV file",
                None,
            )
            .named(
                "columns",
                SyntaxShape::List(Box::new(SyntaxShape::String)),
                "Columns to be selected from csv file. CSV and Parquet file",
                None,
            )
            .category(Category::Custom("dataframe".into()))
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Takes a file name and creates a dataframe",
            example: "dataframe open test.csv",
            result: None,
        }]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        command(engine_state, stack, call)
    }
}

fn command(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<PipelineData, ShellError> {
    let span = call.head;
    let file: Spanned<PathBuf> = call.req(engine_state, stack, 0)?;

    match file.item.extension() {
        Some(e) => match e.to_str() {
            Some("csv") => from_csv(engine_state, stack, call),
            Some("parquet") => from_parquet(engine_state, stack, call),
            Some("json") => from_json(engine_state, stack, call),
            _ => Err(ShellError::FileNotFoundCustom(
                "Not a csv, parquet or json file".into(),
                file.span,
            )),
        },
        None => Err(ShellError::FileNotFoundCustom(
            "File without extension".into(),
            file.span,
        )),
    }
    .map(|df| PipelineData::Value(NuDataFrame::dataframe_into_value(df, span), None))
}

fn from_parquet(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Spanned<PathBuf> = call.req(engine_state, stack, 0)?;
    let columns: Option<Vec<String>> = call.get_flag(engine_state, stack, "columns")?;

    let r = File::open(&file.item).map_err(|e| {
        ShellError::SpannedLabeledError("Error opening file".into(), e.to_string(), file.span)
    })?;
    let reader = ParquetReader::new(r);

    let reader = match columns {
        None => reader,
        Some(columns) => reader.with_columns(Some(columns)),
    };

    reader.finish().map_err(|e| {
        ShellError::SpannedLabeledError(
            "Parquet reader error".into(),
            format!("{:?}", e),
            call.head,
        )
    })
}

fn from_json(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Spanned<PathBuf> = call.req(engine_state, stack, 0)?;

    let r = File::open(&file.item).map_err(|e| {
        ShellError::SpannedLabeledError("Error opening file".into(), e.to_string(), file.span)
    })?;

    let reader = JsonReader::new(r);

    reader.finish().map_err(|e| {
        ShellError::SpannedLabeledError("Json reader error".into(), format!("{:?}", e), call.head)
    })
}

fn from_csv(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
) -> Result<polars::prelude::DataFrame, ShellError> {
    let file: Spanned<PathBuf> = call.req(engine_state, stack, 0)?;
    let delimiter: Option<Spanned<String>> = call.get_flag(engine_state, stack, "delimiter")?;
    let no_header: bool = call.has_flag("no_header");
    let infer_schema: Option<usize> = call.get_flag(engine_state, stack, "infer_schema")?;
    let skip_rows: Option<usize> = call.get_flag(engine_state, stack, "skip_rows")?;
    let columns: Option<Vec<String>> = call.get_flag(engine_state, stack, "columns")?;

    let csv_reader = CsvReader::from_path(&file.item)
        .map_err(|e| {
            ShellError::SpannedLabeledError(
                "Error creating CSV reader".into(),
                e.to_string(),
                file.span,
            )
        })?
        .with_encoding(CsvEncoding::LossyUtf8);

    let csv_reader = match delimiter {
        None => csv_reader,
        Some(d) => {
            if d.item.len() != 1 {
                return Err(ShellError::SpannedLabeledError(
                    "Incorrect delimiter".into(),
                    "Delimiter has to be one character".into(),
                    d.span,
                ));
            } else {
                let delimiter = match d.item.chars().next() {
                    Some(d) => d as u8,
                    None => unreachable!(),
                };
                csv_reader.with_delimiter(delimiter)
            }
        }
    };

    let csv_reader = csv_reader.has_header(!no_header);

    let csv_reader = match infer_schema {
        None => csv_reader,
        Some(r) => csv_reader.infer_schema(Some(r)),
    };

    let csv_reader = match skip_rows {
        None => csv_reader,
        Some(r) => csv_reader.with_skip_rows(r),
    };

    let csv_reader = match columns {
        None => csv_reader,
        Some(columns) => csv_reader.with_columns(Some(columns)),
    };

    csv_reader.finish().map_err(|e| {
        ShellError::SpannedLabeledError(
            "Parquet reader error".into(),
            format!("{:?}", e),
            call.head,
        )
    })
}
