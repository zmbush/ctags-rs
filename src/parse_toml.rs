use std::fmt;
use std::error::Error;
use toml::{self, Value};

#[derive(Debug)]
enum TomlParseError {
    TypeError,
    LookupError(&'static str),
    Default
}

type ParseResult<S> = Result<S, TomlParseError>;

impl Default for TomlParseError {
    fn default() -> TomlParseError {
        TomlParseError::Default
    }
}

impl fmt::Display for TomlParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TomlParseError {
    fn description(&self) -> &str {
        "CTagsError"
    }
}

pub trait ParseValue {
    fn parse<F>(&self) -> Result<F, F::Err> where F: FromValue;
}

impl ParseValue for toml::Value {
    fn parse<F>(&self) -> Result<F, F::Err> where F: FromValue {
        F::from_value(self)
    }
}

impl<'a> ParseValue for Option<&'a toml::Value> {
    fn parse<F>(&self) -> Result<F, F::Err> where F: FromValue {
        match *self {
            Some(s) => s.parse(),
            None => Err(F::Err::default())
        }
    }
}

trait FromValue {
    type Err: Default;

    fn from_value(v: &toml::Value) -> Result<Self, Self::Err>;
}

type PackageString = String;

impl FromValue for PackageString {
    type Err = TomlParseError;

    fn from_value(v: &toml::Value) -> ParseResult<PackageString> {
        match *v {
            Value::String(ref s) => Ok(s.clone()),
            _ => Err(TomlParseError::TypeError)
        }
    }
}

impl<V: FromValue> FromValue for Vec<V> {
    type Err = V::Err;

    fn from_value(v: &toml::Value) -> Result<Vec<V>, V::Err> {
        match *v {
            Value::Array(ref a) => {
                let mut ret = Vec::new();
                for elem in a {
                    ret.push(try!(elem.parse()));
                }
                Ok(ret)
            },
            _ => Err(V::Err::default())
        }
    }
}

macro_rules! try_get {
    ($tab:ident, $name:expr) => (match $tab.get($name).parse() {
        Ok(val) => val,
        Err(_) => {
            return Err(TomlParseError::LookupError($name))
        }
    })
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub source: String,
    pub version: String,
    pub dependencies: Option<Vec<String>>
}

impl FromValue for Package {
    type Err = TomlParseError;

    fn from_value(v: &toml::Value) -> ParseResult<Package> {
        match *v {
            Value::Table(ref table) => {
                Ok(Package {
                    name: try_get!(table, "name"),
                    source: try_get!(table, "source"),
                    version: try_get!(table, "version"),
                    dependencies: table.get("dependencies").parse().ok()
                })
            },
            _ => Err(TomlParseError::TypeError)
        }
    }
}

#[derive(Debug)]
pub struct CargoLock {
    pub name: String,
    pub version: String,
    pub dependencies: Option<Vec<String>>,
    pub packages: Option<Vec<Package>>
}

impl FromValue for CargoLock {
    type Err = TomlParseError;

    fn from_value(v: &toml::Value) -> ParseResult<CargoLock> {
        match *v {
            Value::Table(ref table) => {
                let root = try!(table.get("root").ok_or(TomlParseError::LookupError("root")));

                match *root {
                    Value::Table(ref root) => {
                        Ok(CargoLock {
                            name: try_get!(root, "name"),
                            version: try_get!(root, "version"),
                            dependencies: root.get("dependencies").parse().ok(),
                            packages: table.get("package").parse().ok()
                        })
                    },
                    _ => Err(TomlParseError::TypeError)
                }
            },
            _ => Err(TomlParseError::TypeError)
        }
    }
}
