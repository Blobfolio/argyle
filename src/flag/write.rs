/*!
# Argyle: Flag Builder.
*/

use std::{
	collections::{
		BTreeSet,
		BTreeMap,
	},
	fmt,
};
use super::{
	FlagsBuilder,
	Scope,
};



/// # Filler Names.
///
/// A list of variant names to pull from for unnamed combinatory flags.
static FILLER: [&str; 256] = [
	"None", "Z01", "Z02", "Z03", "Z04", "Z05", "Z06", "Z07", "Z08", "Z09", "Z0a", "Z0b", "Z0c", "Z0d", "Z0e", "Z0f", "Z10", "Z11", "Z12", "Z13", "Z14", "Z15", "Z16", "Z17", "Z18", "Z19", "Z1a", "Z1b", "Z1c", "Z1d", "Z1e", "Z1f", "Z20", "Z21", "Z22", "Z23", "Z24", "Z25", "Z26", "Z27", "Z28", "Z29", "Z2a", "Z2b", "Z2c", "Z2d", "Z2e", "Z2f", "Z30", "Z31", "Z32", "Z33", "Z34", "Z35", "Z36", "Z37", "Z38", "Z39", "Z3a", "Z3b", "Z3c", "Z3d", "Z3e", "Z3f",
	"Z40", "Z41", "Z42", "Z43", "Z44", "Z45", "Z46", "Z47", "Z48", "Z49", "Z4a", "Z4b", "Z4c", "Z4d", "Z4e", "Z4f", "Z50", "Z51", "Z52", "Z53", "Z54", "Z55", "Z56", "Z57", "Z58", "Z59", "Z5a", "Z5b", "Z5c", "Z5d", "Z5e", "Z5f", "Z60", "Z61", "Z62", "Z63", "Z64", "Z65", "Z66", "Z67", "Z68", "Z69", "Z6a", "Z6b", "Z6c", "Z6d", "Z6e", "Z6f", "Z70", "Z71", "Z72", "Z73", "Z74", "Z75", "Z76", "Z77", "Z78", "Z79", "Z7a", "Z7b", "Z7c", "Z7d", "Z7e", "Z7f",
	"Z80", "Z81", "Z82", "Z83", "Z84", "Z85", "Z86", "Z87", "Z88", "Z89", "Z8a", "Z8b", "Z8c", "Z8d", "Z8e", "Z8f", "Z90", "Z91", "Z92", "Z93", "Z94", "Z95", "Z96", "Z97", "Z98", "Z99", "Z9a", "Z9b", "Z9c", "Z9d", "Z9e", "Z9f", "Za0", "Za1", "Za2", "Za3", "Za4", "Za5", "Za6", "Za7", "Za8", "Za9", "Zaa", "Zab", "Zac", "Zad", "Zae", "Zaf", "Zb0", "Zb1", "Zb2", "Zb3", "Zb4", "Zb5", "Zb6", "Zb7", "Zb8", "Zb9", "Zba", "Zbb", "Zbc", "Zbd", "Zbe", "Zbf",
	"Zc0", "Zc1", "Zc2", "Zc3", "Zc4", "Zc5", "Zc6", "Zc7", "Zc8", "Zc9", "Zca", "Zcb", "Zcc", "Zcd", "Zce", "Zcf", "Zd0", "Zd1", "Zd2", "Zd3", "Zd4", "Zd5", "Zd6", "Zd7", "Zd8", "Zd9", "Zda", "Zdb", "Zdc", "Zdd", "Zde", "Zdf", "Ze0", "Ze1", "Ze2", "Ze3", "Ze4", "Ze5", "Ze6", "Ze7", "Ze8", "Ze9", "Zea", "Zeb", "Zec", "Zed", "Zee", "Zef", "Zf0", "Zf1", "Zf2", "Zf3", "Zf4", "Zf5", "Zf6", "Zf7", "Zf8", "Zf9", "Zfa", "Zfb", "Zfc", "Zfd", "Zfe", "Zff",
];



