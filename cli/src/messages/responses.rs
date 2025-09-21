use engine::Move;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum UCIResponse {
    IDName { name: String },
    IDAuthor { author: String },
    UCIOk,
    ReadyOk,
    BestMove { mve: Move, ponder: Option<Move> },
    Option { option: UCIOption },
}

impl std::fmt::Display for UCIResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res_str = match self {
            UCIResponse::IDName { name } => &format!("id name {}", name),
            UCIResponse::IDAuthor { author } => &format!("id author {}", author),
            UCIResponse::UCIOk => "uciok",
            UCIResponse::ReadyOk => "readyok",
            UCIResponse::BestMove { mve, ponder: _ } => {
                &format!("bestmove {}", mve.to_string().to_lowercase())
            }
            UCIResponse::Option { option } => &format!("option {}", option),
        };
        write!(f, "{}", res_str)
    }
}

impl Into<String> for UCIResponse {
    fn into(self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
pub struct UCIOption {
    name: String,
    type_: UCIOptionType,
    default: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum UCIOptionType {
    Check,
    Spin { range_start: i32, range_end: i32 },
    Combo { options: Vec<String> },
    Button,
    String,
}

impl std::fmt::Display for UCIOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_upper = format!("{:?}", self.type_);
        write!(f, "option {} type {}", self.name, type_upper.to_lowercase())?;
        if let Some(default) = &self.default {
            write!(f, " default {}", default)?;
        }
        match &self.type_ {
            UCIOptionType::Spin {
                range_start,
                range_end,
            } => {
                write!(f, "min {} max {}", range_start, range_end)?;
            }
            UCIOptionType::Combo { options } => {
                let options_str_vec: Vec<String> =
                    options.iter().map(|o| format!("option {}", o)).collect();
                let options_str = options_str_vec.join(" ");
                write!(f, "{}", options_str)?;
            }
            _ => {}
        };
        Ok(())
    }
}
