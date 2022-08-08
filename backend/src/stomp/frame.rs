use std::{collections::HashMap, str::FromStr};

use nom::Finish;

#[derive(Debug)]
pub struct Frame {
    pub command: Command,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl FromStr for Frame {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parser::frame(s).finish() {
            Ok((_remainder, frame)) => Ok(frame),
            Err(nom::error::Error { input, code }) => Err(nom::error::Error {
                input: input.to_owned(),
                code,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Connect,
    Connected,
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        match s {
            "CONNECTED" => Command::Connected,
            "CONNECT" => Command::Connect,
            _ => unimplemented!("No other commands available"),
        }
    }
}

mod parser {
    use std::collections::HashMap;

    use nom::{
        branch::alt,
        bytes::complete::{tag, take_until, take_till1},
        error::context,
        multi::{separated_list0, count},
        sequence::{separated_pair, tuple},
        IResult, character::complete::{not_line_ending, line_ending}, combinator::opt,
    };

    use super::{Command, Frame};

    fn command(input: &str) -> IResult<&str, Command> {
        context("command", alt((tag("CONNECTED"), tag("CONNECT"))))(input)
            .map(|(input, res)| (input, res.into()))
    }

    fn header(input: &str) -> IResult<&str, (&str, &str)> {
        context(
            "header",
                separated_pair(take_until(":"), tag(":"), not_line_ending)
        )(input)
    }

    fn headers(input: &str) -> IResult<&str, HashMap<&str, &str>> {
        let (input, parsed) = context("headers", separated_list0(line_ending, header))(input)?;

        let headers = parsed
            .into_iter()
            //.map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        Ok((input, headers))
    }

    fn body(input: &str) -> IResult<&str, &str> {
        context(
            "body",
            take_till1(|c| c == '\0')
        )(input)
    }

    pub fn frame(input: &str) -> IResult<&str, Frame> {
        let (input, (command, _, headers, _, body)) = context(
            "frame",
            tuple((command, line_ending, headers, count(line_ending, 2), opt(body)))
        )(input)?;
        
        let frame = Frame {
            command,
            headers: headers.into_iter().map(|(key, value)| (key.to_owned(), value.to_owned())).collect(),
            body: body.map(str::to_owned)
        };

        Ok((input, frame))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn command_works() {
            let (remainder, parsed) = command("CONNECT\n").unwrap();
            assert_eq!(parsed, Command::Connect);
            assert_eq!(remainder, "\n");

            let (remainder, parsed) = command("CONNECTED\n").unwrap();
            assert_eq!(parsed, Command::Connected);
            assert_eq!(remainder, "\n");
        }

        #[test]
        fn header_works() {
            let (_, (key, value)) = header("content-type:application/json\r\n").unwrap();
            assert_eq!(key, "content-type");
            assert_eq!(value, "application/json");
        }

        #[test]
        fn headers_works() {
            let (remainder, headers) =
                headers("content-type:application/json\ncontent-length:5\r\nsession:gugus\r\n")
                    .unwrap();
            assert_eq!(
                headers.get("content-type"),
                Some(&"application/json")
            );
            assert_eq!(headers.get("content-length"), Some(&"5"));
            assert_eq!(headers.get("session"), Some(&"gugus"));
            assert_eq!(remainder, "\r\n");
        }

        #[test]
        fn empty_headers_works() {
            let (_remainder, _headers) = headers("\r\n").unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frame_no_body() {
        let frame: Frame = "CONNECT\ncontent-type:application/json\n\n\0".parse().unwrap();

        assert_eq!(frame.command, Command::Connect);
        assert_eq!(frame.headers.get("content-type"), Some(&"application/json".to_owned()));
        assert_eq!(frame.body, None);
    }

    #[test]
    fn parse_frame_no_headers() {
        let frame: Frame = "CONNECT\r\n\r\n\r\n\0".parse().unwrap();
        
        assert_eq!(frame.command, Command::Connect);
        assert!(frame.headers.is_empty());
        assert_eq!(frame.body, None);
    }

    #[test]
    fn parse_frame_body() {
        let frame: Frame = "CONNECT\r\ncontent-type:application/json\r\n\r\nbody\0".parse().unwrap();

        assert_eq!(frame.command, Command::Connect);
        assert_eq!(frame.headers.get("content-type"), Some(&"application/json".to_owned()));
        assert_eq!(frame.body, Some("body".to_owned()));
    }
}
