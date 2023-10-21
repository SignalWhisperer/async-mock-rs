# async-mock

[<img alt="github" src="https://img.shields.io/badge/github-SignalWhisperer/async--mock-3A4C7E?style=plastic&labelColor=555555&logo=github" height="20">](https://github.com/SignalWhisperer/async-mock-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/async-mock.svg?style=plastic&color=834B02&logo=rust" height="20">](https://crates.io/crates/async-mock)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-async--mock-28624E?style=plastic&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/async-mock)

Async trait mocking library for Rust. Also supports mocking non-async traits.

### Usage

`async-mock` is normally used for testing only. To do so, add it to `Cargo.toml` as such:
```cargo
[dev-dependencies]
async-mock = "0.1.3"
```

Then you can use it as such:
```rust
#[cfg(test)]
use async_mock::async_mock;
use async_trait::async_trait;

#[cfg_attr(test, async_mock)]
#[async_trait]
trait MyTrait {
    async fn foo(&self, x: i32) -> i32;
}

#[derive(Default)]
struct MyStruct;
impl MyStruct {
    async fn bar(&self, my_trait: &impl MyTrait, x: i32) -> i32 {
        my_trait.foo(x * 2).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn some_test() {
        let mut mock = MockMyTrait::default();
        mock.expect_foo()
            .times(1)
            .returning(|x| x + 1);

        let system_under_test = MyStruct::default();
        assert_eq!(7, system_under_test.bar(&mock, 3).await);
    }
}
```

The traits will have a mock structure created named by prefixing `Mock` to the trait name.
This structure implements the trait and all of its functions. To setup the mocking, call the
`expect_*` functions, where `*` is the name of the function you want to mock. This returns
a mutable reference to an expectation object on which you can set the mocking expectations.
To set the number of expected calls, you can use `once()`, `never()`, or `times(u32)`. To
set the mocked return value, call `returning` with a closure that takes the same arguments
as your trait function. If your trait function has an argument that contains a `&impl`, you
must call `returning_dyn` instead and enclose your closure in a `Box<T>`.

### Limitations

As of v0.1.3, `async-mock` does not support generics and has a hard-coded dependency on
[async-trait](https://crates.io/crates/async-trait), which you should be using already anyway.
There may be many edge cases not covered, it does not support input filtering, and it does not
support passing through a sequence of functions across multiple invocations. There are many
more limitations since this crate is fairly young. Pull requests are welcome!

Non exhaustive list of limitations:
- When mocking an async trait that has an `&impl` argument, you must call `returning_dyn` instead of `returning` and put the closure into a `Box<T>`.
- Calling async functions from the mocking closures passed to `returning()` and `returning_dyn()` is not supported.
- Generics are not supported.

### Acknowledgements

`async-mock` takes great inspiration from [Mockall](https://crates.io/crates/mockall) and actually aims
to complete it by covering the use case of async traits, which Mockall currently does not support very well.
`async-mock` aims to be a nearly drop-in replacement for Mockall when it comes to async traits only,
but not a replacement when it comes to everything else.
