**WARNING**: c3ne is very experimental. At the time of writing, only GNU/Linux x64 has been tested, but Windows should, in theory, work. Anything else is even more experimental so expect things to break. Cross-compilation has also not been tested.

## How to pronounce it
c3ne is pronounced as 'citrine'.

## Why c3ne exists
Interoperating with other languages is not always easy, which is why crates such as `cc-rs` exist.

I tried using it to interoperate with C3, but after experimenting for a while, I got nowhere.

After that, I decided to write my own solution that supported C3, and am now releasing it.

## How to use c3ne
Using c3ne is not very difficult, in general, you should follow these steps:
1. Add c3ne to your `build-dependencies`:
```toml
[build-dependencies]
c3ne = "0.1.0"
# ...
```
```
2. After that, open or create a `build.rs`, and write the following:
```rs
use c3ne::C3FFI;

fn main() {
    C3FFI::new().file("extern/hello.c3").compile("hello");
}
```
```
3. Once you have your `build.rs` defined, create `extern/hello.c3`, and write the following inside:
```c3
module hello @export;

import std::io;

fn void greet()
{
    io::printn("Hello from C3!");
}
```
4. And finally, in your `src/main.rs` write:
```rs
unsafe extern "system" {
    fn greet();
}

fn main() {
    unsafe {
        greet();
    }
}
```

Now compile and run the code and you should see "Hello from C3!" printed out to your console!

## Getting deeper
c3ne provides some more options which may be useful to you depending on your goal, you can view the documentation for the crate to see what is available.

## Extras
I have also written another crate that may be of use, `c3ne-types`, which implements some of C3's more messy types, mainly strings.
