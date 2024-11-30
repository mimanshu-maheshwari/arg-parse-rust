use std::collections::{HashMap, HashSet};

use crate::{error::ParseError, matches::Matches};

#[derive(Debug, Clone, PartialEq)]
pub struct Opt {
    pub name: String, 
    pub short: Option<char>, 
    pub long: Option<String>, 
    pub help: Option<String>,
    pub default: Option<String>, 
    pub action: Action, 
    pub required: bool,
}

impl Opt {

    pub fn name(name: &str) -> Self {
        Self {
            name: name.into(), 
            short:None, 
            long: None, 
            help: None, 
            default: None, 
                     action: Action::Set,
                     required:false
        }
    }

    pub fn short(mut self, short: char) -> Self {
        self.short= Some(short);
        self
    }

    pub fn long(mut self, long: &str) -> Self {
        self.long = Some(long.into()); 
        self
    }

    pub fn help(mut self, help: &str) -> Self {
        self.help = Some(help.into()); 
        self
    }

    pub fn default(mut self, default: &str) -> Self {
        self.default = Some(default.into()); 
        self
    }

    pub fn action(mut self, action: Action) -> Self {
        self.action = action; 
        self
    }

    pub fn required(mut self, required:bool) -> Self {
        self.required = required;
        self
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Set, 
    Append, 
    SetTrue,
    SetFalse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Single(String), 
    Multi(Vec<String>),
    Flag(bool),
}

#[derive(Debug, PartialEq)]
pub struct Opts {
    opts: Vec<Opt>,
}

impl Opts {

    pub fn new(opts: Vec<Opt>) -> Result<Opts, String> {
        let args = Opts { opts };
        args.validate()?;
        Ok(args)
    }

    pub fn add(&mut self, arg: Opt) -> Result<(), String> {
        self.opts.push(arg);
        self.validate()
    }

