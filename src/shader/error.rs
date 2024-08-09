use crate::shader::Preprocessor;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::{Files, SimpleFile},
    term::{emit, Config},
};
use log::error;
use std::error::Error;
use std::ops::Range;
use wgpu::naga::{error::ShaderError, front::glsl::ParseErrors, WithSpan};

trait ToDiagnostic {
    fn to_diagnostic(&self, preprocessor: &Preprocessor) -> Vec<Diagnostic<usize>>;
}

impl ToDiagnostic for ParseErrors {
    fn to_diagnostic(&self, preprocessor: &Preprocessor) -> Vec<Diagnostic<usize>> {
        self.errors
            .iter()
            .map(|err| {
                let mut diagnostic = Diagnostic::error().with_message(err.kind.to_string());

                if let Some(range) = err.meta.to_range() {
                    let (fileid, start) = preprocessor.get_file_and_start(range.start);
                    let range = start..range.end;
                    diagnostic = diagnostic.with_labels(vec![Label::primary(fileid, range)]);
                }
                diagnostic
            })
            .collect::<Vec<_>>()
    }
}

impl<E: Error> ToDiagnostic for WithSpan<E> {
    fn to_diagnostic(&self, preprocessor: &Preprocessor) -> Vec<Diagnostic<usize>> {
        let diagnostic = Diagnostic::error()
            .with_message(self.as_inner().to_string())
            .with_labels(
                self.spans()
                    .flat_map(|&(span, ref desc)| {
                        if let Some(range) = span.to_range() {
                            let (fileid, start) = preprocessor.get_file_and_start(range.start);
                            let range = start..range.end;
                            Some(Label::primary(fileid, range).with_message(desc.to_owned()))
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
            .with_notes({
                let mut notes = Vec::new();
                let mut source: &dyn Error = self.as_inner();
                while let Some(next) = Error::source(source) {
                    notes.push(next.to_string());
                    source = next;
                }
                notes
            });
        vec![diagnostic]
    }
}

pub(crate) trait ErrorLogger {
    fn log_errors(&self, preprocessor: &Preprocessor);
}

impl<T> ErrorLogger for ShaderError<T>
where
    T: ToDiagnostic,
{
    fn log_errors(&self, preprocessor: &Preprocessor) {
        let label = self.label.as_ref().map_or("", |s| s.as_str());
        let error = self.inner.as_ref();

        let mut writer = termcolor::Ansi::new(Vec::new());

        let config = Config::default();

        for diagnostic in &error.to_diagnostic(preprocessor) {
            emit(&mut writer, &config, preprocessor, diagnostic).expect("cannot write error");
        }

        let result = String::from_utf8(writer.into_inner()).unwrap();
        error!("{}: {}", label, result);
    }
}

impl ErrorLogger for wgpu::Error {
    fn log_errors(&self, preprocessor: &Preprocessor) {
        if let wgpu::Error::Validation { source, .. } = self {
            match source
                .source()
                .and_then(|s| s.downcast_ref::<wgpu::core::pipeline::CreateShaderModuleError>())
            {
                Some(wgpu::core::pipeline::CreateShaderModuleError::ParsingGlsl(error)) => {
                    return error.log_errors(preprocessor)
                }
                Some(wgpu::core::pipeline::CreateShaderModuleError::Validation(error)) => {
                    return error.log_errors(preprocessor)
                }
                _ => {}
            }
        };

        error!("{}", self);
    }
}

impl<'a> Files<'a> for Preprocessor {
    type FileId = usize;
    type Name = &'a str;
    type Source = &'a str;

    fn name(&'a self, id: Self::FileId) -> Result<Self::Name, codespan_reporting::files::Error> {
        if id < self.files.len() {
            Ok(&self.files[id].filename)
        } else {
            Err(codespan_reporting::files::Error::FileMissing)
        }
    }

    fn source(
        &'a self,
        id: Self::FileId,
    ) -> Result<Self::Source, codespan_reporting::files::Error> {
        if id < self.files.len() {
            Ok(&self.files[id].content)
        } else {
            Err(codespan_reporting::files::Error::FileMissing)
        }
    }

    fn line_index(
        &self,
        id: Self::FileId,
        byte_index: usize,
    ) -> Result<usize, codespan_reporting::files::Error> {
        if id < self.files.len() {
            let file = SimpleFile::new(&self.files[id].filename, &self.files[id].content);
            file.line_index((), byte_index)
        } else {
            Err(codespan_reporting::files::Error::FileMissing)
        }
    }

    fn line_range(
        &self,
        id: Self::FileId,
        line_index: usize,
    ) -> Result<Range<usize>, codespan_reporting::files::Error> {
        if id < self.files.len() {
            let file = SimpleFile::new(&self.files[id].filename, &self.files[id].content);
            file.line_range((), line_index)
        } else {
            Err(codespan_reporting::files::Error::FileMissing)
        }
    }
}
