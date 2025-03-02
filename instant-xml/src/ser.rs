use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::{self};
use std::mem;

use super::Error;
use crate::ToXml;

pub struct Serializer<'xml, W: fmt::Write + ?Sized> {
    output: &'xml mut W,
    /// Map namespace keys to prefixes.
    ///
    /// The prefix map is updated using `Context` types that are held on the
    /// stack in the relevant `ToXml` implementation. If a prefix is already
    /// defined for a given namespace, we don't update the set the new prefix.
    prefixes: HashMap<&'static str, &'static str>,
    default_ns: &'static str,
    state: State,
}

impl<'xml, W: fmt::Write + ?Sized> Serializer<'xml, W> {
    pub fn new(output: &'xml mut W) -> Self {
        Self {
            output,
            prefixes: HashMap::new(),
            default_ns: "",
            state: State::Element,
        }
    }

    pub fn write_start(
        &mut self,
        name: &str,
        ns: &str,
        scalar: bool,
    ) -> Result<Option<&'static str>, Error> {
        if self.state != State::Element {
            return Err(Error::UnexpectedState);
        }

        let prefix = match ns == self.default_ns {
            true => {
                self.output.write_fmt(format_args!("<{name}"))?;
                None
            }
            // In the case of scalars, we can use the prefix instead of setting a
            // default namespace (since there are no child elements that will
            // expect the default namespace to be set).
            false => match (scalar, self.prefixes.get(ns)) {
                (false, _) | (true, None) => {
                    self.output
                        .write_fmt(format_args!("<{name} xmlns=\"{ns}\""))?;
                    None
                }
                (true, Some(&prefix)) => {
                    self.output.write_fmt(format_args!("<{prefix}:{name}"))?;
                    Some(prefix)
                }
            },
        };

        self.state = State::Attribute;
        Ok(prefix)
    }

    pub fn write_attr<V: ToXml + ?Sized>(
        &mut self,
        name: &str,
        ns: &str,
        value: &V,
    ) -> Result<(), Error> {
        if self.state != State::Attribute {
            return Err(Error::UnexpectedState);
        }

        match ns == self.default_ns {
            true => self.output.write_fmt(format_args!(" {name}=\""))?,
            false => {
                let prefix = self.prefixes.get(ns).ok_or(dbg!(Error::UnexpectedState))?;
                self.output.write_fmt(format_args!(" {prefix}:{name}=\""))?;
            }
        }

        self.state = State::Scalar;
        value.serialize(self)?;
        self.state = State::Attribute;
        self.output.write_char('"')?;
        Ok(())
    }

    pub fn write_str<V: fmt::Display + ?Sized>(&mut self, value: &V) -> Result<(), Error> {
        if !matches!(self.state, State::Element | State::Scalar) {
            return Err(Error::UnexpectedState);
        }

        self.output.write_fmt(format_args!("{}", value))?;
        self.state = State::Element;
        Ok(())
    }

    pub fn end_start(&mut self) -> Result<(), Error> {
        if self.state != State::Attribute {
            return Err(Error::UnexpectedState);
        }

        self.output.write_char('>')?;
        self.state = State::Element;
        Ok(())
    }

    pub fn write_close(&mut self, prefix: Option<&str>, name: &str) -> Result<(), Error> {
        if self.state != State::Element {
            return Err(Error::UnexpectedState);
        }

        match prefix {
            Some(prefix) => self.output.write_fmt(format_args!("</{prefix}:{name}>"))?,
            None => self.output.write_fmt(format_args!("</{name}>"))?,
        }

        Ok(())
    }

    pub fn push<const N: usize>(&mut self, new: Context<N>) -> Result<Context<N>, Error> {
        if self.state != State::Attribute {
            return Err(Error::UnexpectedState);
        }

        let mut old = Context::default();
        let prev = mem::replace(&mut self.default_ns, new.default_ns);
        let _ = mem::replace(&mut old.default_ns, prev);

        let mut used = 0;
        for prefix in new.prefixes.into_iter() {
            if prefix.prefix.is_empty() {
                continue;
            }

            if self.prefixes.contains_key(prefix.ns) {
                continue;
            }

            self.output
                .write_fmt(format_args!(" xmlns:{}=\"{}\"", prefix.prefix, prefix.ns))?;

            let prev = match self.prefixes.entry(prefix.ns) {
                Entry::Occupied(mut entry) => mem::replace(entry.get_mut(), prefix.prefix),
                Entry::Vacant(entry) => {
                    entry.insert(prefix.prefix);
                    ""
                }
            };

            old.prefixes[used] = Prefix {
                ns: prefix.ns,
                prefix: prev,
            };
            used += 1;
        }

        Ok(old)
    }

    pub fn pop<const N: usize>(&mut self, old: Context<N>) {
        let _ = mem::replace(&mut self.default_ns, old.default_ns);
        for prefix in old.prefixes.into_iter() {
            if prefix.ns.is_empty() && prefix.prefix.is_empty() {
                continue;
            }

            let mut entry = match self.prefixes.entry(prefix.ns) {
                Entry::Occupied(entry) => entry,
                Entry::Vacant(_) => unreachable!(),
            };

            match prefix.prefix {
                "" => {
                    entry.remove();
                }
                prev => {
                    let _ = mem::replace(entry.get_mut(), prev);
                }
            }
        }
    }

    pub fn prefix(&self, ns: &str) -> Option<&'static str> {
        self.prefixes.get(ns).copied()
    }

    pub fn default_ns(&self) -> &'static str {
        self.default_ns
    }
}

#[derive(Debug)]
pub struct Context<const N: usize> {
    pub default_ns: &'static str,
    pub prefixes: [Prefix; N],
}

impl<const N: usize> Default for Context<N> {
    fn default() -> Self {
        Self {
            default_ns: Default::default(),
            prefixes: [Prefix { prefix: "", ns: "" }; N],
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Prefix {
    pub prefix: &'static str,
    pub ns: &'static str,
}

#[derive(Debug, Eq, PartialEq)]
enum State {
    Attribute,
    Element,
    Scalar,
}
