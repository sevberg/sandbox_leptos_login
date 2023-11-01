# Sandbox Leptos Login Reactivity

This is a sandbox project which showcases how to implement login reactivity in a Leptos web
application.

The main components are:

- `App`: A top-level app component which defines a `user_data` signal which is intended to be
  reused in it's children
- `Control-Area`: A component which handles login/logout actions taken by a user and, when
  necessary, syncs those results to the App's `user_data` context
- `Display-Area`: A component which watches the App's `user_data` context, and constructs it's
  view accordingly

## Usage

After cloning this repo, run me with:

```bash
trunk serve
```

## The Problem

Since logging-in requires talking to a backend server in an async function call, the Control-Area
component must make use of the `create_local_resource` functionality in Leptos. However I could
not find an obvious (at least, to me, with my limited knowledge of Leptos and websassembly as a
whole) way to set the result of this async function call into the App's `user_data` context.

The only thing which seemed to work involved using the `create_effect` functionality in Leptos to
invoke the `user_data.set()` setter. However, this goes directly against what is stated in the
[Leptos docs](https://docs.rs/leptos/latest/leptos/fn.create_effect.html):

> Effects are intended to run side-effects of the system, not to synchronize state within the
> system. In other words: don’t write to signals within effects, unless you’re coordinating with
> some other non-reactive side effect. (If you need to define a signal that depends on the value
> of other signals, use a derived signal or create_memo).

Unfortunately, using a derived signal or create_memo did not seem to work, either.

## The Question

How would one get around having to use `create_effect`, here? Alternatively, how else would one
implement reactive user-logins resulting from async server communication.
