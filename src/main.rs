mod cli;

use clap::Parser;
use anyhow::Result;
use anyhow::Context;
use std::fs;

use std::path::{Path, PathBuf};

use std::marker::PhantomData;

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let folder = match args.command {
        cli::Command::Folder(f) => f,
        _ => unimplemented!()
    };

    Ok(())
}

fn paths_in_dir(path: &Path) -> Result<impl Iterator<Item = Result<PathBuf>>> {
    let out = fs::read_dir(&path)
        .with_context(|| format!("failed to read directory {}", path.display()))?
        .map(|entry| {
            let entry = entry?;

            let meta = entry.metadata()?;

            if meta.file_type().is_dir() {
                let err = anyhow::anyhow!("found directory instead of file `{}`. Input directory should only contain files", entry.path().display());
                return Err(err);
            }

            Ok(entry.path())
        });

    Ok(out)
}

enum Either<Current, Next> {
    Left(CompareState<Current>),
    Right(CompareState<Next>),
}

enum EitherT<Current, RightOne, RightTwo> {
    Continue(CompareState<Current>),
    RightOne(CompareState<RightOne>),
    RightTwo(CompareState<RightTwo>),
}

#[derive(Debug, Eq, PartialEq)]
struct Comparison<'a> {
    prefix: &'a str,
    zero_padding: usize,
    suffix: &'a str
}

/// type state for [`CompareState`] - we are finding characters that do not match 
/// between the two strings
#[derive(Debug)]
struct Prefix {
    prefix_length: usize
}


/// type state for [`CompareState`] - we are currently counting the characters that 
/// do not match
#[derive(Debug)]
struct Numerics {
    prefix_length: usize,
    numeric_length: usize,
    encountered_mismatch: bool
}

#[derive(Debug)]
struct Suffix {
    prefix_length: usize,
    numeric_length: usize, 
    suffix_start: usize
}

#[derive(Debug)]
struct CompareState<STATE>{
    state: STATE
}

impl CompareState<Prefix> {
    fn prefix_match(mut self, l: char, r: char) -> Result<Either<Prefix, Numerics>> {
        if l == r {
            // they characters are the same, and they are numeric, we should
            // go to the numeric parser
            if l.is_digit(10) {
                let state = CompareState {
                    state: Numerics {
                        prefix_length: self.state.prefix_length,
                        numeric_length: 1,
                        encountered_mismatch: false
                    }
                };

                Ok(Either::Right(state))
            }
            // the characters are the same, and they are also 
            // alphabetic. We should just continue on
            else {
                self.state.prefix_length += l.len_utf8();
                Ok(Either::Left(self))
            }
        } else {
            // if they are different, but still digits, then 
            // we enter the numeric parser
            if l.is_digit(10) && r.is_digit(10) {
                let state = CompareState {
                    state: Numerics {
                        prefix_length: self.state.prefix_length,
                        numeric_length: 1,
                        encountered_mismatch: true
                    }
                };

                Ok(Either::Right(state))
            } else {
                // the characters are different, and they are not numeric
                // so we know that these files do not actually follow the required pattern
                // mechanics
                anyhow::bail!("characters `{l}` and `{r}` do not match in prefix parsing")
            }
        }
    }
}

impl CompareState<Numerics> {
    fn read_numeric(mut self, l: char, r: char) -> Result<EitherT<Numerics, Prefix, Suffix>> {
        if l == r {
            // the characters are the same, and they are also digits
            // so we continue on our current trajectory
            if l.is_digit(10) {
                self.state.numeric_length += 1;
                return Ok(EitherT::Continue(self))
            } 
            // the characters are the same, and the character is NOT 
            // numeric, this means we are still in the prefix
            else {
                // if we have already encountered a difference between numeric characters,
                // then this alphabetic character means that we are in the suffix
                let numeric_len_utf8 = self.state.numeric_length;

                if self.state.encountered_mismatch {
                    let state = Suffix { 
                        prefix_length: self.state.prefix_length,
                        numeric_length: self.state.numeric_length,
                        // do not add the length of the current character to the suffix so that
                        // things slice correctly
                        suffix_start: self.state.prefix_length + numeric_len_utf8
                    };

                    let compare_state = CompareState { state };
                    let either = EitherT::RightTwo(compare_state);
                    Ok(either)
                } else {
                    let state = Prefix { prefix_length: self.state.prefix_length + numeric_len_utf8 + l.len_utf8()};
                    let compare_state = CompareState { state };
                    let either = EitherT::RightOne(compare_state);

                    Ok(either)
                }
            }
        }
        else {
            self.state.encountered_mismatch = true;
            // the characters are not the same

            // if the characters are numeric then we are fine, it can still be the index
            // of the image. For example
            // 0005
            // 0004
            // are in the same padding, but 5 and 4 are different numeric characters
            if l.is_digit(10) && r.is_digit(10) {
                self.state.numeric_length += 1;
                return Ok(EitherT::Continue(self))
            } 
            // they are not numeric digits and they are not the same,
            // this should not happen unless the strings are not the same
            else {
                anyhow::bail!("while parsing numeric portion, characters `{l}` and `{r}` did not match and were not numeric");
            }
        }
    }
}

