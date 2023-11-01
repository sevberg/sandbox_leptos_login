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
