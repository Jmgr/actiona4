use std::fmt::{self, Display, Formatter, Write};

#[derive(Default)]
pub struct DisplayFields {
    buffer: String,
    not_first: bool,
}

impl DisplayFields {
    fn maybe_space(&mut self) {
        if self.not_first {
            let _ = write!(self.buffer, ", ");
        } else {
            self.not_first = true;
        }
    }

    pub fn display<T: Display>(mut self, name: &str, value: T) -> Self {
        self.maybe_space();
        let _ = write!(self.buffer, "{name}: {value}");
        self
    }

    pub fn display_if_some<T: Display>(mut self, name: &str, value: &Option<T>) -> Self {
        if let Some(value) = value {
            self.maybe_space();
            let _ = write!(self.buffer, "{name}: {value}");
        }
        self
    }

    pub fn finish<'a, 'f>(self, f: &'a mut Formatter<'f>) -> fmt::Result {
        write!(f, "({})", self.buffer)
    }

    pub fn finish_as_string(self) -> String {
        format!("({})", self.buffer)
    }
}

pub struct DisplayList<'a, T>(&'a [T]);

impl<'a, T: Display> Display for DisplayList<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            Display::fmt(item, f)?;
        }
        f.write_str("]")
    }
}

pub const fn display_list<T: Display>(v: &[T]) -> DisplayList<'_, T> {
    DisplayList(v)
}

pub struct DisplayMap<I>(pub I);

impl<I, K, V> Display for DisplayMap<I>
where
    I: Clone + IntoIterator<Item = (K, V)>,
    K: Display,
    V: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        let mut it = self.0.clone().into_iter();
        if let Some((k, v)) = it.next() {
            write!(f, "{k}: {v}")?;
            for (k, v) in it {
                write!(f, ", {k}: {v}")?;
            }
        }
        f.write_str("}")
    }
}

pub const fn display_map<I>(iter: I) -> DisplayMap<I> {
    DisplayMap(iter)
}
