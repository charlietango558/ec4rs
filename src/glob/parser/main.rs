use super::alt::AltStack;
use crate::glob::{Glob, Matcher};

pub fn parse(glob: &str) -> Glob {
	let mut retval = Glob(vec![]);
	let mut stack = AltStack::new();
	let mut found_sep: bool = false;
	for segment in glob.split('/') {
		retval.append_char('/');
		found_sep = retval.0.len() > 1;
		let mut chars = segment.chars().peekable();
		while let Some(c) = chars.next() {
			match c {
				'\\' => {
					if let Some(escaped) = chars.next() {
						retval.append_char(escaped);
					}
				}
				'?' => retval.append(Matcher::AnyChar),
				'*' => retval.append(Matcher::AnySeq(matches!(chars.peek(), Some('*')))),
				'[' => {
					(retval, chars) = super::charclass::parse(retval, chars);
				}
				'{' => {
					if let Some((a, b, chars_new)) = super::numrange::parse(chars.clone()) {
						chars = chars_new;
						retval.append(Matcher::Range(
							// Reading the spec strictly,
							// a compliant implementation must handle cases where
							// the left integer is greater than the right integer.
							std::cmp::min(a, b),
							std::cmp::max(a, b),
						));
					} else {
						stack.push(retval);
						retval = Glob(vec![]);
					}
				}
				',' => {
					if let Some(rejected) = stack.add_alt(retval) {
						retval = rejected;
						retval.append_char(',');
					} else {
						retval = Glob(vec![]);
					}
				}
				'}' => {
					let add_brace: bool;
					(retval, add_brace) = stack.add_alt_and_pop(retval);
					if add_brace {
						retval.append_char('}');
					}
				}
				_ => retval.append_char(c),
			}
		}
	}
	loop {
		let is_empty: bool;
		(retval, is_empty) = stack.join_and_pop(retval);
		if is_empty {
			break;
		}
	}
	if found_sep {
		*retval.0.first_mut().unwrap() = Matcher::End;
	}
	if let Some(Matcher::Sep) = retval.0.last() {
		retval.append(Matcher::AnySeq(false));
	}
	retval
}
