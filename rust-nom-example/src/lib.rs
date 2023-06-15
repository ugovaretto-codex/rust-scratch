use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take},
    character::complete::{alpha1, alphanumeric1, digit1, one_of},
    combinator::opt,
    error::{context, ErrorKind, VerboseError, VerboseErrorKind},
    multi::{count, many0, many1, many_m_n, separated_list1},
    sequence::{preceded, separated_pair, terminated, tuple},
    AsChar, Err as NomErr, IResult, InputTakeAtPosition,
};

#[derive(Debug, PartialEq, Eq)]
pub struct URI<'a> {
    scheme: Scheme,
    authority: Option<Authority<'a>>,
    host: Host,
    port: Option<u16>,
    path: Option<QueryParams<'a>>,
    fragment: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Scheme {
    HTTP,
    HTTPS,
}

pub type Authority<'a> = (&'a str, Option<&'a str>);

#[derive(Debug, PartialEq, Eq)]
pub enum Host {
    HOST(String),
    IP([u8; 4]),
}

pub type QueryParam<'a> = (&'a str, &'a str);
pub type QueryParams<'a> = Vec<QueryParam<'a>>;

impl From<&str> for Scheme {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_ref() {
            "http://" => Scheme::HTTP,
            "https://" => Scheme::HTTPS,
            _ => unimplemented!("Only HTTP and HTTPS supported"),
        }
    }
}

type Res<T, U> = IResult<T, U, VerboseError<T>>;

//-----------------------------------------------------------------------------
fn scheme(input: &str) -> Res<&str, Scheme> {
    context(
        "scheme",
        alt((tag_no_case("HTTP://"), tag_no_case("HTTPS://"))),
    )(input)
    .map(|(next_input, res)| (next_input, res.into()))
}

//-----------------------------------------------------------------------------
fn authority(input: &str) -> Res<&str, (&str, Option<&str>)> {
    context(
        "authority",
        terminated(
            separated_pair(alphanumeric1, opt(tag(":")), opt(alphanumeric1)),
            tag("@"),
        ),
    )(input)
}

// fn host(input: &str) -> Res<&str, Host> {
//     context(
//         "host",
//         alt((
//             tuple((many1(terminated(alphanumerichyphen1, tag("."))), alpha1)),
//             tuple((many_m_n(1, 1, alphanumerichyphen1), take(0 as usize))),
//         )),
//     )(input)
//     .map(|(next_input, mut res)| {
//         if !res.1.is_empty() {
//             res.0.push(res.1);
//         }
//         (next_input, Host::HOST(res.0.join(".")))
//     })
// }
//

