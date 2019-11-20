# tocenter

[![Build Status](https://travis-ci.org/silentsokolov/tocenter.svg?branch=master)](https://travis-ci.org/silentsokolov/tocenter)
![](https://github.com/silentsokolov/tocenter/workflows/Release/badge.svg)

**ToCenter** is a video game written in [Rust](https://www.rust-lang.org) using the [ggez](https://github.com/ggez/ggez) engine and the [ECS](https://github.com/amethyst/specs) library. This game is my hobby project and not considerate a best example use ggez.

## play

The game available on windows / macOS / linux. [Download latest release](https://github.com/silentsokolov/tocenter/releases).

#### Linux users

Not all libraries support [musl](https://www.musl-libc.org/) (like `girls-core`). It is impossible to make a fully static binary. You will need `glibc>=2.23` and `alsa` for start game. (`alsa` is `libasound2-dev` on Debian, `alsa-lib-devel` on CentOS, `alsa-lib-dev` on Alpine)

## screenshot

![ToCenter Game](https://raw.githubusercontent.com/silentsokolov/tocenter/master/.github/docs/screen1.png)