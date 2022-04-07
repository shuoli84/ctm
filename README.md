# Intro
ctm(custom toolchain manager) is a tool helps you to build and manage custom rust toolchains.
Maybe you want to tweak some rust compiler code and use it to build against several crates
to see whether anything different. Or try build dust with several profiles (min size, max performance) 
and benchmark it with real data?

# Usage

```bash
git clone <folder>
cd <folder>
cargo install --path .

# create a workbench folder in ~
cd ~ && mkdir ctm_demo
# go to folder
cd ctm_demo
# create an init config
ctm init

# (optional) edit the config
vim config.toml

# clone rust repo and its submodules, this may take a while
ctm bootstrap

# build toolchains
ctm build-toolchain

# build crates for each toolchain-profile
ctm build-crate --crate dust

# outputs in json format, you can use nushell to further filter or sort it
[
  {
    "toolchain": "base",
    "profile": "minsize",
    "krate": "dust",
    "binary_size": 1612232,
    "path": ".../build/dust/target/base_minsize/release/dust"
  },
  {
    "toolchain": "base",
    "profile": "maxspeed",
    "krate": "dust",
    "binary_size": 1903048,
    "path": ".../build/dust/target/base_maxspeed/release/dust"
  }
]

# run each run cmd for crate and print duration statistic
ctm run --crate dust

[
  {
    "toolchain": "base",
    "profile": "minsize",
    "krate": "dust",
    "cmd": "dust_home",
    "binary_size": 1612232,
    "duration_ms_hist_min": 912,
    "duration_ms_hist_p50": 942,
    "duration_ms_hist_p90": 973,
    "duration_ms_hist_max": 986
  },
  {
    "toolchain": "base",
    "profile": "maxspeed",
    "krate": "dust",
    "cmd": "dust_home",
    "binary_size": 1903048,
    "duration_ms_hist_min": 734,
    "duration_ms_hist_p50": 787,
    "duration_ms_hist_p90": 860,
    "duration_ms_hist_max": 961
  }
]
```

# Config

```toml
[global]
# the project root, default is same folder as this file
# project_root = ""

# if the path is relative, then it is relative to project root 
rust_repo = "rust"

# the git revision
rust_rev = "9d1b210"

# all built toolchains will be put in this folder
toolchains_root = "toolchains"

# where to find patch files
patch_root = "patches"

# all crate will be checked out or copied to this folder
build_root = "build"

[[toolchains]]
# base toolchain is build without any patch, it serves as base line
name = "base"
# the profiles for this toolchain, each of them will be used to build crate
profiles = [ "minsize", "maxspeed" ]

# optimize for speed
[[profiles]]
name = "maxspeed"

# control cargo build profile with environment variables, refer to 
# https://doc.rust-lang.org/cargo/reference/environment-variables.html
[profiles.environ]
CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1"
CARGO_PROFILE_RELEASE_LTO = "fat"
CARGO_PROFILE_RELEASE_OPT_LEVEL = "3"

# optimize for size
[[profiles]]
name = "minsize"

[profiles.environ]
CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1"
CARGO_PROFILE_RELEASE_LTO = "fat"
CARGO_PROFILE_RELEASE_OPT_LEVEL = "z"

# sample crates
[[crates]]
name = "dust"
git = "https://github.com/bootandy/dust.git"
output_path = "release/dust"

# run agains home folder
[[crates.runs]]
# name
name = "dust_home"
# how many times this cmd should run
count = 20
# arguments to output_path
args = [ "/home" ]

```

# Q&A
Q: How to use existing rust repo? 

A: modify config.toml global.rust_repo, point to the absolute path of existing rust repo. A git reset --hard will be performed, so backup your un commited changes.