/// # Flag Writer.
///
/// This is a temporary struct used by [`FlagsBuilder`] to handle the actual
/// code generation.
pub(super) struct FlagsWriter<'a> {
	/// # Enum Name.
	name: &'a str,

	/// # Enum Documentation.
	docs: &'a str,

	/// # Enum/Member Scope.
	scope: Scope,

	/// # Default Enum Value.
	default: u8,

	/// # Primary Flag Names.
	primary: Vec<&'a str>,

	/// # Variants (Number, Name).
	by_num: BTreeMap<u8, &'a str>,

	/// # Variants (Name, Number).
	by_var: BTreeMap<&'a str, u8>,

	/// # Flag Documentation (Name, Docs).
	flag_docs: BTreeMap<&'a str, &'a str>,

	/// # Links.
	///
	/// Flags (LHS) that imply other flags (RHS).
	links: Vec<(&'a str, &'a str)>,
}

impl fmt::Display for FlagsWriter<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// There's a lot so we split it up!
		self.write_enum_def(f)?;
		self.write_bitwise(f)?;
		self.write_type_helpers(f)?;
		self.write_self_helpers(f)?;
		self.write_tests(f)
	}
}

impl<'a> FlagsWriter<'a> {
	#[expect(clippy::cast_possible_truncation, reason = "Checked via assertion.")]
	/// # From Flags Builder.
	///
	/// Crunch the bitwise values of the primary and alias flags collected by
	/// the builder, along with the combinations in between, performing various
	/// sanity checks along the way.
	///
	/// ## Panics
	///
	/// This method will panic if:
	/// * There are too few or too many primary flags;
	/// * Circular references are encountered;
	/// * Referenced flags are undefined;
	/// * Name or number collisions occur;
	/// * Miscellaneous bugs are encountered;
	pub(super) fn from_builder(builder: &'a FlagsBuilder) -> Self {
		// The primaries are easy to work out.
		let primary: Vec<&str> = builder.primary.iter()
			.map(|s| s.name.as_str())
			.collect();

		// Can't be too small or too big.
		assert!(
			(1..=8).contains(&primary.len()),
			"The number of primary flags must be between 1..=8. (argyle::FlagsBuilder)",
		);

		// The enum's upper limit is defined by the combination of _all_ flags,
		// which being powers of two, bring the total within one of the _next_
		// power of two. (Eight will overflow, but that's fine; MAX is max in
		// that case.)
		let max = 2_u8.checked_pow(primary.len() as u32)
			.map_or(u8::MAX, |n| n - 1);

		// Sort out the named flags.
		let named = named_flags(builder);
		assert!(
			named.keys().all(|k| *k <= max) &&
			max == named.keys().fold(0_u8, |acc, v| acc | v),
			"BUG: argyle messed up the maximum bit value!",
		);

		// Let's prepopulate the by_num set accordingly.
		let by_num = (0..=max).zip(FILLER)
			.map(|(k, v)|
				// Prefer named to filler.
				named.get(&k).map_or((k, v), |v| (k, *v))
			)
			.collect::<BTreeMap<u8, &str>>();

		// Double-check that max is indeed max.
		assert_eq!(
			max,
			by_num.keys().fold(0_u8, |acc, v| acc | v),
			"BUG: argyle messed up the maximum bit value!",
		);

		// Reverse polarity!
		let by_var = by_num.iter()
			.map(|(k, v)| (*v, *k))
			.collect::<BTreeMap<&str, u8>>();

		// The two should match, obviously.
		assert_eq!(
			by_num.len(),
			by_var.len(),
			"BUG: argyle messed up the flag math!",
		);

		// Now that the numbers are in, we can calculate the default value.
		let default =
			if builder.default_all { max }
			else {
				builder.default.iter().fold(0_u8, |acc, v| {
					let Some(v) = by_var.get(v.as_str()) else {
						panic!("TYPO: flag ({v}) is undefined. (argyle::FlagsBuilder)");
					};
					acc | v
				})
			};

		// Build up the docs list.
		let mut flag_docs = BTreeMap::new();
		flag_docs.insert("None", "# None.\n\nThis variant is the flag equivalent of zero.");
		flag_docs.extend(
			builder.primary.iter()
				.chain(builder.alias.iter())
				.map(|f| (f.name.as_str(), f.docs.as_str()))
		);

		// Let's collect up the links so we can unit test them user-side.
		let mut links = Vec::new();
		for flag in builder.primary.iter().chain(builder.alias.iter()) {
			let lhs = flag.name.as_str();
			for rhs in &flag.deps {
				links.push((lhs, rhs.as_str()));
			}
		}

		// Finally done!
		Self {
			name: builder.name.as_str(),
			docs: builder.docs.as_str(),
			scope: builder.scope,
			default,
			primary,
			by_num,
			by_var,
			flag_docs,
			links,
		}
	}
}

