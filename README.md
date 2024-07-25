![Pipeline Status](https://gitlab.com/Aleksbgbg/opencraft/badges/main/pipeline.svg)

# Opencraft

Opencraft is an open-source video game inspired by Minecraft.

We aim to create the hackable game that the community has always wanted, at
every level:
 - assets and shaders can be modified easily as part of the game distribution;
 - anyone can contribute code to the main repository; and
 - more ambitious players can create a custom fork to implement the features
   they have always wanted.

## Cloning

We use git submodules, mainly for our assets. Submodules do not decouple the
assets from the main repository like an external download does (which has
benefits and drawbacks), however they do avoid bloating the main repository
with many copies of binary files, and offer the flexibility that the entire
repository can be shallow-cloned or even re-created in the future, if it
becomes too large.

To clone the repository:
```
git clone --recurse-submodules https://gitlab.com/Aleksbgbg/opencraft.git
```

## Building and Running

We build our Rust code with [cargo](https://doc.rust-lang.org/cargo). See
[Install Rust](https://www.rust-lang.org/tools/install) to install.

Run the project with:
```
cargo run
```

## Roadmap

Opencraft is currently in its very early stages. See #1 for a list of features
that we are working on and need help with.

## Technology

We use the [wgpu](https://github.com/gfx-rs/wgpu) crate to render our game,
which means Opencraft can be configured to run on any GPU API on several
platforms, and even ported to run in the browser.

Most of our game engine math is implemented by hand. We use
[rotors](https://jacquesheunis.com/post/rotors) instead of quaternions for
rotation computations, as they are easier to understand and use. Rotors are not
widespread in use, therefore we aim to provide a complete implementation that
can be studied or used as a reference by those who wish to implement them in
their own game engines.

We intend to implement a high-quality game engine that is more efficient than
Minecraft and makes better use of hardware. Opencraft servers should have low
system requirements so that players can easily and cheaply host servers for
their own community, even on embedded systems such as the Raspberry Pi.
