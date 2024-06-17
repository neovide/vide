use crate::shader::Shader;

pub(crate) struct PreprocessedFile {
    pub filename: String,
    pub content: String,
    pub line_nr: usize,
    pub num_lines: usize,
}

fn preprocess_line(line: &str, preprocessed_files: &mut Vec<PreprocessedFile>) -> String {
    let filename_pattern: &[char] = &[' ', '"', '\'', '<', '>'];
    if line.starts_with("#include") {
        let ret = "// ".to_string() + line;
        if let Some((_, filename)) = line.split_once(' ') {
            let filename = filename.trim_matches(filename_pattern).to_string();
            let already_processed = preprocessed_files.iter().any(|f| f.filename == filename);
            if already_processed {
                return ret;
            }

            if let Some(file) = Shader::get(&filename) {
                preprocess(&file.data, &filename, preprocessed_files);
            } else {
                return line.to_string() + " // not found";
            }
        }
        return ret;
    }
    line.to_string()
}

fn preprocess(data: &[u8], filename: &str, preprocessed_files: &mut Vec<PreprocessedFile>) {
    let content = std::str::from_utf8(data).unwrap();
    let lines = content
        .lines()
        .map(|line| preprocess_line(line, preprocessed_files))
        .collect::<Vec<String>>();
    let content = lines.join("\n");

    let line_nr = if let Some(file) = preprocessed_files.last() {
        file.line_nr + file.num_lines
    } else {
        0
    };
    preprocessed_files.push(PreprocessedFile {
        content,
        filename: filename.to_string(),
        line_nr,
        num_lines: lines.len(),
    });
}

pub(crate) struct Preprocessor {
    pub content: String,
    pub files: Vec<PreprocessedFile>,
}

impl Preprocessor {
    pub fn new(data: &[u8], filename: &str) -> Self {
        let mut files = Vec::new();
        preprocess(data, filename, &mut files);

        let content: String = files
            .iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<&str>>()
            .join("");

        Self { content, files }
    }

    pub fn get_file_and_start(&self, start: usize) -> (usize, usize) {
        let mut start = start;
        for (index, file) in self.files.iter().enumerate() {
            if start < file.content.len() {
                log::error!("{index}");
                return (index, start);
            }
            start -= file.content.len();
        }
        (self.files.len(), 0)
    }
}