/// # Write Helpers.
impl FlagsWriter<'_> {
	/// # Enum Definition.
	///
	/// Write the type definition for the enum!
	fn write_enum_def(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(
			f,
			"#[allow(
	clippy::allow_attributes,
	clippy::manual_non_exhaustive,
	reason = \"It is exhaustive!\"
)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
#[doc = {:?}]
{}enum {} {{",
			self.docs,
			self.scope,
			self.name,
)?;

		// Generate each arm.
		for (name, bits) in &self.by_var {
			// Add #[default]?
			if *bits == self.default { f.write_str("\t#[default]\n")?; }

			// Named entries get docs.
			if let Some(docs) = self.flag_docs.get(name) {
				writeln!(f, "\t#[doc = {docs:?}]")?;
			}
			// Generated entries get hidden.
			else {
				f.write_str("\t#[doc(hidden)]\n\t/// # Automatically Generated.\n")?;
			}

			// The actual arm!
			writeln!(f, "\t{name} = 0b{},\n", nice_bits(*bits))?;
		}
		f.write_str("}\n")
	}

	/// # Bitwise Implementations.
	///
	/// Write And, Or, and Xor implementations for `Self`.
	fn write_bitwise(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// The last/largest value has all the bits.
		let (_, all) = self.by_num.last_key_value().ok_or(fmt::Error)?;

		writeln!(
			f,
			"impl ::std::ops::BitAnd for {name} {{
	type Output = Self;
	#[inline]
	fn bitand(self, other: Self) -> Self::Output {{
		Self::from_u8((self as u8) & (other as u8))
	}}
}}
impl ::std::ops::BitAndAssign for {name} {{
	#[inline]
	fn bitand_assign(&mut self, other: Self) {{ *self = *self & other; }}
}}
impl ::std::ops::BitOr for {name} {{
	type Output = Self;
	#[inline]
	fn bitor(self, other: Self) -> Self::Output {{ self.with(other) }}
}}
impl ::std::ops::BitOrAssign for {name} {{
	#[inline]
	fn bitor_assign(&mut self, other: Self) {{ *self = *self | other; }}
}}
impl ::std::ops::BitXor for {name} {{
	type Output = Self;
	#[inline]
	fn bitxor(self, other: Self) -> Self::Output {{
		Self::from_u8((self as u8) ^ (other as u8))
	}}
}}
impl ::std::ops::BitXorAssign for {name} {{
	#[inline]
	fn bitxor_assign(&mut self, other: Self) {{ *self = *self ^ other; }}
}}
impl ::std::ops::Not for {name} {{
	type Output = Self;
	#[inline]
	fn not(self) -> Self::Output {{
		let raw = ! (self as u8);
		Self::from_u8(raw & (Self::{all} as u8))
	}}
}}",
			name=self.name,
		)
	}

	/// # Miscellaneous (Type) Helpers.
	///
	/// Write the `FLAGS` constant and other top-level helpers.
	fn write_type_helpers(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		/// # Primary Flag Array Values.
		///
		/// Print the values for the array, comma-separated, no terminating
		/// line.
		struct FlagsFmt<'a>(&'a [&'a str]);

		impl fmt::Display for FlagsFmt<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				let mut iter = self.0.iter();
				if let Some(next) = iter.next() {
					write!(f, "Self::{next},")?;
					for next in iter {
						write!(f, " Self::{next},")?;
					}
				}
				Ok(())
			}
		}

		/// # Writer: `Enum::from_u8`.
		///
		/// Write the match arms for the `from_u8` method.
		struct FromU8Fmt<'a>(&'a BTreeMap<u8, &'a str>);

		impl fmt::Display for FromU8Fmt<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				// Write the arms
				let full = self.0.len() == 256;
				for (bits, name) in self.0 {
					if *bits == 0 {
						if full {
							f.write_str("\t\t\t0b0000_0000 => Self::None,\n")?;
						}
						continue;
					}
					writeln!(f, "\t\t\t0b{} => Self::{name},", nice_bits(*bits))?;
				}

				// Add a wildcard if we aren't full.
				if ! full {
					f.write_str("\t\t\t_ => Self::None,\n")?;
				}

				Ok(())
			}
		}

		// Write everything!
		writeln!(
			f,
			"#[allow(
	clippy::allow_attributes,
	clippy::too_many_lines,
	dead_code,
	reason = \"Automatically generated.\"
)]
impl {name} {{
	/// # (Primary) Flags.
	{scope}const FLAGS: [Self; {}] = [
		{}
	];

