### Enable macOS notifications with the Rusty Pomo icon

On macOS, notifications use the icon of the app bundle that posts them. You cannot set a per-notification PNG. To show the Rusty Pomo logo, attribute notifications to an installed app bundle that has your icon.

- Install the bundler and build the app:

```bash
cargo install cargo-bundle
cargo bundle --release
```

- Move into Applications so macOS treats it as an installed app:

```bash
mv target/release/bundle/osx/Rusty\ Pomo.app/ /Applications/
```

- Run the CLI using the bundle’s identifier
The app already supports a flag to attribute notifications to a bundle id.

```bash
cargo run -- --macos-bundle-id dev.jorbush.rusty-pomo
```

Now notifications should appear with the Rusty Pomo icon. Notice that now the Rusty Pomo app is installed in the Applications folder but it doesn't open when you click on it because
