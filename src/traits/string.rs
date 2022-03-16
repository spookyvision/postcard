use core::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    str,
};

use serde::Serialize;

// sed -n 'p;/impl FromUtf8Error {/q' < "$HOME/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/string.rs" | grep -A1 " #\[stable" | grep 'pub.*self' | grep -v '<' | sed 's/pub //' | sort | uniq
// | sed 's/ {/;/' | pbcopy
// ... plus manual purge
pub trait StringRO {
    fn as_str(&self) -> &str;
}

// sed -n 'p;/impl FromUtf8Error {/q' < "$HOME/.rustup/toolchains/nightly-x86_64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/string.rs" | grep -A1 " #\[stable" | grep 'pub.*self' | grep -v '<' | sed 's/pub //' | sort | uniq
// | perl -lne 'print; /.*fn (.*)\(.*self(, (.*?): )?(.*((,.*?):))?.*\)/ && print "        self.$1($3$6)\n    }"' | pbcopy
// ... plus manual purge

#[cfg(feature = "use-std")]
impl StringRO for std::string::String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl StringRO for &str {
    fn as_str(&self) -> &str {
        self
    }
}

impl<const N: usize> StringRO for heapless::String<N> {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

pub trait StringRW {
    type Error;
    fn capacity(&self) -> usize;
    fn push(&mut self, ch: char) -> Result<(), Self::Error>;
    fn push_str(&mut self, string: &str) -> Result<(), Self::Error>;
    fn as_mut_str(&mut self) -> &mut str;
    fn clear(&mut self);
    fn truncate(&mut self, new_len: usize);
}

#[cfg(feature = "use-std")]
impl StringRW for std::string::String {
    type Error = Infallible;

    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn push(&mut self, ch: char) -> Result<(), Self::Error> {
        self.push(ch);
        Ok(())
    }

    fn push_str(&mut self, string: &str) -> Result<(), Self::Error> {
        self.push_str(string);
        Ok(())
    }
    fn as_mut_str(&mut self) -> &mut str {
        self.as_mut_str()
    }
    fn clear(&mut self) {
        self.clear()
    }
    fn truncate(&mut self, new_len: usize) {
        self.truncate(new_len)
    }
}

impl<const N: usize> StringRW for heapless::String<N> {
    type Error = ();
    fn capacity(&self) -> usize {
        self.capacity()
    }
    fn push(&mut self, ch: char) -> Result<(), Self::Error> {
        self.push(ch)
    }

    fn push_str(&mut self, string: &str) -> Result<(), Self::Error> {
        self.push_str(string)
    }

    fn as_mut_str(&mut self) -> &mut str {
        self.as_mut_str()
    }
    fn clear(&mut self) {
        self.clear()
    }
    fn truncate(&mut self, new_len: usize) {
        self.truncate(new_len)
    }
}

// grep ' for String' "$HOME/.cargo/registry/src/github.com-1ecc6299db9ec823/heapless-0.7.10/src/string.rs" | grep -v '\$\|N1\|hash32' | sed -E "s/('a, |'a )//g" | awk '{print $4}' | tr -cd '[:alnum:]:&<>[]\n' | sed 's/^/ + /' | tr -d '\n' | pbcopy
pub trait PostcardString: StringRO + Serialize + AsRef<str> + Deref {}
pub trait PostcardStringRW: PostcardString + StringRW + AsMut<str> + DerefMut {}

impl<T: StringRO + Serialize + AsRef<str> + Deref> PostcardString for T {}

impl<T: PostcardString + StringRW + AsMut<str> + DerefMut> PostcardStringRW for T {}

#[cfg(all(test, feature = "use-std"))]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct ToMcu<STRING: PostcardString> {
        #[allow(unused)]
        s: STRING,
    }

    #[test]
    fn read() {
        type HS32 = heapless::String<32>;

        let _str = ToMcu {
            s: "test moon please ignore",
        };

        let std = ToMcu {
            s: "test moon please ignore".to_string(),
        };

        let h: HS32 = "test moon please ignore".into();
        let heapless = ToMcu { s: h };

        let ser_std = serde_json::to_string(&std).unwrap();
        let ser_heapless = serde_json::to_string(&heapless).unwrap();

        assert_eq!(ser_std, ser_heapless);

        // the LHS/RHS swap is intentional
        let _de_std: ToMcu<String> = serde_json::from_str(&ser_heapless).unwrap();
        let _de_heapless: ToMcu<HS32> = serde_json::from_str(&ser_std).unwrap();
    }

    #[derive(Serialize, Deserialize)]
    struct ToMcuRW<STRING: PostcardStringRW> {
        #[allow(unused)]
        s: STRING,
    }

    #[test]
    fn read_write() {
        type HS32 = heapless::String<32>;

        let mut std = ToMcu {
            s: "test moon please ignore".to_string(),
        };

        // meh, hitting the limits of type inference
        assert_eq!(StringRW::push_str(&mut std.s, "RWRWRWRWRWRWRWRW"), Ok(()));

        let h: HS32 = "test moon please ignore".into();
        let mut heapless = ToMcu { s: h };

        assert_eq!(heapless.s.push_str("RW"), Ok(()));
        assert_eq!(heapless.s.push_str("RWRWRWRWRWRWRWRW"), Err(()));
    }

    /// Publish/Subscribe Path - Short or Long
    #[derive(Debug, Serialize, Eq, PartialEq, Clone)]
    pub enum PubSubPath<'a, STRING: PostcardStringRW> {
        /// A long form, UTF-8 Path
        #[serde(borrow)]
        Long(&'a STRING),
        Short(u16),
    }

    #[test]
    fn arachno() {
        let critters = "🕷🕷🕷";
        let path = PubSubPath::Long(&"actually short".to_string());
        todo!();
    }
}