fn mydigit1(input: &str) -> Res<&str, &str> {
    match digit1::<&str, VerboseError<&str>>(input) {
        Ok((next, parsed)) => Ok((next, parsed)),
        Err(e) => Err(e),
    }
}
fn host(input: &str) -> Res<&str, Host> {
    let mut res;
    if let Ok((next, parsed)) =
        tuple((many_m_n(3, 3, terminated(mydigit1, tag("."))), mydigit1))(input)
    {
        res = parsed.0;
        res.push(parsed.1);
        let ip_address: Result<Vec<_>, _> = res
            .iter()
            .map(|d| {
                let x = d.parse::<u8>().map_err(|_| d.to_owned())?;
                if x < 1_u8 || x > 254_u8 {
                    return Err(d.to_owned());
                } else {
                    return Ok(x);
                }
            })
            .collect();
        match ip_address {
            Ok(ip) => return Ok((next, Host::IP([ip[0], ip[1], ip[2], ip[3]]))),
            Err(_) => {
                return Err(NomErr::Error(VerboseError {
                    errors: vec![(input, VerboseErrorKind::Nom(ErrorKind::Digit))],
                }))
            }
        }
    }
    match context(
        "host",
        tuple((many0(terminated(alphanumerichyphen1, tag("."))), alpha1)),
    )(input)
    {
        Ok((next, parsed)) => {
            res = parsed.0;
            res.push(parsed.1);
            return Ok((next, Host::HOST(res.join("."))));
        }
        Err(e) => return Err(e),
    }
}
fn alphanumerichyphen1<T>(i: T) -> Res<T, T>
where
    T: InputTakeAtPosition,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    i.split_at_position1_complete(
        |item| {
            let char_item = item.as_char();
            !(char_item == '-') && !char_item.is_alphanum()
        },
        ErrorKind::AlphaNumeric,
    )
}
/// TEST
///

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{ErrorKind, VerboseError, VerboseErrorKind},
        Err as NomErr,
    };
    #[test]
    fn test_scheme() {
        assert_eq!(scheme("https://yay"), Ok(("yay", Scheme::HTTPS)));
        assert_eq!(scheme("http://yay"), Ok(("yay", Scheme::HTTP)));
        assert_eq!(
            scheme("bla://yay"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("bla://yay", VerboseErrorKind::Nom(ErrorKind::Alt)),
                    ("bla://yay", VerboseErrorKind::Context("scheme")),
                ]
            }))
        )
    }
    #[test]
    fn test_authority() {
        assert_eq!(
            authority("username:password@zupzup.org"),
            Ok(("zupzup.org", ("username", Some("password"))))
        );
        assert_eq!(
            authority("username@zupzup.org"),
            Ok(("zupzup.org", ("username", None)))
        );
        assert_eq!(
            authority("zupzup.org"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (".org", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    ("zupzup.org", VerboseErrorKind::Context("authority")),
                ]
            }))
        );
        assert_eq!(
            authority(":zupzup.org"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (
                        ":zupzup.org",
                        VerboseErrorKind::Nom(ErrorKind::AlphaNumeric)
                    ),
                    (":zupzup.org", VerboseErrorKind::Context("authority")),
                ]
            }))
        );
        assert_eq!(
            authority("username:passwordzupzup.org"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (".org", VerboseErrorKind::Nom(ErrorKind::Tag)),
                    (
                        "username:passwordzupzup.org",
                        VerboseErrorKind::Context("authority")
                    ),
                ]
            }))
        );
        assert_eq!(
            authority("@zupzup.org"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (
                        "@zupzup.org",
                        VerboseErrorKind::Nom(ErrorKind::AlphaNumeric)
                    ),
                    ("@zupzup.org", VerboseErrorKind::Context("authority")),
                ]
            }))
        )
    }
    #[test]
    fn test_host() {
        assert_eq!(
            host("localhost:8080"),
            Ok((":8080", Host::HOST("localhost".to_string())))
        );
        assert_eq!(
            host("example.org:8080"),
            Ok((":8080", Host::HOST("example.org".to_string())))
        );
        assert_eq!(
            host("some-subsite.example.org:8080"),
            Ok((":8080", Host::HOST("some-subsite.example.org".to_string())))
        );
        // assert_eq!(
        //     host("example.123"),
        //     Ok((".123", Host::HOST("example".to_string())))
        // );
        assert_eq!(
            host("$$$.com"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    ("$$$.com", VerboseErrorKind::Nom(ErrorKind::Alpha)),
                    ("$$$.com", VerboseErrorKind::Context("host")),
                ]
            }))
        );
        assert_eq!(host("124.124.44.3"), Ok(("", Host::IP([124, 124, 44, 3]))));
        assert_eq!(
            host("300.124.44.3"),
            Err(NomErr::Error(VerboseError {
                errors: vec![("300.124.44.3", VerboseErrorKind::Nom(ErrorKind::Digit))]
            }))
        );
        assert_eq!(
            host(".com"),
            Err(NomErr::Error(VerboseError {
                errors: vec![
                    (".com", VerboseErrorKind::Nom(ErrorKind::Alpha)),
                    (".com", VerboseErrorKind::Context("host")),
                ]
            }))
        );
    }
}
// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