	#[must_use]
	/// # From `u8`.
	///
	/// Find and return the flag corresponding to the `u8`. If out of range,
	/// `Self::None` is returned.
	{scope}const fn from_u8(num: u8) -> Self {{
		match num {{
{arms}\t\t}}
	}}
}}",
			self.primary.len(),
			FlagsFmt(self.primary.as_slice()),
			name=self.name,
			scope=self.scope,
			arms=FromU8Fmt(&self.by_num),
		)
	}

	/// # Miscellaneous (Self) Helpers.
	///
	/// Write methods working on `self`.
	fn write_self_helpers(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Write everything!
		writeln!(
			f,
			"#[allow(
	clippy::allow_attributes,
	dead_code,
	reason = \"Automatically generated.\"
)]
impl {name} {{
	#[must_use]
	#[inline]
	/// # Contains Flag?
	///
	/// Returns `true` if `self` is or comprises `other`, `false` if not.
	{scope}const fn contains(self, other: Self) -> bool {{
		(other as u8) == (self as u8) & (other as u8)
	}}

	#[must_use]
	/// # Contains Any Part of Flag?
	///
	/// Returns the bits common to `self` and `other`, if any.
	{scope}const fn contains_any(self, other: Self) -> Option<Self> {{
		let any = Self::from_u8((self as u8) & (other as u8));
		if any.is_none() {{ None }}
		else {{ Some(any) }}
	}}

	#[must_use]
	#[inline]
	/// # Is None?
	///
	/// Returns `true` if no bits are set (i.e. [`{name}::None`]).
	{scope}const fn is_none(self) -> bool {{ matches!(self, Self::None) }}

	#[must_use]
	/// # With Flag Bits.
	///
	/// Return the combination of `self` and `other`.
	///
	/// This is equivalent to `self | other`, but constant.
	{scope}const fn with(self, other: Self) -> Self {{
		Self::from_u8((self as u8) | (other as u8))
	}}

