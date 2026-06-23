# get-size2

[![Crates.io](https://img.shields.io/crates/v/get-size2)](https://crates.io/crates/get-size2)
[![docs.rs](https://img.shields.io/docsrs/get-size2)](https://docs.rs/get-size2)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/bircni/get-size2/blob/main/crates/get-size2/LICENSE)

> This repo is a fork of get-size, as it is not maintained anymore. The original repo can be found [here](https://github.com/DKerp/get-size)

Determine the size in bytes an object occupies inside RAM.

The [`GetSize`] trait can be used to determine the size of an object inside the stack as well as in the heap. The [`size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html) function provided by the standard library can already be used to determine the size of an object in the stack, but many application (e.g. for caching) do also need to know the number of bytes occupied inside the heap, for which this library provides an appropriate trait.

## Example

We use [`GetSize`] to determine number of bytes occupied by both a `String` and a `Vec` of bytes. Note that the `Vec` has already allocated a capacity of `1024` bytes, and does thus correctly show a heap size of `1024`, even if only `1` byte is currently in use.

```rust
use get_size2::GetSize;

fn main() {
  let value = String::from("Hello World!");

  assert_eq!(String::get_stack_size(), std::mem::size_of::<String>());
  assert_eq!(value.get_heap_size(), 12);

  assert_eq!(value.get_size(), std::mem::size_of::<String>() + 12);


  let mut buffer = Vec::with_capacity(1024); // 1KB allocated on the heap.
  buffer.push(1u8); // 1 byte in use.

  assert_eq!(buffer.len(), 1);
  assert_eq!(buffer.get_heap_size(), 1024);
}
```

## Ownership based accounting

This library follows the idea that only bytes owned by a certain object should be accounted for, and not bytes owned by different objects which are only borrowed. This means in particular that objects referenced by pointers are ignored.

On the other hand references implemented as shared ownership are treated as owned values. It is your responsibility to ensure that the bytes occupied by them are not counted twice in your application. The `ignore` attribute might be helpful, [see below](#ignoring-certain-values).

## How to implement

The [`GetSize`] trait is already implemented for most objects defined by the standard library, like `Vec`, `HashMap`, `String` as well as all the primitive values, like `u8`, `i32` etc.

Unless you have a complex datastructure which requires a manual implementation, you can easily derive [`GetSize`] for your own structs and enums. The derived implementation will implement [`get_heap_size`] by simply calling [`get_heap_size`] on all values contained inside the struct or enum variant and return the sum of them.

You will need to activate the `derive` feature first, which is disabled by default. Add the following to your `cargo.toml`:

```toml
get-size2 = { version = "^0.7", features = ["derive"] }
```

Note that the derive macro _does not support unions_. You have to manually implement it for them.
The derive macro does also work with generics. The generated trait implementation will by default require all generic types to implement [`GetSize`] themselves, but this [can be changed](#ignoring-certain-generic-types).

### Dealing with external types which do not implement GetSize

Deriving [`GetSize`] is straight forward if all the types contained in your data structure implement [`GetSize`] themselves, but this might not always be the case. For that reason the derive macro offers some helpers to assist you in that case.

Note that the helper attributes are supported for structs (named and tuple; `size_fn` requires named fields). For enums, only `ignore` is supported on named fields.

#### Ignoring certain values

You can tell the derive macro to ignore certain struct fields by adding the `ignore` attribute to them. The generated implementation of [`get_heap_size`] will then simple skip this field.

But you may also use this as a band aid, if a certain struct fields type does not implement [`GetSize`].

Be aware though that this will result in an implementation which will return incorrect results, unless the heap size of that type is indeed always zero and can thus be ignored.
It is therefor advisable to use one of the next two helper options instead.

#### Returning a fixed value

In same cases you may be dealing with external types which allocate a fixed amount of bytes at the heap. In this case you may use the `size` attribute to always account the given field with a fixed value.

#### Using a helper function

In same cases you may be dealing with an external data structure for which you know how to calculate its heap size using its public methods. In that case you may either use the newtype pattern to implement [`GetSize`] for it directly, or you can use the `size_fn` attribute, which will call the given function in order to calculate the fields heap size.

The latter is especially useful if you can make use of a certain trait to calculate the heap size for multiple types.

Note that unlike in other crates, the name of the function to be called is **not** encapsulated by double-quotes ("), but rather given directly.

#### Ignoring certain generic types

If your struct uses generics, but the fields at which they are stored are ignored or get handled by helpers because the generic does not implement [`GetSize`], you will have to mark these generics with a special struct level `ignore` attribute. Otherwise the derived [`GetSize`] implementation would still require these generics to implement [`GetSize`], even though there is no need for it.

## Tracking shared ownership

To avoid double-counting shared ownership (e.g. `Rc`, `Arc`), use a tracker:

```rust
use get_size2::{GetSize, StandardTracker};

fn main() {
  let value = std::sync::Arc::new(String::from("hello"));
  let (heap_size, _tracker) = value.get_heap_size_with_tracker(StandardTracker::new());
  assert_eq!(heap_size, std::mem::size_of::<String>() + 5);
}
```

## License

This library is licensed under the [MIT license](http://opensource.org/licenses/MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this library by you, shall be licensed as MIT, without any additional terms or conditions.

[`GetSize`]: https://docs.rs/get-size2/latest/get_size2/trait.GetSize.html
[`get_heap_size`]: https://docs.rs/get-size2/latest/get_size2/trait.GetSize.html#method.get_heap_size