    pub fn parse(&self, args: Vec<String>) -> Result<Matches, ParseError> {
        let mut args = args.into_iter();
        let exec_name = match args.next() {
            Some(s) => s, 
            None => return Err(ParseError::MissingProgramName),
        };
        let mut positional = vec![];
        let mut named = HashMap::new();
        self.populate_defaults(&mut named);
        while let Some(arg) = args.next() {
            if arg.starts_with("-") {
                let opt = self.find_opt(&arg)?;
                match opt.action {
                    Action::Set      => {
                        if let Some(value) = args.next() {
                            named.insert(opt.name.clone(), Value::Single(value));
                        } else {
                            return Err(ParseError::MissingValue(opt.name.to_owned()));
                        }
                    },
                    Action::Append   => {
                        match (args.next(), named.get_mut(&opt.name)) {
                            (None, _) => {
                                return Err(ParseError::MissingValue(opt.name.clone()));
                            }, 
                            (Some(val), Some(Value::Multi(vals))) => {
                                vals.push(val);
                            },
                            (Some(val), None) => {
                                named.insert(opt.name.clone(), Value::Multi(vec![val]));
                            },
                            _ => return Err(ParseError::BadInternalState),
                        }
                    },
                    Action::SetTrue  => {
                        named.insert(opt.name.clone(), Value::Flag(true));
                    },
                    Action::SetFalse => {
                        named.insert(opt.name.clone(), Value::Flag(false));
                    },
                }
            } else {
                positional.push(arg);
            }
        }
        Ok(Matches::new(exec_name, positional, named))
    }
    pub fn populate_defaults(&self, named: &mut HashMap<String, Value>) {
        for opt in &self.opts {
            if let Some(default) = &opt.default {
                named.insert(opt.name.clone(), Value::Single(default.to_owned()));
            } else {
                match opt.action {
                    Action::Set => {}, 
                    Action::Append => {
                        named.insert(opt.name.clone(), Value::Multi(vec![]));
                    }, 
                    Action::SetTrue => {
                        named.insert(opt.name.clone(), Value::Flag(true));
                    }, 
                    Action::SetFalse => {
                        named.insert(opt.name.clone(), Value::Flag(false));
                    },
                }
            }
        }
    }
    fn find_opt(&self, arg: &str) -> Result<&Opt, ParseError> {
        let opt = if arg.starts_with("--") {
            let long = arg.strip_prefix("--").unwrap();
            self.opts.iter().find(|o| o.long.as_deref() == Some(long))
        } else if arg.starts_with("-") {
            if arg.chars().count() != 2 {
                return Err(ParseError::MalformedOption(arg.to_string()));
            }
            let short = arg.chars().nth(1);
            self.opts.iter().find(|o| o.short == short)
        } else {
            return Err(ParseError::UnexpectedOption(arg.to_string()));
        };
        if let Some(opt) = opt {
            Ok(opt)
        } else {
            Err(ParseError::UnexpectedOption(arg.to_string()))
        }
    }
    fn validate(&self) -> Result<(), String> {
        let mut names: HashSet<String> = HashSet::new();
        let mut short: HashSet<char>   = HashSet::new();
        let mut long : HashSet<String> = HashSet::new();

        for arg in &self.opts {
            if names.contains(&arg.name) {
                return Err(format!(
                        "Optument names must be unique; found two with name {}",
                        arg.name
                ));
            } else if arg.short.is_some() && short.contains(&arg.short.unwrap()) {
                return Err(format!(
                        "Short flags must be unique; found two with short flag -{}",
                        arg.short.unwrap()
                ));
            } else if arg.long.is_some() && long.contains(arg.long.as_ref().unwrap()) {
                return Err(format!(
                        "Long flags must be unique; found two with long flag --{}",
                        arg.long.as_ref().unwrap()
                ));
            }

            names.insert(arg.name.to_string());
            if let Some(c) = arg.short {
                short.insert(c);
            }
            if let Some(s) = &arg.long {
                long.insert(s.to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validates_empty_args() {
        let _ = Opts::new(vec![]).expect("should validate");
    }

    #[test]
    fn detects_duplicate_names() {
        let opts = Opts::new(vec![
            Opt::name("host"),
            Opt::name("port"),
            Opt::name("port"),
        ]);
        assert_eq!(
            opts,
            Err(format!(
                "Optument names must be unique; found two with name port"
            ))
        );
    }

    #[test]
    fn detects_duplicate_short() {
        let opts = Opts::new(vec![
            Opt::name("host").short('p'),
            Opt::name("port").short('p'),
            Opt::name("threads").short('t'),
        ]);
        assert_eq!(
            opts,
            Err(format!(
                "Short flags must be unique; found two with short flag -p"
            ))
        );
    }

    #[test]
    fn detects_duplicate_long() {
        let opts = Opts::new(vec![
            Opt::name("host").long("host"),
            Opt::name("port").long("host"),
            Opt::name("threads").long("threads"),
        ]);
        assert_eq!(
            opts,
            Err(format!(
                "Long flags must be unique; found two with long flag --host"
            ))
        );
    }

    #[test]
    fn parses_positional_args() {
        let opts = Opts::new(vec![Opt::name("host").long("host")]).unwrap();
        let args: Vec<_> = ["myprogram", "1", "2", "blue"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let expected_positional: Vec<_> = args.iter().skip(1).cloned().collect();

        let matches = opts.parse(args);
        assert!(matches.is_ok());
        let matches = matches.unwrap();

        assert_eq!(matches.positional(), expected_positional);
    }

    #[test]
    fn parses_named_args() {
        let opts = Opts::new(vec![
            Opt::name("host").long("host"),
            Opt::name("verbose").long("verbose").action(Action::SetTrue),
            Opt::name("queue").short('q').action(Action::Append),
            Opt::name("nocolor")
                .short('n')
                .long("nocolor")
                .action(Action::SetFalse),
            Opt::name("missing").default("something"),
        ])
        .unwrap();
        let args: Vec<String> = vec![
            "myprogram",
            "1",
            "2",
            "--verbose",
            "-q",
            "items",
            "--host",
            "localhost",
            "-q",
            "-queue-name-with-dash",
            "-n",
            "blue",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let expected_positional: Vec<_> = vec!["1", "2", "blue"];

        let matches = opts.parse(args);
        dbg!(&matches);
        assert!(matches.is_ok());
        let matches = matches.unwrap();

        assert_eq!(matches.positional(), expected_positional);
        assert_eq!(matches.flag("verbose").unwrap(), Some(true));
        assert_eq!(matches.one("host").unwrap(), Some("localhost".to_string()));
        let queues: Vec<String> = matches.all("queue").unwrap();
        assert_eq!(
            queues,
            vec!["items".to_string(), "-queue-name-with-dash".to_string()]
        );

        assert_eq!(
            matches.one("missing").unwrap(),
            Some("something".to_string())
        );
    }
}
