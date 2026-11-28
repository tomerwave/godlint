use crate::{
    facts::DependencySpecFact,
    source::{SourceRange, TextFile},
};

const SECTIONS: [&str; 3] = ["dependencies", "devDependencies", "optionalDependencies"];

pub fn dependency_specs(file: &TextFile) -> Vec<DependencySpecFact> {
    let mut reader = Reader {
        text: file.text().as_bytes(),
        at: 0,
    };
    let mut found = Vec::new();

    let read = reader.object(|reader, key| {
        match SECTIONS
            .into_iter()
            .find(|section| reader.unquoted(key) == section.as_bytes())
        {
            Some(section) if reader.opens(b'{') => reader.object(|reader, package| {
                if reader.opens(b'"') {
                    let spec = reader.string()?;
                    found.push((section, package, spec));
                    Some(())
                } else {
                    reader.skip_value()
                }
            }),
            _ => reader.skip_value(),
        }
    });

    read.map(|()| {
        found
            .into_iter()
            .map(|(section, package, spec)| {
                DependencySpecFact::new(file.clone(), section, package, spec)
            })
            .collect()
    })
    .unwrap_or_default()
}

struct Reader<'text> {
    text: &'text [u8],
    at: usize,
}

impl Reader<'_> {
    fn peek(&self) -> Option<u8> {
        self.text.get(self.at).copied()
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(|byte| byte.is_ascii_whitespace()) {
            self.at += 1;
        }
    }

    fn opens(&mut self, byte: u8) -> bool {
        self.skip_space();
        self.peek() == Some(byte)
    }

    fn eat(&mut self, byte: u8) -> bool {
        let opens = self.opens(byte);
        if opens {
            self.at += 1;
        }
        opens
    }

    fn unquoted(&self, token: SourceRange) -> &[u8] {
        &self.text[token.start() + 1..token.end() - 1]
    }

    fn string(&mut self) -> Option<SourceRange> {
        let start = self.at;
        self.at += 1;

        loop {
            match self.peek()? {
                b'"' => break,
                b'\\' => self.at += 2,
                _ => self.at += 1,
            }
        }

        self.at += 1;
        Some(SourceRange::new(start, self.at))
    }

    fn object(
        &mut self,
        mut member: impl FnMut(&mut Self, SourceRange) -> Option<()>,
    ) -> Option<()> {
        self.expect(b'{')?;
        if self.eat(b'}') {
            return Some(());
        }

        while self.member(&mut member)? {}

        self.expect(b'}')
    }

    fn member(
        &mut self,
        member: &mut impl FnMut(&mut Self, SourceRange) -> Option<()>,
    ) -> Option<bool> {
        let key = self.key()?;
        member(self, key)?;

        Some(self.eat(b','))
    }

    fn expect(&mut self, byte: u8) -> Option<()> {
        self.eat(byte).then_some(())
    }

    fn key(&mut self) -> Option<SourceRange> {
        let key = self.opens(b'"').then(|| self.string())??;
        self.expect(b':')?;
        Some(key)
    }

    fn skip_value(&mut self) -> Option<()> {
        let mut depth = 0_usize;
        self.skip_space();

        loop {
            depth = self.step(depth)?;

            if depth == 0 && self.ends_scalar() {
                return Some(());
            }
        }
    }

    fn step(&mut self, depth: usize) -> Option<usize> {
        let byte = self.peek()?;
        if byte == b'"' {
            return self.string().map(|_| depth);
        }

        self.at += 1;
        match byte {
            b'{' | b'[' => Some(depth + 1),
            b'}' | b']' => depth.checked_sub(1),
            _ => Some(depth),
        }
    }

    fn ends_scalar(&mut self) -> bool {
        self.skip_space();
        matches!(self.peek(), None | Some(b',' | b'}' | b']'))
    }
}
