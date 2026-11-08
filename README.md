# Barney

**Bring your own operating system.**

Barney is an open-source engineering platform for describing, building, composing and maintaining operating systems.

It isn't another Linux distribution, nor is it simply a package manager or a collection of build scripts. Barney is intended to provide the engineering machinery behind an operating system, so that people can build and own systems without having to assemble every part of that machinery themselves.

[Website](https://barney-build.github.io/) · [Discussions](https://github.com/orgs/barney-build/discussions) · [Issues](https://github.com/barney-build/barney/issues)

## Why Barney?

Operating system engineering involves much more than building packages. It means managing recipes, dependencies, build environments, variants, bootstrap stages, composition and the relationships between all of them.

These concerns are often handled by separate tools, ad-hoc scripts and knowledge that exists only in the heads of the people maintaining a system.

Barney is an attempt to bring that engineering into a coherent, reusable model.

The goal is to make operating system engineering more transferable and composable, without prescribing what the resulting operating system must look like.

## The engineering model

At the centre of Barney is `.bny`, a structured, schema-driven language for describing recipes and their relationships.

Rather than treating build definitions as arbitrary executable scripts, Barney aims to make their structure and intent explicit.

The model is being developed around:

- **Recipes and modules** — reusable descriptions of packages and build work.
- **Typed properties and arguments** — explicit inputs and configuration.
- **Variants** — alternative forms of a recipe for different requirements and build contexts.
- **Dependencies** — relationships between components and the work needed to produce them.
- **Phases** — explicit stages for organising the build process.
- **Composition** — combining reusable building blocks into complete systems.

This structured approach provides a foundation for validation, reproducibility, tooling and eventually richer integrations with external tools and agents.

The language and underlying schema are actively evolving.

## An ecosystem, not a prescribed distribution

Barney separates the platform from the building blocks and systems created with it.

| Project | Purpose |
|---|---|
| [Barney](https://github.com/barney-build/barney) | Core platform and engineering machinery |
| [bny-base](https://github.com/barney-build/bny-base) | Reference recipes and reusable modules |
| [Pavee OS](https://github.com/barney-build/pavee-os) | An example system built with the ecosystem |

bny-base is optional. It provides a useful starting point, but you're free to adapt it, replace it or create your own collection.

Pavee OS demonstrates one possible direction for a system built on the platform. It isn't the definition of what Barney should produce.

## Development status

Barney is under active development.

The recipe language, schema and validation model are being established as the foundations for the wider platform. Other parts of the intended engineering pipeline are still being developed.

Not every capability described by the long-term direction is available today. Interfaces and implementation details may change as the architecture takes shape.

Follow the [roadmap](https://barney-build.github.io/roadmap/) and [GitHub Discussions](https://github.com/orgs/barney-build/discussions) for development updates.

## Get involved

Barney is developed independently and in the open.

There are several ways to take part:

- Explore the source and follow development.
- Report issues and share feedback.
- Experiment with recipes and modules.
- Help test the language and tooling.
- Improve documentation and examples.
- Discuss ideas and possible directions.

Start with the [Barney repositories](https://github.com/orgs/barney-build/repositories) or join the [community discussions](https://github.com/orgs/barney-build/discussions).

## License

Barney is available under the terms of the [Mozilla Public License 2.0](https://spdx.org/licenses/MPL-2.0.html).
