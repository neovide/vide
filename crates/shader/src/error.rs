use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{emit, Config},
};
use log::error;
use std::{error::Error, path::Path};
use wgpu::naga::{front::glsl::ParseErrors, WithSpan};

trait ToDiagnostic {
    fn to_diagnostic(&self) -> Vec<Diagnostic<usize>>;
}

impl ToDiagnostic for ParseErrors {
    fn to_diagnostic(&self) -> Vec<Diagnostic<usize>> {
        self.errors
            .iter()
            .map(|err| {
                let mut diagnostic = Diagnostic::error().with_message(err.kind.to_string());

                if let Some(range) = err.meta.to_range() {
                    diagnostic = diagnostic.with_labels(vec![Label::primary(0, range)]);
                }
                diagnostic
            })
            .collect::<Vec<_>>()
    }
}

impl<E: Error> ToDiagnostic for WithSpan<E> {
    fn to_diagnostic(&self) -> Vec<Diagnostic<usize>> {
        let diagnostic = Diagnostic::error()
            .with_message(self.as_inner().to_string())
            .with_labels(
                self.spans()
                    .map(|&(span, ref desc)| {
                        Label::primary(0, span.to_range().unwrap()).with_message(desc.to_owned())
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
    fn log_errors(&self, path: &Path);
}

impl<T> ErrorLogger for wgpu::naga::error::ShaderError<T>
where
    T: ToDiagnostic,
{
    fn log_errors(&self, path: &Path) {
        let path = path.to_str().unwrap_or_default();
        let label = self.label.as_ref().map_or("", |s| s.as_str());
        let error = self.inner.as_ref();

        let mut writer = termcolor::Ansi::new(Vec::new());

        let path = path.to_string();
        let mut files = SimpleFiles::new();
        files.add(path, &self.source);
        let config = Config::default();

        for diagnostic in &error.to_diagnostic() {
            emit(&mut writer, &config, &files, diagnostic).expect("cannot write error");
        }

        let result = String::from_utf8(writer.into_inner()).unwrap();
        error!("{}: {}", label, result);
    }
}

impl ErrorLogger for wgpu::Error {
    fn log_errors(&self, path: &Path) {
        if let wgpu::Error::Validation { source, .. } = self {
            match source
                .source()
                .and_then(|s| s.downcast_ref::<wgpu::core::pipeline::CreateShaderModuleError>())
            {
                Some(wgpu::core::pipeline::CreateShaderModuleError::ParsingGlsl(error)) => {
                    return error.log_errors(path)
                }
                Some(wgpu::core::pipeline::CreateShaderModuleError::Validation(error)) => {
                    return error.log_errors(path)
                }
                _ => {}
            }
        };

        error!("{}", self);
    }
}