	#[must_use]
	/// # Without Flag Bits.
	///
	/// Remove `other` from `self`, returning the difference.
	///
	/// This is equivalent to `self & ! other`, but constant.
	{scope}const fn without(self, other: Self) -> Self {{
		Self::from_u8((self as u8) & ! (other as u8))
	}}
}}",
			name=self.name,
			scope=self.scope,
		)
	}

	#[expect(clippy::literal_string_with_formatting_args, reason = "Sure does.")]
	/// # Write Tests.
	fn write_tests(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// The last/largest value has all the bits.
		let (_, all) = self.by_num.last_key_value().ok_or(fmt::Error)?;

		writeln!(
			f,
			"#[cfg(test)]
mod test_{snake} {{
	use super::*;

	#[test]
	/// # Test `{name}::Default`.
	///
	/// Ensure the default value and flag resolve as expected.
	fn t_default() {{
		let default = {name}::default();
		assert_eq!(
			{name}::{default_var},
			default,
			\"Default implementation returned unexpected flag.\",
		);
		assert_eq!(
			{default_num},
			default as u8,
			\"Default implementation returned unexpected value.\",
		);
	}}

	#[test]
	/// # Test Bitwise Impls.
	///
	/// Ensure flags can be added and subtracted from one another.
	fn t_bitwise() {{
		assert_eq!({name}::None, ! {name}::{all}, \"!{all} should be None!\");
		assert_eq!({name}::{all}, ! {name}::None, \"!None should be {all}!\");

		for pair in {name}::FLAGS.windows(2) {{
			let a = pair[0];
			let b = pair[1];
			let ab = a | b;

			// Confirm the combined value contains both.
			assert!(
				ab.contains(a),
				\"Union of {{a:?}} and {{b:?}} missing the former?!\",
			);
			assert!(
				ab.contains(b),
				\"Union of {{a:?}} and {{b:?}} missing the latter?!\",
			);

			// For simple flags, confirm negation returns the status quo.
			if (a as u8).is_power_of_two() && (b as u8).is_power_of_two() {{
				assert_eq!(a, ab & ! b, \"ab & ! b doesn't equal a?!\");
				assert_eq!(b, ab & ! a, \"ab & ! a doesn't equal b?!\");
			}}
		}}
	}}

	#[test]
	/// # Test Conversions.
	fn t_conversion() {{
		let mut all = std::collections::BTreeSet::new();
		let mut max = 0_u8;
		for i in 0..=u8::MAX {{
			let cur = {name}::from_u8(i);
			if i == 0 || ! cur.is_none() {{
				all.insert(i);
				assert_eq!(cur as u8, i, \"{name}/u8 conversion failed for {{i}}\");
				if max < i {{ max = i; }}
			}}
		}}

		assert_eq!(max, {name}::{all} as u8, \"Max valid value not {name}::{all}…\");
		assert_eq!(
			all.len(),
			usize::from(max) + 1,
			\"Found {{}} elements instead of {{}}\",
			all.len(),
			usize::from(max) + 1,
		);
	}}

	#[test]
	/// # Test `{name}::contains`.
	///
	/// Ensure `{name}::None` contains none of the primary flags, and
	/// `{name}::{all}` contains all of them.
	fn t_contains() {{
		for flag in {name}::FLAGS {{
			assert!(
				! {name}::None.contains(flag),
				\"None should not contain {{flag:?}}.\",
			);
			assert!(
				{name}::{all}.contains(flag),
				\"{all} should contain {{flag:?}}.\",
			);
		}}
	}}

	{links}
}}",
			name=self.name,
			snake=super::to_snake_case(self.name),
			default_num=self.default,
			default_var=self.by_num.get(&self.default).ok_or(fmt::Error)?,
			links=TLinksFmt {
				name: self.name,
				links: self.links.as_slice(),
			},
		)
	}
}



/// # Test Links.
///
/// Write the entire `t_links` test method, which we only need
/// conditionally.
struct TLinksFmt<'a> {
	/// # Enum Name.
	name: &'a str,

	/// # Link Pairs.
	links: &'a [(&'a str, &'a str)],
}

impl fmt::Display for TLinksFmt<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.links.is_empty() { return Ok(()); }

		// Method opener.
		f.write_str("#[test]
	/// # Test Complex Flag and Alias Links.
	///
	/// Ensure the variants referencing other variants actually do.
	fn t_links() {\n")?;

		// Each condition.
		for (lhs, rhs) in self.links {
			writeln!(
				f,
				"\t\tassert!(
			{name}::{lhs}.contains({name}::{rhs}),
			\"{name}::{lhs} does not contain {name}::{rhs}.\",
		);",
				name=self.name,
			)?;
		}

		// Method closer.
		f.write_str("\t}")
	}
}

/// # Build Named Bitflags.
///
/// Figure out the corresponding bit values for all named flags, returning a
/// definitive map indexed by said values.
///
/// This isn't quite as terrible as the line count suggests; the datasets have
/// to be traversed multiple times to account for recursive flag references.
///
/// ## Panics
///
/// Panics if any flags are undefined, contain circular or duplicate
/// references, or we wind up with too few or too many of them.
fn named_flags(builder: &FlagsBuilder) -> BTreeMap<u8, &str> {
	// Primary flags and dependencies.
	let mut primaries = builder.primary.iter()
		.map(|f| (
			f.name.as_str(),
			f.deps.iter().map(String::as_str).collect::<Vec<_>>(),
		))
		.collect::<BTreeMap<&str, Vec<&str>>>();

	// Aliases and dependencies.
	let mut aliases = builder.alias.iter()
		.map(|f| (
			f.name.as_str(),
			f.deps.iter().map(String::as_str).collect::<Vec<_>>(),
		))
		.collect::<BTreeMap<&str, Vec<&str>>>();

	// All the named flags.
	let named = primaries.keys()
		.copied()
		.chain(aliases.keys().copied())
		.collect::<BTreeSet<&str>>();

	// Make sure all dependent flags are defined.
	for flag in primaries.values().chain(aliases.values()).flatten() {
		assert!(
			named.contains(flag),
			"TYPO: flag ({flag}) is undefined. (argyle::FlagsBuilder)",
		);
	}

	// Assign all primary flags a unique power of two.
	let mut out = (0..8_u32).zip(primaries.keys().copied())
		.map(|(i, v)| (v, 2_u8.pow(i)))
		.collect::<BTreeMap<&str, u8>>();

	// If there are complex primaries, backfill the extra bits now.
	primaries.retain(|_, deps| ! deps.is_empty());
	while ! primaries.is_empty() {
		let mut changed = false;
		let mut multi = primaries.keys().copied().collect::<BTreeSet<&str>>();
		primaries.retain(|name, deps| {
			// We can add some bits now and others later, provided they're
			// not also TBD.
			let mut extra = 0;
			deps.retain(|v| {
				if ! multi.contains(v) {
					if let Some(bit) = out.get(v) {
						extra |= bit;
						return false;
					}
				}
				true
			});

			if extra == 0 { true }
			else {
				let Some(e) = out.get_mut(name) else {
					panic!("BUG: missing flag entry ({name})! (argyle::FlagsBuilder)");
				};
				*e |= extra;
				changed = true;

				// We're done backfilling this flag!
				if deps.is_empty() {
					multi.remove(name);
					false
				}
				else { true }
			}
		});

		// If nothing changed this time around, the next time won't be any
		// better.
		if ! changed { break; }
	}

	// All primary flags should be gone now.
	assert!(
		primaries.is_empty(),
		"FAIL: unable to resolve circular flag references. (argyle::FlagsBuilder)",
	);

	// Aliases might alias themselves, requiring some loop-and-repeat to fully
	// resolve.
	while ! aliases.is_empty() {
		let mut changed = false;
		aliases.retain(|k, v| {
			let mut bits = 0_u8;
			for k2 in v {
				// Can't process undefined flags yet; skip this alias for now.
				let Some(bit) = out.get(k2) else { return true; };
				bits |= *bit;
			}

			// Can't be zero.
			assert!(
				bits != 0,
				"TYPO: Alias ({k}) doesn't alias anything! (argyle::FlagsBuilder)",
			);

			// Can't already exist.
			assert!(
				! out.values().any(|v| *v == bits),
				"TYPO: Duplicate flag alias ({k}). (argyle::FlagsBuilder)",
			);

			// Save it!
			out.insert(k, bits);
			changed = true;
			false // We can drop it.
		});

		// If nothing changed this time around, the next time won't be any
		// better.
		if ! changed { break; }
	}

	// If we have named entries remaining, they're unresolvable!
	assert!(
		aliases.is_empty(),
		"Unable to reconcile recursive flag aliases. (argyle::FlagsBuilder)",
	);

	// Reverse the polarity.
	let out2 = out.iter().map(|(k, v)| (*v, *k)).collect::<BTreeMap<u8, &str>>();

	// Sanity check: everything in named should be accounted for, and both
	// versions of out should have the same length.
	assert!(
		! out2.is_empty() && out.len() == out2.len() && named.into_iter().all(|v| out.contains_key(v)),
		"BUG: argyle messed up the flag math!",
	);

	// We're done!
	out2
}

/// # Format Binary Bits Nicely.
///
/// Return all eight bits as ASCII, with a `_` in the middle to appease clippy.
fn nice_bits(num: u8) -> String {
	let mut out = format!("{num:08b}");
	out.insert(4, '_');
	out
}
