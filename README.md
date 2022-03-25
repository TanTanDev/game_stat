<p align="center">
  <img src="https://github.com/TanTanDev/game_stat/blob/main/branding/gamestat_logo.png">
</p>

[![Crates.io](https://img.shields.io/crates/v/game_stat.svg)](https://crates.io/crates/game_stat)
[![docs.rs](https://img.shields.io/docsrs/game_stat.svg)](https://docs.rs/game_stat/latest/game_stat/)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/d/game_stat.svg)](https://crates.io/crates/game_stat)

# game_stat

`game_stat` is a small Rust library for handling stats that can change with modifiers. Equipped an epic sword? Then your attack stats could **increase by 40**. Received a debuff? Your movement speed could **decrease by 50%**.

## Example code

```rs
let mut armor_stat: Stat<2> = Stat::new(10f32);
{
    let _modifier_handle = armor_stat.add_modifier(StatModifier::Flat(5f32));
    println!("armor_stat is: {} it should be 15!", armor_stat.value());
}
println!("armor_stat is: {}, It should be 10!", armor_stat.value());
```

* This code creates a `Stat<2>`, meaning it can hold a maximum of 2 modifiers (a design decision explained later).
* `armor_stat.value()` returns our stat value based on what modifiers are active.
* We add a flat modifier, it is valid as long as the `_modifier_key` exists, which is why our value goes back to 10 when it gets dropped from the stack.

## Features

* Say goodbye to `stat.remove_modifier()`. This library has no such feature, instead a modifier is valid as long as a handle to it exists. It's a cool idea, but I don't know yet if this design choice will be practical.
* Everything lives on the stack. (blessing or curse, you tell me... I just wanted to experiment with const generics). This is achieved by stating in stat creation what the max modifier count is `Stat<4>`.
* Customizable Modifier order (optional), some games might require a more customizable Modifier application, use `stat.add_modifier_with_order()` instead of `stat.add_modifier()`.

## Is it battle ready?

I have not used this library in any real project yet. My design choices sounds good on paper, but is it useful? Time will tell.

## API design goals

* **Tiny**, laser focused. no external dependencies (except for std)

## License

gamestat is free and open source! All code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](docs/LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](docs/LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
