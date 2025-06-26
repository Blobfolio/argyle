##
# Development Recipes
#
# This justfile is intended to be run from inside a Docker sandbox:
# https://github.com/Blobfolio/righteous-sandbox
#
# docker run \
#	--rm \
#	-v "{{ invocation_directory() }}":/share \
#	-it \
#	--name "righteous_sandbox" \
#	"righteous/sandbox:debian"
#
# Alternatively, you can just run cargo commands the usual way and ignore these
# recipes.
##

pkg_id      := "argyle"
pkg_name    := "Argyle"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
doc_dir     := justfile_directory() + "/doc"



# Clean Cargo crap.
@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"

	cargo update -w


# Clippy.
@clippy:
	clear
	cargo clippy \
		--release \
		--target-dir "{{ cargo_dir }}"

	cargo clippy \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"

	# Flag builder tests.
	cd "{{ justfile_dir() }}/examples/flags_builder" && cargo clippy \
		--release \
		--target-dir "{{ cargo_dir }}"


# Generate CREDITS.
@credits:
	cargo bashman --no-bash --no-man
	just _fix-chown "{{ justfile_directory() }}/CREDITS.md"


# Build and Run Args Example.
@debug-args +ARGS:
	clear
	cargo run \
		-q \
		--release \
		--all-features \
		--example "debug" \
		--target-dir "{{ cargo_dir }}" \
		-- {{ ARGS }}


# Build and Run Flags Example.
@debug-flags:
	clear
	cd "{{ justfile_dir() }}/examples/flags_builder" && cargo run \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Docs.
@doc:
	cargo rustdoc \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"

	# Move the docs and clean up ownership.
	[ ! -d "{{ doc_dir }}" ] || rm -rf "{{ doc_dir }}"
	mv "{{ cargo_dir }}/doc" "{{ justfile_directory() }}"
	just _fix-chown "{{ doc_dir }}"


# Miri tests!
@miri:
	# Pre-clean.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"

	fyi task "Testing native/default target."
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --all-features

	fyi task "Testing i686-unknown-linux-gnu (32-bit) target."
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --all-features --target i686-unknown-linux-gnu

	fyi task "Testing mps64 (big endian) target."
	MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --all-features --target mips64-unknown-linux-gnuabi64

	# Post-clean.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"


# Unit tests!
@test:
	clear
	cargo test \
		--all-features \
		--target-dir "{{ cargo_dir }}"

	cargo test \
		--target-dir "{{ cargo_dir }}"

	cargo test \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"

	cargo test \
		--release \
		--target-dir "{{ cargo_dir }}"

	# Flag builder tests.
	cd "{{ justfile_dir() }}/examples/flags_builder" && cargo test \
		--release \
		--target-dir "{{ cargo_dir }}"


# Get/Set version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( toml get "{{ justfile_directory() }}/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting version to $_ver2."

	# Set the release version!
	just _version "{{ justfile_directory() }}" "$_ver2"
	just credits


# Set version for real.
@_version DIR VER:
	[ -f "{{ DIR }}/Cargo.toml" ] || exit 1

	# Set the release version!
	toml set "{{ DIR }}/Cargo.toml" package.version "{{ VER }}" > /tmp/Cargo.toml
	just _fix-chown "/tmp/Cargo.toml"
	mv "/tmp/Cargo.toml" "{{ DIR }}/Cargo.toml"


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