impl CompareState<Suffix> {
    fn suffix_match(&mut self, l: char, r: char) -> Result<()> {
        // we dont need to update anything here, since we only store 
        // the starting point of the suffix
        if l == r {
            Ok(())
        } else {
            anyhow::bail!("characters `{l}` and `{r}` did not match");
        }
    }
}

fn compare_paths<'a>(one: &'a str, two: &str) -> Result<Comparison<'a>> {
    let mut prefix_length : usize = 0;
    let suffix_start: usize;

    let mut iter = one.chars().zip(two.chars());

    let mut prefix = CompareState {
        state: Prefix { prefix_length: 0 }
    };

    let mut suffix;

    'outer: loop {
        let mut numeric;
        loop {
            let (l, r) = iter.next().ok_or_else(|| anyhow::anyhow!("ran out of characters while waiting for a difference in files. This should not happen"))?;

            let next_state = prefix.prefix_match(l,r)
                .with_context(|| "while parsing prefix")?;

            match next_state {
                Either::Left(pre) => prefix = pre,
                Either::Right(num) => {
                    numeric = num;
                    break
                }
            };
        }

        loop {
            let (l, r) = iter.next().ok_or_else(|| anyhow::anyhow!("ran out of characters while waiting for a difference in files (numeric). This should not happen"))?;

            let next_state = numeric.read_numeric(l,r)
                .with_context(|| "while parsing numerics")?;

            match next_state {
                EitherT::Continue(num) => numeric = num,
                EitherT::RightOne(pre) => {
                    prefix = pre;
                    break
                }
                EitherT::RightTwo(suff) => {
                    suffix = suff;
                    break 'outer;
                }
            }

        }
    }

    loop {
        if let Some((l,r)) = iter.next() {
            suffix.suffix_match(l,r)
                .with_context(|| "while parsing suffix")?;
        } else {
            break
        }
    }

    dbg!(&suffix);

    let zero_padding = suffix.state.numeric_length;
    let prefix = one.get(0..suffix.state.prefix_length).unwrap();
    let suffix = one.get(suffix.state.suffix_start..).unwrap();

    let compare = Comparison {
        prefix,
        suffix,
        zero_padding
    };

    Ok(compare)
}

#[cfg(test)]
mod tests {
    use super::compare_paths;
    use super::Comparison;

    #[test]
    fn simple_numerics() {
        let one = "some_prefix_0001suffix.ext";
        let two = "some_prefix_0002suffix.ext";
        let res = compare_paths(one, two).unwrap();

        let expected = Comparison {
            prefix: "some_prefix_",
            suffix: "suffix.ext",
            zero_padding: 4
        };

        assert_eq!(res, expected);
    }

    #[test]
    fn nonsimple_numerics() {
        let one = "some_prefix_0123suffix.ext";
        let two = "some_prefix_4123suffix.ext";
        let res = compare_paths(one, two).unwrap();

        let expected = Comparison {
            prefix: "some_prefix_",
            suffix: "suffix.ext",
            zero_padding: 4
        };

        assert_eq!(res, expected);
    }

    #[test]
    fn leading_useless_numerics() {
        let one = "text_prefix_0001_other_stuff_0001.ext";
        let two = "text_prefix_0001_other_stuff_0002.ext";

        let res = compare_paths(one, two).unwrap();

        let expected = Comparison {
            prefix: "text_prefix_0001_other_stuff_",
            suffix: ".ext",
            zero_padding: 4
        };

        assert_eq!(res, expected);
    }
}

