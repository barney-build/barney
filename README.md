# barney

> This project is developed by [Ikey Doherty](https://github.com/TravellingEngineer) and is
part of the [barney](https://github.com/barney-build) ecosystem initiative.

![GitHub Sponsors](https://img.shields.io/github/sponsors/TravellingEngineer?style=for-the-badge&logo=github)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6Q326CJIX)

`barney` is not a distro, or a package manager. It is a full composition pipeline for producing atomic, immutable Linux based operating systems in an entirely reproducible fashion including the entire bootstrap.

Quite the mouthful!

For some background information about the motivation for this project and how I was drawn out of retirement, check the [TravellingEngineer.com blogpost](https://travellingengineer.com/blog/first-post/).

## Status (simplified list)

 - [ ] Domain Specific Language
    - [x] Modules
    - [ ] Recipes
    - [ ] Distro definition (`distro.kdl`)
- [ ] Compiler
- [ ] Execution engine
- [ ] Binary artefacts (caching / binary distro)

## Supporting the project

I'm no millionare and never will be. However if you like my work or content I'm of the mind that caffeine is compulsory and I don't mind help paying for it. Also any little help enables better hardware for me and others (ie a Hetzner builder or whatever).

 - [Ko-fi](https://ko-fi.com/TravellingEngineer) - also exclusive access to content
 - [GitHub sponsors](https://github.com/sponsors/TravellingEngineer) - primarily to support my **code** directly.

## The problem

It's all well and good writing "yet another package manager" without actually having a problem to solve. In a nut shell - binary is great but there is no `1:1` mapping to source (temporal drift, etc).

### Incremental binary model doesn't work.

Binary-first distributions are amazing for end users but an absolute nightmare in terms of source recipe drift and bootstrap. They rely entirely on being self hosting to be able to produce future packages, which may in turn "brick the world" while performing mass builds or simple package updates.

Yes, distros have many workarounds but the reality is there is no perfect world where you can automatically produce the minimal queue of work needed to provide updates and still know that full from-source bootstrap is possible and valid.

### Pure source model isn't exactly adoption-friendly.

I mean, unless you're the CEO of an electric company maybe.

## What we need is another language!

OK, I'm half joking. While I firmly believe any packaging recipe format that requires execution for evaluation has no place in the modern world, I also don't think we need to go turing complete. What we need is sensible control flow, but most importantly, fully hashable inputs without escape hatches that introduce variance (shell escapes, etc.).

In the interest of keeping it `KISS`, we're parsing our files first as legal [KDL](https://kdl.dev) documents, and then producing our `AST` from the input nodes to produce a recipe compiler architecture.

Think of these as well structured documents that happen to support variable/property bindings and extremely simplified control flow:

 - List enumeration (`ForEach`)
 - Boolean or non-empty test (`With`)

The semantics are bolted directly on top of KDL `nodes`, no special magic. Consequently we're able to fully bind our variables, symbols, etc, to gain a true chain of custody into each callsite in a recipe and produce true hashes for our dependency graph invalidation.

## And a whole lot of experience.

No shit. I've been building distros and tooling for way too long. It was always obvious to me that source distros are the only ones without any temporal or incidental drift between source recipes and published packages. However, there's never been a "fast" way to use these.

The plan then is to produce a binary distribution with all the standard bells and whistles but from source:

 - Content addressable storage (hashed & deduplicated in on-disk pool, per-object granularity, no file sharding)
 - Atomic update between state changes
 - Immutable runtime
 - Stateless OS (these days they call this "hermetic")
 - Minimal downtime / fuss
 - Unbrickable boot config (ala `clr-boot-manager` or `moss-boot`)

In our case we're currently aiming for efficient binary artefacts (packages) that function as hash checks: "oh this build is missing I need to do it". We'll assemble the resulting artefacts for rootfs assessmbly either in package builds or client installs using something like a dynamically generated `erofs` client-side backed by the on-disk CA pool to eliminate block copies and enable immediate atomic writes/pivots (mount tucking).

## But actually solve bootstrapping.

Any binary distro relies on a hand-mangled toolchain being staged with some serious expert-level voodoo. That's great, but its not reproducible in any sense. It also doesn't help with "we have a minor libc bump" and we need to full distro rebuild with minimal fallout but not shit the bed next time we rebuild the compiler.

For that we need a new approach. Barney is being built around `phases`, explicitly described bootstrap stages, in configuration files. Additionally we're relying on our fancy-pants conditionals to have **non-default variants** of recipes.

Let that one sink in a moment. If we assume all recipes are in their "default" variant, we can permit non-default variants that are explicitly requested as part of a build chain. By being **non-default** - they do not "publish" to the final repository / artefact tree. Essentially they're free interim builds designed to break the cycles of packages. 

Quick example. Our "world" needs `util-linux`. It also needs `systemd`. However `util-linux` has a build dependency on `systemd` and guess what? `systemd` has a build dependency (multiple) on `util-linux` ...

Until `systemd` actually has a build dependency on `util-linux(minimal)` eliminating the circles, and `util-linux(default)` being pulled in as a later build dependency in the chain for everything else (as well as being an installation candidate).

TLDR: Break atoms down for variants, introduce tier gates as control units inside a directed acyclical graph, forbid cycle breaking in builds (permit in runtime dependencies), and group the tiers within phases.

In case you're wondering.. yes you can still have rich build dependencies (`pkgconfig(libpng)`) - assuming the build queue can differentiate the groups, and track the so-called "in degrees" while supporting dynamic resolution.


## License

Copyright © 2026 Ikey Doherty

`barney` is available under the terms of the [MPL-2.0](https://spdx.org/licenses/MPL-2.0.html) license.
